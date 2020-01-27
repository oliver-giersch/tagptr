//! Implementation for [`MarkedPtr`].

use core::cmp;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};

use crate::{MarkedNonNull, MarkedPtr};

/********** impl Clone ****************************************************************************/

impl<T, N> Clone for MarkedPtr<T, N> {
    impl_ptr_clone!();
}

/********** impl Copy *****************************************************************************/

impl<T, N> Copy for MarkedPtr<T, N> {}

/********** impl inherent (const) *****************************************************************/

impl<T, N> MarkedPtr<T, N> {
    impl_ptr_const!(MarkedPtr);
}

/********** impl inherent *************************************************************************/

// calculate_tag_bits(...) -> usize {}
// compose() -> Self
// decompose_ptr() -> *mut T
// decompose_tag() -> usize

impl<T, const N: usize> MarkedPtr<T, N> {
    /// The number of available tag bits for this type.
    pub const TAG_BITS: usize = N;
    /// The bitmask for the lower bits available for storing the tag value.
    pub const TAG_MASK: usize = crate::mark_mask::<T>(Self::TAG_BITS);
    /// The bitmask for the (higher) bits for storing the pointer itself.
    pub const POINTER_MASK: usize = !Self::TAG_MASK;

    /// Composes a new [`MarkedPtr`] from a raw `ptr` and a `tag` value.
    ///
    /// The supplied `ptr` is assumed to be well-aligned (i.e. has no tag bits
    /// set), so this function may lead to unexpected results when this is not
    /// the case.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    ///
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, 2>;
    ///
    /// let raw = &1 as *const i32 as *mut i32;
    /// let ptr = MarkedPtr::compose(raw, 0b11);
    /// assert_eq!(ptr.decompose(), (raw, 0b11));
    /// // any excess bits are silently truncated.
    /// let ptr = MarkedPtr::compose(raw, 0b101);
    /// assert_eq!(ptr.decompose(), (raw, 0b01));
    /// ```
    #[inline]
    pub fn compose(ptr: *mut T, tag: usize) -> Self {
        crate::assert_alignment::<T, N>();
        Self::new(crate::compose(ptr, tag, Self::TAG_BITS))
    }

    /// Returns `true` if the [`MarkedPtr`] is `null`.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    ///
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, 1>;
    ///
    /// // a `null` pointer with tag bits is still considered to be `null`.
    /// let ptr = MarkedPtr::compose(ptr::null_mut(), 0b1);
    /// assert!(ptr.is_null())
    /// ```
    #[inline]
    pub fn is_null(self) -> bool {
        self.decompose_ptr().is_null()
    }

    /// Clears the tag from `self` and returns the same pointer but stripped of
    /// its tag value.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    ///
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, 2>;
    ///
    /// let raw = &1 as *const i32 as *mut i32;
    /// let ptr = MarkedPtr::compose(raw, 0b11);
    /// assert_eq!(ptr.clear_tag().decompose(), (raw, 0));
    /// ```
    #[inline]
    pub fn clear_tag(self) -> Self {
        Self::new(self.decompose_ptr())
    }

    /// Splits the tag from `self` and returns the same pointer but stripped of
    /// its tag value and the separated tag.
    ///
    /// ```
    /// use core::ptr;
    ///
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, 2>;
    ///
    /// let raw = &1 as *const i32 as *mut i32;
    /// let ptr = MarkedPtr::compose(raw, 0b11);
    /// assert_eq!(ptr.split_tag(), (MarkedPtr::new(raw), 0b11));
    /// ```
    #[inline]
    pub fn split_tag(self) -> (Self, usize) {
        let (ptr, tag) = self.decompose();
        (Self::new(ptr), tag)
    }

    /// Clears the tag from `self` and returns the same pointer but with the
    /// previous tag replaced with `tag`.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::ptr;
    ///
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, 2>;
    ///
    /// let raw = &1 as *const i32 as *mut i32;
    /// let ptr = MarkedPtr::compose(raw, 0b11);
    /// assert_eq!(ptr.set_tag(0b10).decompose(), (raw, 0b10));
    /// ```
    #[inline]
    pub fn set_tag(self, tag: usize) -> Self {
        Self::compose(self.decompose_ptr(), tag)
    }

    /// Updates the tag value of `self` using `func`, which receives the current
    /// tag value as argument and returns the same pointer with the updated tag
    /// value.
    ///
    /// # Examples
    ///
    /// ```
    /// type MarkedPtr = conquer_pointer::MarkedPtr<i32, 2>;
    ///
    /// let ptr = MarkedPtr::compose(&mut 1, 0b11);
    /// let ptr = ptr.update_tag(|tag| tag - 1);
    /// assert_eq!(ptr.decompose_tag(), 0b10);
    /// ```
    #[inline]
    pub fn update_tag(self, func: impl FnOnce(usize) -> usize) -> Self {
        let (ptr, tag) = self.decompose();
        Self::compose(ptr, func(tag))
    }

    /// Adds `value` to the current tag *without* regard for the previous value.
    ///
    /// This method does not perform any checks, so it may overflow the tag
    /// bits, result in a pointer to a different value, a null pointer or an
    /// unaligned pointer.
    #[inline]
    pub fn add_tag(self, value: usize) -> Self {
        Self::from_usize(self.into_usize() + value)
    }

    /// Subtracts `value` from the current tag *without* regard for the previous
    /// value.
    ///
    /// This method does not perform any checks, so it may underflow the tag
    /// bits, result in a pointer to a different value, a null pointer or an
    /// unaligned pointer.
    #[inline]
    pub fn sub_tag(self, value: usize) -> Self {
        Self::from_usize(self.into_usize() - value)
    }

    /// Decomposes the [`MarkedPtr`], returning the separated raw pointer and
    /// its tag.
    #[inline]
    pub fn decompose(self) -> (*mut T, usize) {
        crate::decompose::<T>(self.inner as usize, Self::TAG_BITS)
    }

    /// Decomposes the [`MarkedPtr`], returning only the separated raw pointer.
    #[inline]
    pub fn decompose_ptr(self) -> *mut T {
        crate::decompose_ptr::<T>(self.inner as usize, Self::TAG_BITS)
    }

    /// Decomposes the [`MarkedPtr`], returning only the separated tag value.
    #[inline]
    pub fn decompose_tag(self) -> usize {
        crate::decompose_tag::<T>(self.inner as usize, Self::TAG_BITS)
    }

    /// Decomposes the marked pointer, returning an optional reference and the
    /// separated tag.
    ///
    /// In case the pointer stripped of its tag is null, [`None`] is returned as
    /// part of the tuple. Otherwise, the reference is wrapped in a [`Some`].
    ///
    /// # Safety
    ///
    /// While this method and its mutable counterpart are useful for
    /// null-safety, it is important to note that this is still an unsafe
    /// operation because the returned value could be pointing to invalid
    /// memory.
    ///
    /// Additionally, the lifetime 'a returned is arbitrarily chosen and does
    /// not necessarily reflect the actual lifetime of the data.
    #[inline]
    pub unsafe fn decompose_ref<'a>(self) -> (Option<&'a T>, usize) {
        (self.as_ref(), self.decompose_tag())
    }

    /// Decomposes the marked pointer returning an optional mutable reference
    /// and the separated tag.
    ///
    /// In case the pointer stripped of its tag is null, [`None`] is returned as
    /// part of the tuple. Otherwise, the mutable reference is wrapped in a
    /// [`Some`].
    ///
    /// # Safety
    ///
    /// As with [`decompose_ref`][MarkedPtr::decompose_ref], this is unsafe
    /// because it cannot verify the validity of the returned pointer, nor can
    /// it ensure that the lifetime `'a` returned is indeed a valid lifetime for
    /// the contained data.
    #[inline]
    pub unsafe fn decompose_mut<'a>(self) -> (Option<&'a mut T>, usize) {
        (self.as_mut(), self.decompose_tag())
    }

    /// Decomposes the marked pointer, returning an optional reference and
    /// discarding the tag.
    ///
    /// # Safety
    ///
    /// The same caveats as with [`decompose_ref`][MarkedPtr::decompose_ref]
    /// apply for this method as well.
    #[inline]
    pub unsafe fn as_ref<'a>(self) -> Option<&'a T> {
        self.decompose_ptr().as_ref()
    }

    /// Decomposes the marked pointer, returning an optional mutable reference
    /// and discarding the tag.
    ///
    /// # Safety
    ///
    /// The same caveats as with [`decompose_mut`][MarkedPtr::decompose_mut]
    /// apply for this method as well.
    #[inline]
    pub unsafe fn as_mut<'a>(self) -> Option<&'a mut T> {
        self.decompose_ptr().as_mut()
    }
}

/********** impl From (*mut T) ********************************************************************/

impl<T, const N: usize> From<*mut T> for MarkedPtr<T, N> {
    impl_from_raw!(*mut T);
}

/********** impl From (*const T) ******************************************************************/

impl<T, const N: usize> From<*const T> for MarkedPtr<T, N> {
    impl_from_raw!(*const T);
}

/********** impl From (&T) ************************************************************************/

impl<T, const N: usize> From<&T> for MarkedPtr<T, N> {
    impl_from_reference!(&T);
}

/********** impl From (&mut T) ********************************************************************/

impl<T, const N: usize> From<&mut T> for MarkedPtr<T, N> {
    impl_from_reference!(&mut T);
}

/********** impl From (NonNull) *******************************************************************/

impl<T, const N: usize> From<NonNull<T>> for MarkedPtr<T, N> {
    impl_from_non_null!();
}

/********** impl From (MarkedNonNull) *************************************************************/

impl<T, const N: usize> From<MarkedNonNull<T, N>> for MarkedPtr<T, N> {
    #[inline]
    fn from(non_null: MarkedNonNull<T, N>) -> Self {
        non_null.into_marked_ptr()
    }
}

/********** impl Debug ****************************************************************************/

impl<T, const N: usize> fmt::Debug for MarkedPtr<T, N> {
    impl_debug!("MarkedPtr");
}

/********** impl Default **************************************************************************/

impl<T, const N: usize> Default for MarkedPtr<T, N> {
    impl_default!();
}

/********** impl Pointer **************************************************************************/

impl<T, const N: usize> fmt::Pointer for MarkedPtr<T, N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.decompose_ptr(), f)
    }
}

/********** impl PartialEq ************************************************************************/

impl<T, const N: usize> PartialEq for MarkedPtr<T, N> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

/********** impl PartialOrd ***********************************************************************/

impl<T, const N: usize> PartialOrd for MarkedPtr<T, N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

/********** impl Eq *******************************************************************************/

impl<T, const N: usize> Eq for MarkedPtr<T, N> {}

/********** impl Ord ******************************************************************************/

impl<T, const N: usize> Ord for MarkedPtr<T, N> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

/********** impl Hash *****************************************************************************/

impl<T, const N: usize> Hash for MarkedPtr<T, N> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

#[cfg(test)]
mod tests {
    use core::ptr;

    type MarkedPtr = crate::MarkedPtr<i32, 2>;

    #[test]
    #[should_panic]
    fn illegal_type() {
        type InvMarkedPtr = crate::MarkedPtr<i32, 3>;
        let _ptr = InvMarkedPtr::compose(ptr::null_mut(), 0b111);
    }

    #[test]
    fn cast() {
        let ptr = MarkedPtr::compose(ptr::null_mut(), 0b11);
        let cast: crate::MarkedPtr<i64, 2> = ptr.cast();
        assert_eq!(cast.decompose(), (ptr::null_mut(), 0b11));
    }

    #[test]
    #[should_panic]
    fn illegal_cast() {
        let ptr = MarkedPtr::compose(ptr::null_mut(), 0b11);
        let _cast: crate::MarkedPtr<i8, 2> = ptr.cast();
    }

    #[test]
    fn from_usize() {
        let reference = &1 as *const i32 as usize;
        let ptr = MarkedPtr::from_usize(reference | 0b1);
        assert_eq!(ptr.into_usize(), reference | 0b1);
        assert_eq!(unsafe { ptr.decompose_ref() }, (Some(&1), 0b1))
    }

    #[test]
    fn from() {
        let mut val = 1;

        let from_ref = MarkedPtr::from(&val);
        let from_mut = MarkedPtr::from(&mut val);
        let from_const_ptr = MarkedPtr::from(&val as *const _);
        let from_mut_ptr = MarkedPtr::from(&mut val as *mut _);

        assert_eq!(from_ref, from_mut);
        assert_eq!(from_mut, from_const_ptr);
        assert_eq!(from_const_ptr, from_mut_ptr);
    }

    #[test]
    fn clear_tag() {
        let raw = &mut 1 as *mut i32;
        let ptr = MarkedPtr::compose(raw, 0);
        assert_eq!(ptr.clear_tag().into_raw(), raw);
        assert_eq!(ptr.clear_tag().decompose(), (raw, 0));

        let ptr = MarkedPtr::compose(raw, 0b11);
        assert_eq!(ptr.clear_tag().into_raw(), raw);
        assert_eq!(ptr.clear_tag().decompose(), (raw, 0));
    }

    #[test]
    fn with_tag() {
        let raw = &mut 1 as *mut i32;
        let unmarked = MarkedPtr::compose(raw, 0);
        let marked_ptr = MarkedPtr::compose(raw, 0b11);

        assert_eq!(unmarked.set_tag(0b1), MarkedPtr::compose(raw, 0b1));
        assert_eq!(marked_ptr.set_tag(0b1), MarkedPtr::compose(raw, 0b1));
        assert_eq!(unmarked.set_tag(0b101), MarkedPtr::compose(raw, 0b1));
        assert_eq!(marked_ptr.set_tag(0b101), MarkedPtr::compose(raw, 0b1));
        assert_eq!(unmarked.set_tag(0).into_raw(), raw);
        assert_eq!(marked_ptr.set_tag(0).into_raw(), raw);
    }

    #[test]
    fn decompose() {
        assert_eq!(MarkedPtr::compose(ptr::null_mut(), 0).decompose(), (ptr::null_mut(), 0));
        assert_eq!(MarkedPtr::compose(ptr::null_mut(), 0b11).decompose(), (ptr::null_mut(), 0b11));
        assert_eq!(MarkedPtr::compose(ptr::null_mut(), 0b100).decompose(), (ptr::null_mut(), 0));

        let ptr = &mut 0xBEEF as *mut i32;
        assert_eq!(MarkedPtr::compose(ptr, 0).decompose(), (ptr, 0));
        assert_eq!(MarkedPtr::compose(ptr, 0b11).decompose(), (ptr, 0b11));
        assert_eq!(MarkedPtr::compose(ptr, 0b100).decompose(), (ptr, 0));
    }

    #[test]
    fn decompose_ref() {
        let null = MarkedPtr::null();
        assert_eq!(unsafe { null.decompose_ref() }, (None, 0));
        let marked_null = MarkedPtr::compose(ptr::null_mut(), 0b11);
        assert_eq!(unsafe { marked_null.decompose_ref() }, (None, 0b11));
        let ptr = MarkedPtr::compose(&mut 1, 0b01);
        assert_eq!(unsafe { ptr.decompose_ref() }, (Some(&1), 0b01));
    }

    #[test]
    fn decompose_mut() {
        let ptr = MarkedPtr::compose(&mut 1, 0b01);
        assert_eq!(unsafe { ptr.decompose_mut() }, (Some(&mut 1), 0b01));
    }
}
