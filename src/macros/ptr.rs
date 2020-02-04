macro_rules! impl_ptr_inherent_const {
    (example_type_path = $example_type_path:path) => {
        doc_comment! {
            doc_null!($example_type_path),
            #[inline]
            pub const fn null() -> Self {
                Self::new(ptr::null_mut())
            }
        }

        doc_comment! {
            doc_new!($example_type_path),
            #[inline]
            pub const fn new(ptr: *mut T) -> Self {
                Self { inner: ptr, _marker: PhantomData }
            }
        }

        doc_comment! {
            doc_from_usize!("nullable", $example_type_path),
            #[inline]
            pub const fn from_usize(val: usize) -> Self {
                Self::new(val as _)
            }
        }

        doc_comment! {
            doc_into_raw!(),
            #[inline]
            pub const fn into_raw(self) -> *mut T {
                self.inner
            }
        }

        doc_comment! {
            doc_into_usize!(),
            #[inline]
            pub fn into_usize(self) -> usize {
                self.inner as usize
            }
        }
    };
}

/// A macro for implementing non-const (trait bound) inherent methods and constants for marked
/// pointers.
macro_rules! impl_ptr_inherent {
    (
        ty_ident = $ty_ident:ident,
        tag_type = $tag_type:ty,
        example_type_path = $example_type_path:path
    ) => {
        doc_comment! {
            doc_is_null!(),
            #[inline]
            pub fn is_null(self) -> bool {
                self.decompose_ptr().is_null()
            }
        }

        doc_comment! {
            doc_clear_tag!($example_type_path),
            #[inline]
            pub fn clear_tag(self) -> Self {
                Self::new(self.decompose_ptr())
            }
        }

        doc_comment! {
            doc_split_tag!($example_type_path),
            #[inline]
            pub fn split_tag(self) -> (Self, $tag_type) {
                let (ptr, tag) = self.decompose();
                (Self::new(ptr), tag)
            }
        }

        doc_comment! {
            doc_set_tag!($example_type_path),
            #[inline]
            pub fn set_tag(self, tag: $tag_type) -> Self {
                let ptr = self.decompose_ptr();
                Self::compose(ptr, tag)
            }
        }

        doc_comment! {
            doc_update_tag!($example_type_path),
            #[inline]
            pub fn update_tag(self, func: impl FnOnce($tag_type) -> $tag_type) -> Self {
                let (ptr, tag) = self.decompose();
                Self::compose(ptr, func(tag))
            }
        }

        doc_comment! {
            doc_add_tag!(),
            #[inline]
            pub fn add_tag(self, value: usize) -> Self {
                Self::from_usize(self.into_usize().wrapping_add(value))
            }
        }

        doc_comment! {
            doc_sub_tag!(),
            #[inline]
            pub fn sub_tag(self, value: usize) -> Self {
                Self::from_usize(self.into_usize().wrapping_sub(value))
            }
        }

        doc_comment! {
            doc_decompose!(),
            #[inline]
            pub fn decompose(self) -> (*mut T, $tag_type) {
                (self.decompose_ptr(), self.decompose_tag())
            }
        }

        doc_comment! {
            doc_as_ref!("unbounded"),
            #[inline]
            pub unsafe fn as_ref<'a>(self) -> Option<&'a T> {
                self.decompose_ptr().as_ref()
            }
        }

        doc_comment! {
            doc_as_mut!($ty_ident),
            #[inline]
            pub unsafe fn as_mut<'a>(self) -> Option<&'a mut T> {
                self.decompose_ptr().as_mut()
            }
        }

        doc_comment! {
            doc_decompose_ref!($ty_ident),
            #[inline]
            pub unsafe fn decompose_ref<'a>(self) -> (Option<&'a T>, $tag_type) {
                (self.as_ref(), self.decompose_tag())
            }
        }

        doc_comment! {
            doc_decompose_mut!($ty_ident),
            #[inline]
            pub unsafe fn decompose_mut<'a>(self) -> (Option<&'a mut T>, $tag_type) {
                (self.as_mut(), self.decompose_tag())
            }
        }
    };
}

/// A macro for implementing `From` from raw pointer types.
macro_rules! impl_from_raw {
    ($ptr:ty) => {
        #[inline]
        fn from(ptr: $ptr) -> Self {
            Self::new(ptr as _)
        }
    };
}

/// A macro for implementing `From` from reference types.
macro_rules! impl_ptr_from_reference {
    ($reference:ty) => {
        #[inline]
        fn from(reference: $reference) -> Self {
            Self::from(reference as *const _)
        }
    };
}

/// A macro for implementing `From` from `NonNull` pointers.
macro_rules! impl_ptr_from_non_null {
    () => {
        #[inline]
        fn from(non_null: NonNull<T>) -> Self {
            Self::new(non_null.as_ptr())
        }
    };
}
