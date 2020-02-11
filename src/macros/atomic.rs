macro_rules! impl_atomic_inherent_const {
    (ptr_type = $ptr_type:ty, ptr_ident = $ptr_ident:ident) => {
        doc_comment! {
            doc_null!(),
            pub const fn null() -> Self {
                Self { inner: AtomicUsize::new(0), _marker: PhantomData }
            }
        }

        #[inline]
        pub fn new(marked_ptr: $ptr_type) -> Self {
            Self { inner: AtomicUsize::new(marked_ptr.into_usize()), _marker: PhantomData }
        }

        #[inline]
        pub fn into_inner(self) -> $ptr_type {
            $ptr_ident::from_usize(self.inner.into_inner())
        }

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

        #[inline]
        pub fn store(&self, ptr: $ptr_type, order: Ordering) {
            self.inner.store(ptr.into_usize(), order)
        }

        #[inline]
        pub fn swap(&self, ptr: $ptr_type, order: Ordering) -> $ptr_type {
            $ptr_ident::from_usize(self.inner.swap(ptr.into_usize(), order))
        }
    };
}

macro_rules! impl_atomic_inherent {
    (ptr_type = $ptr_type:ty, ptr_ident = $ptr_ident:ident) => {};
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

macro_rules! impl_atomic_pointer {
    () => {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Pointer::fmt(&self.load(Ordering::SeqCst), f)
        }
    }
}
