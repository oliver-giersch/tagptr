macro_rules! impl_atomic_inherent_const {
    () => {
        doc_comment! {
            doc_null!(),
            pub const fn null() -> Self {
                Self { inner: AtomicUsize::new(0), _marker: PhantomData }
            }
        }
    };
}

macro_rules! impl_atomic_inherent {
    (ptr_type = $ptr_type:ty) => {
        doc_comment! {
            doc_load!(),
            #[inline]
            pub fn load(&self, order: Ordering) -> $ptr_type {
                todo!()
            }
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

macro_rules! impl_atomic_pointer {
    () => {
        #[inline]
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Pointer::fmt(&self.load(Ordering::SeqCst), f)
        }
    }
}
