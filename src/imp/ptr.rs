use core::cmp;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};

use typenum::Unsigned;

use crate::MarkedPtr;

/********** impl Clone ****************************************************************************/

impl<T, N> Clone for MarkedPtr<T, N> {
    impl_clone!();
}

/********** impl Copy *****************************************************************************/

impl<T, N> Copy for MarkedPtr<T, N> {}

/********** impl inherent (const) *****************************************************************/

impl<T, N> MarkedPtr<T, N> {
    impl_ptr_inherent_const!(
        example_type_path = conquer_pointer::MarkedPtr<i32, conquer_pointer::typenum::U2>
    );
}

/********** impl inherent *************************************************************************/

impl<T, N: Unsigned> MarkedPtr<T, N> {
    impl_constants!(
        tag_bits = N::USIZE,
        tag_type = usize,
        tag_mask = crate::mark_mask::<T>(N::USIZE)
    );

    doc_comment! {
        doc_compose!(),
        ///
        /// # Examples
        ///
        /// ```
        /// use core::ptr;
        ///
        /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, conquer_pointer::typenum::U2>;
        ///
        /// let raw = &1 as *const i32 as *mut i32;
        /// let ptr = MarkedPtr::compose(raw, 0b11);
        /// assert_eq!(ptr.decompose(), (raw, 0b11));
        /// // excess bits are silently truncated
        /// let ptr = MarkedPtr::compose(raw, 0b101);
        /// assert_eq!(ptr.decompose(), (raw, 0b01));
        /// ```
        #[inline]
        pub fn compose(ptr: *mut T, tag: usize) -> Self {
            crate::assert_alignment::<T, N>();
            Self::new(crate::compose(ptr, tag, Self::TAG_BITS))
        }
    }

    impl_ptr_inherent!(
        ty_ident = MarkedPtr,
        tag_type = usize,
        example_type_path = conquer_pointer::MarkedPtr<i32, conquer_pointer::typenum::U2>
    );

    doc_comment! {
        doc_decompose_ptr!(),
        #[inline]
        pub fn decompose_ptr(self) -> *mut T {
            crate::decompose_ptr::<T>(self.inner as usize, Self::TAG_BITS)
        }
    }

    doc_comment! {
        doc_decompose_tag!(),
        #[inline]
        pub fn decompose_tag(self) -> usize {
            crate::decompose_tag::<T>(self.inner as usize, Self::TAG_BITS)
        }
    }
}

/********** impl Debug ****************************************************************************/

impl<T, N: Unsigned> fmt::Debug for MarkedPtr<T, N> {
    impl_debug!("MarkedPtr");
}

/********** impl Default **************************************************************************/

impl<T, N> Default for MarkedPtr<T, N> {
    impl_default!();
}

/********** impl From (*mut T) ********************************************************************/

impl<T, N> From<*mut T> for MarkedPtr<T, N> {
    impl_from_raw!(*mut T);
}

/********** impl From (*const T) ******************************************************************/

impl<T, N> From<*const T> for MarkedPtr<T, N> {
    impl_from_raw!(*const T);
}

/********** impl From (&T) ************************************************************************/

impl<T, N> From<&T> for MarkedPtr<T, N> {
    impl_ptr_from_reference!(&T);
}

/********** impl From (&mut T) ********************************************************************/

impl<T, N> From<&mut T> for MarkedPtr<T, N> {
    impl_ptr_from_reference!(&mut T);
}

/********** impl From (NonNull) *******************************************************************/

impl<T, N> From<NonNull<T>> for MarkedPtr<T, N> {
    impl_ptr_from_non_null!();
}

/********** impl PartialEq ************************************************************************/

impl<T, N> PartialEq for MarkedPtr<T, N> {
    impl_partial_eq!();
}

/********** impl PartialOrd ***********************************************************************/

impl<T, N> PartialOrd for MarkedPtr<T, N> {
    impl_partial_ord!();
}

/********** impl Pointer **************************************************************************/

impl<T, N: Unsigned> fmt::Pointer for MarkedPtr<T, N> {
    impl_pointer!();
}

/********** impl Eq *******************************************************************************/

impl<T, N> Eq for MarkedPtr<T, N> {}

/********** impl Ord ******************************************************************************/

impl<T, N> Ord for MarkedPtr<T, N> {
    impl_ord!();
}

/********** impl Hash *****************************************************************************/

impl<T, N> Hash for MarkedPtr<T, N> {
    impl_hash!();
}
