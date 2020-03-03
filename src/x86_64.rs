//! Additional 128-bit marked pointer types for the x86-64 architecture only.
//!
//! At the time of writing, the x86-64 CPU architecture is the only 64-bit
//! architecture providing an atomic instruction for realizing a double-wide
//! compare-and-swap (DWCAS) operation, which is useful for many concurrent
//! algorithms.
//! This module provides the [`AtomicMarkedPtr128`] and [`MarkedPtr128`] types
//! that can safely make use of this instruction.
//! Unlike the architecture-independent marked pointer types provided by this
//! crate, these allow storing a full 64-bit tag value in a separate memory word
//! alongside the pointer itself.
//! This means the tag value is practically unrestricted and very unlikely to
//! ever overflow and puts no alignment restrictions on the pointer itself,
//! because it does not need to fit in its lower bits.
//!
//! # A Note on Memory Orderings
//!
//! Rust's core (or standard) atomic types follow the C++11 pattern of supplying
//! the required memory ordering specifications as function arguments.
//! This module also follows this pattern for the sake of consistency, although
//! this is somewhat nonsensical:
//! Given that it is x86-64 only, any orderings other than [`SeqCst`] and
//! [`Relaxed`] is ineffectual, since the CPU architecture inherently is
//! strongly ordered.
//! The `cmpxchg16b` instruction is also inherently sequentially consistent.
//! Only the compiler *may* decide to reorder this instruction, if a weaker
//! ordering was specified.   

use core::fmt;
use core::mem::transmute;
use core::ptr;
use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

#[cfg(not(feature = "nightly"))]
use self::ffi::cmpxchg16b;
#[cfg(feature = "nightly")]
use core::arch::x86_64::cmpxchg16b;

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedPtr128
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A raw 64-bit pointer type which can be safely shared between threads and
/// which can store an additional 64-bit tag value alongside the pointer.
///
/// This types enables atomic operations on 128-bit wide (pointer, tag) tuples,
/// allowing for the elementary synchronization operation *compare-and-swap*.
#[repr(C, align(16))]
pub struct AtomicMarkedPtr128<T> {
    /// The (atomic) 64-bit pointer.
    pub ptr: AtomicPtr<T>,
    /// The (atomic) 64-bit tag value.
    pub tag: AtomicU64,
}

/********** impl Send + Sync **********************************************************************/

unsafe impl<T> Send for AtomicMarkedPtr128<T> {}
unsafe impl<T> Sync for AtomicMarkedPtr128<T> {}

/********** impl inherent *************************************************************************/

impl<T> AtomicMarkedPtr128<T> {
    doc_comment! {
        doc_atomic_new!(),
        #[inline]
        pub const fn new(marked_ptr: MarkedPtr128<T>) -> Self {
            let (ptr, tag) = marked_ptr.decompose();
            Self { ptr: AtomicPtr::new(ptr), tag: AtomicU64::new(tag) }
        }
    }

    #[inline]
    pub fn into_inner(self) -> MarkedPtr128<T> {
        MarkedPtr128 { ptr: self.ptr.into_inner(), tag: self.tag.into_inner() }
    }

    /// Loads the value of the atomic marked 128-bit pointer.
    ///
    /// `load` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation.
    /// Possible values are [`SeqCst`][seq_cst], [`Acquire`][acq] and
    /// [`Relaxed`][rlx].
    ///
    /// # Panics
    ///
    /// Panics if `order` is [`Release`][rel] or [`AcqRel`][acq_rel].
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    /// [acq_rel]: Ordering::AcqRel
    /// [seq_cst]: Ordering::SeqCst
    #[inline]
    pub fn load(&self, order: Ordering) -> MarkedPtr128<T> {
        match order {
            Ordering::Relaxed | Ordering::Acquire | Ordering::SeqCst => {
                self.compare_and_swap(MarkedPtr128::null(), MarkedPtr128::null(), order)
            }
            _ => panic!(),
        }
    }

    #[inline]
    pub fn compare_and_swap(
        &self,
        current: MarkedPtr128<T>,
        new: MarkedPtr128<T>,
        order: Ordering,
    ) -> MarkedPtr128<T> {
        match self.compare_exchange(current, new, order, strongest_failure_ordering(order)) {
            Ok(res) => res,
            Err(res) => res,
        }
    }

    #[inline]
    pub fn compare_exchange(
        &self,
        current: MarkedPtr128<T>,
        new: MarkedPtr128<T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<MarkedPtr128<T>, MarkedPtr128<T>> {
        unsafe {
            let dst = &self as *const _ as *mut u128;
            let old_u128 = current.into_u128();
            let new_u128 = new.into_u128();

            match cmpxchg16b(dst, old_u128, new_u128, success, failure) {
                res if res == old_u128 => Ok(current),
                _ => Err(new),
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// MarkedPtr128
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A tuple of a 64-bit raw `*mut T` pointer composed with a 64-bit tag value
/// into a 128-bit tuple.
#[repr(C)]
pub struct MarkedPtr128<T> {
    /// The 64-bit raw pointer.
    pub ptr: *mut T,
    /// The 64-bit tag value.
    pub tag: u64,
}

/********** impl Clone ****************************************************************************/

impl<T> Clone for MarkedPtr128<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self { ptr: self.ptr, tag: self.tag }
    }
}

/********** impl Copy *****************************************************************************/

impl<T> Copy for MarkedPtr128<T> {}

/********** impl inherent *************************************************************************/

impl<T> MarkedPtr128<T> {
    doc_comment! {
        doc_null!(),
        pub const fn null() -> Self {
            Self { ptr: ptr::null_mut(), tag: 0 }
        }
    }

    doc_comment! {
        doc_new!(),
        #[inline]
        pub const fn new(ptr: *mut T) -> Self {
            Self { ptr, tag: 0}
        }
    }

    doc_comment! {
        doc_compose!(),
        ///
        /// # Examples
        ///
        /// ```
        /// type MarkedPtr128 = conquer_pointer::x86_64::MarkedPtr128<i32>;
        ///
        /// let reference = &mut 1;
        /// let ptr = MarkedPtr128::compose(reference, 0b11);
        ///
        /// assert_eq!(ptr.decompose(), (reference as *mut _, 0b11));
        /// ```
        #[inline]
        pub const fn compose(ptr: *mut T, tag: u64) -> Self {
            Self { ptr, tag }
        }
    }

    doc_comment! {
        doc_cast!(),
        pub const fn cast<U>(self) -> MarkedPtr128<U> {
            MarkedPtr128 { ptr: self.ptr.cast(), tag: self.tag }
        }
    }

    doc_comment! {
        doc_into_usize!(),
        pub fn into_u128(self) -> u128 {
            unsafe { transmute(self) }
        }
    }

    doc_comment! {
        doc_decompose!(),
        #[inline]
        pub const fn decompose(self) -> (*mut T, u64) {
            (self.ptr, self.tag)
        }
    }

    doc_comment! {
        doc_decompose_ptr!(),
        #[inline]
        pub const fn decompose_ptr(self) -> *mut T {
            self.ptr
        }
    }

    doc_comment! {
        doc_decompose_tag!(),
        #[inline]
        pub const fn decompose_tag(self) -> u64 {
            self.tag
        }
    }

    doc_comment! {
        doc_as_ref!("nullable"),
        #[inline]
        pub unsafe fn as_ref<'a>(self) -> Option<&'a T> {
            todo!()
        }
    }

    doc_comment! {
        doc_as_mut!("nullable", MarkedPtr128),
        #[inline]
        pub unsafe fn as_mut<'a>(self) -> Option<&'a mut T> {
            self.ptr.as_mut()
        }
    }

    /// Decomposes the marked pointer, returning an optional reference and the
    /// separated tag.
    ///
    /// # Safety
    ///
    /// The same safety caveats as with [`as_ref`][MarkedPtr128::as_ref] apply.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    ///
    /// use conquer_pointer::typenum::U2;
    ///
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, U2>;
    ///
    /// let reference = &1;
    /// let ptr = MarkedPtr::compose(reference as *const _ as *mut _, 0b11);
    ///
    /// unsafe {
    ///     assert_eq!(ptr.decompose_ref(), (Some(reference), 0b11));
    /// }
    /// ```
    #[inline]
    pub unsafe fn decompose_ref<'a>(self) -> (Option<&'a T>, u64) {
        (self.as_ref(), self.decompose_tag())
    }

    /// Decomposes the marked pointer, returning an optional *mutable* reference
    /// and the separated tag.
    ///
    /// # Safety
    ///
    /// The same safety caveats as with [`as_mut`][MarkedPtr128::as_mut] apply.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    ///
    /// use conquer_pointer::typenum::U2;
    ///
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, U2>;
    ///
    /// let reference = &mut 1;
    /// let ptr = MarkedPtr::compose(reference, 0b11);
    ///
    /// unsafe {
    ///     assert_eq!(ptr.decompose_mut(), (Some(reference), 0b11));
    /// }
    /// ```
    #[inline]
    pub unsafe fn decompose_mut<'a>(self) -> (Option<&'a mut T>, u64) {
        (self.as_mut(), self.decompose_tag())
    }
}

/********** impl Debug ****************************************************************************/

impl<T> fmt::Debug for MarkedPtr128<T> {
    impl_debug!("MarkedPtr128");
}

/********** impl Default **************************************************************************/

impl<T> Default for MarkedPtr128<T> {
    impl_default!();
}

/********** ffi module ****************************************************************************/

#[cfg(not(feature = "nightly"))]
mod ffi {
    use core::sync::atomic::Ordering;

    #[inline]
    pub unsafe fn cmpxchg16b(
        dst: *mut u128,
        old: u128,
        new: u128,
        _: Ordering,
        _: Ordering,
    ) -> u128 {
        match dwcas_compare_exchange_128(dst as _, old.into(), new.into()) {
            0 => old,
            _ => *dst,
        }
    }

    #[repr(C)]
    struct U128(u64, u64);

    impl From<u128> for U128 {
        #[inline]
        fn from(val: u128) -> Self {
            unsafe { core::mem::transmute(val) }
        }
    }

    extern "C" {
        fn dwcas_compare_exchange_128(dst: *mut U128, old: U128, new: U128) -> u8;
    }
}

#[inline]
fn strongest_failure_ordering(order: Ordering) -> Ordering {
    match order {
        Ordering::Release => Ordering::Relaxed,
        Ordering::Relaxed => Ordering::Relaxed,
        Ordering::SeqCst => Ordering::SeqCst,
        Ordering::Acquire => Ordering::Acquire,
        Ordering::AcqRel => Ordering::Acquire,
        _ => unreachable!(),
    }
}
