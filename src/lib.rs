#![no_std]

#[macro_use]
mod macros;

// this module relies on 48-bit virtual addresses and thus explicitly names each
// supported architecture with this property.
#[cfg(any(
    target_arch = "x86_64",
    target_arch = "powerpc64",
    target_arch = "aarch64"
))]
pub mod arch64;

mod imp;

use core::marker::PhantomData;
use core::mem;
use core::ptr::{self, NonNull};
use core::sync::atomic::AtomicUsize;

// public re-export
pub use typenum;

use typenum::Unsigned;

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

/********** internal helper functions *************************************************************/

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

//#[deny(const_err)]
#[inline]
const fn mark_mask<T>(tag_bits: usize) -> usize {
    //let _assert_sufficient_alignment = lower_bits::<T>() - tag_bits;
    (1 << tag_bits) - 1
}

#[inline]
fn compose<T>(ptr: *mut T, tag: usize, tag_bits: usize) -> *mut T {
    debug_assert_eq!(ptr as usize & mark_mask::<T>(tag_bits), 0);
    ((ptr as usize) | (mark_mask::<T>(tag_bits) & tag)) as *mut _
}
