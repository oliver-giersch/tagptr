use core::ptr::NonNull;

use typenum::Unsigned;

use crate::MarkedPtr;

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

    /// Converts `ptr` into a [`MarkedPtr`] without consuming it.
    fn as_marked_ptr(ptr: &Self) -> MarkedPtr<Self::Item, Self::MarkBits>;

    /// Converts `ptr` into a [`MarkedPtr`] and consumes it.
    fn into_marked_ptr(ptr: Self) -> MarkedPtr<Self::Item, Self::MarkBits>;

    /// Clears (zeroes) `ptr`'s tag and returns the same pointer..
    fn clear_tag(ptr: Self) -> Self;

    /// Splits the tag from `ptr` and returns the unmarked pointer and the
    /// previous tag.
    fn split_tag(ptr: Self) -> (Self, usize);

    /// Sets the tag of `ptr` to `tag` and returns the marked value.
    fn set_tag(ptr: Self, tag: usize) -> Self;

    /// Updates the tag of `ptr` with `func` and returns the pointer with the
    /// updated tag.
    fn update_tag(ptr: Self, func: impl FnOnce(usize) -> usize) -> Self;

    /// Decomposes `ptr` and returns the separated raw [`NonNull`] and its tag
    /// value.
    fn decompose(ptr: &Self) -> (NonNull<Self::Item>, usize);

    /// Decomposes `ptr` and returns the separated raw pointer.
    fn decompose_ptr(ptr: &Self) -> *mut Self::Item;

    /// Decomposes `ptr` and returns the separated raw [`NonNull`].
    fn decompose_non_null(ptr: &Self) -> NonNull<Self::Item>;

    /// Decomposes `ptr` and returns the separated tag value.
    fn decompose_tag(ptr: &Self) -> usize;
}
