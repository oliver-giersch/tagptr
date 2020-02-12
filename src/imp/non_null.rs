use core::cmp;
use core::convert::TryFrom;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ptr::NonNull;

use typenum::Unsigned;

use crate::{MarkedNonNull, MarkedPtr, Null};

/********** impl Clone ****************************************************************************/

impl<T, N> Clone for MarkedNonNull<T, N> {
    impl_clone!();
}

/********** impl Copy *****************************************************************************/

impl<T, N> Copy for MarkedNonNull<T, N> {}

/********** impl inherent (const) *****************************************************************/

impl<T, N> MarkedNonNull<T, N> {
    impl_non_null_inherent_const!(ptr_type = MarkedPtr<T, N>, ptr_ident = MarkedPtr);

    doc_comment! {
        doc_dangling!(),
        #[inline]
        pub fn dangling() -> Self {
            todo!()
        }
    }
}

/********** impl inherent *************************************************************************/

impl<T, N: Unsigned> MarkedNonNull<T, N> {
    impl_constants!(
        tag_bits = N::USIZE,
        tag_type = usize,
        tag_mask = crate::mark_mask::<T>(N::USIZE)
    );

    const COMPOSE_ERR_MSG: &'static str = "argument `ptr` is mis-aligned for `N` tag bits and could be parsed as marked `null` pointer.";

    impl_non_null_inherent!(
        self_ident = MarkedNonNull,
        ptr_type = MarkedPtr<T, N>,
        tag_type = usize,
        example_type_path = conquer_pointer::MarkedNonNull<T, conquer_pointer::typenum::U2>
    );

    doc_comment! {
        doc_compose!(),
        /// # Panics
        ///
        /// This function panics if `ptr` is mis-aligned for `N` tag bits and
        /// could be parsed as a marked `null` pointer.
        #[inline]
        pub fn compose(ptr: NonNull<T>, tag: usize) -> Self {
            Self::try_compose(ptr, tag).expect(Self::COMPOSE_ERR_MSG)
        }
    }

    /// Attempts to compose a new marked pointer from a raw `ptr` and a `tag`
    /// value.
    ///
    /// # Errors
    ///
    /// Panics if ...
    #[inline]
    pub fn try_compose(ptr: NonNull<T>, tag: usize) -> Result<Self, Null> {
        match ptr.as_ptr() as usize & Self::POINTER_MASK {
            0 => Ok(unsafe { Self::compose_unchecked(ptr, tag) }),
            _ => Err(Null(ptr.as_ptr() as usize)),
        }
    }

    #[inline]
    pub unsafe fn compose_unchecked(ptr: NonNull<T>, tag: usize) -> Self {
        Self::new_unchecked(MarkedPtr::compose(ptr.as_ptr(), tag))
    }

    doc_comment! {
        doc_set_tag!("non-null" conquer_pointer::MarkedNonNull<i32, conquer_pointer::typenum::U2>),
        #[inline]
        pub fn set_tag(self, tag: usize) -> Self {
            let ptr = self.decompose_non_null();
            unsafe { Self::compose_unchecked(ptr, tag) }
        }
    }

    doc_comment! {
        doc_update_tag!("non-null" conquer_pointer::MarkedNonNull<i32, conquer_pointer::typenum::U2>),
        #[inline]
        pub fn update_tag(self, func: impl FnOnce(usize) -> usize) -> Self {
            let (ptr, tag) = self.decompose();
            unsafe { Self::compose_unchecked(ptr, func(tag)) }
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
}

/********** impl Debug ****************************************************************************/

impl<T, N: Unsigned> fmt::Debug for MarkedNonNull<T, N> {
    impl_debug!("MarkedNonNull");
}

/********** impl Pointer **************************************************************************/

impl<T, N: Unsigned> fmt::Pointer for MarkedNonNull<T, N> {
    impl_pointer!();
}

/********** impl From (&T) ************************************************************************/

impl<T, N> From<&T> for MarkedNonNull<T, N> {
    impl_non_null_from_reference!(&T);
}

/********** impl From (&mut T) ********************************************************************/

impl<T, N> From<&mut T> for MarkedNonNull<T, N> {
    impl_non_null_from_reference!(&mut T);
}

/********** impl PartialEq ************************************************************************/

impl<T, N> PartialEq for MarkedNonNull<T, N> {
    impl_partial_eq!();
}

/********** impl PartialOrd ***********************************************************************/

impl<T, N> PartialOrd for MarkedNonNull<T, N> {
    impl_partial_ord!();
}

/********** impl Eq *******************************************************************************/

impl<T, N> Eq for MarkedNonNull<T, N> {}

/********** impl Ord ******************************************************************************/

impl<T, N> Ord for MarkedNonNull<T, N> {
    impl_ord!();
}

/********** impl Hash *****************************************************************************/

impl<T, N> Hash for MarkedNonNull<T, N> {
    impl_hash!();
}

/********** impl TryFrom (*mut T) *****************************************************************/

impl<T, N: Unsigned> TryFrom<*mut T> for MarkedNonNull<T, N> {
    impl_non_null_try_from_raw_mut!();
}

/********** impl TryFrom (*const T) ***************************************************************/

impl<T, N: Unsigned> TryFrom<*const T> for MarkedNonNull<T, N> {
    type Error = Null;

    #[inline]
    fn try_from(ptr: *const T) -> Result<Self, Self::Error> {
        Self::try_from(ptr as *mut _)
    }
}

/********** impl TryFrom (MarkedPtr) **************************************************************/

impl<T, N: Unsigned> TryFrom<MarkedPtr<T, N>> for MarkedNonNull<T, N> {
    type Error = Null;

    #[inline]
    fn try_from(ptr: MarkedPtr<T, N>) -> Result<Self, Self::Error> {
        Self::try_from(ptr.into_raw())
    }
}

/********** impl TryFrom (NonNull) ****************************************************************/

impl<T, N: Unsigned> TryFrom<NonNull<T>> for MarkedNonNull<T, N> {
    type Error = Null;

    #[inline]
    fn try_from(ptr: NonNull<T>) -> Result<Self, Self::Error> {
        Self::try_from(ptr.as_ptr())
    }
}
