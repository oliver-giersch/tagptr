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
//! Given that it is x86-64 only, any orderings other than [`SeqCst`][seq_cst]
//! and [`Relaxed`][rlx] is ineffectual, since the CPU architecture is
//! inherently strongly ordered.
//! The `cmpxchg16b` instruction is also inherently sequentially consistent.
//! Only the compiler *may* decide to reorder this instruction, if a weaker
//! ordering was specified.
//!
//! [rlx]: Ordering::Relaxed
//! [seq_cst]: Ordering::SeqCst

use core::fmt;
use core::mem::transmute;
use core::ptr;
use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

#[cfg(not(feature = "nightly"))]
use self::ffi::cmpxchg16b;
#[cfg(feature = "nightly")]
use core::arch::x86_64::cmpxchg16b;

// *************************************************************************************************
// AtomicMarkedPtr128
// *************************************************************************************************

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

    doc_comment! {
        doc_atomic_into_inner!(),
        #[inline]
        pub fn into_inner(self) -> MarkedPtr128<T> {
            MarkedPtr128 { ptr: self.ptr.into_inner(), tag: self.tag.into_inner() }
        }
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

    /// Stores a value into the pointer if the current value is the same as
    /// `current`.
    ///
    /// The return value is always the previous value.
    /// If it is equal to `current`, then the value was updated.
    /// `compare_and_swap` also takes an [`Ordering`] argument which describes
    /// the memory ordering of this operation.
    /// Notice that even when using [`AcqRel`][acq_rel], the operation might
    /// fail and hence just perform an `Acquire` load, but not have `Release`
    /// semantics.
    /// Using [`Acquire`][acq] makes the store part of this operation
    /// [`Relaxed`][rlx] if it happens, and using [`Release`][rel] makes the
    /// load part [`Relaxed`][rlx].
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    /// [acq_rel]: Ordering::AcqRel
    /// [seq_cst]: Ordering::SeqCst
    ///
    /// # Examples
    ///
    /// ```
    /// use core::sync::atomic::Ordering;
    ///
    /// use conquer_pointer::x86_64::{AtomicMarkedPtr128, MarkedPtr128};
    ///
    /// let raw = &mut 1 as *mut _;
    ///
    /// let expected = MarkedPtr128::new(raw);
    /// let ptr = AtomicMarkedPtr128::new(expected);
    /// let prev =
    ///     ptr.compare_and_swap(expected, MarkedPtr128::compose(raw, 1), Ordering::Relaxed);
    /// assert_eq!(prev, expected);
    /// ```
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

    /// Stores a value into the pointer if the current value is the same as
    /// `current`.
    ///
    /// The return value is a result indicating whether the new value was
    /// written and containing the previous value.
    /// On success this value is guaranteed to be equal to `current`.
    ///
    /// `compare_exchange` takes takes two [`Ordering`] arguments to describe
    /// the memory ordering of this operation.
    /// The first describes the required ordering if the operation succeeds
    /// while the second describes the required ordering when the operation
    /// fails.
    /// Using [`Acquire`][acq] as success ordering makes store part of this
    /// operation [`Relaxed`][rlx], and using [`Release`][rel] makes the
    /// successful load [`Relaxed`][rlx].
    /// The failure ordering can only be [`SeqCst`][seq_cst], [`Acquire`][acq]
    /// or [`Relaxed`][rlx] and must be equivalent or weaker than the success
    /// ordering.
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    /// [seq_cst]: Ordering::SeqCst
    #[inline]
    pub fn compare_exchange(
        &self,
        current: MarkedPtr128<T>,
        new: MarkedPtr128<T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<MarkedPtr128<T>, MarkedPtr128<T>> {
        unsafe {
            let dst = self as *const AtomicMarkedPtr128<T> as *mut u128;
            let old_u128 = current.into_u128();
            let new_u128 = new.into_u128();

            match cmpxchg16b(dst, old_u128, new_u128, success, failure) {
                res if res == old_u128 => Ok(current),
                res => Err(MarkedPtr128::from_u128(res)),
            }
        }
    }
}

// *************************************************************************************************
// MarkedPtr128
// *************************************************************************************************

/// A tuple of a 64-bit raw `*mut T` pointer composed with a 64-bit tag value
/// into a 128-bit tuple.
#[derive(Ord, PartialOrd, Eq, PartialEq)]
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
            self.ptr.as_ref()
        }
    }

    doc_comment! {
        doc_as_mut!("nullable", MarkedPtr128),
        #[inline]
        pub unsafe fn as_mut<'a>(self) -> Option<&'a mut T> {
            self.ptr.as_mut()
        }
    }

    #[inline]
    fn from_u128(val: u128) -> Self {
        unsafe { transmute(val) }
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
    /// use conquer_pointer::x86_64::MarkedPtr128;
    ///
    /// let reference = &1;
    /// let ptr = MarkedPtr128::compose(reference as *const _ as *mut _, 0b11);
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
    /// use conquer_pointer::x86_64::MarkedPtr128;
    ///
    /// let reference = &mut 1;
    /// let ptr = MarkedPtr128::compose(reference, 0b11);
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
    const fn ordering_to_u8(order: Ordering) -> u8 {
        match order {
            Ordering::Relaxed => 0,
            Ordering::Acquire => 1,
            Ordering::Release => 2,
            Ordering::AcqRel => 3,
            Ordering::SeqCst => 4,
        }
    }

    #[inline]
    pub unsafe fn cmpxchg16b(
        dst: *mut u128,
        mut old: u128,
        new: u128,
        success: Ordering,
        failure: Ordering,
    ) -> u128 {
        let old_ptr = &mut old as *mut u128 as *mut u64;
        let new_ptr = &new as *const u128 as *const u64;

        let _ = dwcas_compare_exchange_128(
            dst as _,
            old_ptr,
            new_ptr,
            ordering_to_u8(success),
            ordering_to_u8(failure),
        );

        old
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
        fn dwcas_compare_exchange_128(
            dst: *mut U128,
            old: *mut u64,
            new: *const u64,
            success: u8,
            failure: u8,
        ) -> bool;
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

#[cfg(test)]
mod tests {
    use core::sync::atomic::Ordering;

    use super::{AtomicMarkedPtr128, MarkedPtr128};

    #[test]
    fn compare_exchange() {
        let ptr = &mut 1 as *mut i32;

        let current = MarkedPtr128::compose(ptr, 1);
        let new = MarkedPtr128::compose(ptr, 2);

        let atomic = AtomicMarkedPtr128::new(current);

        let res = atomic.compare_exchange(current, new, Ordering::Relaxed, Ordering::Relaxed);
        assert_eq!(res, Ok(current));

        let res = atomic.compare_exchange(current, new, Ordering::Relaxed, Ordering::Relaxed);
        assert_eq!(res, Err(new));
    }

    #[test]
    fn compare_and_swap() {
        let ptr = &mut 1 as *mut i32;

        let current = MarkedPtr128::compose(ptr, 1);
        let new = MarkedPtr128::compose(ptr, 2);

        let atomic = AtomicMarkedPtr128::new(current);

        let prev = atomic.compare_and_swap(current, new, Ordering::Relaxed);
        assert_eq!(prev, current);

        let prev = atomic.compare_and_swap(current, new, Ordering::Relaxed);
        assert_eq!(prev, new);
    }

    #[test]
    fn load() {
        let ptr = &mut 1 as *mut i32;

        let current = MarkedPtr128::new(ptr);

        let atomic = AtomicMarkedPtr128::new(current);
        assert_eq!(atomic.load(Ordering::Relaxed), current);
    }
}
