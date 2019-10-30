//! Native marked pointers without alignment requirements exploiting the
//! property of current 64-bit architectures, which only use 48-bit virtual
//! addresses.
//! This leaves the upper 16-bit of any 64-bit pointer available for storing
//! additional tag bits.

#[cfg(all(target_arch = "x86_64", feature = "nightly"))]
mod dwcas;

use core::cmp;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(all(target_arch = "x86_64", feature = "nightly"))]
pub use dwcas::{AtomicTagPtr, TagPtr};

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedNativePtr
////////////////////////////////////////////////////////////////////////////////////////////////////

/// An atomic native 64-bit marked pointer with 16 available bits for storing a
/// tag value.
///
/// This type's API is almost identical to the more general
/// [`AtomicMarkedPtr`][crate::AtomicMarkedPtr].
/// It's advantage is its ability to store 16 bit tags regardless of the
/// alignment of type `T`.
/// However, it is also only available on 64-bit architectures that use 48-bit
/// virtual addresses, which, as of 2019, is practically every 64-bit
/// architecture, although this may change for future architectures.
pub struct AtomicMarkedNativePtr<T> {
    inner: AtomicUsize,
    _marker: PhantomData<*mut T>,
}

/********** impl Send + Sync **********************************************************************/

unsafe impl<T> Send for AtomicMarkedNativePtr<T> {}
unsafe impl<T> Sync for AtomicMarkedNativePtr<T> {}

/********** impl Default **************************************************************************/

impl<T> Default for AtomicMarkedNativePtr<T> {
    #[inline]
    fn default() -> Self {
        Self::null()
    }
}

/********** impl inherent *************************************************************************/

impl<T> AtomicMarkedNativePtr<T> {
    /// The number of available mark bits for this type.
    pub const MARK_BITS: usize = 16;
    /// The bitmask for the lower markable bits.
    pub const MARK_MASK: usize = 0xFFFF << Self::MARK_SHIFT;
    /// The bitmask for the (higher) pointer bits.
    pub const POINTER_MASK: usize = !Self::MARK_MASK;

    const MARK_SHIFT: usize = 48;

    /// Creates a new and unmarked `null` pointer.
    pub const fn null() -> Self {
        Self { inner: AtomicUsize::new(0), _marker: PhantomData }
    }

    /// Creates a new [`AtomicMarkedNativePtr`].
    #[inline]
    pub fn new(marked_ptr: MarkedNativePtr<T>) -> Self {
        Self { inner: AtomicUsize::new(marked_ptr.into_usize()), _marker: PhantomData }
    }

    /// Consumes `self` and returns the inner [`MarkedNativePtr`].
    #[inline]
    pub fn into_inner(self) -> MarkedNativePtr<T> {
        MarkedNativePtr::from_usize(self.inner.into_inner())
    }

    /// Returns a mutable reference to the underlying [`MarkedNativePtr`].
    ///
    /// This is safe because the mutable reference guarantees that no other
    /// threads are concurrently accessing the atomic data.
    #[inline]
    pub fn get_mut(&mut self) -> &mut MarkedNativePtr<T> {
        unsafe { &mut *(self.inner.get_mut() as *mut usize as *mut _) }
    }

    /// Loads the value of the [`AtomicMarkedNativePtr`].
    ///
    /// `load` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation. Possible values are [`SeqCst`][seq_cst],
    /// [`Acquire`][acq] and [`Relaxed`][rlx].
    ///
    /// # Panics
    ///
    /// Panics if `order` is [`Release`][rel] or [`AcqRel`][acq_rel].
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    /// [acq_rel]: Ordering::AcqRel
    /// [seq_cst]: Ordering::SeqCst
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::atomic::Ordering;
    ///
    /// use conquer_pointer::arch64::{AtomicMarkedNativePtr, MarkedNativePtr};
    ///
    /// let atomic = AtomicMarkedNativePtr::new(MarkedNativePtr::compose(&mut 5, 0b1));
    ///
    /// let load = atomic.load(Ordering::SeqCst);
    /// assert_eq!((Some(&mut 5), 0b1), unsafe { load.decompose_mut() });
    /// ```
    #[inline]
    pub fn load(&self, order: Ordering) -> MarkedNativePtr<T> {
        MarkedNativePtr::from_usize(self.inner.load(order))
    }

    /// Stores a value into the [`AtomicMarkedNativePtr`].
    ///
    /// `store` takes an [`Ordering`] argument which describes the
    /// memory ordering of this operation. Possible values are
    /// [`SeqCst`][seq_cst], [`Release`][rel] and [`Relaxed`][rlx].
    ///
    /// # Panics
    ///
    /// Panics if `order` is [`Acquire`][acq] or [`AcqRel`][acq_rel].
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    /// [acq_rel]: Ordering::AcqRel
    /// [seq_cst]: Ordering::SeqCst
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::atomic::Ordering;
    ///
    /// use conquer_pointer::arch64::{AtomicMarkedNativePtr, MarkedNativePtr};
    ///
    /// let atomic = AtomicMarkedNativePtr::null();
    /// let store = MarkedNativePtr::new(&mut 10);
    ///
    /// atomic.store(store, Ordering::SeqCst);
    /// ```
    #[inline]
    pub fn store(&self, ptr: MarkedNativePtr<T>, order: Ordering) {
        self.inner.store(ptr.into_usize(), order);
    }

    /// Stores a value into the pointer if the current value is the same
    /// as `current`.
    ///
    /// The return value is always the previous value.
    /// If it is equal to `current`, then the value was updated.
    ///
    /// `compare_and_swap` also takes an [`Ordering`] argument which describes
    /// the memory ordering of this operation.
    /// Notice that even when using [`AcqRel`][acq_rel], the operation might
    /// fail and hence just perform an `Acquire` load, but not have `Release`
    /// semantics.
    /// Using [`Acquire`][acq] makes the store part of this operation
    /// [`Relaxed`][rlx] if it happens, and using [`Release`][rel] makes the
    /// load part [`Relaxed`][rlx].
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    /// [acq_rel]: Ordering::AcqRel
    #[inline]
    pub fn compare_and_swap(
        &self,
        current: MarkedNativePtr<T>,
        new: MarkedNativePtr<T>,
        order: Ordering,
    ) -> MarkedNativePtr<T> {
        MarkedNativePtr::from_usize(self.inner.compare_and_swap(
            current.into_usize(),
            new.into_usize(),
            order,
        ))
    }

    /// Stores a value into the pointer if the current value is the same
    /// as `current`.
    ///
    /// The return value is a result indicating whether the new value was
    /// written and containing the previous value.
    /// On success this value is guaranteed to be equal to `current`.
    ///
    /// `compare_exchange` takes two [`Ordering`] arguments to describe the
    /// memory ordering of this operation.
    /// The first describes the required ordering if the operation succeeds
    /// while the second describes the required ordering when the operation
    /// fails.
    /// Using [`Acquire`][acq] as success ordering makes the store part of this
    /// operation [`Relaxed`][rlx], and using [`Release`][rel] makes the
    /// successful load [`Relaxed`][rlx].
    /// The failure ordering can only be [`SeqCst`][seq_cst], [`Acquire`][acq]
    /// or [`Relaxed`][rlx] and must be equivalent to or weaker than the success
    /// ordering.
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    /// [seq_cst]: Ordering::SeqCst
    #[inline]
    pub fn compare_exchange(
        &self,
        current: MarkedNativePtr<T>,
        new: MarkedNativePtr<T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<MarkedNativePtr<T>, MarkedNativePtr<T>> {
        self.inner
            .compare_exchange(current.into_usize(), new.into_usize(), success, failure)
            .map(MarkedNativePtr::from_usize)
            .map_err(MarkedNativePtr::from_usize)
    }

    /// Stores a value into the pointer if the current value is the same
    /// as `current`.
    ///
    /// Unlike [`compare_exchange`][AtomicMarkedPtr::compare_exchange], this
    /// function is allowed to spuriously fail even when the comparison
    /// succeeds, which can result in more efficient code on some platforms.
    /// The return value is a result indicating whether the new value was
    /// written and containing the previous value.
    ///
    /// `compare_exchange_weak` takes two [`Ordering`] arguments to describe the
    /// memory ordering of this operation.
    /// The first describes the required ordering if the operation succeeds
    /// while the second describes the required ordering when the operation
    /// fails.
    /// Using [`Acquire`][acq] as success ordering makes the store part of this
    /// operation [`Relaxed`][rlx], and using [`Release`][rel] makes the
    /// successful load [`Relaxed`][rlx].
    /// The failure ordering can only be [`SeqCst`][seq_cst], [`Acquire`][acq]
    /// or [`Relaxed`][rlx] and must be equivalent to or weaker than the success
    /// ordering.
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    /// [seq_cst]: Ordering::SeqCst
    #[inline]
    pub fn compare_exchange_weak(
        &self,
        current: MarkedNativePtr<T>,
        new: MarkedNativePtr<T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<MarkedNativePtr<T>, MarkedNativePtr<T>> {
        self.inner
            .compare_exchange_weak(current.into_usize(), new.into_usize(), success, failure)
            .map(MarkedNativePtr::from_usize)
            .map_err(MarkedNativePtr::from_usize)
    }

    /// Adds to the current tag value, returning the previous [`MarkedNativePtr`].
    ///
    /// Fetch-and-add operates on the entire [`AtomicMarkedNativePtr`] and has
    /// no notion of any tag bits or a maximum number thereof.
    /// Since the operation is also infallible, it may be impossible to
    /// guarantee that incrementing the tag value can not overflow into the
    /// pointer bits, which would corrupt both values and lead to undefined
    /// behaviour as soon as the pointer is de-referenced.
    ///
    /// `fetch_add` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation.
    /// All ordering modes are possible.
    /// Note that using [`Acquire`][acq] makes the store part of this operation
    /// [`Relaxed`][rlx], and using [`Release`][rel] makes the load part
    /// [`Relaxed`][rlx].
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    ///
    /// # Panics
    ///
    /// This method panics **in debug mode** if it is detected (after the fact)
    /// that an overflow has occurred.
    /// Note, that this does not guarantee that no other thread can observe the
    /// corrupted pointer value before the panic occurs.
    #[inline]
    pub fn fetch_add(&self, value: u16, order: Ordering) -> MarkedNativePtr<T> {
        let prev = MarkedNativePtr::from_usize(
            self.inner.fetch_add((value as usize) << Self::MARK_SHIFT, order),
        );
        debug_assert!(
            Self::MARK_MASK - value as usize >= prev.decompose_tag() as usize,
            "overflow of tag bits detected"
        );
        prev
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedNativePtr
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A marked native (64-bit) pointer of which the upper 16 bits can be used for
/// storing additional information.
#[repr(transparent)]
pub struct MarkedNativePtr<T> {
    inner: *mut T,
}

/********** impl Clone ****************************************************************************/

impl<T> Clone for MarkedNativePtr<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}

/********** impl Copy *****************************************************************************/

impl<T> Copy for MarkedNativePtr<T> {}

/********** impl Default ***************************************************************************/

impl<T> Default for MarkedNativePtr<T> {
    #[inline]
    fn default() -> Self {
        Self::null()
    }
}

/********** impl inherent *************************************************************************/

impl<T> MarkedNativePtr<T> {
    /// The number of available mark bits for this type.
    pub const MARK_BITS: usize = 16;
    /// The bitmask for the lower markable bits.
    pub const MARK_MASK: usize = 0xFFFF << Self::MARK_SHIFT;
    /// The bitmask for the (higher) pointer bits.
    pub const POINTER_MASK: usize = !Self::MARK_MASK;

    const MARK_SHIFT: usize = 48;

    /// Creates a new unmarked `null` pointer.
    #[inline]
    pub const fn null() -> Self {
        Self::new(ptr::null_mut())
    }

    /// Creates a new unmarked [`MarkedNativePtr`].
    #[inline]
    pub const fn new(ptr: *mut T) -> Self {
        Self { inner: ptr }
    }

    /// Creates a [`MarkedNativePtr`] from the integer (numeric) representation
    /// of a potentially marked pointer.
    #[inline]
    pub const fn from_usize(val: usize) -> Self {
        Self { inner: val as *mut _ }
    }

    /// Casts to a pointer of type `U`.
    #[inline]
    pub const fn cast<U>(self) -> MarkedNativePtr<U> {
        MarkedNativePtr { inner: self.inner.cast() }
    }

    /// Returns the inner pointer *as is*, meaning any potential tag is not
    /// stripped.
    ///
    /// De-referencing the returned pointer results in undefined behaviour, if
    /// the pointer is still marked and even if the pointer itself points to a
    /// valid and live value.
    #[inline]
    pub const fn into_ptr(self) -> *mut T {
        self.inner
    }

    /// Returns the integer representation of the pointer with its tag.
    #[inline]
    pub fn into_usize(self) -> usize {
        self.inner as usize
    }

    /// Clears the tag from `self` and returns the same but unmarked pointer.
    #[inline]
    pub fn clear_tag(self) -> Self {
        Self::new(self.decompose_ptr())
    }

    /// Clears the tag from `self` and replaces it with `tag`.
    #[inline]
    pub fn with_tag(self, tag: u16) -> Self {
        Self::compose(self.decompose_ptr(), tag)
    }

    /// Adds `value` to the current tag without regard for the previous value.
    ///
    /// This method does not perform any checks, so it may overflow the tag
    /// bits, result in a pointer to a different value, a null pointer or an
    //  unaligned pointer.
    #[inline]
    pub fn add_tag(self, value: usize) -> Self {
        Self::from_usize(self.into_usize() + value)
    }

    /// Subtracts `value` to the current tag without regard for the previous
    /// value.
    ///
    /// This method does not perform any checks, so it may underflow the tag
    /// bits, result in a pointer to a different value, a null pointer or an
    /// unaligned pointer.
    #[inline]
    pub fn sub_tag(self, value: usize) -> Self {
        Self::from_usize(self.into_usize() - value)
    }

    /// Composes a new [`MarkedNativePtr`] from a raw `ptr` and a `tag` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    ///
    /// type MarkedNativePtr = conquer_pointer::arch64::MarkedNativePtr<i32>;
    ///
    /// let raw = &1 as *const i32 as *mut _;
    /// let ptr = MarkedNativePtr::compose(raw, 0b11);
    /// assert_eq!(ptr.decompose(), (raw, 0b11));
    /// ```
    #[inline]
    pub fn compose(ptr: *mut T, tag: u16) -> Self {
        Self::new((ptr as usize | (tag as usize) << Self::MARK_SHIFT) as *mut _)
    }

    /// Decomposes the [`MarkedNativePtr`], returning the separated raw pointer
    /// and its tag.
    #[inline]
    pub fn decompose(self) -> (*mut T, u16) {
        (self.decompose_ptr(), self.decompose_tag())
    }

    /// Decomposes the [`MarkedNativePtr`], returning only the separated raw
    /// pointer.
    #[inline]
    pub fn decompose_ptr(self) -> *mut T {
        (self.inner as usize & Self::POINTER_MASK) as *mut _
    }

    /// Decomposes the [`MarkedNativePtr`], returning only the separated tag
    /// value.
    #[inline]
    pub fn decompose_tag(self) -> u16 {
        (self.inner as usize >> Self::MARK_SHIFT) as u16
    }

    /// Decomposes the [`MarkedNativePtr`], returning an optional reference and the
    /// separated tag.
    ///
    /// In case the pointer stripped of its tag is null, [`None`] is returned as
    /// part of the tuple. Otherwise, the reference is wrapped in a [`Some`].
    ///
    /// # Safety
    ///
    /// While this method and its mutable counterpart are useful for
    /// null-safety, it is important to note that this is still an unsafe
    /// operation because the returned value could be pointing to invalid
    /// memory.
    ///
    /// Additionally, the lifetime 'a returned is arbitrarily chosen and does
    /// not necessarily reflect the actual lifetime of the data.
    #[inline]
    pub unsafe fn decompose_ref<'a>(self) -> (Option<&'a T>, u16) {
        (self.as_ref(), self.decompose_tag())
    }

    /// Decomposes the [`MarkedNativePtr`] returning an optional mutable reference
    /// and the separated tag.
    ///
    /// In case the pointer stripped of its tag is null, [`None`] is returned as
    /// part of the tuple. Otherwise, the mutable reference is wrapped in a
    /// [`Some`].
    ///
    /// # Safety
    ///
    /// As with [`decompose_ref`][crate::MarkedPtr::decompose_ref], this is
    /// unsafe because it cannot verify the validity of the returned pointer,
    /// nor can it ensure that the lifetime `'a` returned is indeed a valid
    /// lifetime for the contained data.
    #[inline]
    pub unsafe fn decompose_mut<'a>(self) -> (Option<&'a mut T>, u16) {
        (self.as_mut(), self.decompose_tag())
    }

    /// Decomposes the [`MarkedNativePtr`], returning an optional reference and
    /// discarding the tag.
    ///
    /// # Safety
    ///
    /// The same caveats as with
    /// [`decompose_ref`][crate::MarkedPtr::decompose_ref] apply for this method
    /// as well.
    #[inline]
    pub unsafe fn as_ref<'a>(self) -> Option<&'a T> {
        self.decompose_ptr().as_ref()
    }

    /// Decomposes the [`MarkedNativePtr`], returning an optional mutable reference
    /// and discarding the tag.
    ///
    /// # Safety
    ///
    /// The same caveats as with
    /// [`decompose_mut`][crate::MarkedPtr::decompose_mut] apply for this method
    /// as well.
    #[inline]
    pub unsafe fn as_mut<'a>(self) -> Option<&'a mut T> {
        self.decompose_ptr().as_mut()
    }
}

impl<T> From<*mut T> for MarkedNativePtr<T> {
    #[inline]
    fn from(ptr: *mut T) -> Self {
        Self::new(ptr)
    }
}

impl<T> From<*const T> for MarkedNativePtr<T> {
    #[inline]
    fn from(ptr: *const T) -> Self {
        Self::new(ptr as *mut _)
    }
}

impl<'a, T> From<&'a T> for MarkedNativePtr<T> {
    #[inline]
    fn from(reference: &'a T) -> Self {
        Self::from(reference as *const _)
    }
}

impl<'a, T> From<&'a mut T> for MarkedNativePtr<T> {
    #[inline]
    fn from(reference: &'a mut T) -> Self {
        Self::new(reference)
    }
}

impl<T> From<NonNull<T>> for MarkedNativePtr<T> {
    #[inline]
    fn from(non_null: NonNull<T>) -> Self {
        Self::new(non_null.as_ptr())
    }
}

/********** impl Debug ****************************************************************************/

impl<T> fmt::Debug for MarkedNativePtr<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MarkedPtr")
            .field("ptr", &self.decompose_ptr())
            .field("tag", &self.decompose_tag())
            .finish()
    }
}

/********** impl Pointer **************************************************************************/

impl<T> fmt::Pointer for MarkedNativePtr<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.decompose_ptr(), f)
    }
}

/********** impl PartialEq ************************************************************************/

impl<T> PartialEq for MarkedNativePtr<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

/********** impl PartialOrd ***********************************************************************/

impl<T> PartialOrd for MarkedNativePtr<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

/********** impl Eq *******************************************************************************/

impl<T> Eq for MarkedNativePtr<T> {}

/********** impl Ord ******************************************************************************/

impl<T> Ord for MarkedNativePtr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

/********** impl Hash *****************************************************************************/

impl<T> Hash for MarkedNativePtr<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}
