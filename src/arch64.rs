//! A module containing additional marked pointer types for 64-bit processor
//! architectures.
//!
//! Current 64-bit processor architectures except the very latest Intel x86-64
//! micro-architectures (Sunny Cove and later) use 64-bit registers but only
//! 48-bit virtual addresses and 4-level page table structure.
//! Consequently, the upper 16 bits of any pointer are always unused and can be
//! used to store tag values instead, regardless of alignment.
//!
//! The x86-64 architecture in particular provides the `cmpxchg16` CPU
//! instruction, which can be used to implement the double-width
//! compare-and-swap (DWCAS or DCAS) operation, for which this module also
//! exports the appropriate marked pointer types.

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
