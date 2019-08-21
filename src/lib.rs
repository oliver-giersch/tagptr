//! Strongly typed marked pointers for storing bit patterns alongside pointers
//!

#![allow(clippy::should_implement_trait)]
#![warn(missing_docs)]
#![cfg_attr(all(target_arch = "x86_64", feature = "nightly"), feature(stdsimd))]
#![no_std]

#[cfg(any(target_arch = "x86_64", target_arch = "powerpc64", target_arch = "aarch64"))]
pub mod arch64;

mod atomic;
mod option;
mod raw;

use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;

pub use typenum;

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedPtr
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A raw pointer type which can be safely shared between threads and which
/// can store additional information in its lower (unused) bits.
///
/// This type has the same in-memory representation as a `*mut T`. It is mostly
/// identical to [`AtomicPtr`][atomic], except that all of its methods involve
/// a [`MarkedPtr`] instead of `*mut T`.
///
/// [atomic]: core::sync::atomic::AtomicPtr
pub struct AtomicMarkedPtr<T, N> {
    inner: AtomicUsize,
    _marker: PhantomData<(*mut T, N)>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedPtr
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A raw, unsafe pointer type like `*mut T` in which up to `N` of the pointer's
/// lower bits can be used to store additional information (the *tag*).
///
/// Note, that the upper bound for `N` is dictated by the alignment of `T`.
/// A type with an alignment of `8` (e.g. a `usize` on 64-bit architectures) can
/// have up to `3` mark bits.
/// Attempts to use types with insufficient alignment will result in a compile-
/// time error.
pub struct MarkedPtr<T, N> {
    inner: *mut T,
    _marker: PhantomData<N>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedNonNull
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A non-nullable marked raw pointer type like [`NonNull`].
///
/// Note, that unlike [`MarkedPtr`] this also **excludes** marked
/// null-pointers.
pub struct MarkedNonNull<T, N> {
    inner: NonNull<T>,
    _marker: PhantomData<N>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedOption
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A value that represents the possible states of a nullable marked pointer.
///
/// This type is similar to [`Option<T>`][Option] but can also express `null`
/// pointers with mark bits.
#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum MarkedOption<T: NonNullable> {
    /// Some reference or non-nullable pointer type
    Value(T),
    /// Null pointer, potentially marked
    Null(usize),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// NonNullable (trait)
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A trait for non-nullable pointer and reference types.
pub trait NonNullable: Sized {
    /// The referenced or pointed-to type.
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
