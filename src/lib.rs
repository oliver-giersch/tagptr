//! Strongly typed pointers with reserved bits for storing additional bit
//! patterns within pointer-width memory words.
//!
//! # Motivation
//!
//! Many lock-free algorithms require storing additional state information
//! together with pointers (i.e., within the same 32-bit or 64-bit memory word)
//! in order to be able to exchange both the pointer and the state with a single
//! atomic instruction.
//! The marked pointer types provided by this crate encapsulate and abstract the
//! required functionality and logic for composing, decomposing and mutating
//! such pointers with tag values.
//!
//! # Tag Bits and Type Alignment
//!
//! The number of unused lower bits in a pointer is directly determined by the
//! alignment of the pointed-to type, as long as the pointer itself is
//! well-aligned (e.g., not packed).
//! For example, the `u64` type has an alignment of 8 (or 2^3) and, therefore,
//! no well-aligned pointer to an `u64` uses its lower 3 bits.
//! Consequently, constructing, e.g., a `MarkedPtr<u64, 4>` is most likely an
//! error on the part of the user of this crate, since the resulting type would
//! consider the bit at index 3 to be part of the tag value, although it in fact
//! is part of the pointer itself.
//! The [`has_sufficient_alignment`] and [`assert_alignment`] can be used to
//! explicitly check for this property.
//!
//! There is, however, one exception in which using "ill-formed" marked pointer
//! types is valid:
//! When a well-formed marked pointer is constructed (e.g., a
//! `MarkedPtr<u64, 3>`) and then later cast to a pointer to a type with a
//! smaller alignment, e.g., a `MarkedPtr<(), 3>` for the purpose of type
//! erasure.
//! The type-erased pointer can then safely modify its tag value without
//! corrupting the original pointer.

// TODO: atomic doc examples
// TODO: module/crate docs
// TODO: missing impls
// TODO: unit tests

#![feature(min_const_generics)]
#![cfg_attr(feature = "nightly", feature(stdsimd))]

#![no_std]

#[macro_use]
mod macros;

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

mod imp;

use core::marker::PhantomData;
use core::mem;
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedPtr (impl in "imp/atomic.rs")
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A raw pointer type which can be safely shared between threads and which can
/// store additional information in its lower (unused) bits.
///
/// This type has the same in-memory representation as a `*mut T`.
/// It is mostly identical to [`AtomicPtr`][atomic], except that all of its
/// methods take or return a [`MarkedPtr`] instead of `*mut T`.
///
/// [atomic]: core::sync::atomic::AtomicPtr
#[repr(transparent)]
pub struct AtomicMarkedPtr<T, const N: usize> {
    inner: AtomicUsize,
    _marker: PhantomData<*mut T>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedPtr (impl in "imp/ptr.rs")
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A raw, unsafe pointer type like `*mut T` in which up to `N` of the pointer's
/// lower bits can be used to store additional information (the *tag*).
///
/// Note, that the logical upper bound for `N` is dictated by the alignment of
/// type `T`.
/// A type with an alignment of 8 (2^3), e.g., an `u64`, can safely store up to
/// 3 tag bits.
/// A type with an alignment of 16 (2^4) can safely store up to 4 tag bits, etc.
#[repr(transparent)]
pub struct MarkedPtr<T, const N: usize> {
    inner: *mut T,
    _marker: PhantomData<()>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedNonNull (impl in "imp/non_null.rs")
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A non-nullable marked raw pointer type like [`NonNull`].
///
/// Note, that the logical upper bound for `N` is dictated by the alignment of
/// type `T`.
/// A type with an alignment of 8 (2^3), e.g., an `u64`, can safely store up to
/// 3 tag bits.
/// A type with an alignment of 16 (2^4) can safely store up to 4 tag bits, etc.
///
/// # Invariants
///
/// Unlike [`NonNull`], this type does not permit values that would be `null`
/// pointers after its first `N` bits are parsed as the tag value.
/// For instance, a pointer value `0x1`, despite not pointing at valid memory,
/// is still valid for constructing a [`NonNull`] value.
/// For any `N > 0`, however, this value is not a valid [`MarkedNonNull`], since
/// it would be interpreted as a `null` pointer with a tag value of `1`.
/// For regular, well-aligned pointers, this is usually not an issue.
#[repr(transparent)]
pub struct MarkedNonNull<T, const N: usize> {
    inner: NonNull<T>,
    _marker: PhantomData<()>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Null
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A type representing a `null` pointer with potential tag bits.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Null(pub usize);

/********** public functions **********************************************************************/

/// Returns `true` if the alignment of `T` is large enough so a pointer to an
/// instance may store the given number of `tag_bits`.
#[inline]
pub const fn has_sufficient_alignment<T>(tag_bits: usize) -> bool {
    lower_bits::<T>() >= tag_bits
}

/// Asserts that the alignment of `U` is large enough so a pointer to an
/// instance may store `N` tag bits.
///
/// # Panics
///
/// This function panics if the alignment of `U` is insufficient for storing
/// `N` tag bits.
#[inline]
pub fn assert_alignment<T, const N: usize>() {
    assert!(
        has_sufficient_alignment::<T>(N),
        "the respective type has insufficient alignment for storing N tag bits"
    );
}

/********** helper functions **********************************************************************/

/// Decomposes the integer representation of a `marked_ptr` for a given number
/// of `tag_bits` into only a raw pointer.
#[inline]
const fn decompose_ptr<T>(ptr: usize, tag_bits: usize) -> *mut T {
    (ptr & !mark_mask::<T>(tag_bits)) as *mut _
}

/// Decomposes the integer representation of a `marked_ptr` for a given number
/// of `tag_bits` into only a separated tag value.
#[inline]
const fn decompose_tag<T>(ptr: usize, tag_bits: usize) -> usize {
    ptr & mark_mask::<T>(tag_bits)
}

/// Returns the (alignment-dependent) number of unused lower bits in a pointer
/// to type `T`.
#[inline]
const fn lower_bits<T>() -> usize {
    mem::align_of::<T>().trailing_zeros() as usize
}

/// Returns the bit-mask for the lower bits containing the tag value.
#[inline]
const fn mark_mask<T>(tag_bits: usize) -> usize {
    (1 << tag_bits) - 1
}

/// Composes the given `ptr` with `tag` and returns the composed marked pointer
/// as a raw `*mut T`.
///
/// # Panics
///
/// Panics in *debug builds only* if `ptr` is not well aligned, i.e., if it
/// contains any bits in its lower bits reserved for the tag value.
#[inline]
fn compose<T, const N: usize>(ptr: *mut T, tag: usize) -> *mut T {
    debug_assert_eq!(ptr as usize & mark_mask::<T>(N), 0);
    ((ptr as usize) | (mark_mask::<T>(N) & tag)) as *mut _
}
