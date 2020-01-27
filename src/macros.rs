/// A macro for generating arbitrary documented code items
macro_rules! doc_comment {
    ($docs:expr, $($item:tt)*) => {
        #[doc = $docs]
        $($item)*
    };
}

macro_rules! doc_clear_tag {
    () => {
        "Clears the pointers tag value."
    };
}

macro_rules! impl_ptr_clone {
    () => {
        #[inline]
        fn clone(&self) -> Self {
            Self { inner: self.inner, _marker: PhantomData }
        }
    };
}

/// A macro for generating the inherent (const) implementation for a marked
/// pointer.
macro_rules! impl_ptr_const {
(
    $ptr_ident:ident
) => {
    /// Creates a new unmarked `null` pointer.
    #[inline]
    pub const fn null() -> Self {
        Self::new(ptr::null_mut())
    }

    /// Creates a new unmarked pointer.
    #[inline]
    pub const fn new(ptr: *mut T) -> Self {
        Self { inner: ptr, _marker: PhantomData }
    }

    /// Creates a new pointer from the numeric (integer) representation of a
    /// potentially marked pointer.
    #[inline]
    pub const fn from_usize(val: usize) -> Self {
        Self::new(val as _)
    }

    /// Casts to a pointer of another type.
    #[inline]
    pub const fn cast<U>(self) -> $ptr_ident<U, N> {
        $ptr_ident::new(self.inner.cast())
    }

    /// Returns the internal representation of the pointer *as is*, i.e. any
    /// potential tag value is **not** stripped.
    #[inline]
    pub const fn into_raw(self) -> *mut T {
        self.inner
    }

    /// Returns the numeric (integer) representation of the pointer with its tag
    /// value.
    #[inline]
    pub fn into_usize(self) -> usize {
        self.inner as usize
    }
};
}

macro_rules! impl_ptr_non_const {
    (
    tag_bits = $tag_bits:expr,
    tag_type = $tag_type:ty,
) => {
        /// The number of available tag bits for this type.
        pub const TAG_BITS: $tag_type = $tag_bits;
        /// The bitmask for the lower bits available for storing the tag value.
        pub const TAG_MASK: usize = crate::mark_mask::<T>(Self::TAG_BITS);
        /// The bitmask for the (higher) bits for storing the pointer itself.
        pub const POINTER_MASK: usize = !Self::TAG_MASK;

        /// Returns `true` if the marked pointer is `null`.
        #[inline]
        pub fn is_null(self) -> bool {
            self.decompose_ptr().is_null()
        }

        #[doc = doc_clear_tag!()]
        #[inline]
        pub fn clear_tag(self) -> Self {
            Self::new(self.decompose_ptr())
        }
    };
}

// a macro for generating the `Debug` implementation for atomic pointers.
macro_rules! impl_atomic_debug {
    ($type_name:expr) => {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let (ptr, tag) = self.load(Ordering::SeqCst).decompose();
            f.debug_struct($type_name).field("ptr", &ptr).field("tag", &tag).finish()
        }
    };
}

macro_rules! impl_debug {
    ($type_name:expr) => {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let (ptr, tag) = self.decompose();
            f.debug_struct($type_name).field("ptr", &ptr).field("tag", &tag).finish()
        }
    };
}

macro_rules! impl_default {
    () => {
        #[inline]
        fn default() -> Self {
            Self::null()
        }
    };
}

macro_rules! impl_from_raw {
    ($ptr:ty) => {
        #[inline]
        fn from(ptr: $ptr) -> Self {
            Self::new(ptr as _)
        }
    };
}

macro_rules! impl_from_reference {
    ($reference:ty) => {
        #[inline]
        fn from(reference: $reference) -> Self {
            Self::from(reference as *const _)
        }
    };
}

macro_rules! impl_from_non_null {
    () => {
        #[inline]
        fn from(non_null: NonNull<T>) -> Self {
            Self::new(non_null.as_ptr())
        }
    };
}

macro_rules! impl_pointer {
    () => {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Pointer::fmt(&self.decompose_ptr(), f)
        }
    };
}

// implementation for (nullable) marked pointer types
macro_rules! impl_atomic_marked_ptr {
(
    atomic_name = $atomic_name:ident,
    atomic_type = $atomic_type:ty,
    atomic_bounds = { $($atomic_bounds:tt)* },
    regular_name = $regular_name:ident,
    regular_type = $regular_type:ty,
    tag_type = $tag_type:ty,
    tag_bits = $tag_bits:expr,
    load_example = $load_example:expr,
    store_example = $store_example:expr,
    swap_example = $swap_example:expr,
    cas_example = $cas_example:expr,
    cex_example = $cex_example:expr,
    cex_weak_example = $cex_weak_example:expr,
    fadd_example = $fadd_example:expr,
    fsub_example = $fsub_example:expr,
    fand_example = $fand_example:expr,
    fnand_example = $fnand_example:expr,
    for_example = $for_example:expr,
    fxor_example = $fxor_example:expr
) => {
/********** impl Send + Sync **********************************************************************/

unsafe impl<$($atomic_bounds)*> Send for $atomic {}
unsafe impl<$($atomic_bounds)*> Sync for $atomic {}

/********** impl inherent *************************************************************************/

impl<$($atomic_bounds)*> $atomic_type {
    /// The number of available tag bits for this type.
    pub const TAG_BITS: $tag_type = $tag_bits;
    /// The bitmask for the lower bits available for storing the tag value.
    pub const MARK_MASK: usize = crate::mark_mask::<T>(Self::TAG_BITS);
    /// The bitmask for the bits reserved for storing the pointer itself.
    pub const POINTER_MASK: usize = !Self::MARK_MASK;

    /// Creates a new and unmarked `null` pointer.
    #[inline]
    pub const fn null() -> Self {
        Self { inner: AtomicUsize::new(0), _marker: PhantomData }
    }

    doc_comment! {
        concat!("Creates a new [`", stringify!($atomic_name), "`]."),
        #[inline]
        pub fn new(marked_ptr: $regular_type) -> Self {
            Self { inner: AtomicUsize::new(marked_ptr.into_usize(), _marked: PhantomdData) }
        }
    }

    /// Consumes `self` and returns the contained marked pointer.
    ///
    /// This is safe because passing self by value guarantees that no other
    /// threads are concurrently accessing the atomic data.
    pub fn into_inner(self) -> $regular_type {
        $regular_name::from_usize(self.inner.into_inner())
    }

    /// Returns a mutable reference to the underlying marked pointer.
    ///
    /// This is safe because the mutable reference guarantees that no other
    /// threads are concurrently accessing the atomic data.
    #[inline]
    pub fn get_mut(&mut self) -> &mut $regular_type {
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
    ///
    /// # Examples
    ///
    #[doc = $load_example]
    #[inline]
    pub fn load(&self, order: Ordering) -> $regular {
        $regular_name::from_usize(self.inner.load(order))
    }

    /// Stores a value into the atomic marked pointer.
    ///
    /// `store` takes an [`Ordering`] argument which describes the
    /// memory ordering of this operation.
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
    ///
    /// # Examples
    ///
    #[doc = $store_example]
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
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    ///
    /// # Examples
    ///
    #[doc = $swap_example]
    #[inline]
    pub fn swap(&self, ptr: MarkedPtr<T, N>, order: Ordering) -> MarkedPtr<T, N> {
        $regular_name::from_usize(self.inner.swap(ptr.into_usize(), order))
    }

    /// Stores a value into the pointer if the current value is the same
    /// as `current`.
    ///
    /// The return value is always the previous value.
    /// If it is equal to `current`, then the value was updated.
    ///
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
    ///
    /// # Examples
    ///
    #[doc = $cas_example]
    #[inline]
    pub fn compare_and_swap(
        &self,
        current: $regular_type,
        new: $regular_type,
        order: Ordering,
    ) -> $regular_type {
        $regular_name::from_usize(self.inner.compare_and_swap(
            current.into_usize(),
            new.into_usize(),
            order,
        ))
    }

    /// Stores a value into the pointer if the current value is the same
    /// as `current`.
    ///
    /// The return value is a result indicating whether the new value was
    /// written and containing the previous value.
    /// On success this value is guaranteed to be equal to `current`.
    ///
    /// `compare_exchange` takes two [`Ordering`] arguments to describe the
    /// memory ordering of this operation.
    /// The first describes the required ordering if the operation succeeds
    /// while the second describes the required ordering when the operation
    /// fails.
    /// Using [`Acquire`][acq] as success ordering makes the store part of this
    /// operation [`Relaxed`][rlx], and using [`Release`][rel] makes the
    /// successful load [`Relaxed`][rlx].
    /// The failure ordering can only be [`SeqCst`][seq_cst], [`Acquire`][acq]
    /// or [`Relaxed`][rlx] and must be equivalent to or weaker than the success
    /// ordering.
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    /// [seq_cst]: Ordering::SeqCst
    ///
    /// # Examples
    ///
    #[doc = $cex_example]
    #[inline]
    pub fn compare_exchange(
        &self,
        current: $regular_type,
        new: $regular_type,
        success: Ordering,
        failure: Ordering,
    ) -> Result<$regular_type, $regular_type> {
        self.inner
            .compare_exchange(current.into_usize(), new.into_usize(), success, failure)
            .map(|_| current)
            .map_err($regular_name::from_usize)
    }

    /// Stores a value into the pointer if the current value is the same
    /// as `current`.
    ///
    /// Unlike `compare_exchange`, this function is allowed to spuriously fail
    /// even when the comparison succeeds, which can result in more efficient
    /// code on some platforms.
    /// The return value is a result indicating whether the new value was
    /// written and containing the previous value.
    ///
    /// `compare_exchange_weak` takes two [`Ordering`] arguments to describe the
    /// memory ordering of this operation.
    /// The first describes the required ordering if the operation succeeds
    /// while the second describes the required ordering when the operation
    /// fails.
    /// Using [`Acquire`][acq] as success ordering makes the store part of this
    /// operation [`Relaxed`][rlx], and using [`Release`][rel] makes the
    /// successful load [`Relaxed`][rlx].
    /// The failure ordering can only be [`SeqCst`][seq_cst], [`Acquire`][acq]
    /// or [`Relaxed`][rlx] and must be equivalent to or weaker than the success
    /// ordering.
    ///
    /// [rlx]: Ordering::Relaxed
    /// [acq]: Ordering::Acquire
    /// [rel]: Ordering::Release
    /// [seq_cst]: Ordering::SeqCst
    ///
    /// # Examples
    ///
    #[doc = $cex_weak_example]
    #[inline]
    pub fn compare_exchange_weak(
        &self,
        current: $regular_type,
        new: $regular_type,
        success: Ordering,
        failure: Ordering,
    ) -> Result<$regular_type, $regular_type> {
        self.inner
            .compare_exchange_weak(current.into_usize(), new.into_usize(), success, failure)
            .map(|_| current)
            .map_err($regular_name::from_usize)
    }

    #[inline]
    pub fn fetch_add(&self, value: $tag_type, order: Ordering) -> $regular_type {
        let value = Self::calculate_tag_bits(value);
        $regular_name::from_usize(self.inner.fetch_add(value, order))
    }

    #[inline]
    pub fn fetch_sub(&self, value: $tag_type, order: Ordering) -> $regular_type {
        let value = Self::calculate_tag_bits(value);
        $regular_name::from_usize(self.inner.fetch_sub(value, order))
    }

    #[inline]
    pub fn fetch_and(&self, value: $tag_type, order: Ordering) -> $regular_type {
        let value = Self::calculate_tag_bits(value);
        $regular_name::from_usize(self.inner.fetch_and(value, order))
    }

    #[inline]
    pub fn fetch_nand(&self, value: $tag_type, order: Ordering) -> $regular_type {
        let value = Self::calculate_tag_bits(value);
        $regular_name::from_usize(self.inner.fetch_nand(value, order))
    }

    #[inline]
    pub fn fetch_or(&self, value: $tag_type, order: Ordering) -> $regular_type {
        let value = Self::calculate_tag_bits(value);
        $regular_name::from_usize(self.inner.fetch_or(value, order))
    }

    #[inline]
    pub fn fetch_xor(&self, value: $tag_type, order: Ordering) -> $regular_type {
        let value = Self::calculate_tag_bits(value);
        $regular_name::from_usize(self.inner.fetch_xor(value, order))
    }
}

/********** impl Debug ****************************************************************************/

impl<$($atomic_bounds)*> fmt::Debug for $atomic_type {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (ptr, tag) = self.load(Ordering::SeqCst).decompose();
        f.debug_struct(stringify!($atomic_name))
            .field("ptr", &ptr)
            .field("tag", &tag)
            .finish()
    }
}

/********** impl Default **************************************************************************/

impl<$($atomic_bounds)*> Default for $atomic_type {
    #[inline]
    fn default() -> Self {
        Self::null()
    }
}

/********** impl From *****************************************************************************/

impl<$($atomic_bounds)*> From<$regular_type> for $atomic_type {
    #[inline]
    fn from(marked_ptr: $regular_type) -> Self {
        Self::new(marked_ptr)
    }
}
};
}
