use core::ptr::NonNull;

use typenum::Unsigned;

use crate::{MarkedNonNull, MarkedPtr};

////////////////////////////////////////////////////////////////////////////////////////////////////
// NonNullable (trait)
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A trait for non-nullable (thin) pointer and reference types.
pub trait NonNullable: Sized {
    /// The referenced or pointed-to type.
    type Item: Sized;

    /// Returns the value of `ptr` as a `const` pointer that is guaranteed
    /// to be non-null.
    fn as_const_ptr(ptr: &Self) -> *const Self::Item;

    /// Returns the value of `ptr` as a `mut` pointer that is guaranteed to
    /// be non-null.
    fn as_mut_ptr(ptr: &Self) -> *mut Self::Item;

    /// Returns the value of `ptr` as a [`NonNull`].
    fn as_non_null(ptr: &Self) -> NonNull<Self::Item>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedNonNullable (trait)
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A trait for non-nullable marked pointer and reference types.
pub trait MarkedNonNullable: NonNullable {
    /// The number of mark bits.
    type MarkBits: Unsigned;

    /// TODO: docs...
    fn as_marked_ptr(ptr: &Self) -> MarkedPtr<Self::Item, Self::MarkBits>;

    /// TODO: docs...
    fn into_marked_ptr(ptr: Self) -> MarkedPtr<Self::Item, Self::MarkBits>;

    /// Clears (zeroes) `ptr`'s tag and returns the same pointer..
    fn clear_tag(ptr: Self) -> Self;

    ///
    fn split_tag(ptr: Self) -> (Self, usize);

    /// Returns `arg` with its tag set to `tag`.
    fn set_tag(ptr: Self, tag: usize) -> Self;

    /// TODO: docs...
    fn decompose(ptr: &Self) -> (NonNull<Self::Item>, usize);

    /// Decomposes the `arg`, returning the separated raw pointer.
    fn decompose_ptr(ptr: &Self) -> *mut Self::Item;

    /// Decomposes the `arg`, returning the separated [`NonNull`]
    /// pointer and its tag.
    fn decompose_non_null(ptr: &Self) -> NonNull<Self::Item>;

    /// Decomposes the `arg`, returning the separated tag value.
    fn decompose_tag(ptr: &Self) -> usize;
}
