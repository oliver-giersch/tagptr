macro_rules! impl_non_null_inherent_const {
    (
        ptr_type = $ptr_type:ty,
        ptr_ident = $ptr_ident:ident
    ) => {
        /// Creates a new marked non-null pointer from `marked_ptr` without
        /// checking for `null`.
        ///
        /// # Safety
        ///
        /// The caller has to ensure that `marked_ptr` is not `null`.
        #[inline]
        pub const unsafe fn new_unchecked(marked_ptr: $ptr_type) -> Self {
            Self {
                inner: NonNull::new_unchecked(marked_ptr.inner),
                _marker: PhantomData
            }
        }

        doc_comment! {
            doc_from_usize!(),
            /// Safety
            ///
            /// The caller has to ensure that `val` represents neither a marked nor an unmarked
            /// `null` pointer.
            #[inline]
            pub const unsafe fn from_usize(val: usize) -> Self {
                Self { inner: NonNull::new_unchecked(val as *mut _), _marker: PhantomData }
            }
        }

        doc_comment! {
            doc_into_raw!(),
            #[inline]
            pub const fn into_raw(self) -> NonNull<T> {
                self.inner
            }
        }

        #[inline]
        pub const fn into_marked_ptr(self) -> $ptr_type {
            $ptr_ident::new(self.inner.as_ptr())
        }

        doc_comment! {
            doc_into_usize!(),
            #[inline]
            pub fn into_usize(self) -> usize {
                self.inner.as_ptr() as _
            }
        }
    };
}

macro_rules! impl_non_null_inherent {
    (
        self_ident = $self_ident:ident,
        ptr_type = $ptr_type:ty,
        tag_type = $tag_type:ty,
        example_type_path = $example_type_path:path
    ) => {
        /// Creates a new non-null pointer from `marked_ptr`.
        ///
        /// # Errors
        ///
        /// This function fails if `marked_ptr` is `null` in which case a
        /// [`Null`] instance is returned containing argument pointer's tag
        /// value.
        #[inline]
        pub fn new(marked_ptr: $ptr_type) -> Result<Self, Null> {
            todo!()
        }

        doc_comment! {
            doc_clear_tag!("non-null" $example_type_path),
            #[inline]
            pub fn clear_tag(self) -> Self {
                Self { inner: self.decompose_non_null(), _marker: PhantomData }
            }
        }

        doc_comment! {
            doc_split_tag!("non-null" $example_type_path),
            #[inline]
            pub fn split_tag(self) -> (Self, $tag_type) {
                let (inner, tag) = self.decompose();
                (Self { inner, _marker: PhantomData }, tag)
            }
        }

        doc_comment! {
            doc_add_tag!(),
            #[inline]
            pub unsafe fn add_tag(self, value: usize) -> Self {
                Self::from_usize(self.into_usize().wrapping_add(value))
            }
        }

        doc_comment! {
            doc_sub_tag!(),
            #[inline]
            pub unsafe fn sub_tag(self, value: usize) -> Self {
                Self::from_usize(self.into_usize().wrapping_sub(value))
            }
        }

        doc_comment! {
            doc_decompose!(),
            #[inline]
            pub fn decompose(self) -> (NonNull<T>, $tag_type) {
                (self.decompose_non_null(), self.decompose_tag())
            }
        }

        doc_comment! {
            doc_as_ref!("bounded"),
            #[inline]
            pub unsafe fn as_ref(&self) -> &T {
                &*self.decompose_non_null().as_ptr()
            }
        }

        doc_comment! {
            doc_as_ref!("unbounded"),
            #[inline]
            pub unsafe fn as_ref_unbounded<'a>(self) -> &'a T {
                &*self.decompose_non_null().as_ptr()
            }
        }

        doc_comment! {
            doc_as_mut!($self_ident),
            #[inline]
            pub unsafe fn as_mut(&mut self) -> &mut T {
                self.as_mut_unbounded()
            }
        }

        doc_comment! {
            doc_as_mut!($self_ident),
            #[inline]
            pub unsafe fn as_mut_unbounded<'a>(self) -> &'a mut T {
                &mut *self.decompose_non_null().as_ptr()
            }
        }

        doc_comment! {
            doc_decompose_ref!($self_ident),
            #[inline]
            pub unsafe fn decompose_ref(&self) -> (&T, $tag_type) {
                let (ptr, tag) = self.decompose();
                (&*ptr.as_ptr(), tag)
            }
        }

        doc_comment! {
            doc_decompose_mut!($self_ident),
            #[inline]
            pub unsafe fn decompose_mut(&mut self) -> (&mut T, $tag_type) {
                let (ptr, tag) = self.decompose();
                (&mut *ptr.as_ptr(), tag)
            }
        }
    };
}
