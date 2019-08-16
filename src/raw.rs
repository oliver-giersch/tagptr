//! Trait and inherent impls for the [`MarkedPtr`] type.

use core::cmp;
use core::convert::TryFrom;
use core::fmt;
use core::marker::PhantomData;
use core::ptr::{self, NonNull};

use typenum::Unsigned;

use crate::{MarkedPtr, MarkedNonNull, MarkedOption::{self, Null, Value}};

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedPtr
////////////////////////////////////////////////////////////////////////////////////////////////////

/********** impl Clone ****************************************************************************/

impl<T, N> Clone for MarkedPtr<T, N> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

/********** impl Copy *****************************************************************************/

impl<T, N> Copy for MarkedPtr<T, N> {}

/********** impl Default ***************************************************************************/

impl<T, N> Default for MarkedPtr<T, N> {
    #[inline]
    fn default() -> Self {
        Self::null()
    }
}

/********** impl inherent (const) *****************************************************************/

impl<T, N> MarkedPtr<T, N> {
    /// Creates a new unmarked [`MarkedPointer`].
    #[inline]
    pub const fn new(ptr: *mut T) -> Self {
        Self {
            ptr,
            _marker: PhantomData
        }
    }

    /// Creates a new unmarked `null` pointer.
    #[inline]
    pub const fn null() -> Self {
        Self::new(ptr::null_mut())
    }

    /// Casts to a pointer of type `U`.
    #[inline]
    pub const fn cast<U>(self) -> MarkedPtr<U, N> {
        MarkedPtr { ptr: self.ptr.cast(), _marker: PhantomData }
    }

    /// Creates a [`MarkedPtr`] from the integer (numeric) representation of a
    /// potentially marked pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    ///
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, typenum::U1>;
    ///
    /// let ptr = MarkedPtr::from_usize(1);
    /// assert_eq!(ptr.decompose(), (ptr::null_mut(), 1));
    /// ```
    #[inline]
    pub const fn from_usize(val: usize) -> Self {
        Self { ptr: val as *mut _, _marker: PhantomData }
    }

    /// Returns the inner pointer *as is*, meaning any potential tag is not
    /// stripped.
    ///
    /// De-referencing the returned pointer results in undefined behaviour, if
    /// the pointer is still marked and even if the pointer itself points to a
    /// valid and live value.
    #[inline]
    pub const fn into_ptr(self) -> *mut T {
        self.ptr
    }
}

/********** impl inherent *************************************************************************/

impl<T, N: Unsigned> MarkedPtr<T, N> {
    const MARK_BITS: usize = N::USIZE;

    /// Composes a new [`MarkedPtr`] from a raw `ptr` and a `tag` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    ///
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, typenum::U2>;
    ///
    /// let raw = &1 as *const i32 as *mut i32;
    /// let ptr = MarkedPtr::compose(raw, 0b11);
    /// assert_eq!(ptr.decompose(), (raw, 0b11));
    /// ```
    #[inline]
    pub fn compose(ptr: *mut T, tag: usize) -> Self {
        Self::new(crate::compose(Self::MARK_BITS, ptr, tag))
    }

    /// Returns the integer representation of the pointer with its tag.
    #[inline]
    pub fn into_usize(self) -> usize {
        self.ptr as usize
    }

    /// Clears the tag from `self` and returns the same but unmarked pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    ///
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, typenum::U2>;
    ///
    /// let raw = &1 as *const i32 as *mut i32;
    /// let ptr = MarkedPtr::compose(raw, 0b11);
    /// assert_eq!(ptr.clear_tag().decompose(), (raw, 0));
    /// ```
    #[inline]
    pub fn clear_tag(self) -> Self {
        Self::new(self.decompose_ptr())
    }

    /// Clears the tag from `self` and replaces it with `tag`.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    ///
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, typenum::U2>;
    ///
    /// let raw = &1 as *const i32 as *mut i32;
    /// let ptr = MarkedPtr::compose(raw, 0b11);
    /// assert_eq!(ptr.with_tag(0b10).decompose(), (raw, 0b10));
    /// ```
    #[inline]
    pub fn with_tag(self, tag: usize) -> Self {
        Self::compose(self.decompose_ptr(), tag)
    }

    /// Decomposes the [`MarkedPtr`], returning the separated raw pointer and
    /// its tag.
    #[inline]
    pub fn decompose(self) -> (*mut T, usize) {
        crate::decompose::<T>(self.ptr as usize, Self::MARK_BITS)
    }

    /// Decomposes the [`MarkedPtr`], returning only the separated raw pointer.
    #[inline]
    pub fn decompose_ptr(self) -> *mut T {
        crate::decompose_ptr::<T>(self.ptr as usize, Self::MARK_BITS)
    }

    /// Decomposes the [`MarkedPtr`], returning only the separated tag value.
    #[inline]
    pub fn decompose_tag(self) -> usize {
        crate::decompose_tag::<T>(self.ptr as usize, Self::MARK_BITS)
    }

    #[inline]
    pub unsafe fn decompose_ref<'a>(self) -> (Option<&'a T>, usize) {
        (self.as_ref(), self.decompose_tag())
    }

    #[inline]
    pub unsafe fn decompose_mut<'a>(self) -> (Option<&'a mut T>, usize) {
        (self.as_mut(), self.decompose_tag())
    }

    #[inline]
    pub unsafe fn as_ref<'a>(self) -> Option<&'a T> {
        self.decompose_ptr().as_ref()
    }

    #[inline]
    pub unsafe fn as_mut<'a>(self) -> Option<&'a mut T> {
        self.decompose_ptr().as_mut()
    }
}

/********** impl From *****************************************************************************/

impl<T, N> From<*mut T> for MarkedPtr<T, N> {
    #[inline]
    fn from(ptr: *mut T) -> Self {
        Self::new(ptr)
    }
}

impl<T, N> From<*const T> for MarkedPtr<T, N> {
    #[inline]
    fn from(ptr: *const T) -> Self {
        Self::new(ptr as *mut _)
    }
}

/********** impl Debug ****************************************************************************/

impl<T, N: Unsigned> fmt::Debug for MarkedPtr<T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MarkedPtr")
            .field("ptr", &self.decompose_ptr())
            .field("tag", &self.decompose_tag())
            .finish()
    }
}

/********** impl Pointer **************************************************************************/

impl<T, N: Unsigned> fmt::Pointer for MarkedPtr<T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.decompose_ptr(), f)
    }
}

/********** impl PartialEq ************************************************************************/

impl<T, N> PartialEq for MarkedPtr<T, N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.ptr.eq(&other.ptr)
    }
}

/********** impl PartialOrd ***********************************************************************/

impl<T, N> PartialOrd for MarkedPtr<T, N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.ptr.partial_cmp(&other.ptr)
    }
}

/********** impl Eq *******************************************************************************/

impl<T, N> Eq for MarkedPtr<T, N> {}

/********** impl Ord ******************************************************************************/

impl<T, N> Ord for MarkedPtr<T, N> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.ptr.cmp(&other.ptr)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedNonNull
////////////////////////////////////////////////////////////////////////////////////////////////////

/********** impl Clone ****************************************************************************/

impl<T, N> Clone for MarkedNonNull<T, N> {
    #[inline]
    fn clone(&self) -> Self {
        Self { ptr: self.ptr, _marker: PhantomData }
    }
}

/********** impl Copy *****************************************************************************/

impl<T, N> Copy for MarkedNonNull<T, N> {}

/********** impl inherent (const) *****************************************************************/

impl<T, N> MarkedNonNull<T, N> {
    /// Creates a new [`MarkedNonNull`] that is dangling, but well-aligned.
    ///
    /// This is useful for initializing types which lazily allocate, like
    /// `Vec::new` does.
    ///
    /// Note that the pointer value may potentially represent a valid pointer to
    /// a `T`, which means this must not be used as a "not yet initialized"
    /// sentinel value. Types that lazily allocate must track initialization by
    /// some other means.
    #[inline]
    pub const fn dangling() -> Self {
        Self { ptr: NonNull::dangling(), _marker: PhantomData }
    }

    /// Creates a new [`MarkedNonNull`] from a marked pointer without checking
    /// for `null`.
    ///
    /// # Safety
    ///
    /// `ptr` may be marked, but must be be neither an unmarked nor a marked
    /// null pointer.
    /// In other words, the numeric representation of `ptr` must be greater
    /// than the smallest possible well-aligned pointer for type `T`.
    #[inline]
    pub const unsafe fn new_unchecked(ptr: MarkedPtr<T, N>) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr.ptr),
            _marker: PhantomData,
        }
    }

    /// Cast to a pointer of another type.
    #[inline]
    pub const fn cast<U>(self) -> MarkedNonNull<U, N> {
        MarkedNonNull { ptr: self.ptr.cast(), _marker: PhantomData }
    }

    /// Returns the inner pointer *as is*, meaning any potential tag is not
    /// stripped.
    ///
    /// De-referencing the returned pointer results in undefined behaviour, if
    /// the pointer is still marked and even if the pointer itself points to a
    /// valid and live value.
    ///
    /// Use e.g. [`decompose`][MarkedNonNull::decompose] instead to get the
    /// actual pointer without the tag.
    #[inline]
    pub const fn into_non_null(self) -> NonNull<T> {
        self.ptr
    }

    /// Converts the pointer to the equivalent [`MarkedPtr`].
    #[inline]
    pub const fn into_marked_ptr(self) -> MarkedPtr<T, N> {
        MarkedPtr::new(self.ptr.as_ptr())
    }
}

/********** impl inherent *************************************************************************/

impl<T, N: Unsigned> MarkedNonNull<T, N> {
    /// The number of available mark bits for this type.
    pub const MARK_BITS: usize = N::USIZE;
    /// The bitmask for the lower markable bits.
    pub const MARK_MASK: usize = crate::mark_mask::<T>(Self::MARK_BITS);
    /// The bitmask for the (higher) pointer bits.
    pub const POINTER_MASK: usize = !Self::MARK_MASK;

    #[inline]
    pub fn new(ptr: *mut T) -> Option<MarkedNonNull<T, N>> {
        NonNull::new(ptr).map(Self::from)
    }

    #[inline]
    pub fn from_marked_ptr(marked_ptr: MarkedPtr<T, N>) -> MarkedOption<Self> {
        match marked_ptr.decompose() {
            (ptr, _) if !ptr.is_null() => Value(unsafe { Self::new_unchecked(marked_ptr) }),
            (_, tag) => Null(tag),
        }
    }

    #[inline]
    pub fn compose(ptr: NonNull<T>, tag: usize) -> Self {
        assert_eq!(ptr.as_ptr() as usize & Self::MARK_MASK, 0, "`ptr` is not well aligned");
        unsafe {
            Self::from(NonNull::new_unchecked(crate::compose(Self::MARK_BITS, ptr.as_ptr(), tag)))
        }
    }

    #[inline]
    pub fn decompose(self) -> (NonNull<T>, usize) {
        (self.decompose_non_null(), self.decompose_tag())
    }

    #[inline]
    pub fn decompose_ptr(self) -> *mut T {
        crate::decompose_ptr(self.ptr.as_ptr() as usize, Self::MARK_BITS)
    }

    #[inline]
    pub fn decompose_non_null(self) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(self.decompose_ptr()) }
    }

    #[inline]
    pub fn decompose_tag(self) -> usize {
        crate::decompose_tag::<T>(self.ptr.as_ptr() as usize, Self::MARK_BITS)
    }

    /// Decomposes the pointer, dereferences the the raw pointer and returns
    /// both the reference and the separated tag.
    ///
    /// The resulting lifetime is bound to self so this behaves "as if"
    /// it were actually an instance of T that is getting borrowed. If a longer
    /// (unbound) lifetime is needed, use e.g.
    /// [`decompose_ref_unbounded`][MarkedNonNull::decompose_ref_unbounded]
    /// or `&*my_ptr.decompose_ptr()`.
    ///
    /// # Safety
    ///
    /// This is unsafe because it cannot verify the validity of the returned
    /// pointer.
    #[inline]
    pub unsafe fn decompose_ref(&self) -> (&T, usize) {
        let (ptr, tag) = self.decompose();
        (&*ptr.as_ptr(), tag)
    }

    /// Decomposes the pointer, dereferences the the raw pointer and returns
    /// both the reference and the separated tag. The returned reference is not
    /// bound to the lifetime of the [`MarkedNonNull`].
    ///
    /// # Safety
    ///
    /// This is unsafe because it cannot verify the validity of the returned
    /// pointer, nor can it ensure that the lifetime `'a` returned is indeed a
    /// valid lifetime for the contained data.
    #[inline]
    pub unsafe fn decompose_ref_unbounded<'a>(self) -> (&'a T, usize) {
        let (ptr, tag) = self.decompose();
        (&*ptr.as_ptr(), tag)
    }

    /// Decomposes the pointer, *mutably* dereferences the the raw pointer and
    /// returns both the mutable reference and the separated tag.
    ///
    /// The resulting lifetime is bound to self so this behaves "as if"
    /// it were actually an instance of T that is getting borrowed. If a longer
    /// (unbound) lifetime is needed, use e.g.
    /// [`decompose_mut_unbounded`][MarkedNonNull::decompose_mut_unbounded] or
    /// `&mut *my_ptr.decompose_ptr()`.
    ///
    /// # Safety
    ///
    /// This is unsafe because it cannot verify the validity of the returned
    /// pointer.
    #[inline]
    pub unsafe fn decompose_mut(&mut self) -> (&mut T, usize) {
        let (ptr, tag) = self.decompose();
        (&mut *ptr.as_ptr(), tag)
    }

    /// Decomposes the marked pointer, mutably dereferences the the raw pointer
    /// and returns both the mutable reference and the separated tag. The
    /// returned reference is not bound to the lifetime of the `MarkedNonNull`.
    ///
    /// # Safety
    ///
    /// This is unsafe because it cannot verify the validity of the returned
    /// pointer, nor can it ensure that the lifetime `'a` returned is indeed a
    /// valid lifetime for the contained data.
    #[inline]
    pub unsafe fn decompose_mut_unbounded<'a>(&mut self) -> (&'a mut T, usize) {
        let (ptr, tag) = self.decompose();
        (&mut *ptr.as_ptr(), tag)
    }

    /// Decomposes the marked pointer, returning only the de-referenced raw
    /// pointer.
    ///
    /// The resulting lifetime is bound to self so this behaves "as if" it were
    /// actually an instance of T that is getting borrowed. If a longer
    /// (unbound) lifetime is needed, use e.g. `&*my_ptr.decompose_ptr()`
    /// or [`as_ref_unbounded`][MarkedNonNull::as_ref_unbounded].
    ///
    /// # Safety
    ///
    /// This is unsafe because it cannot verify the validity of the returned
    /// pointer.
    #[inline]
    pub unsafe fn as_ref(&self) -> &T {
        &*self.decompose_non_null().as_ptr()
    }

    /// Decomposes the marked pointer, returning only the de-referenced raw
    /// pointer, which is not bound to the lifetime of the `MarkedNonNull`.
    ///
    /// # Safety
    ///
    /// This is unsafe because it cannot verify the validity of the returned
    /// pointer, nor can it ensure that the lifetime `'a` returned is indeed a
    /// valid lifetime for the contained data.
    #[inline]
    pub unsafe fn as_ref_unbounded<'a>(self) -> &'a T {
        &*self.decompose_non_null().as_ptr()
    }

    /// Decomposes the marked pointer, returning only the mutably de-referenced
    /// raw pointer.
    ///
    /// The resulting lifetime is bound to self so this behaves "as if"
    /// it were actually an instance of T that is getting borrowed. If a longer
    /// (unbound) lifetime is needed, use e.g. `&mut *my_ptr.decompose_ptr()`
    /// or [`as_mut_unbounded`][MarkedNonNull::as_ref_unbounded].
    ///
    /// # Safety
    ///
    /// This is unsafe because it cannot verify the validity of the returned
    /// pointer.
    #[inline]
    pub unsafe fn as_mut(&mut self) -> &mut T {
        &mut *self.decompose_non_null().as_ptr()
    }

    /// Decomposes the marked pointer, returning only the mutably de-referenced
    /// raw pointer, which is not bound to the lifetime of the `MarkedNonNull`.
    ///
    /// # Safety
    ///
    /// This is unsafe because it cannot verify the validity of the returned
    /// pointer, nor can it ensure that the lifetime `'a` returned is indeed a
    /// valid lifetime for the contained data.
    #[inline]
    pub unsafe fn as_mut_unbounded<'a>(self) -> &'a mut T {
        &mut *self.decompose_non_null().as_ptr()
    }
}

/********** impl Debug ****************************************************************************/

impl<T, N: Unsigned> fmt::Debug for MarkedNonNull<T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (ptr, tag) = self.decompose();
        f.debug_struct("MarkedNonNull")
            .field("ptr", &ptr)
            .field("tag", &tag)
            .finish()
    }
}

/********** impl Pointer **************************************************************************/

impl<T, N: Unsigned> fmt::Pointer for MarkedNonNull<T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.decompose_non_null(), f)
    }
}

/********** impl From *****************************************************************************/

impl<T, N> From<NonNull<T>> for MarkedNonNull<T, N> {
    #[inline]
    fn from(ptr: NonNull<T>) -> Self {
        Self { ptr, _marker: PhantomData }
    }
}

impl<'a, T, N> From<&'a T> for MarkedNonNull<T, N> {
    #[inline]
    fn from(reference: &T) -> Self {
        Self { ptr: NonNull::from(reference), _marker: PhantomData }
    }
}

impl<'a, T, N> From<&'a mut T> for MarkedNonNull<T, N> {
    #[inline]
    fn from(reference: &mut T) -> Self {
        Self { ptr: NonNull::from(reference), _marker: PhantomData }
    }
}

/********** impl TryFrom **************************************************************************/

impl<T, N: Unsigned> TryFrom<MarkedPtr<T, N>> for MarkedNonNull<T, N> {
    type Error = NullError;

    #[inline]
    fn try_from(marked_ptr: MarkedPtr<T, N>) -> Result<Self, Self::Error> {
        match Self::from_marked_ptr(marked_ptr) {
            Value(ptr) => Ok(ptr),
            Null(_) => Err(NullError(())),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// NullError
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NullError(());
