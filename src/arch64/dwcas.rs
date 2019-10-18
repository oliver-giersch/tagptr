use core::arch::x86_64::cmpxchg16b;
use core::cell::UnsafeCell;
use core::cmp;
use core::fmt;
use core::mem;
use core::ptr;
use core::sync::atomic::Ordering;

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedWidePtr
////////////////////////////////////////////////////////////////////////////////////////////////////

/// An atomic 128-bit value that is composed of a pointer and a 64-bit tag.
///
/// This type uses the x86/64 specific `cmpxchg16b` instruction to allow safe
/// concurrent mutation.
/// The tag value is usually used to prevent the **ABA Problem** with CAS
/// operations.
#[repr(align(16))]
pub struct AtomicTagPtr<T> {
    inner: UnsafeCell<TagPtr<T>>,
}

/********** impl Send + Sync **********************************************************************/

unsafe impl<T> Send for AtomicTagPtr<T> {}
unsafe impl<T> Sync for AtomicTagPtr<T> {}

/********** impl Default **************************************************************************/

impl<T> Default for AtomicTagPtr<T> {
    #[inline]
    fn default() -> Self {
        Self::new(TagPtr::null())
    }
}

/********** impl inherent *************************************************************************/

impl<T> AtomicTagPtr<T> {
    const NULL: TagPtr<T> = TagPtr::null();

    /// Creates a new [`AtomicTagPtr`].
    #[inline]
    pub const fn new(ptr: TagPtr<T>) -> Self {
        Self { inner: UnsafeCell::new(ptr) }
    }

    /// Loads the value of the [`AtomicTagPtr`].
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
    #[inline]
    pub fn load(&self, order: Ordering) -> TagPtr<T> {
        assert!(order != Ordering::Release && order != Ordering::AcqRel);
        self.compare_and_swap(Self::NULL, Self::NULL, order)
    }

    #[inline]
    pub fn compare_and_swap(
        &self,
        current: TagPtr<T>,
        new: TagPtr<T>,
        order: Ordering,
    ) -> TagPtr<T> {
        match self.compare_exchange(current, new, order, Ordering::Relaxed) {
            Ok(res) => res,
            Err(res) => res,
        }
    }

    #[inline]
    pub fn compare_exchange(
        &self,
        current: TagPtr<T>,
        new: TagPtr<T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<TagPtr<T>, TagPtr<T>> {
        unsafe {
            let dest = self.inner.get().cast();
            let curr_u128 = mem::transmute(current);
            let new_u128 = mem::transmute(new);
            let res = cmpxchg16b(dest, curr_u128, new_u128, success, failure);

            if res == curr_u128 {
                Ok(mem::transmute(res))
            } else {
                Err(mem::transmute(res))
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// TagPtr
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A double-world tuple consisting of a raw pointer and an associated 64-bit
/// tag value.
pub struct TagPtr<T>(pub *mut T, pub u64);

/*********** impl Clone ***************************************************************************/

impl<T> Clone for TagPtr<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

/*********** impl Copy ****************************************************************************/

impl<T> Copy for TagPtr<T> {}

/*********** impl Default *************************************************************************/

impl<T> Default for TagPtr<T> {
    #[inline]
    fn default() -> Self {
        Self::null()
    }
}

/*********** impl inherent ************************************************************************/

impl<T> TagPtr<T> {
    /// Creates a new unmarked `null` pointer.
    #[inline]
    pub const fn null() -> Self {
        Self(ptr::null_mut(), 0)
    }

    /// Casts to a pointer of type `U`.
    #[inline]
    pub const fn cast<U>(self) -> TagPtr<U> {
        TagPtr(self.0.cast(), self.1)
    }

    /// Creates a new unmarked [`TagPtr`].
    #[inline]
    pub const fn new(ptr: *mut T) -> Self {
        Self(ptr, 0)
    }

    /// Composes a new [`TagPtr`] from a raw `ptr` and a 64-bit `tag` value.
    #[inline]
    pub const fn compose(ptr: *mut T, tag: u64) -> Self {
        Self(ptr, tag)
    }

    /// Decomposes the [`TagPtr`], returning the separated raw pointer and
    /// its 64-bit tag.
    #[inline]
    pub const fn decompose(self) -> (*mut T, u64) {
        (self.ptr(), self.tag())
    }

    /// Decomposes the [`TagPtr`], returning only the separated raw pointer.
    #[inline]
    pub const fn decompose_ptr(self) -> *mut T {
        self.ptr()
    }

    /// Decomposes the [`TagPtr`], returning only the separated 64-bit tag.
    #[inline]
    pub const fn decompose_tag(self) -> u64 {
        self.tag()
    }

    /// Returns the raw pointer.
    #[inline]
    pub const fn ptr(self) -> *mut T {
        self.0
    }

    /// Returns the 64-bit tag.
    #[inline]
    pub const fn tag(self) -> u64 {
        self.1
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
    pub unsafe fn decompose_ref<'a>(self) -> (Option<&'a T>, u64) {
        (self.0.as_ref(), self.1)
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
    pub unsafe fn decompose_mut<'a>(self) -> (Option<&'a mut T>, u64) {
        (self.0.as_mut(), self.1)
    }

    #[inline]
    pub unsafe fn as_ref<'a>(self) -> Option<&'a T> {
        self.0.as_ref()
    }

    #[inline]
    pub unsafe fn as_mut<'a>(self) -> Option<&'a mut T> {
        self.0.as_mut()
    }
}

/*********** impl Debug ***************************************************************************/

impl<T> fmt::Debug for TagPtr<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TagPtr").field("ptr", &self.0).field("tag", &self.1).finish()
    }
}

/*********** impl PartialEq ***********************************************************************/

impl<T> PartialEq for TagPtr<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0) && self.1.eq(&other.1)
    }
}

/*********** impl PartialOrd **********************************************************************/

impl<T> PartialOrd for TagPtr<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match self.0.partial_cmp(&other.0) {
            Some(cmp::Ordering::Equal) => self.1.partial_cmp(&self.1),
            any => any,
        }
    }
}

/*********** impl Eq ******************************************************************************/

impl<T> Eq for TagPtr<T> {}

/*********** impl Ord *****************************************************************************/

impl<T> Ord for TagPtr<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.0.cmp(&other.0) {
            cmp::Ordering::Equal => self.1.cmp(&other.1),
            any => any,
        }
    }
}

#[cfg(test)]
mod tests {
    use core::sync::atomic::Ordering;

    use super::{AtomicTagPtr, TagPtr};

    #[test]
    fn atomic_load() {
        let ptr = &mut 1;
        let atomic = AtomicTagPtr::new(TagPtr(ptr, u64::max_value()));
        assert_eq!(atomic.load(Ordering::SeqCst), TagPtr(ptr, u64::max_value()));
    }
}
