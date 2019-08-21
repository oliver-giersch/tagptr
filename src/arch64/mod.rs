//! Native marked pointers without alignment requirements exploiting the
//! property of current 64-bit architectures, which only use 48-bit virtual
//! addresses.
//! This leaves the upper 16-bit of any 64-bit pointer available for storing
//! additional tag bits.

#[cfg(all(target_arch = "x86_64", feature = "nightly"))]
mod dwcas;

use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::AtomicUsize;

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
/// virtual addresses.
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
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedNativePtr
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A marked native (64-bit) pointer of which the upper 16 bits can be used for
/// storing additional information.
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

    /// Decomposes the marked pointer, returning an optional reference and the
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

    /// Decomposes the marked pointer returning an optional mutable reference
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

    /// Decomposes the marked pointer, returning an optional reference and
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

    /// Decomposes the marked pointer, returning an optional mutable reference
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
