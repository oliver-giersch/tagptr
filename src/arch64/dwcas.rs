use core::fmt;
use core::mem::transmute;
use core::ptr;
use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

#[cfg(not(feature = "nightly"))]
use self::ffi::cmpxchg16b;
#[cfg(feature = "nightly")]
use core::arch::x86_64::cmpxchg16b;

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

/********** impl inherent *************************************************************************/

impl<T> AtomicMarkedPtr128<T> {
    #[inline]
    pub const fn new(marked_ptr: MarkedPtr128<T>) -> Self {
        let (ptr, tag) = marked_ptr.decompose();
        Self { ptr: AtomicPtr::new(ptr), tag: AtomicU64::new(tag) }
    }

    #[inline]
    pub fn load(&self, order: Ordering) -> MarkedPtr128<T> {
        match order {
            Ordering::Relaxed | Ordering::Acquire | Ordering::SeqCst => {
                self.compare_and_swap(MarkedPtr128::null(), MarkedPtr128::null(), order)
            }
            _ => panic!(),
        }
    }

    #[inline]
    pub fn compare_and_swap(
        &self,
        current: MarkedPtr128<T>,
        new: MarkedPtr128<T>,
        order: Ordering,
    ) -> MarkedPtr128<T> {
        match self.compare_exchange(current, new, order, strongest_failure_ordering(order)) {
            Ok(res) => res,
            Err(res) => res,
        }
    }

    #[inline]
    pub fn compare_exchange(
        &self,
        current: MarkedPtr128<T>,
        new: MarkedPtr128<T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<MarkedPtr128<T>, MarkedPtr128<T>> {
        unsafe {
            let dst = &self as *const _ as *mut u128;
            let old_u128 = current.into_u128();
            let new_u128 = new.into_u128();

            match cmpxchg16b(dst, old_u128, new_u128, success, failure) {
                res if res == old_u128 => Ok(current),
                _ => Err(new),
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedPtr128
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A tuple of a 64-bit raw `*mut T` pointer composed with a 64-bit tag value
/// into a 128-bit tuple.
#[repr(C)]
pub struct MarkedPtr128<T> {
    /// The 64-bit raw pointer.
    pub ptr: *mut T,
    /// The 64-bit tag value.
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
        doc_new!(),
        #[inline]
        pub const fn new(ptr: *mut T) -> Self {
            Self { ptr, tag: 0}
        }
    }

    doc_comment! {
        doc_cast!(),
        pub const fn cast<U>(self) -> MarkedPtr128<U> {
            MarkedPtr128 { ptr: self.ptr.cast(), tag: self.tag }
        }
    }

    doc_comment! {
        doc_into_usize!(),
        pub fn into_u128(self) -> u128 {
            unsafe { transmute(self) }
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

/********** ffi module ****************************************************************************/

#[cfg(not(feature = "nightly"))]
mod ffi {
    use core::sync::atomic::Ordering;

    #[inline]
    pub unsafe fn cmpxchg16b(
        dst: *mut u128,
        old: u128,
        new: u128,
        _: Ordering,
        _: Ordering,
    ) -> u128 {
        match dwcas_compare_exchange_128(dst as _, old.into(), new.into()) {
            0 => old,
            _ => *dst,
        }
    }

    #[repr(C)]
    struct U128(u64, u64);

    impl From<u128> for U128 {
        #[inline]
        fn from(val: u128) -> Self {
            unsafe { core::mem::transmute(val) }
        }
    }

    extern "C" {
        fn dwcas_compare_exchange_128(dst: *mut U128, old: U128, new: U128) -> u8;
    }
}

#[inline]
fn strongest_failure_ordering(order: Ordering) -> Ordering {
    match order {
        Ordering::Release => Ordering::Relaxed,
        Ordering::Relaxed => Ordering::Relaxed,
        Ordering::SeqCst => Ordering::SeqCst,
        Ordering::Acquire => Ordering::Acquire,
        Ordering::AcqRel => Ordering::Acquire,
        _ => unreachable!(),
    }
}
