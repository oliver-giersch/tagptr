use core::cmp;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};

use crate::arch64::MarkedPtr64;

/********** impl Clone ****************************************************************************/

impl<T> Clone for MarkedPtr64<T> {
    impl_clone!();
}

/********** impl Copy *****************************************************************************/

impl<T> Copy for MarkedPtr64<T> {}

/********** impl inherent *************************************************************************/

impl<T> MarkedPtr64<T> {
    impl_constants!(tag_bits = 16, tag_type = u16, tag_mask = 0xFFFF << Self::TAG_SHIFT);

    const TAG_SHIFT: usize = 48;

    impl_ptr_inherent_const!(
        ptr_type = MarkedPtr64,
        example_type_path = conquer_pointer::arch64::MarkedPtr64<i32>
    );

    doc_comment! {
        doc_cast!(),
        #[inline]
        pub const fn cast<U>(self) -> MarkedPtr64<U> {
            MarkedPtr64::new(self.inner.cast())
        }
    }

    doc_comment! {
        doc_compose!(),
        #[inline]
        pub fn compose(ptr: *mut T, tag: u16) -> Self {
            Self::new(crate::arch64::compose(ptr, tag))
        }
    }

    impl_ptr_inherent!(
        ty_ident = MarkedPtr64,
        tag_type = u16,
        example_type_path = conquer_pointer::arch64::MarkedPtr64<i32>
    );

    doc_comment! {
        doc_decompose_ptr!(),
        #[inline]
        pub fn decompose_ptr(self) -> *mut T {
            (self.inner as usize & Self::POINTER_MASK) as *mut _
        }
    }

    doc_comment! {
        doc_decompose_tag!(),
        #[inline]
        pub fn decompose_tag(self) -> u16 {
            (self.inner as usize >> Self::TAG_SHIFT) as u16
        }
    }
}

/********** impl Debug ****************************************************************************/

impl<T> fmt::Debug for MarkedPtr64<T> {
    impl_debug!("MarkedPtr");
}

/********** impl Default **************************************************************************/

impl<T> Default for MarkedPtr64<T> {
    impl_default!();
}

/********** impl From (*mut T) ********************************************************************/

impl<T> From<*mut T> for MarkedPtr64<T> {
    impl_from_raw!(*mut T);
}

/********** impl From (*const T) ******************************************************************/

impl<T> From<*const T> for MarkedPtr64<T> {
    impl_from_raw!(*const T);
}

/********** impl From (&T) ************************************************************************/

impl<T> From<&T> for MarkedPtr64<T> {
    impl_ptr_from_reference!(&T);
}

/********** impl From (&mut T) ********************************************************************/

impl<T> From<&mut T> for MarkedPtr64<T> {
    impl_ptr_from_reference!(&mut T);
}

/********** impl From (NonNull) *******************************************************************/

impl<T> From<NonNull<T>> for MarkedPtr64<T> {
    impl_ptr_from_non_null!();
}

/********** impl PartialEq ************************************************************************/

impl<T> PartialEq for MarkedPtr64<T> {
    impl_partial_eq!();
}

/********** impl PartialOrd ***********************************************************************/

impl<T> PartialOrd for MarkedPtr64<T> {
    impl_partial_ord!();
}

/********** impl Pointer **************************************************************************/

impl<T> fmt::Pointer for MarkedPtr64<T> {
    impl_pointer!();
}

/********** impl Eq *******************************************************************************/

impl<T> Eq for MarkedPtr64<T> {}

/********** impl Ord ******************************************************************************/

impl<T> Ord for MarkedPtr64<T> {
    impl_ord!();
}

/********** impl Hash *****************************************************************************/

impl<T> Hash for MarkedPtr64<T> {
    impl_hash!();
}
