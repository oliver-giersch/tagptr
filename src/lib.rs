//! Strongly typed pointers with reserved bits for storing additional bit
//! patterns within a single pointer-width word.

#![no_std]
#![warn(missing_docs)]
#![allow(clippy::should_implement_trait)]
#![cfg_attr(all(target_arch = "x86_64", feature = "nightly"), feature(stdsimd))]

#[cfg(any(target_arch = "x86_64", target_arch = "powerpc64", target_arch = "aarch64"))]
pub mod arch64;

mod atomic;
mod imp;
mod traits;

use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;

pub use typenum;

pub use crate::traits::{MarkedNonNullable, NonNullable};

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
#[repr(transparent)]
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
#[repr(transparent)]
pub struct MarkedPtr<T, N> {
    inner: *mut T,
    _marker: PhantomData<N>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedNonNull
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A non-nullable marked raw pointer type like [`NonNull`].
///
/// # Invariants
///
/// Unlike [`NonNull`] this type does not permit values that would be `null`
/// pointers if its first `N` bits are interpreted as tag.
/// For instance, a pointer value `0x1`, despite not pointing at valid memory,
/// is still valid for constructing a [`NonNull`] value.
/// For any `N > 0`, however, this value is not a valid [`MarkedNonNull`], since
/// it would be interpreted as a `null` pointer with a tag value of `1`.
/// For regular, well-aligned pointers, this is usually not an issue and the
/// type enforces at compile-time that no value `N` can be instantiated that
/// exceeds `T`'s inherent alignment.
#[repr(transparent)]
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
/// Note that unlike [`Option`] this type `enum` can not benefit from
/// Null-Pointer-Optimization and hence takes up at least *two* memory words.
#[derive(Clone, Copy, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub enum MaybeNull<P: NonNullable> {
    /// Some reference or non-nullable pointer
    NotNull(P),
    /// Null pointer, potentially marked
    Null(usize),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// NullError
////////////////////////////////////////////////////////////////////////////////////////////////////

/// An error type for fallible conversion from [`MarkedPtr`] to
/// [`MarkedNonNull`].
#[derive(Clone, Copy, Debug, Default, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct NullError;

/// Returns `true` if the alignment of `T` is large enough so a pointer pointer
/// to an instance may store the given number of `mark_bits`.
#[inline]
pub const fn check_sufficient_bits<T>(mark_bits: usize) -> bool {
    lower_bits::<T>() > mark_bits
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
fn compose<T>(ptr: *mut T, tag: usize, mark_bits: usize) -> *mut T {
    debug_assert_eq!(ptr as usize & mark_mask::<T>(mark_bits), 0);
    ((ptr as usize) | (mark_mask::<T>(mark_bits) & tag)) as *mut _
}
