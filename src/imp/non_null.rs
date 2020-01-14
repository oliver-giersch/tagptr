use core::cmp;
use core::convert::TryFrom;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::mem;
use core::ptr::NonNull;

use crate::{MarkedNonNull, MarkedPtr, Null};

/********** impl Clone ****************************************************************************/

impl<T, const N: usize> Clone for MarkedNonNull<T, N> {
    #[inline]
    fn clone(&self) -> Self {
        Self { inner: self.inner }
    }
}

/********** impl Copy *****************************************************************************/

impl<T, const N: usize> Copy for MarkedNonNull<T, N> {}

/********** impl inherent *************************************************************************/

impl<T, const N: usize> MarkedNonNull<T, N> {
    /// The number of available mark bits for this type.
    pub const MARK_BITS: usize = N;
    /// The bitmask for the lower markable bits.
    pub const MARK_MASK: usize = crate::mark_mask::<T>(Self::MARK_BITS);
    /// The bitmask for the (higher) pointer bits.
    pub const POINTER_MASK: usize = !Self::MARK_MASK;

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
        Self { inner: NonNull::new_unchecked(ptr.inner) }
    }

    /// Creates a new [`MarkedNonNull`] from a [`NonNull`] without checking its
    /// validity.
    #[inline]
    pub const unsafe fn from_non_null_unchecked(non_null: NonNull<T>) -> Self {
        Self { inner: non_null }
    }

    #[inline]
    pub fn dangling() -> Self {
        let ptr = cmp::max(Self::MARK_MASK + 1, mem::align_of::<T>()) as *mut _;
        Self { inner: unsafe { NonNull::new_unchecked(ptr) } }
    }

    /// Creates a new [`MarkedNonNull`] from a [`MarkedPtr`] and fails, if
    /// `marked_ptr` is `null`.
    ///
    /// # Errors
    ///
    /// This methods returns an error  containing the tag value of the supplied
    /// pointer if it is `null`
    #[inline]
    pub fn new(marked_ptr: MarkedPtr<T, N>) -> Result<Self, Null> {
        match marked_ptr.decompose() {
            (ptr, _) if !ptr.is_null() => Ok(unsafe { Self::new_unchecked(marked_ptr) }),
            (_, tag) => Err(Null(tag)),
        }
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
    pub fn try_compose(ptr: NonNull<T>, tag: usize) -> Result<Self, Null> {
        match ptr.as_ptr() as usize & Self::POINTER_MASK {
            0 => Ok(unsafe { Self::compose_unchecked(ptr, tag) }),
            _ => Err(Null(ptr.as_ptr() as usize)),
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

    /// Cast to a pointer of another type.
    #[inline]
    pub fn cast<U>(self) -> MarkedNonNull<U, N> {
        crate::assert_alignment::<U, N>();
        MarkedNonNull { inner: self.inner.cast() }
    }

    #[inline]
    pub fn into_usize(self) -> usize {
        self.inner.as_ptr() as usize
    }

    /// Clears the tag from `self` and returns the same pointer stripped of its
    /// tag value.
    #[inline]
    pub fn clear_tag(self) -> Self {
        let clear = crate::decompose_ptr::<T>(self.inner.as_ptr() as usize, Self::MARK_BITS);
        Self { inner: unsafe { NonNull::new_unchecked(clear) } }
    }

    /// Splits the tag from `self` and returns the same pointer stripped of its
    /// tag value and the separated tag.
    #[inline]
    pub fn split_tag(self) -> (Self, usize) {
        let (inner, tag) = self.decompose();
        (Self { inner }, tag)
    }

    /// Clears the tag value from `self` and replaces it with `tag`.
    #[inline]
    pub fn set_tag(self, tag: usize) -> Self {
        Self::compose(self.decompose_non_null(), tag)
    }

    /// Updates the tag value of `self` using `func`, which receives the current
    /// tag value as argument and returns the same pointer with the updated tag
    /// value.
    #[inline]
    pub fn update_tag(self, func: impl FnOnce(usize) -> usize) -> Self {
        let (ptr, tag) = self.decompose();
        unsafe { Self::compose_unchecked(ptr, func(tag)) }
    }

    #[inline]
    pub unsafe fn add_tag(self, value: usize) -> Self {
        Self::new_unchecked(MarkedPtr::from_usize(self.inner.as_ptr() as usize + value))
    }

    #[inline]
    pub unsafe fn sub_tag(self, value: usize) -> Self {
        Self::new_unchecked(MarkedPtr::from_usize(self.inner.as_ptr() as usize - value))
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

impl<T, const N: usize> fmt::Debug for MarkedNonNull<T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (ptr, tag) = self.decompose();
        f.debug_struct("MarkedNonNull").field("ptr", &ptr).field("tag", &tag).finish()
    }
}

/********** impl Pointer **************************************************************************/

impl<T, const N: usize> fmt::Pointer for MarkedNonNull<T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.decompose_non_null(), f)
    }
}

/********** impl From (&T) ************************************************************************/

impl<T, const N: usize> From<&T> for MarkedNonNull<T, N> {
    #[inline]
    fn from(reference: &T) -> Self {
        Self { inner: NonNull::from(reference) }
    }
}

/********** impl From (&mut T) ********************************************************************/

impl<T, const N: usize> From<&mut T> for MarkedNonNull<T, N> {
    #[inline]
    fn from(reference: &mut T) -> Self {
        Self { inner: NonNull::from(reference) }
    }
}

/********** impl TryFrom (*mut T) *****************************************************************/

impl<T, const N: usize> TryFrom<*mut T> for MarkedNonNull<T, N> {
    type Error = Null;

    #[inline]
    fn try_from(ptr: *mut T) -> Result<Self, Self::Error> {
        if ptr as usize & Self::POINTER_MASK == 0 {
            Err(Null(ptr as usize))
        } else {
            Ok(Self { inner: unsafe { NonNull::new_unchecked(ptr) } })
        }
    }
}

/********** impl TryFrom (NonNull) ****************************************************************/

impl<T, const N: usize> TryFrom<NonNull<T>> for MarkedNonNull<T, N> {
    type Error = Null;

    #[inline]
    fn try_from(non_null: NonNull<T>) -> Result<Self, Self::Error> {
        Self::try_from(non_null.as_ptr())
    }
}

/********** impl TryFrom (MarkedPtr) **************************************************************/

impl<T, const N: usize> TryFrom<MarkedPtr<T, N>> for MarkedNonNull<T, N> {
    type Error = Null;

    #[inline]
    fn try_from(marked_ptr: MarkedPtr<T, N>) -> Result<Self, Self::Error> {
        Self::new(marked_ptr)
    }
}

/********** impl PartialEq ************************************************************************/

impl<T, const N: usize> PartialEq for MarkedNonNull<T, N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

/********** impl PartialOrd ***********************************************************************/

impl<T, const N: usize> PartialOrd for MarkedNonNull<T, N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

/********** impl Eq *******************************************************************************/

impl<T, const N: usize> Eq for MarkedNonNull<T, N> {}

/********** impl Ord ******************************************************************************/

impl<T, const N: usize> Ord for MarkedNonNull<T, N> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

/********** impl Hash *****************************************************************************/

impl<T, const N: usize> Hash for MarkedNonNull<T, N> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}
