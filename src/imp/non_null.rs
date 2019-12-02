use core::cmp;
use core::convert::TryFrom;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ptr::NonNull;

use typenum::Unsigned;

use crate::{
    MarkedNonNull, MarkedNonNullable,
    MaybeNull::{self, Null, NotNull},
    MarkedPtr, NonNullable, NullError,
};

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedNonNull
////////////////////////////////////////////////////////////////////////////////////////////////////

/********** impl Clone ****************************************************************************/

impl<T, N> Clone for MarkedNonNull<T, N> {
    #[inline]
    fn clone(&self) -> Self {
        Self { inner: self.inner, _marker: PhantomData }
    }
}

/********** impl Copy *****************************************************************************/

impl<T, N> Copy for MarkedNonNull<T, N> {}

/********** impl inherent (const) *****************************************************************/

impl<T, N> MarkedNonNull<T, N> {
    /// Creates a new [`MarkedNonNull`] from a marked pointer without checking
    /// its validity.
    ///
    /// # Safety
    ///
    /// `ptr` may be marked, but must be be neither an unmarked nor a marked
    /// null pointer.
    /// In other words, the numeric representation of `ptr` must be greater
    /// than the smallest possible well-aligned pointer for type `T`.
    #[inline]
    pub const unsafe fn new_unchecked(ptr: MarkedPtr<T, N>) -> Self {
        Self { inner: NonNull::new_unchecked(ptr.inner), _marker: PhantomData }
    }

    /// Creates a new [`MarkedNonNull`] from a [`NonNull`] without checking its
    /// validity.
    #[inline]
    pub const unsafe fn from_non_null_unchecked(non_null: NonNull<T>) -> Self {
        Self { inner: non_null, _marker: PhantomData }
    }

    /// Cast to a pointer of another type.
    #[inline]
    pub const fn cast<U>(self) -> MarkedNonNull<U, N> {
        MarkedNonNull { inner: self.inner.cast(), _marker: PhantomData }
    }

    /// Returns the inner pointer *as is*, meaning any potential tag is not
    /// stripped.
    ///
    /// De-referencing the returned pointer results in undefined behaviour, if
    /// the pointer is still marked, even if the pointer itself points to a
    /// valid and live value.
    ///
    /// Use e.g. [`decompose`][MarkedNonNull::decompose] instead to get the
    /// actual pointer without the tag.
    #[inline]
    pub const fn into_non_null(self) -> NonNull<T> {
        self.inner
    }

    /// Converts the pointer to the equivalent [`MarkedPtr`].
    #[inline]
    pub const fn into_marked_ptr(self) -> MarkedPtr<T, N> {
        MarkedPtr::new(self.inner.as_ptr())
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

    /// Creates a new [`MarkedNonNull`] from a raw pointer.
    ///
    /// If `ptr` is `null`, [`None`] is returned and any potential mark bits
    /// are simply ignored.
    #[inline]
    pub fn try_from_ptr(ptr: *mut T) -> Option<MarkedNonNull<T, N>> {
        if !crate::decompose_ptr::<T>(ptr as usize, Self::MARK_BITS).is_null() {
            Some(unsafe { MarkedNonNull::new_unchecked(MarkedPtr::new(ptr)) })
        } else {
            None
        }
    }

    /// Creates a new [`MarkedNonNull`] from a [`MarkedPtr`].
    ///
    /// If `ptr` is `null`, [`None`] is returned and any potential mark bits
    /// are simply ignored.
    #[inline]
    pub fn try_from_marked_ptr(marked_ptr: MarkedPtr<T, N>) -> Option<Self> {
        Self::try_from(marked_ptr).ok()
    }

    /// Composes a new [`MarkedNonNull`] from a raw `ptr` and a `tag` value.
    ///
    /// For a fallible version of this function, see
    /// [`try_compose`][MarkedNonNull::try_compose].
    ///
    /// # Panics
    ///
    /// This function panics, if `ptr` is a actually `null` pointer if its first
    /// `N` bits are interpreted as tag.
    /// For instance, `0x1` would be a valid value for `NonNull`, even though
    /// de-referencing it would be undefined behaviour.
    /// The same value is not valid for `MarkedNonNull` with `N > 0`, however,
    /// since it would be interpreted as a `null` pointer with tag value of `1`.
    /// This is usually not a problem as long as `ptr` is well aligned, since
    /// the number of mark bits can not exceed `T`'s alignment.
    #[inline]
    pub fn compose(ptr: NonNull<T>, tag: usize) -> Self {
        Self::try_compose(ptr, tag)
            .expect("`ptr` is misaligned for `N` mark bits - could be interpreted as a marked `null` pointer.")
    }

    /// Attempts to compose a new [`MarkedNonNull`] from a raw `ptr` and a `tag`
    /// value.
    ///
    /// # Errors
    ///
    /// This function fails, if `ptr` is a actually `null` pointer if its first
    /// `N` bits are interpreted as tag.
    /// For instance, `0x1` would be a valid value for `NonNull`, even though
    /// de-referencing it would be undefined behaviour.
    /// The same value is not valid for `MarkedNonNull` with `N > 0`, however,
    /// since it would be interpreted as a `null` pointer with tag value of `1`.
    /// This is usually not a problem as long as `ptr` is well aligned, since
    /// the number of mark bits can not exceed `T`'s alignment.
    #[inline]
    pub fn try_compose(ptr: NonNull<T>, tag: usize) -> Result<Self, NullError> {
        if !crate::decompose_ptr::<T>(ptr.as_ptr() as usize, Self::MARK_BITS).is_null() {
            Ok(unsafe { Self::compose_unchecked(ptr, tag) })
        } else {
            Err(NullError)
        }
    }

    /// Composes a new [`MarkedNonNull`] from a raw `ptr` and a `tag` value
    /// without checking if `ptr` is actually a `null` pointer if it is first
    /// `N` bits are interpreted as tag.
    ///
    /// # Safety
    ///
    /// The caller has to ensure, that `ptr` is not a marked `null` pointer for
    /// `N` mark bits.
    #[inline]
    pub unsafe fn compose_unchecked(ptr: NonNull<T>, tag: usize) -> Self {
        Self::new_unchecked(MarkedPtr::compose(ptr.as_ptr(), tag))
    }

    /// Clears the tag from `self` and returns the same pointer but stripped of
    /// its mark bits.
    #[inline]
    pub fn clear_tag(self) -> Self {
        let clear = crate::decompose_ptr::<T>(self.inner.as_ptr() as usize, Self::MARK_BITS);
        Self { inner: unsafe { NonNull::new_unchecked(clear) }, _marker: PhantomData }
    }

    /// TODO: docs..
    #[inline]
    pub fn split_tag(self) -> (Self, usize) {
        let (inner, tag) = self.decompose();
        (Self { inner, _marker: PhantomData }, tag)
    }

    /// Clears the tag from `self` and replaces it with `tag`.
    #[inline]
    pub fn set_tag(self, tag: usize) -> Self {
        Self::compose(self.decompose_non_null(), tag)
    }

    #[inline]
    pub unsafe fn add_tag(self, value: usize) -> Self {
        Self::new_unchecked(MarkedPtr::from_usize(self.inner.as_ptr() as usize + value))
    }

    #[inline]
    pub fn wrapping_add_tag(self, value: usize) -> Self {
        let (ptr, tag) = self.decompose();
        unsafe { Self::compose(ptr, tag + value) }
    }

    #[inline]
    pub unsafe fn sub_tag(self, value: usize) -> Self {
        Self::new_unchecked(MarkedPtr::from_usize(self.inner.as_ptr() as usize - value))
    }

    #[inline]
    pub fn wrapping_sub_tag(self, value: usize) -> Self {
        let (ptr, tag) = self.decompose();
        unsafe { Self::compose(ptr, tag - value) }
    }

    /// Decomposes the [`MarkedNonNull`], returning the separated raw
    /// [`NonNull`] pointer and its tag.
    #[inline]
    pub fn decompose(self) -> (NonNull<T>, usize) {
        (self.decompose_non_null(), self.decompose_tag())
    }

    /// Decomposes the marked pointer, returning only the separated raw pointer.
    #[inline]
    pub fn decompose_ptr(self) -> *mut T {
        crate::decompose_ptr(self.inner.as_ptr() as usize, Self::MARK_BITS)
    }

    /// Decomposes the marked pointer, returning only the separated raw
    /// [`NonNull`] pointer.
    #[inline]
    pub fn decompose_non_null(self) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(self.decompose_ptr()) }
    }

    /// Decomposes the marked pointer, returning only the separated tag.
    #[inline]
    pub fn decompose_tag(self) -> usize {
        crate::decompose_tag::<T>(self.inner.as_ptr() as usize, Self::MARK_BITS)
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
    #[allow(clippy::trivially_copy_pass_by_ref)]
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
    /// returned reference is not bound to the lifetime of the
    /// [`MarkedNonNull`].
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
    /// or [`as_mut_unbounded`][MarkedNonNull::as_mut_unbounded].
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
        f.debug_struct("MarkedNonNull").field("ptr", &ptr).field("tag", &tag).finish()
    }
}

/********** impl MarkedNonNullable ****************************************************************/

impl<T, N: Unsigned> MarkedNonNullable for MarkedNonNull<T, N> {
    type MarkBits = N;

    #[inline]
    fn as_marked_ptr(arg: &Self) -> MarkedPtr<Self::Item, Self::MarkBits> {
        arg.into_marked_ptr()
    }

    #[inline]
    fn into_marked_ptr(arg: Self) -> MarkedPtr<Self::Item, Self::MarkBits> {
        arg.into_marked_ptr()
    }

    #[inline]
    fn as_marked_non_null(arg: &Self) -> MarkedNonNull<Self::Item, Self::MarkBits> {
        *arg
    }

    #[inline]
    fn into_marked_non_null(arg: Self) -> MarkedNonNull<Self::Item, Self::MarkBits> {
        arg
    }

    #[inline]
    fn clear_tag(arg: Self) -> Self {
        arg.clear_tag()
    }

    #[inline]
    fn set_tag(arg: Self, tag: usize) -> Self {
        arg.set_tag(tag)
    }

    #[inline]
    fn decompose(arg: Self) -> (Self, usize) {
        let tag = arg.decompose_tag();
        (arg.clear_tag(), tag)
    }

    #[inline]
    fn decompose_ptr(arg: &Self) -> *mut Self::Item {
        arg.decompose_ptr()
    }

    #[inline]
    fn decompose_non_null(arg: &Self) -> NonNull<Self::Item> {
        arg.decompose_non_null()
    }

    #[inline]
    fn decompose_tag(arg: &Self) -> usize {
        arg.decompose_tag()
    }
}

/********** impl NonNullable **********************************************************************/

impl<T, N: Unsigned> NonNullable for MarkedNonNull<T, N> {
    type Item = T;

    #[inline]
    fn as_const_ptr(arg: &Self) -> *const Self::Item {
        arg.decompose_ptr() as *const _
    }

    #[inline]
    fn as_mut_ptr(arg: &Self) -> *mut Self::Item {
        arg.decompose_ptr()
    }

    #[inline]
    fn as_non_null(arg: &Self) -> NonNull<Self::Item> {
        arg.decompose_non_null()
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

impl<'a, T, N> From<&'a T> for MarkedNonNull<T, N> {
    #[inline]
    fn from(reference: &T) -> Self {
        Self { inner: NonNull::from(reference), _marker: PhantomData }
    }
}

impl<'a, T, N> From<&'a mut T> for MarkedNonNull<T, N> {
    #[inline]
    fn from(reference: &mut T) -> Self {
        Self { inner: NonNull::from(reference), _marker: PhantomData }
    }
}

/********** impl TryFrom **************************************************************************/

impl<T, N: Unsigned> TryFrom<MarkedPtr<T, N>> for MarkedNonNull<T, N> {
    type Error = NullError;

    #[inline]
    fn try_from(marked_ptr: MarkedPtr<T, N>) -> Result<Self, Self::Error> {
        match MaybeNull::from(marked_ptr) {
            NotNull(ptr) => Ok(ptr),
            Null(_) => Err(NullError),
        }
    }
}

impl<T, N: Unsigned> TryFrom<NonNull<T>> for MarkedNonNull<T, N> {
    type Error = NullError;

    #[inline]
    fn try_from(non_null: NonNull<T>) -> Result<Self, Self::Error> {
        if non_null.as_ptr() as usize & Self::POINTER_MASK == 0 {
            Err(NullError)
        } else {
            Ok(Self { inner: non_null, _marker: PhantomData })
        }
    }
}

/********** impl PartialEq ************************************************************************/

impl<T, N> PartialEq for MarkedNonNull<T, N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

/********** impl PartialOrd ***********************************************************************/

impl<T, N> PartialOrd for MarkedNonNull<T, N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

/********** impl Eq *******************************************************************************/

impl<T, N> Eq for MarkedNonNull<T, N> {}

/********** impl Ord ******************************************************************************/

impl<T, N> Ord for MarkedNonNull<T, N> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

/********** impl Hash *****************************************************************************/

impl<T, N> Hash for MarkedNonNull<T, N> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}
