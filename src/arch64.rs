#[cfg(target_arch = "x86_64")]
mod dwcas;
mod imp;

#[cfg(target_arch = "x86_64")]
pub use self::dwcas::{AtomicMarkedPtr128, MarkedPtr128};

use core::marker::PhantomData;
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;

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

/********** helper function(s) ********************************************************************/

#[inline]
fn compose<T>(ptr: *mut T, tag: u16) -> *mut T {
    (ptr as usize | (tag as usize) << TAG_SHIFT) as *mut _
}

/********** constant(s) ***************************************************************************/

const TAG_SHIFT: usize = 48;
const TAG_BITS: u16 = 16;
const TAG_MASK: usize = 0xFFFF << TAG_SHIFT;
