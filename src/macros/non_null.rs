macro_rules! impl_non_null_inherent {
    () => {
        /// Creates a new dangling but well aligned pointer.
        #[inline]
        pub fn dangling() -> Self {
            todo!()
        }

        #[inline]
        pub fn new(marked_ptr: $ptr_type) -> Result<Self, Null> {
            todo!()
        }

        // compose is impl'd differently

        #[inline]
        pub fn into_usize(self) -> usize {
            todo!()
        }

        #[inline]
        pub fn clear_tag(self) -> Self {
            todo!()
        }
    }
}
