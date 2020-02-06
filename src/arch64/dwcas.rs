use core::sync::atomic::{AtomicPtr, AtomicU64};

#[cfg(not(feature = "nightly"))]
extern "C" {
    fn dwcas_compare_exchange_128(
        ptr: *const AtomicMarkedPtr128<()>,
        current: MarkedPtr128<()>,
        new: MarkedPtr128<()>,
    );
}

#[repr(C, align(16))]
pub struct AtomicMarkedPtr128<T> {
    ptr: AtomicPtr<T>,
    tag: AtomicU64,
}

#[repr(C)]
pub struct MarkedPtr128<T>(pub *mut T, pub u64);
