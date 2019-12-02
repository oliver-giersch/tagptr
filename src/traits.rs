use typenum::Unsigned;

use crate::{MarkedPtr, MarkedNonNull};

pub trait IntoMarkedPtr {
    type Item: Sized;
    type MarkBits: Unsigned;

    fn as_marked_ptr(ptr: &Self) -> MarkedPtr<Self::Item, Self::MarkBits>;
    fn into_marked_ptr(ptr: Self) -> MarkedPtr<Self::Item, Self::MarkBits>;
}

pub trait FromMarkedPtr {
    type Item: Sized;
    type MarkBits: Unsigned;

    unsafe fn from_marked_ptr(ptr: MarkedPtr<Self::Item, Self::MarkBits>) -> Self;
    unsafe fn from_marked_non_null(ptr: MarkedNonNull<Self::Item, Self::MarkBits>) -> Self;
}
