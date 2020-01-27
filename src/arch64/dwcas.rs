use core::sync::atomic::{AtomicPtr, AtomicU64};

#[repr(C, align(16))]
pub struct AtomicMarkedPtr128<T> {
    ptr: AtomicPtr<T>,
    tag: AtomicU64,
}

#[repr(C)]
pub struct MarkedPtr128<T>(pub *mut T, pub u64);
