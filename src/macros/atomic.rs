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
