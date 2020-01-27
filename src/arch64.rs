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
pub use self::{AtomicTagPtr, TagPtr};

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedPtr64
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
pub struct AtomicMarkedPtr64<T> {
    inner: AtomicUsize,
    _marker: PhantomData<*mut T>,
}

/********** impl Send + Sync **********************************************************************/

unsafe impl<T> Send for AtomicMarkedPtr64<T> {}
unsafe impl<T> Sync for AtomicMarkedPtr64<T> {}

/********** impl inherent *************************************************************************/

impl<T> AtomicMarkedPtr64<T> {
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

    /// Creates a new [`AtomicMarkedPtr64`].
    #[inline]
    pub fn new(marked_ptr: MarkedPtr64<T>) -> Self {
        Self { inner: AtomicUsize::new(marked_ptr.into_usize()), _marker: PhantomData }
    }

    /// Consumes `self` and returns the inner [`MarkedPtr64`].
    #[inline]
    pub fn into_inner(self) -> MarkedPtr64<T> {
        MarkedPtr64::from_usize(self.inner.into_inner())
    }

    /// Returns a mutable reference to the underlying [`MarkedPtr64`].
    ///
    /// This is safe because the mutable reference guarantees that no other
    /// threads are concurrently accessing the atomic data.
    #[inline]
    pub fn get_mut(&mut self) -> &mut MarkedPtr64<T> {
        unsafe { &mut *(self.inner.get_mut() as *mut usize as *mut _) }
    }

    /// Loads the value of the [`AtomicMarkedPtr64`].
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
    /// use conquer_pointer::arch64::{AtomicMarkedPtr64, MarkedPtr64};
    ///
    /// let atomic = AtomicMarkedPtr64::new(MarkedPtr64::compose(&mut 5, 0b1));
    ///
    /// let load = atomic.load(Ordering::SeqCst);
    /// assert_eq!((Some(&mut 5), 0b1), unsafe { load.decompose_mut() });
    /// ```
    #[inline]
    pub fn load(&self, order: Ordering) -> MarkedPtr64<T> {
        MarkedPtr64::from_usize(self.inner.load(order))
    }

    /// Stores a value into the [`AtomicMarkedPtr64`].
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
    /// use conquer_pointer::arch64::{AtomicMarkedPtr64, MarkedPtr64};
    ///
    /// let atomic = AtomicMarkedPtr64::null();
    /// let store = MarkedPtr64::new(&mut 10);
    ///
    /// atomic.store(store, Ordering::SeqCst);
    /// ```
    #[inline]
    pub fn store(&self, ptr: MarkedPtr64<T>, order: Ordering) {
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
        current: MarkedPtr64<T>,
        new: MarkedPtr64<T>,
        order: Ordering,
    ) -> MarkedPtr64<T> {
        MarkedPtr64::from_usize(self.inner.compare_and_swap(
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
        current: MarkedPtr64<T>,
        new: MarkedPtr64<T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<MarkedPtr64<T>, MarkedPtr64<T>> {
        self.inner
            .compare_exchange(current.into_usize(), new.into_usize(), success, failure)
            .map(MarkedPtr64::from_usize)
            .map_err(MarkedPtr64::from_usize)
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
        current: MarkedPtr64<T>,
        new: MarkedPtr64<T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<MarkedPtr64<T>, MarkedPtr64<T>> {
        self.inner
            .compare_exchange_weak(current.into_usize(), new.into_usize(), success, failure)
            .map(MarkedPtr64::from_usize)
            .map_err(MarkedPtr64::from_usize)
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
    pub fn fetch_add(&self, value: u16, order: Ordering) -> MarkedPtr64<T> {
        let prev = MarkedPtr64::from_usize(
            self.inner.fetch_add((value as usize) << Self::MARK_SHIFT, order),
        );
        debug_assert!(
            Self::MARK_MASK - value as usize >= prev.decompose_tag() as usize,
            "overflow of tag bits detected"
        );
        prev
    }
}

/********** impl Default **************************************************************************/

impl<T> Default for AtomicMarkedPtr64<T> {
    #[inline]
    fn default() -> Self {
        Self::null()
    }
}

/********** impl Debug ****************************************************************************/

impl<T> fmt::Debug for AtomicMarkedPtr64<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (ptr, tag) = self.load(Ordering::SeqCst).decompose();
        f.debug_struct("AtomicMarkedPtr64").field("ptr", &ptr).field("tag", &tag).finish()
    }
}

/********** impl Pointer **************************************************************************/

impl<T> fmt::Pointer for AtomicMarkedPtr64<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ptr = self.load(Ordering::SeqCst);
        fmt::Pointer::fmt(&ptr, f)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedPtr64
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A marked native (64-bit) pointer of which the upper 16 bits can be used for
/// storing additional information.
#[repr(transparent)]
pub struct MarkedPtr64<T> {
    inner: *mut T,
}

/********** impl Clone ****************************************************************************/

impl<T> Clone for MarkedPtr64<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}

/********** impl Copy *****************************************************************************/

impl<T> Copy for MarkedPtr64<T> {}

/********** impl inherent *************************************************************************/

impl<T> MarkedPtr64<T> {
    /// The number of available mark bits for this type.
    pub const MARK_BITS: usize = 16;
    /// The bitmask for the lower markable bits.
    pub const MARK_MASK: usize = 0xFFFF << Self::TAG_SHIFT;
    /// The bitmask for the (higher) pointer bits.
    pub const POINTER_MASK: usize = !Self::MARK_MASK;

    /// The number of bits the tag value is shifted to the left.
    const TAG_SHIFT: usize = 48;

    /// Creates a new unmarked `null` pointer.
    #[inline]
    pub const fn null() -> Self {
        Self::new(ptr::null_mut())
    }

    /// Creates a new unmarked [`MarkedPtr64`].
    #[inline]
    pub const fn new(ptr: *mut T) -> Self {
        Self { inner: ptr }
    }

    /// Creates a [`MarkedPtr64`] from the integer (numeric) representation
    /// of a potentially marked pointer.
    #[inline]
    pub const fn from_usize(val: usize) -> Self {
        Self { inner: val as *mut _ }
    }

    /// Casts to a pointer of type `U`.
    #[inline]
    pub const fn cast<U>(self) -> MarkedPtr64<U> {
        MarkedPtr64 { inner: self.inner.cast() }
    }

    /// Returns the inner pointer *as is*, meaning any potential tag is not
    /// stripped.
    ///
    /// De-referencing the returned pointer results in undefined behaviour, if
    /// the pointer is still marked and even if the pointer itself points to a
    /// valid and live value.
    #[inline]
    pub const fn into_raw(self) -> *mut T {
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

    /// Adds `value` to the current tag *without* regard for the previous value.
    ///
    /// This method does not perform any checks, so it may overflow the tag
    /// bits, result in a pointer to a different value, a null pointer or an
    ///  unaligned pointer.
    #[inline]
    pub fn add_tag(self, value: usize) -> Self {
        Self::from_usize(self.into_usize() + value)
    }

    /// Subtracts `value` from the current tag *without* regard for the previous
    /// value.
    ///
    /// This method does not perform any checks, so it may underflow the tag
    /// bits, result in a pointer to a different value, a null pointer or an
    /// unaligned pointer.
    #[inline]
    pub fn sub_tag(self, value: usize) -> Self {
        Self::from_usize(self.into_usize() - value)
    }

    /// Composes a new [`MarkedPtr64`] from a raw `ptr` and a `tag` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    ///
    /// type MarkedNativePtr = conquer_pointer::arch64::MarkedPtr64<i32>;
    ///
    /// let raw = &1 as *const i32 as *mut _;
    /// let ptr = MarkedNativePtr::compose(raw, 0b11);
    /// assert_eq!(ptr.decompose(), (raw, 0b11));
    /// ```
    #[inline]
    pub fn compose(ptr: *mut T, tag: u16) -> Self {
        Self::new((ptr as usize | (tag as usize) << Self::TAG_SHIFT) as *mut _)
    }

    /// Decomposes the [`MarkedPtr64`], returning the separated raw pointer
    /// and its tag.
    #[inline]
    pub fn decompose(self) -> (*mut T, u16) {
        (self.decompose_ptr(), self.decompose_tag())
    }

    /// Decomposes the [`MarkedPtr64`], returning only the separated raw
    /// pointer.
    #[inline]
    pub fn decompose_ptr(self) -> *mut T {
        (self.inner as usize & Self::POINTER_MASK) as *mut _
    }

    /// Decomposes the [`MarkedPtr64`], returning only the separated tag
    /// value.
    #[inline]
    pub fn decompose_tag(self) -> u16 {
        (self.inner as usize >> Self::TAG_SHIFT) as u16
    }

    /// Decomposes the [`MarkedPtr64`], returning an optional reference and the
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

/********** impl Default **************************************************************************/

impl<T> Default for MarkedPtr64<T> {
    #[inline]
    fn default() -> Self {
        Self::null()
    }
}

/********** impl From (*mut T) ********************************************************************/

impl<T> From<*mut T> for MarkedPtr64<T> {
    #[inline]
    fn from(ptr: *mut T) -> Self {
        Self::new(ptr)
    }
}

/********** impl From (*const T) ******************************************************************/

impl<T> From<*const T> for MarkedPtr64<T> {
    #[inline]
    fn from(ptr: *const T) -> Self {
        Self::new(ptr as *mut _)
    }
}

/********** impl From (&T) ************************************************************************/

impl<T> From<&T> for MarkedPtr64<T> {
    #[inline]
    fn from(reference: &T) -> Self {
        Self::from(reference as *const _)
    }
}

/********** impl From (&mut T) ********************************************************************/

impl<T> From<&mut T> for MarkedPtr64<T> {
    #[inline]
    fn from(reference: &mut T) -> Self {
        Self::new(reference)
    }
}

/********** impl From (NonNull<T>) ****************************************************************/

impl<T> From<NonNull<T>> for MarkedPtr64<T> {
    #[inline]
    fn from(non_null: NonNull<T>) -> Self {
        Self::new(non_null.as_ptr())
    }
}

/********** impl Debug ****************************************************************************/

impl<T> fmt::Debug for MarkedPtr64<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (ptr, tag) = self.decompose();
        f.debug_struct("MarkedNativePtr").field("ptr", &ptr).field("tag", &tag).finish()
    }
}

/********** impl Pointer **************************************************************************/

impl<T> fmt::Pointer for MarkedPtr64<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.decompose_ptr(), f)
    }
}

/********** impl PartialEq ************************************************************************/

impl<T> PartialEq for MarkedPtr64<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

/********** impl PartialOrd ***********************************************************************/

impl<T> PartialOrd for MarkedPtr64<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

/********** impl Eq *******************************************************************************/

impl<T> Eq for MarkedPtr64<T> {}

/********** impl Ord ******************************************************************************/

impl<T> Ord for MarkedPtr64<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

/********** impl Hash *****************************************************************************/

impl<T> Hash for MarkedPtr64<T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}