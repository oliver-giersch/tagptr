#![allow(clippy::should_implement_trait)]

#![cfg_attr(feature = "nightly", const_generics)]

#![no_std]

#[cfg(feature = "nightly")]
pub mod nightly;

#[cfg(pointer_width = "64")]
mod arch64;

mod atomic;
mod option;
mod raw;

use core::mem;
use core::marker::PhantomData;
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedPtr
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AtomicMarkedPtr<T, N> {
    ptr: AtomicUsize,
    _marker: PhantomData<(*mut T, N)>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedPtr
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct MarkedPtr<T, N> {
    ptr: *mut T,
    _marker: PhantomData<N>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedNonNull
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct MarkedNonNull<T, N> {
    ptr: NonNull<T>,
    _marker: PhantomData<N>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedOption
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy)]
pub enum MarkedOption<T: NonNullable> {
    Value(T),
    Null(usize),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// NonNullable (trait)
////////////////////////////////////////////////////////////////////////////////////////////////////

/// TODO: Docs...
pub trait NonNullable: Sized {
    /// TODO: Docs...
    type Item: ?Sized;
}

/********** impl for NonNull<T> *******************************************************************/

impl<T: ?Sized> NonNullable for NonNull<T> {
    type Item = T;
}

/********** impl for &'a T ************************************************************************/

impl<'a, T: ?Sized> NonNullable for &'a T {
    type Item = T;
}

/********** impl for &'a mut T ********************************************************************/

impl<'a, T: ?Sized> NonNullable for &'a mut T {
    type Item = T;
}

/********** impl for MarkedNonNull ****************************************************************/

impl<T, N> NonNullable for MarkedNonNull<T, N> {
    type Item = T;
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
fn compose<T>(mark_bits: usize, ptr: *mut T, tag: usize) -> *mut T {
    debug_assert_eq!(ptr as usize & mark_mask::<T>(mark_bits), 0);
    ((ptr as usize) | (mark_mask::<T>(mark_bits) & tag)) as *mut _
}
