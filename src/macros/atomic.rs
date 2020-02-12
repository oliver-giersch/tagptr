macro_rules! impl_atomic_inherent_const {
    (ptr_type = $ptr_type:ty, ptr_ident = $ptr_ident:ident) => {
        doc_comment! {
            doc_null!(),
            pub const fn null() -> Self {
                Self { inner: AtomicUsize::new(0), _marker: PhantomData }
            }
        }

        /// Creates a new atomic marked pointer.
        #[inline]
        pub fn new(marked_ptr: $ptr_type) -> Self {
            Self { inner: AtomicUsize::new(marked_ptr.into_usize()), _marker: PhantomData }
        }

        /// Consumes the atomic marked pointer and returns its contained value.
        ///
        /// This is safe because passing `self` by value guarantees no other
        /// threads are concurrently accessing the atomic pointer.
        #[inline]
        pub fn into_inner(self) -> $ptr_type {
            $ptr_ident::from_usize(self.inner.into_inner())
        }

        /// Returns a mutable reference to the underlying marked pointer.
        ///
        /// This is safe because the mutable reference guarantees no other
        /// threads are concurrently accessing the atomic pointer.
        #[inline]
        pub fn get_mut(&mut self) -> &mut $ptr_type {
            unsafe { &mut *(self.inner.get_mut() as *mut usize as *mut _) }
        }

        doc_comment! {
            doc_load!(),
            #[inline]
            pub fn load(&self, order: Ordering) -> $ptr_type {
                $ptr_ident::from_usize(self.inner.load(order))
            }
        }

        doc_comment! {
            doc_store!(),
            #[inline]
            pub fn store(&self, ptr: $ptr_type, order: Ordering) {
                self.inner.store(ptr.into_usize(), order)
            }
        }

        #[inline]
        pub fn swap(&self, ptr: $ptr_type, order: Ordering) -> $ptr_type {
            $ptr_ident::from_usize(self.inner.swap(ptr.into_usize(), order))
        }

        #[inline]
        pub fn compare_and_swap(
            &self,
            current: $ptr_type,
            new: $ptr_type,
            order: Ordering
        ) -> $ptr_type {
            $ptr_ident::from_usize(self.inner.compare_and_swap(
                current.into_usize(),
                new.into_usize(),
                order
            ))
        }

        #[inline]
        pub fn compare_exchange(
            &self,
            current: $ptr_type,
            new: $ptr_type,
            success: Ordering,
            failure: Ordering
        ) -> Result<$ptr_type, $ptr_type> {
            self.inner
                .compare_exchange(current.into_usize(), new.into_usize(), success, failure)
                .map(|_| current)
                .map_err($ptr_ident::from_usize)
        }

        #[inline]
        pub fn compare_exchange_weak(
            &self,
            current: $ptr_type,
            new: $ptr_type,
            success: Ordering,
            failure: Ordering
        ) -> Result<$ptr_type, $ptr_type> {
            self.inner
                .compare_exchange_weak(current.into_usize(), new.into_usize(), success, failure)
                .map(|_| current)
                .map_err($ptr_ident::from_usize)
        }
    };
}

macro_rules! impl_atomic_inherent {
(
    ptr_type = $ptr_type:ty,
    ptr_ident = $ptr_ident:ident,
    tag_type = $tag_type:ty,
    example_type_path = $example_type_path:path
) => {
        doc_comment! {
            doc_fetch_add!("`fetch_add`", $example_type_path),
            #[inline]
            pub fn fetch_add(&self, value: $tag_type, order: Ordering) -> $ptr_type {
                todo!()
            }
        }

        #[inline]
        pub fn fetch_sub(&self, value: $tag_type, order: Ordering) -> $ptr_type {
            todo!()
        }

        #[inline]
        pub fn fetch_or(&self, value: $tag_type, order: Ordering) -> $ptr_type {
            todo!()
        }

        #[inline]
        pub fn fetch_xor(&self, value: $tag_type, order: Ordering) -> $ptr_type {
            todo!()
        }

        #[inline]
        pub fn fetch_and(&self, value: $tag_type, order: Ordering) -> $ptr_type {
            todo!()
        }

        #[inline]
        pub fn fetch_nand(&self, value: $tag_type, order: Ordering) -> $ptr_type {
            todo!()
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

macro_rules! impl_atomic_from_raw {
    ($ptr:ty) => {
        #[inline]
        fn from(ptr: $ptr) -> Self {
            Self::new(ptr.into())
        }
    };
}

macro_rules! impl_atomic_pointer {
    () => {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Pointer::fmt(&self.load(Ordering::SeqCst), f)
        }
    }
}
