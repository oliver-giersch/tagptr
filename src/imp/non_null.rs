use core::cmp;
use core::convert::TryFrom;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;

use crate::{MarkedNonNull, MarkedPtr, Null};

/********** impl Clone ****************************************************************************/

impl<T, const N: usize> Clone for MarkedNonNull<T, N> {
    impl_clone!();
}

/********** impl Copy *****************************************************************************/

impl<T, const N: usize> Copy for MarkedNonNull<T, N> {}

/********** impl inherent *************************************************************************/

impl<T, const N: usize> MarkedNonNull<T, N> {
    doc_comment! {
        doc_tag_bits!(),
        pub const TAG_BITS: usize = N;
    }

    doc_comment! {
        doc_tag_mask!(),
        pub const TAG_MASK: usize = crate::mark_mask::<T>(Self::TAG_BITS);
    }

    doc_comment! {
        doc_ptr_mask!(),
        pub const POINTER_MASK: usize = !Self::TAG_MASK;
    }

    const COMPOSE_ERR_MSG: &'static str =
        "argument `ptr` is mis-aligned for `N` tag bits and could be parsed as marked `null` \
        pointer.";

    /// Creates a new marked non-null pointer from `marked_ptr` without
    /// checking if it is `null`.
    ///
    /// # Safety
    ///
    /// The caller has to ensure that `marked_ptr` is not `null`.
    /// This includes `null` pointers with non-zero tag values.
    #[inline]
    pub const unsafe fn new_unchecked(marked_ptr: MarkedPtr<T, N>) -> Self {
        Self { inner: NonNull::new_unchecked(marked_ptr.inner), _marker: PhantomData }
    }

    doc_comment! {
        doc_from_usize!(),
        #[inline]
        pub const unsafe fn from_usize(val: usize) -> Self {
            Self { inner: NonNull::new_unchecked(val as *mut _), _marker: PhantomData }
        }
    }

    doc_comment! {
        doc_into_raw!(),
        #[inline]
        pub const fn into_raw(self) -> NonNull<T> {
            self.inner
        }
    }

    doc_comment! {
        doc_cast!(),
        pub const fn cast<U>(self) -> MarkedNonNull<U, N> {
            MarkedNonNull { inner: self.inner.cast(), _marker: PhantomData }
        }
    }

    doc_comment! {
        doc_into_usize!(),
        #[inline]
        pub fn into_usize(self) -> usize {
            self.inner.as_ptr() as _
        }
    }

    /// Converts `self` into a (nullable) marked pointer.
    #[inline]
    pub const fn into_marked_ptr(self) -> MarkedPtr<T, N> {
        MarkedPtr::new(self.inner.as_ptr())
    }

    /// Creates a new non-null pointer from `marked_ptr`.
    ///
    /// # Errors
    ///
    /// Fails if `marked_ptr` is `null`, in which case a [`Null`] instance is
    /// returned containing argument pointer's tag value.
    #[inline]
    pub fn new(marked_ptr: MarkedPtr<T, N>) -> Result<Self, Null> {
        Self::try_from(marked_ptr)
    }

    /// Creates a new pointer that is dangling but well aligned.
    #[inline]
    pub fn dangling() -> Self {
        // TODO: could be const fn with const generics
        // safety: a type's alignment is never zero, so the result of max is always non-zero
        unsafe { Self::from_usize(cmp::max(mem::align_of::<T>(), Self::TAG_MASK + 1)) }
    }

    doc_comment! {
        doc_compose!(),
        /// # Panics
        ///
        /// Panics if `ptr` is mis-aligned for `N` tag bits and contains only
        /// zero bits in the upper bits, i.e., it would be parsed as a marked
        /// `null` pointer.
        #[inline]
        pub fn compose(ptr: NonNull<T>, tag: usize) -> Self {
            Self::try_compose(ptr, tag).expect(Self::COMPOSE_ERR_MSG)
        }
    }

    /// Attempts to compose a new marked pointer from a raw (non-null) `ptr` and
    /// a `tag` value.
    ///
    /// # Errors
    ///
    /// Fails if `ptr` is mis-aligned for `N` tag bits and contains only
    /// zero bits in the upper bits, i.e., it would be parsed as a marked
    /// `null` pointer.
    /// In this case a [`Null`] instance is returned containing the argument
    /// pointer's tag value.
    #[inline]
    pub fn try_compose(ptr: NonNull<T>, tag: usize) -> Result<Self, Null> {
        match ptr.as_ptr() as usize & Self::POINTER_MASK {
            0 => Ok(unsafe { Self::compose_unchecked(ptr, tag) }),
            _ => Err(Null(ptr.as_ptr() as usize)),
        }
    }

    /// Composes a new marked pointer from a raw (non-null) `ptr` and a `tag`
    /// value without checking if `ptr` is valid.
    ///
    /// # Safety
    ///
    /// The caller has to ensure that `ptr` ...
    #[inline]
    pub unsafe fn compose_unchecked(ptr: NonNull<T>, tag: usize) -> Self {
        Self::new_unchecked(MarkedPtr::compose(ptr.as_ptr(), tag))
    }

    doc_comment! {
        doc_clear_tag!(),
        #[inline]
        pub fn clear_tag(self) -> Self {
            Self { inner: self.decompose_non_null(), _marker: PhantomData }
        }
    }

    doc_comment! {
        doc_split_tag!(),
        #[inline]
        pub fn split_tag(self) -> (Self, usize) {
            let (inner, tag) = self.decompose();
            (Self { inner, _marker: PhantomData }, tag)
        }
    }

    doc_comment! {
        doc_set_tag!(),
        #[inline]
        pub fn set_tag(self, tag: usize) -> Self {
            let ptr = self.decompose_non_null();
            unsafe { Self::compose_unchecked(ptr, tag) }
        }
    }

    doc_comment! {
        doc_update_tag!(),
        #[inline]
        pub fn update_tag(self, func: impl FnOnce(usize) -> usize) -> Self {
            let (ptr, tag) = self.decompose();
            unsafe { Self::compose_unchecked(ptr, func(tag)) }
        }
    }

    doc_comment! {
        doc_add_tag!(),
        /// # Safety
        ///
        /// The caller has to ensure that the resulting pointer is not
        /// `null` (neither marked nor unmarked).
        #[inline]
        pub unsafe fn add_tag(self, value: usize) -> Self {
            Self::from_usize(self.into_usize().wrapping_add(value))
        }
    }

    doc_comment! {
        doc_sub_tag!(),
        /// # Safety
        ///
        /// The caller has to ensure that the resulting pointer is not
        /// `null` (neither marked nor unmarked).
        #[inline]
        pub unsafe fn sub_tag(self, value: usize) -> Self {
            Self::from_usize(self.into_usize().wrapping_sub(value))
        }
    }

    doc_comment! {
        doc_decompose!(),
        #[inline]
        pub fn decompose(self) -> (NonNull<T>, usize) {
            (self.decompose_non_null(), self.decompose_tag())
        }
    }

    doc_comment! {
        doc_decompose_ptr!(),
        #[inline]
        pub fn decompose_ptr(self) -> *mut T {
            crate::decompose_ptr(self.inner.as_ptr() as usize, Self::TAG_BITS)
        }
    }

    doc_comment! {
        doc_decompose_non_null!(),
        #[inline]
        pub fn decompose_non_null(self) -> NonNull<T> {
            unsafe { NonNull::new_unchecked(self.decompose_ptr()) }
        }
    }

    doc_comment! {
        doc_decompose_tag!(),
        #[inline]
        pub fn decompose_tag(self) -> usize {
            crate::decompose_tag::<T>(self.inner.as_ptr() as usize, Self::TAG_BITS)
        }
    }

    doc_comment! {
        doc_as_ref!("non-nullable"),
        #[inline]
        pub unsafe fn as_ref(&self) -> &T {
            &*self.decompose_non_null().as_ptr()
        }
    }

    doc_comment! {
        doc_as_mut!("non-nullable", MarkedNonNull),
        #[inline]
        pub unsafe fn as_mut(&mut self) -> &mut T {
            &mut *self.decompose_non_null().as_ptr()
        }
    }

    /// Decomposes the marked pointer, returning a reference and the separated
    /// tag.
    ///
    /// # Safety
    ///
    /// The same safety caveats as with [`as_ref`][MarkedNonNull::as_ref] apply.
    #[inline]
    pub unsafe fn decompose_ref(&self) -> (&T, usize) {
        let (ptr, tag) = self.decompose();
        (&*ptr.as_ptr(), tag)
    }

    /// Decomposes the marked pointer, returning a *mutable* reference and the
    /// separated tag.
    ///
    /// # Safety
    ///
    /// The same safety caveats as with [`as_mut`][MarkedNonNull::as_mut] apply.
    #[inline]
    pub unsafe fn decompose_mut(&mut self) -> (&mut T, usize) {
        let (ptr, tag) = self.decompose();
        (&mut *ptr.as_ptr(), tag)
    }
}

/********** impl Debug ****************************************************************************/

impl<T, const N: usize> fmt::Debug for MarkedNonNull<T, N> {
    impl_debug!("MarkedNonNull");
}

/********** impl Pointer **************************************************************************/

impl<T, const N: usize> fmt::Pointer for MarkedNonNull<T, N> {
    impl_pointer!();
}

/********** impl From (&T) ************************************************************************/

impl<T, const N: usize> From<&T> for MarkedNonNull<T, N> {
    #[inline]
    fn from(reference: &T) -> Self {
        Self { inner: NonNull::from(reference), _marker: PhantomData }
    }
}

/********** impl From (&mut T) ********************************************************************/

impl<T, const N: usize> From<&mut T> for MarkedNonNull<T, N> {
    #[inline]
    fn from(reference: &mut T) -> Self {
        Self { inner: NonNull::from(reference), _marker: PhantomData }
    }
}

/********** impl PartialEq ************************************************************************/

impl<T, const N: usize> PartialEq for MarkedNonNull<T, N> {
    impl_partial_eq!();
}

/********** impl PartialOrd ***********************************************************************/

impl<T, const N: usize> PartialOrd for MarkedNonNull<T, N> {
    impl_partial_ord!();
}

/********** impl Eq *******************************************************************************/

impl<T, const N: usize> Eq for MarkedNonNull<T, N> {}

/********** impl Ord ******************************************************************************/

impl<T, const N: usize> Ord for MarkedNonNull<T, N> {
    impl_ord!();
}

/********** impl Hash *****************************************************************************/

impl<T, const N: usize> Hash for MarkedNonNull<T, N> {
    impl_hash!();
}

/********** impl TryFrom (*mut T) *****************************************************************/

impl<T, const N: usize> TryFrom<*mut T> for MarkedNonNull<T, N> {
    type Error = Null;

    #[inline]
    fn try_from(ptr: *mut T) -> Result<Self, Self::Error> {
        if ptr as usize & Self::POINTER_MASK == 0 {
            Err(Null(ptr as usize))
        } else {
            Ok(Self { inner: unsafe { NonNull::new_unchecked(ptr) }, _marker: PhantomData })
        }
    }
}

/********** impl TryFrom (*const T) ***************************************************************/

impl<T, const N: usize> TryFrom<*const T> for MarkedNonNull<T, N> {
    type Error = Null;

    #[inline]
    fn try_from(ptr: *const T) -> Result<Self, Self::Error> {
        Self::try_from(ptr as *mut _)
    }
}

/********** impl TryFrom (MarkedPtr) **************************************************************/

impl<T, const N: usize> TryFrom<MarkedPtr<T, N>> for MarkedNonNull<T, N> {
    type Error = Null;

    #[inline]
    fn try_from(ptr: MarkedPtr<T, N>) -> Result<Self, Self::Error> {
        Self::try_from(ptr.into_raw())
    }
}

/********** impl TryFrom (NonNull) ****************************************************************/

impl<T, const N: usize> TryFrom<NonNull<T>> for MarkedNonNull<T, N> {
    type Error = Null;

    #[inline]
    fn try_from(ptr: NonNull<T>) -> Result<Self, Self::Error> {
        Self::try_from(ptr.as_ptr())
    }
}
