use core::fmt;
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

    const COMPOSE_ERR_MSG: &'static str = "argument `ptr` is mis-aligned for `N` tag bits (would be parsed as marked `null` pointer).";

    impl_non_null_inherent!(
        self_ident = MarkedNonNull,
        ptr_type = MarkedPtr<T, N>,
        tag_type = usize,
        example_type_path = conquer_pointer::MarkedNonNull<T, conquer_pointer::typenum::U2>
    );

    #[inline]
    pub fn compose(ptr: NonNull<T>, tag: usize) -> Self {
        Self::try_compose(ptr, tag).expect(Self::COMPOSE_ERR_MSG)
    }

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

    #[inline]
    pub fn set_tag(self, tag: usize) -> Self {
        let ptr = self.decompose_non_null();
        unsafe { Self::compose_unchecked(ptr, tag) }
    }

    #[inline]
    pub fn update_tag(self, func: impl FnOnce(usize) -> usize) -> Self {
        let (ptr, tag) = self.decompose();
        unsafe { Self::compose_unchecked(ptr, func(tag)) }
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
