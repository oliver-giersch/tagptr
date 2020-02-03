mod dwcas;

use core::cmp;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};
use core::sync::atomic::AtomicUsize;

use crate::Null;

pub(crate) const TAG_SHIFT: usize = 48;
pub(crate) const TAG_BITS: u16 = 16;
pub(crate) const TAG_MASK: usize = 0xFFFF << TAG_SHIFT;

#[inline]
pub(crate) fn compose<T>(ptr: *mut T, tag: u16) -> *mut T {
    (ptr as usize | (tag as usize) << TAG_SHIFT) as *mut _
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedPtr64
////////////////////////////////////////////////////////////////////////////////////////////////////

#[repr(transparent)]
pub struct AtomicMarkedPtr64<T> {
    inner: AtomicUsize,
    _marker: PhantomData<*mut T>,
}

/********** impl Send + Sync **********************************************************************/

unsafe impl<T> Send for AtomicMarkedPtr64<T> {}
unsafe impl<T> Sync for AtomicMarkedPtr64<T> {}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedPtr64
////////////////////////////////////////////////////////////////////////////////////////////////////

#[repr(transparent)]
pub struct MarkedPtr64<T> {
    inner: *mut T,
    _marker: PhantomData<()>,
}

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
            Self::new(compose(ptr, tag))
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedNonNull64
////////////////////////////////////////////////////////////////////////////////////////////////////

#[repr(transparent)]
pub struct MarkedNonNull64<T> {
    inner: NonNull<T>,
    _marker: PhantomData<()>,
}

/********** impl Clone ****************************************************************************/

impl<T> Clone for MarkedNonNull64<T> {
    impl_clone!();
}

/********** impl Copy *****************************************************************************/

impl<T> Copy for MarkedNonNull64<T> {}

/********** impl inherent *************************************************************************/

impl<T> MarkedNonNull64<T> {
    impl_constants!(tag_bits = TAG_BITS, tag_type = u16, tag_mask = TAG_MASK);
    impl_non_null_inherent_const!(ptr_type = MarkedPtr64<T>, ptr_ident = MarkedPtr64);
    impl_non_null_inherent!(
        self_ident = MarkedNonNull64,
        ptr_type = MarkedPtr64<T>,
        tag_type = u16,
        example_type_path = conquer_pointer::arch64::MarkedNonNull64<T>
    );

    #[inline]
    pub fn compose(ptr: NonNull<T>, tag: u16) -> Self {
        Self {
            inner: unsafe { NonNull::new_unchecked(compose(ptr.as_ptr(), tag)) },
            _marker: PhantomData,
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
