#![feature(const_generics)]

#![no_std]

mod pointer;

use core::marker::PhantomData;
use core::mem;
use core::sync::atomic::AtomicUsize;
use core::ptr::NonNull;

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedPtr
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AtomicMarkedPtr<T, const N: usize> {
    ptr: AtomicUsize,
    _marker: PhantomData<*mut T>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedPtr
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct MarkedPtr<T, const N: usize> {
    ptr: *mut T,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedNonNull
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct MarkedNonNull<T, const N: usize> {
    ptr: NonNull<T>,
}

/********** helper functions **********************************************************************/

#[inline]
const fn decompose<T>(marked: usize, mark_bits: usize) -> (*mut T, usize) {
    (decompose_ptr::<T>(marked, mark_bits), decompose_tag::<T>(marked, mark_bits))
}

#[inline]
const fn decompose_ptr<T>(marked: usize, mark_bits: usize) -> *mut T {
    (marked & !mark_mask::<T>(mark_bits)) as *mut _
}

#[inline]
const fn decompose_tag<T>(marked: usize, mark_bits: usize) -> usize {
    marked & mark_mask::<T>(mark_bits)
}

#[inline]
const fn lower_bits<T>() -> usize {
    mem::align_of::<T>().trailing_zeros() as usize
}

#[deny(const_err)]
#[inline]
const fn mark_mask<T>(mark_bits: usize) -> usize {
    let _assert_sufficient_alignment = lower_bits::<T>() - mark_bits;
    (1 << mark_bits) - 1
}

#[inline]
fn compose<T, const N: usize>(ptr: *mut T, tag: usize) -> *mut T {
    debug_assert_eq!(ptr as usize & mark_mask::<T>(N), 0);
    ((ptr as usize) | (mark_mask::<T>(N) & tag)) as *mut _
}
