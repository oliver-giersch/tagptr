use core::fmt;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicUsize, Ordering};

use typenum::Unsigned;

use crate::{AtomicMarkedPtr, MarkedPtr};

/********** impl Send + Sync **********************************************************************/

unsafe impl<T, N> Send for AtomicMarkedPtr<T, N> {}
unsafe impl<T, N> Sync for AtomicMarkedPtr<T, N> {}

/********** impl inherent (const) *****************************************************************/

impl<T, N> AtomicMarkedPtr<T, N> {
    doc_comment! {
        doc_null!(),
        ///
        /// # Examples
        ///
        /// ```
        /// use core::ptr;
        /// use core::sync::atomic::Ordering;
        ///
        /// use conquer_pointer::typenum::U2;
        ///
        /// type AtomicMarkedPtr = conquer_pointer::AtomicMarkedPtr<i32, U2>;
        ///
        /// let ptr = AtomicMarkedPtr::null();
        /// assert_eq!(
        ///     ptr.load(Ordering::Relaxed).decompose(),
        ///     (ptr::null_mut(), 0)
        /// );
        /// ```
        pub const fn null() -> Self {
            Self { inner: AtomicUsize::new(0), _marker: PhantomData }
        }
    }

    doc_comment! {
        doc_atomic_new!(),
        #[inline]
        pub fn new(marked_ptr: MarkedPtr<T, N>) -> Self {
            Self { inner: AtomicUsize::new(marked_ptr.into_usize()), _marker: PhantomData }
        }
    }

    doc_comment! {
        doc_atomic_into_inner!(),
        #[inline]
        pub fn into_inner(self) -> MarkedPtr<T, N> {
            MarkedPtr::from_usize(self.inner.into_inner())
        }
    }

    /// Returns a mutable reference to the underlying marked pointer.
    ///
    /// This is safe because the mutable reference guarantees no other
    /// threads are concurrently accessing the atomic pointer.
    #[inline]
    pub fn get_mut(&mut self) -> &mut MarkedPtr<T, N> {
        unsafe { &mut *(self.inner.get_mut() as *mut usize as *mut _) }
    }

    /// Loads the value of the atomic marked pointer.
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
    pub fn load(&self, order: Ordering) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.load(order))
    }

    /// Stores a value into the atomic marked pointer.
    ///
    /// `store` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation.
    /// Possible values are [`SeqCst`][seq_cst], [`Release`][rel] and
    /// [`Relaxed`][rlx].
    ///
    /// # Panics
    ///
    /// Panics if `order` is [`Acquire`][acq] or [`AcqRel`][acq_rel].
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    /// [acq_rel]: Ordering::AcqRel
    /// [seq_cst]: Ordering::SeqCst
    #[inline]
    pub fn store(&self, ptr: MarkedPtr<T, N>, order: Ordering) {
        self.inner.store(ptr.into_usize(), order)
    }

    /// Stores a value into the atomic marked pointer and returns the previous
    /// value.
    ///
    /// `swap` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation.
    /// All ordering modes are possible.
    /// Note that using [`Acquire`][acq] makes the store part of this operation
    /// [`Relaxed`][rlx], and using [`Release`][rel] makes the load part
    /// [`Relaxed`][rlx].
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    /// use core::sync::atomic::Ordering;
    ///
    /// use conquer_pointer::typenum::U2;
    ///
    /// type AtomicMarkedPtr = conquer_pointer::AtomicMarkedPtr<i32, U2>;
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, U2>;
    ///
    /// let ptr = AtomicMarkedPtr::null();
    /// let prev = ptr.swap(MarkedPtr::new(&mut 1), Ordering::Relaxed);
    ///
    /// assert!(prev.is_null());
    /// ```
    pub fn swap(&self, ptr: MarkedPtr<T, N>, order: Ordering) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.swap(ptr.into_usize(), order))
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
    /// use core::ptr;
    /// use core::sync::atomic::Ordering;
    ///
    /// use conquer_pointer::typenum::U2;
    ///
    /// type AtomicMarkedPtr = conquer_pointer::AtomicMarkedPtr<i32, U2>;
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, U2>;
    ///
    /// let ptr = AtomicMarkedPtr::null();
    /// let new = MarkedPtr::new(&mut 1);
    /// let prev = ptr.compare_and_swap(MarkedPtr::null(), new, Ordering::Relaxed);
    ///
    /// assert_eq!(prev, MarkedPtr::null());
    /// ```
    #[inline]
    pub fn compare_and_swap(
        &self,
        current: MarkedPtr<T, N>,
        new: MarkedPtr<T, N>,
        order: Ordering,
    ) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.compare_and_swap(
            current.into_usize(),
            new.into_usize(),
            order,
        ))
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
        current: MarkedPtr<T, N>,
        new: MarkedPtr<T, N>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<MarkedPtr<T, N>, MarkedPtr<T, N>> {
        self.inner
            .compare_exchange(current.into_usize(), new.into_usize(), success, failure)
            .map(|_| current)
            .map_err(MarkedPtr::from_usize)
    }

    /// Stores a value into the pointer if the current value is the same as
    /// `current`.
    ///
    /// The return value is a result indicating whether the new value was
    /// written and containing the previous value.
    /// On success this value is guaranteed to be equal to `current`.
    ///
    /// Unlike `compare_exchange`, this function is allowed to spuriously fail,
    /// even when the comparison succeeds, which can result in more efficient
    /// code on some platforms.
    /// The return value is a result indicating whether the new value was
    /// written and containing the previous value.
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
    pub fn compare_exchange_weak(
        &self,
        current: MarkedPtr<T, N>,
        new: MarkedPtr<T, N>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<MarkedPtr<T, N>, MarkedPtr<T, N>> {
        self.inner
            .compare_exchange_weak(current.into_usize(), new.into_usize(), success, failure)
            .map(|_| current)
            .map_err(MarkedPtr::from_usize)
    }
}

/********** impl inherent *************************************************************************/

impl<T, N: Unsigned> AtomicMarkedPtr<T, N> {
    doc_comment! {
        doc_tag_bits!(),
        pub const TAG_BITS: usize = N::USIZE;
    }

    doc_comment! {
        doc_tag_mask!(),
        pub const TAG_MASK: usize = crate::mark_mask::<T>(Self::TAG_BITS);
    }

    doc_comment! {
        doc_ptr_mask!(),
        pub const POINTER_MASK: usize = !Self::TAG_MASK;
    }

    /// Adds `value` to the current tag value, returning the previous marked
    /// pointer.
    ///
    /// This operation directly and unconditionally alters the internal numeric
    /// representation of the atomic marked pointer.
    /// Hence there is no way to reliably guarantee the operation only affects
    /// the tag bits and does not overflow into the pointer bits.
    ///
    /// `fetch_add` takes takes an [`Ordering`] argument which describes the
    /// memory ordering of this operation.
    /// All ordering modes are possible.
    /// Note that using [`Acquire`][acq] makes the store part of this operation
    /// [`Relaxed`][rlx] and using [`Release`][rel] makes the load part
    /// [`Relaxed`][rlx].
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    /// use core::sync::atomic::Ordering;
    ///
    /// use conquer_pointer::typenum::U2;
    ///
    /// type AtomicMarkedPtr = conquer_pointer::AtomicMarkedPtr<i32, U2>;
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, U2>;
    ///
    /// let reference = &mut 1;
    /// let ptr = AtomicMarkedPtr::new(MarkedPtr::new(reference));
    ///
    /// assert_eq!(
    ///     ptr.fetch_add(1, Ordering::Relaxed).decompose(),
    ///     (reference as *mut _, 0)
    /// );
    ///
    /// assert_eq!(
    ///     ptr.load(Ordering::Relaxed).decompose(),
    ///     (reference as *mut _, 0b01)
    /// );
    /// ```
    #[inline]
    pub fn fetch_add(&self, value: usize, order: Ordering) -> MarkedPtr<T, N> {
        debug_assert!(value < Self::TAG_MASK, "`value` exceeds tag bits (would overflow)");
        MarkedPtr::from_usize(self.inner.fetch_add(value, order))
    }

    /// Subtracts `value` from the current tag value, returning the previous
    /// marked pointer.
    ///
    /// This operation directly and unconditionally alters the internal numeric
    /// representation of the atomic marked pointer.
    /// Hence there is no way to reliably guarantee the operation only affects
    /// the tag bits and does not overflow into the pointer bits.
    ///
    /// `fetch_sub` takes takes an [`Ordering`] argument which describes the
    /// memory ordering of this operation.
    /// All ordering modes are possible.
    /// Note that using [`Acquire`][acq] makes the store part of this operation
    /// [`Relaxed`][rlx] and using [`Release`][rel] makes the load part
    /// [`Relaxed`][rlx].
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    /// use core::sync::atomic::Ordering;
    ///
    /// use conquer_pointer::typenum::U2;
    ///
    /// type AtomicMarkedPtr = conquer_pointer::AtomicMarkedPtr<i32, U2>;
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, U2>;
    ///
    /// let reference = &mut 1;
    /// let ptr = AtomicMarkedPtr::new(MarkedPtr::compose(reference, 0b10));
    ///
    /// assert_eq!(
    ///     ptr.fetch_sub(1, Ordering::Relaxed).decompose(),
    ///     (reference as *mut _, 0b10)
    /// );
    ///
    /// assert_eq!(
    ///     ptr.load(Ordering::Relaxed).decompose(),
    ///     (reference as *mut _, 0b01)
    /// );
    /// ```
    #[inline]
    pub fn fetch_sub(&self, value: usize, order: Ordering) -> MarkedPtr<T, N> {
        debug_assert!(value < Self::TAG_MASK, "`value` exceeds tag bits (would underflow)");
        MarkedPtr::from_usize(self.inner.fetch_sub(value, order))
    }

    /// Performs a bitwise "or" of `value` with the current tag value, returning
    /// the previous marked pointer.
    ///
    /// This operation directly and unconditionally alters the internal numeric
    /// representation of the atomic marked pointer.
    /// Hence there is no way to reliably guarantee the operation only affects
    /// the tag bits and does not overflow into the pointer bits.
    ///
    /// `fetch_or` takes takes an [`Ordering`] argument which describes the
    /// memory ordering of this operation.
    /// All ordering modes are possible.
    /// Note that using [`Acquire`][acq] makes the store part of this operation
    /// [`Relaxed`][rlx] and using [`Release`][rel] makes the load part
    /// [`Relaxed`][rlx].
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    /// use core::sync::atomic::Ordering;
    ///
    /// use conquer_pointer::typenum::U2;
    ///
    /// type AtomicMarkedPtr = conquer_pointer::AtomicMarkedPtr<i32, U2>;
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, U2>;
    ///
    /// let reference = &mut 1;
    /// let ptr = AtomicMarkedPtr::new(MarkedPtr::compose(reference, 0b10));
    ///
    /// assert_eq!(
    ///     ptr.fetch_or(0b11, Ordering::Relaxed).decompose(),
    ///     (reference as *mut _, 0b10)
    /// );
    ///
    /// assert_eq!(
    ///     ptr.load(Ordering::Relaxed).decompose(),
    ///     (reference as *mut _, 0b11)
    /// );
    /// ```
    #[inline]
    pub fn fetch_or(&self, value: usize, order: Ordering) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.fetch_or(Self::TAG_MASK & value, order))
    }

    /// Performs a bitwise "and" of `value` with the current tag value,
    /// returning the previous marked pointer.
    ///
    /// This operation directly and unconditionally alters the internal numeric
    /// representation of the atomic marked pointer.
    /// Hence there is no way to reliably guarantee the operation only affects
    /// the tag bits and does not overflow into the pointer bits.
    ///
    /// `fetch_and` takes takes an [`Ordering`] argument which describes the
    /// memory ordering of this operation.
    /// All ordering modes are possible.
    /// Note that using [`Acquire`][acq] makes the store part of this operation
    /// [`Relaxed`][rlx] and using [`Release`][rel] makes the load part
    /// [`Relaxed`][rlx].
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    /// use core::sync::atomic::Ordering;
    ///
    /// use conquer_pointer::typenum::U2;
    ///
    /// type AtomicMarkedPtr = conquer_pointer::AtomicMarkedPtr<i32, U2>;
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, U2>;
    ///
    /// let reference = &mut 1;
    /// let ptr = AtomicMarkedPtr::new(MarkedPtr::compose(reference, 0b10));
    ///
    /// // fetch_x returns previous value
    /// assert_eq!(
    ///     ptr.fetch_and(0b11, Ordering::Relaxed).decompose(),
    ///     (reference as *mut _, 0b10)
    /// );
    ///
    /// assert_eq!(
    ///     ptr.load(Ordering::Relaxed).decompose(),
    ///     (reference as *mut _, 0b10)
    /// );
    /// ```
    #[inline]
    pub fn fetch_and(&self, value: usize, order: Ordering) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.fetch_and(Self::POINTER_MASK | value, order))
    }
}

/********** impl Debug ****************************************************************************/

impl<T, N: Unsigned> fmt::Debug for AtomicMarkedPtr<T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (ptr, tag) = self.load(Ordering::SeqCst).decompose();
        f.debug_struct("AtomicMarkedPtr").field("ptr", &ptr).field("tag", &tag).finish()
    }
}

/********** impl Default **************************************************************************/

impl<T, N> Default for AtomicMarkedPtr<T, N> {
    impl_default!();
}

/********** impl From (*mut T) ********************************************************************/

impl<T, N> From<*mut T> for AtomicMarkedPtr<T, N> {
    #[inline]
    fn from(ptr: *mut T) -> Self {
        Self::new(ptr.into())
    }
}

/********** impl From (MarkedPtr<T, N>) ***********************************************************/

impl<T, N> From<MarkedPtr<T, N>> for AtomicMarkedPtr<T, N> {
    #[inline]
    fn from(ptr: MarkedPtr<T, N>) -> Self {
        Self::new(ptr)
    }
}

/********** impl Pointer **************************************************************************/

impl<T, N: Unsigned> fmt::Pointer for AtomicMarkedPtr<T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.load(Ordering::SeqCst), f)
    }
}
