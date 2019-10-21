//! Inherent and trait implementations for `MarkedNonNull`, `MarkedPtr`,
//! `MarkedOption` types.

mod non_null;
mod option;
mod ptr;

use core::ptr::NonNull;

use crate::NonNullable;

/********** impl NonNullable **********************************************************************/

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
