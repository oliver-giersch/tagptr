//! Strongly typed pointers with reserved bits for storing additional bit
//! patterns within a single pointer-width word.

#![no_std]
#![warn(missing_docs)]
#![allow(clippy::should_implement_trait)]
#![cfg_attr(all(target_arch = "x86_64", feature = "nightly"), feature(stdsimd))]

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

use typenum::Unsigned;

pub use crate::raw::NullError;

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
pub enum MarkedOption<T: NonNullable> {
    /// Some reference or non-nullable pointer type
    Value(T),
    /// Null pointer, potentially marked
    Null(usize),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedNonNullable (trait)
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A trait for non-nullable marked pointer and reference types.
pub trait MarkedNonNullable: NonNullable {
    /// The number of mark bits.
    type MarkBits: Unsigned;

    /// Converts `self` into a raw [`MarkedNonNull].
    fn into_marked_non_null(self) -> MarkedNonNull<Self::Item, Self::MarkBits>;

    /// TODO: Docs...
    fn decompose(&self) -> (NonNull<Self::Item>, usize);

    /// TODO: Docs...
    fn decompose_ptr(&self) -> *mut Self::Item;

    /// TODO: Docs...
    fn decompose_non_null(&self) -> NonNull<Self::Item>;

    /// TODO: Docs...
    fn decompose_tag(&self) -> usize;
}

/********** impl for MarkedNonNull<T> *************************************************************/

impl<T, N: Unsigned> MarkedNonNullable for MarkedNonNull<T, N> {
    type MarkBits = N;

    #[inline]
    fn into_marked_non_null(self) -> MarkedNonNull<Self::Item, Self::MarkBits> {
        self
    }

    #[inline]
    fn decompose(&self) -> (NonNull<Self::Item>, usize) {
        MarkedNonNull::decompose(*self)
    }

    #[inline]
    fn decompose_ptr(&self) -> *mut Self::Item {
        MarkedNonNull::decompose_ptr(*self)
    }

    #[inline]
    fn decompose_non_null(&self) -> NonNull<Self::Item> {
        MarkedNonNull::decompose_non_null(*self)
    }

    #[inline]
    fn decompose_tag(&self) -> usize {
        MarkedNonNull::decompose_tag(*self)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// NonNullable (trait)
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A trait for non-nullable (thin) pointer and reference types.
pub trait NonNullable: Sized {
    /// The referenced or pointed-to type.
    type Item: Sized;

    /// Returns the value of `self` as a `const` pointer that is guaranteed to
    /// be non-null.
    fn as_const_ptr(&self) -> *const Self::Item;

    /// Returns the value of `self` as a `mut` pointer that is guaranteed to
    /// be non-null.
    fn as_mut_ptr(&self) -> *mut Self::Item;

    /// Returns the value of `self` as a [`NonNull`].
    fn as_non_null(&self) -> NonNull<Self::Item>;
}

/********** impl for NonNull<T> *******************************************************************/

impl<T: Sized> NonNullable for NonNull<T> {
    type Item = T;

    #[inline]
    fn as_const_ptr(&self) -> *const Self::Item {
        self.as_ptr() as *const _
    }

    #[inline]
    fn as_mut_ptr(&self) -> *mut Self::Item {
        self.as_ptr()
    }

    #[inline]
    fn as_non_null(&self) -> NonNull<Self::Item> {
        *self
    }
}

/********** impl for &'a T ************************************************************************/

impl<'a, T: Sized> NonNullable for &'a T {
    type Item = T;

    #[inline]
    fn as_const_ptr(&self) -> *const Self::Item {
        *self as *const _
    }

    #[inline]
    fn as_mut_ptr(&self) -> *mut Self::Item {
        *self as *const _ as *mut _
    }

    #[inline]
    fn as_non_null(&self) -> NonNull<Self::Item> {
        NonNull::from(*self)
    }
}

/********** impl for &'a mut T ********************************************************************/

impl<'a, T: Sized> NonNullable for &'a mut T {
    type Item = T;

    #[inline]
    fn as_const_ptr(&self) -> *const Self::Item {
        *self as *const _
    }

    #[inline]
    fn as_mut_ptr(&self) -> *mut Self::Item {
        *self as *const _ as *mut _
    }

    #[inline]
    fn as_non_null(&self) -> NonNull<Self::Item> {
        NonNull::from(&**self)
    }
}

/********** impl for MarkedNonNull ****************************************************************/

impl<T, N: Unsigned> NonNullable for MarkedNonNull<T, N> {
    type Item = T;

    #[inline]
    fn as_const_ptr(&self) -> *const Self::Item {
        self.decompose_ptr() as *const _
    }

    #[inline]
    fn as_mut_ptr(&self) -> *mut Self::Item {
        self.decompose_ptr()
    }

    #[inline]
    fn as_non_null(&self) -> NonNull<Self::Item> {
        self.decompose_non_null()
    }
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
