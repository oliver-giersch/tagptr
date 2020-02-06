use core::fmt;
use core::ptr;
use core::sync::atomic::{AtomicPtr, AtomicU64};

#[cfg(not(feature = "nightly"))]
extern "C" {
    fn dwcas_compare_exchange_128(
        ptr: *const AtomicMarkedPtr128<()>,
        current: MarkedPtr128<()>,
        new: MarkedPtr128<()>,
    );
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedPtr128
////////////////////////////////////////////////////////////////////////////////////////////////////

#[repr(C, align(16))]
pub struct AtomicMarkedPtr128<T> {
    pub ptr: AtomicPtr<T>,
    pub tag: AtomicU64,
}

/********** impl Send + Sync **********************************************************************/

unsafe impl<T> Send for AtomicMarkedPtr128<T> {}
unsafe impl<T> Sync for AtomicMarkedPtr128<T> {}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedPtr128
////////////////////////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
pub struct MarkedPtr128<T> {
    pub ptr: *mut T,
    pub tag: u64,
}

/********** impl Clone ****************************************************************************/

impl<T> Clone for MarkedPtr128<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self { ptr: self.ptr, tag: self.tag }
    }
}

/********** impl Copy *****************************************************************************/

impl<T> Copy for MarkedPtr128<T> {}

/********** impl inherent *************************************************************************/

impl<T> MarkedPtr128<T> {
    doc_comment! {
        doc_null!(),
        pub const fn null() -> Self {
            Self { ptr: ptr::null_mut(), tag: 0 }
        }
    }

    doc_comment! {
        doc_decompose!(),
        #[inline]
        pub const fn decompose(self) -> (*mut T, u64) {
            (self.ptr, self.tag)
        }
    }

    doc_comment! {
        doc_decompose_ptr!(),
        #[inline]
        pub const fn decompose_ptr(self) -> *mut T {
            self.ptr
        }
    }

    doc_comment! {
        doc_decompose_tag!(),
        #[inline]
        pub const fn decompose_tag(self) -> u64 {
            self.tag
        }
    }
}

/********** impl Debug ****************************************************************************/

impl<T> fmt::Debug for MarkedPtr128<T> {
    impl_debug!("MarkedPtr128");
}

/********** impl Default **************************************************************************/

impl<T> Default for MarkedPtr128<T> {
    impl_default!();
}
