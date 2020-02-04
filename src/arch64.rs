mod dwcas;
mod imp;

use core::cmp;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};
use core::sync::atomic::AtomicUsize;

use crate::Null;

const TAG_SHIFT: usize = 48;
const TAG_BITS: u16 = 16;
const TAG_MASK: usize = 0xFFFF << TAG_SHIFT;

/********** helper function(s) ********************************************************************/

#[inline]
fn compose<T>(ptr: *mut T, tag: u16) -> *mut T {
    (ptr as usize | (tag as usize) << TAG_SHIFT) as *mut _
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedPtr64
////////////////////////////////////////////////////////////////////////////////////////////////////

#[repr(transparent)]
pub struct AtomicMarkedPtr64<T> {
    inner: AtomicUsize,
    _marker: PhantomData<*mut T>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedPtr64
////////////////////////////////////////////////////////////////////////////////////////////////////

#[repr(transparent)]
pub struct MarkedPtr64<T> {
    inner: *mut T,
    _marker: PhantomData<()>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedNonNull64
////////////////////////////////////////////////////////////////////////////////////////////////////

#[repr(transparent)]
pub struct MarkedNonNull64<T> {
    inner: NonNull<T>,
    _marker: PhantomData<()>,
}
