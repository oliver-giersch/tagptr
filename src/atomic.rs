use core::fmt;
use core::marker::PhantomData;
use core::mem;
use core::sync::atomic::{AtomicUsize, Ordering};

use typenum::Unsigned;

use crate::{AtomicMarkedPtr, MarkedPtr};

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedPtr
////////////////////////////////////////////////////////////////////////////////////////////////////

/********** impl Send + Sync **********************************************************************/

unsafe impl<T, N> Send for AtomicMarkedPtr<T, N> {}
unsafe impl<T, N> Sync for AtomicMarkedPtr<T, N> {}

/********** impl Default **************************************************************************/

impl<T, N> Default for AtomicMarkedPtr<T, N> {
    #[inline]
    fn default() -> Self {
        Self::null()
    }
}

/********** impl inherent (const) *****************************************************************/

impl<T, N> AtomicMarkedPtr<T, N> {
    /// Creates a new and unmarked `null` pointer.
    #[inline]
    pub const fn null() -> Self {
        Self {
            inner: AtomicUsize::new(0),
            _marker: PhantomData,
        }
    }

    /// Creates a new [`AtomicMarkedPtr`].
    #[inline]
    pub fn new(marked_ptr: MarkedPtr<T, N>) -> Self {
        Self {
            inner: AtomicUsize::new(marked_ptr.into_usize()),
            _marker: PhantomData,
        }
    }

    /// Consumes `self` and returns the inner [`MarkedPtr`].
    #[inline]
    pub fn into_inner(self) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.into_inner())
    }

    /// Returns a mutable reference to the underlying [`MarkedPtr`].
    ///
    /// This is safe because the mutable reference guarantees that no other
    /// threads are concurrently accessing the atomic data.
    #[inline]
    pub fn get_mut(&mut self) -> &mut MarkedPtr<T, N> {
        unsafe { &mut *(self.inner.get_mut() as *mut usize as *mut _) }
    }

    /// Loads a value from the pointer.
    ///
    /// `load` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation. Possible values are [`SeqCst`][seq_cst],
    /// [`Acquire`][acq] and [`Relaxed`][rlx].
    ///
    /// # Panics
    ///
    /// Panics if `order` is [`Release`][rel] or [`AcqRel`][acq_rel].
    ///
    /// [rlx]: core::sync::atomic::Ordering::Relaxed
    /// [acq]: core::sync::atomic::Ordering::Acquire
    /// [rel]: core::sync::atomic::Ordering::Release
    /// [acq_rel]: core::sync::atomic::Ordering::AcqRel
    /// [seq_cst]: core::sync::atomic::Ordering::SeqCst
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::atomic::Ordering;
    ///
    /// type MarkedPtr<T> = conquer_pointer::MarkedPtr<T, conquer_pointer::typenum::U1>;
    /// type AtomicMarkedPtr<T> = conquer_pointer::AtomicMarkedPtr<T, conquer_pointer::typenum::U1>;
    ///
    /// let atomic = AtomicMarkedPtr::new(MarkedPtr::compose(&mut 5, 0b1));
    ///
    /// let load = atomic.load(Ordering::SeqCst);
    /// assert_eq!((Some(&mut 5), 0b1), unsafe { load.decompose_mut() });
    /// ```
    #[inline]
    pub fn load(&self, order: Ordering) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.load(order))
    }

    /// Stores a value into the pointer.
    ///
    /// `store` takes an [`Ordering`] argument which describes the
    /// memory ordering of this operation. Possible values are
    /// [`SeqCst`][seq_cst], [`Release`][rel] and [`Relaxed`][rlx].
    ///
    /// # Panics
    ///
    /// Panics if `order` is [`Acquire`][acq] or [`AcqRel`][acq_rel].
    ///
    /// [rlx]: core::sync::atomic::Ordering::Relaxed
    /// [acq]: core::sync::atomic::Ordering::Acquire
    /// [rel]: core::sync::atomic::Ordering::Release
    /// [acq_rel]: core::sync::atomic::Ordering::AcqRel
    /// [seq_cst]: core::sync::atomic::Ordering::SeqCst
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::atomic::Ordering;
    ///
    /// type MarkedPtr<T> = conquer_pointer::MarkedPtr<T, reclaim::typenum::U0>;
    /// type AtomicMarkedPtr<T> = conquer_pointer::AtomicMarkedPtr<T, reclaim::typenum::U0>;
    ///
    /// let atomic = AtomicMarkedPtr::null();
    ///
    /// let store = MarkedPtr::new(&mut 10);
    ///
    /// atomic.store(store, Ordering::SeqCst);
    /// ```
    #[inline]
    pub fn store(&self, ptr: MarkedPtr<T, N>, order: Ordering) {
        self.inner.store(ptr.into_usize(), order);
    }

    /// Stores a value into the pointer, returning the previous value.
    ///
    /// `swap` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation.
    /// All ordering modes are possible.
    /// Note that using [`Acquire`][acq] makes the store part of this operation
    /// [`Relaxed`][rlx], and using [`Release`][rel] makes the load part
    /// [`Relaxed`][rlx].
    ///
    /// [rlx]: core::sync::atomic::Ordering::Relaxed
    /// [acq]: core::sync::atomic::Ordering::Acquire
    /// [rel]: core::sync::atomic::Ordering::Release
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::atomic::Ordering;
    ///
    /// type MarkedPtr<T> = conquer_pointer::MarkedPtr<T, typenum::U0>;
    /// ```
    #[inline]
    pub fn swap(&self, ptr: MarkedPtr<T, N>, order: Ordering) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.swap(ptr.into_usize(), order))
    }

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
            .map(MarkedPtr::from_usize)
            .map_err(MarkedPtr::from_usize)
    }

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
            .map(MarkedPtr::from_usize)
            .map_err(MarkedPtr::from_usize)
    }

    /// # Note
    ///
    /// Since fetch-and-add increments the entire memory word and is infallible,
    /// it may be impossible to guarantee that the tag value can not overflow
    /// into the pointer, which would corrupt both values and lead to undefined
    /// behaviour, when the pointer is de-referenced.
    #[inline]
    pub fn fetch_add(&self, value: usize, order: Ordering) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.fetch_add(value, order))
    }

    #[inline]
    pub fn fetch_sub(&self, value: usize, order: Ordering) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.fetch_sub(value, order))
    }

    /// Bitwise `and` with the current tag value.
    ///
    /// Performs a bitwise `and` operation on the current tag and the argument `value` and sets the
    /// new value to the result.
    ///
    /// Returns the [`MarkedPtr`] with the previous tag, the pointer itself can not change.
    /// It `value` is larger than the mask of markable bits of this type it is silently truncated.
    ///
    /// `fetch_and` takes an [`Ordering`] argument, which describes the memory ordering of this
    /// operation.
    /// All orderings modes are possible.
    /// Note, that using [`Acquire`][acq] makes the store part of this operation [`Relaxed`][rlx]
    /// and using [`Release`][rel] makes the load part [`Relaxed][rlx]
    ///
    /// [acq]: core::sync::atomic::Ordering::Acquire
    /// [rel]: core::sync::atomic::Ordering::Release
    /// [rlx]: core::sync::atomic::Ordering::Relaxed
    #[inline]
    pub fn fetch_and(&self, value: usize, order: Ordering) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.fetch_and(value, order))
    }

    /// Bitwise `nand` with the current tag value.
    ///
    /// Performs a bitwise `nand` operation on the current tag and the argument `value` and sets the
    /// new value to the result.
    ///
    /// Returns the [`MarkedPtr`] with the previous tag, the pointer itself can not change.
    /// It `value` is larger than the mask of markable bits of this type it is silently truncated.
    ///
    /// `fetch_nand` takes an [`Ordering`] argument, which describes the memory ordering of this
    /// operation.
    /// All orderings modes are possible.
    /// Note, that using [`Acquire`][acq] makes the store part of this operation [`Relaxed`][rlx]
    /// and using [`Release`][rel] makes the load part [`Relaxed][rlx]
    ///
    /// [acq]: core::sync::atomic::Ordering::Acquire
    /// [rel]: core::sync::atomic::Ordering::Release
    /// [rlx]: core::sync::atomic::Ordering::Relaxed
    #[inline]
    pub fn fetch_nand(&self, value: usize, order: Ordering) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.fetch_nand(value, order))
    }

    /// Bitwise `or` with the current tag value.
    ///
    /// Performs a bitwise `or` operation on the current tag and the argument `value` and sets the
    /// new value to the result.
    ///
    /// Returns the [`MarkedPtr`] with the previous tag, the pointer itself can not change.
    /// It `value` is larger than the mask of markable bits of this type it is silently truncated.
    ///
    /// `fetch_or` takes an [`Ordering`] argument, which describes the memory ordering of this
    /// operation.
    /// All orderings modes are possible.
    /// Note, that using [`Acquire`][acq] makes the store part of this operation [`Relaxed`][rlx]
    /// and using [`Release`][rel] makes the load part [`Relaxed][rlx]
    ///
    /// [acq]: core::sync::atomic::Ordering::Acquire
    /// [rel]: core::sync::atomic::Ordering::Release
    /// [rlx]: core::sync::atomic::Ordering::Relaxed
    #[inline]
    pub fn fetch_or(&self, value: usize, order: Ordering) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.fetch_or(value, order))
    }

    /// Bitwise `xor` with the current tag value.
    ///
    /// Performs a bitwise `xor` operation on the current tag and the argument `value` and sets the
    /// new value to the result.
    ///
    /// Returns the [`MarkedPtr`] with the previous tag, the pointer itself can not change.
    /// It `value` is larger than the mask of markable bits of this type it is silently truncated.
    ///
    /// `fetch_xor` takes an [`Ordering`] argument, which describes the memory ordering of this
    /// operation.
    /// All orderings modes are possible.
    /// Note, that using [`Acquire`][acq] makes the store part of this operation [`Relaxed`][rlx]
    /// and using [`Release`][rel] makes the load part [`Relaxed][rlx]
    ///
    /// [acq]: core::sync::atomic::Ordering::Acquire
    /// [rel]: core::sync::atomic::Ordering::Release
    /// [rlx]: core::sync::atomic::Ordering::Relaxed
    #[inline]
    pub fn fetch_xor(&self, value: usize, order: Ordering) -> MarkedPtr<T, N> {
        MarkedPtr::from_usize(self.inner.fetch_xor(value, order))
    }
}

impl<T, N: Unsigned> AtomicMarkedPtr<T, N> {
    /// The number of available mark bits for this type.
    pub const MARK_BITS: usize = N::USIZE;
    /// The bitmask for the lower markable bits.
    pub const MARK_MASK: usize = crate::mark_mask::<T>(Self::MARK_BITS);
    /// The bitmask for the (higher) pointer bits.
    pub const POINTER_MASK: usize = !Self::MARK_MASK;
}

/********** impl Debug ****************************************************************************/

impl<T, N: Unsigned> fmt::Debug for AtomicMarkedPtr<T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (ptr, tag) = self.load(Ordering::SeqCst).decompose();
        f.debug_struct("AtomicMarkedPtr")
            .field("ptr", &ptr)
            .field("tag", &tag)
            .finish()
    }
}

/********** impl From *****************************************************************************/

impl<T, N> From<MarkedPtr<T, N>> for AtomicMarkedPtr<T, N> {
    #[inline]
    fn from(marked_ptr: MarkedPtr<T, N>) -> Self {
        Self {
            inner: AtomicUsize::new(marked_ptr.into_usize()),
            _marker: PhantomData,
        }
    }
}
