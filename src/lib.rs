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
//! # Examples

// TODO: atomic doc examples
// TODO: module/crate docs
// TODO: missing impls
// TODO: unit tests

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

// public re-export(s)
pub use typenum;

use typenum::Unsigned;

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedPtr
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A raw pointer type which can be safely shared between threads and which can
/// store additional information in its lower (unused) bits.
///
/// This type has the same in-memory representation as a `*mut T`.
/// It is mostly identical to [`AtomicPtr`][atomic], except that all of its
/// methods take or return a [`MarkedPtr`] instead of `*mut T`.
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
/// Note, that the upper bound for `N` is dictated by the alignment of `T`.
/// A type with an alignment of `8` (e.g. a `usize` on 64-bit architectures) can
/// have up to `3` mark bits.
///
/// # Invariants
///
/// Unlike [`NonNull`] this type does not permit values that would be `null`
/// pointers after its first `N` bits are parsed as the tag value.
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
// Null
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A type representing a `null` pointer with potential tag bits.
#[derive(Clone, Copy, Debug, Default, Hash, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
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
pub fn assert_alignment<T, N: Unsigned>() {
    assert!(
        has_sufficient_alignment::<T>(N::USIZE),
        "the respective type has insufficient alignment for storing N tag bits"
    );
}

/********** helper function(s) ********************************************************************/

/// Decomposes the integer representation of a `marked_ptr` for a given number
/// of `tag_bits` into only a raw pointer.
#[inline]
const fn decompose_ptr<T>(marked_ptr: usize, tag_bits: usize) -> *mut T {
    (marked_ptr & !mark_mask::<T>(tag_bits)) as *mut _
}

/// Decomposes the integer representation of a `marked_ptr` for a given number
/// of `tag_bits` into only a separated tag value.
#[inline]
const fn decompose_tag<T>(marked_ptr: usize, tag_bits: usize) -> usize {
    marked_ptr & mark_mask::<T>(tag_bits)
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
/// Panics in *debug* builds if `ptr` is not well aligned, i.e., if it contains
/// any bits in its lower bits reserved for the tag value.
#[inline]
fn compose<T>(ptr: *mut T, tag: usize, tag_bits: usize) -> *mut T {
    debug_assert_eq!(ptr as usize & mark_mask::<T>(tag_bits), 0);
    ((ptr as usize) | (mark_mask::<T>(tag_bits) & tag)) as *mut _
}
