//! Inherent and trait implementations for `MarkedNonNull`, `MarkedPtr`,
//! `MarkedOption` types.

mod maybe;
mod non_null;
mod ptr;

use core::ptr::NonNull;

use crate::traits::NonNullable;

/********** impl NonNullable **********************************************************************/

impl<T: Sized> NonNullable for NonNull<T> {
    type Item = T;

    #[inline]
    fn as_const_ptr(arg: &Self) -> *const Self::Item {
        arg.as_ptr() as *const _
    }

    #[inline]
    fn as_mut_ptr(arg: &Self) -> *mut Self::Item {
        arg.as_ptr()
    }

    #[inline]
    fn as_non_null(arg: &Self) -> NonNull<Self::Item> {
        *arg
    }
}

impl<'a, T: Sized> NonNullable for &'a T {
    type Item = T;

    #[inline]
    fn as_const_ptr(arg: &Self) -> *const Self::Item {
        *arg as *const _
    }

    #[inline]
    fn as_mut_ptr(arg: &Self) -> *mut Self::Item {
        *arg as *const _ as *mut _
    }

    #[inline]
    fn as_non_null(arg: &Self) -> NonNull<Self::Item> {
        NonNull::from(*arg)
    }
}

impl<'a, T: Sized> NonNullable for &'a mut T {
    type Item = T;

    #[inline]
    fn as_const_ptr(arg: &Self) -> *const Self::Item {
        *arg as *const _
    }

    #[inline]
    fn as_mut_ptr(arg: &Self) -> *mut Self::Item {
        *arg as *const _ as *mut _
    }

    #[inline]
    fn as_non_null(arg: &Self) -> NonNull<Self::Item> {
        NonNull::from(&**arg)
    }
}
