use core::cmp;
use core::fmt;
use core::marker::PhantomData;
use core::ptr::{self, NonNull};

use crate::arch64::{MarkedNonNull64, MarkedPtr64};
use crate::Null;

/********** impl Clone ****************************************************************************/

impl<T> Clone for MarkedNonNull64<T> {
    impl_clone!();
}

/********** impl Copy *****************************************************************************/

impl<T> Copy for MarkedNonNull64<T> {}

/********** impl inherent *************************************************************************/

impl<T> MarkedNonNull64<T> {
    impl_constants!(
        tag_bits = crate::arch64::TAG_BITS,
        tag_type = u16,
        tag_mask = crate::arch64::TAG_MASK
    );
    impl_non_null_inherent_const!(ptr_type = MarkedPtr64<T>, ptr_ident = MarkedPtr64);

    doc_comment! {
        doc_dangling!(),
        #[inline]
        pub const fn dangling() -> Self {
            Self { inner: NonNull::dangling(), _marker: PhantomData }
        }
    }

    impl_non_null_inherent!(
        self_ident = MarkedNonNull64,
        ptr_type = MarkedPtr64<T>,
        tag_type = u16,
        example_type_path = conquer_pointer::arch64::MarkedNonNull64<T>
    );

    doc_comment! {
        doc_compose!(),
        #[inline]
        pub fn compose(ptr: NonNull<T>, tag: u16) -> Self {
            Self {
                inner: unsafe { NonNull::new_unchecked(crate::arch64::compose(ptr.as_ptr(), tag)) },
                _marker: PhantomData,
            }
        }
    }

    #[inline]
    pub fn set_tag(self, tag: u16) -> Self {
        Self::compose(self.decompose_non_null(), tag)
    }

    #[inline]
    pub fn update_tag(self, func: impl FnOnce(u16) -> u16) -> Self {
        let (ptr, tag) = self.decompose();
        Self::compose(ptr, func(tag))
    }

    doc_comment! {
        doc_decompose_ptr!(),
        #[inline]
        pub fn decompose_ptr(self) -> *mut T {
            todo!()
        }
    }

    doc_comment! {
        doc_decompose_non_null!(),
        #[inline]
        pub fn decompose_non_null(self) -> NonNull<T> {
            todo!()
        }
    }

    doc_comment! {
        doc_decompose_tag!(),
        #[inline]
        pub fn decompose_tag(self) -> u16 {
            todo!()
        }
    }
}

/********** impl Debug ****************************************************************************/

impl<T> fmt::Debug for MarkedNonNull64<T> {
    impl_debug!("MarkedNonNull64");
}

/********** impl Pointer **************************************************************************/

/********** impl From (&T) ************************************************************************/

/********** impl From (&mut T) ********************************************************************/

/********** impl From (NonNull<T>) ****************************************************************/

/********** impl PartialEq ************************************************************************/

/********** impl PartialOrd ***********************************************************************/

/********** impl Eq *******************************************************************************/

/********** impl Ord ******************************************************************************/

/********** impl Hash *****************************************************************************/

/********** impl TryFrom (*mut T) *****************************************************************/

/********** impl TryFrom (*const T) ***************************************************************/

/********** impl TryFrom (MarkedPtr64) ************************************************************/

/********** impl TryFrom (NonNull) ****************************************************************/
