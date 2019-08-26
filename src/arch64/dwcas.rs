use core::arch::x86_64::cmpxchg16b;
use core::cell::UnsafeCell;
use core::cmp;
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

    #[inline]
    pub fn load(&self, order: Ordering) -> TagPtr<T> {
        self.compare_and_swap(Self::NULL, Self::NULL, order)
    }

    pub fn store(&self, ptr: TagPtr<T>, order: Ordering) {
        let mut curr = Self::NULL;
        while curr != ptr {
            curr = self.compare_and_swap(curr, ptr, order);
        }
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
// MarkedWidePtr
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
    #[inline]
    pub const fn null() -> Self {
        Self(ptr::null_mut(), 0)
    }

    /// Casts to a pointer of type `U`.
    #[inline]
    pub const fn cast<U>(self) -> TagPtr<U> {
        TagPtr(self.0.cast(), self.1)
    }

    #[inline]
    pub const fn new(ptr: *mut T) -> Self {
        Self(ptr, 0)
    }

    #[inline]
    pub const fn compose(ptr: *mut T, tag: u64) -> Self {
        Self(ptr, tag)
    }

    #[inline]
    pub const fn decompose(self) -> (*mut T, u64) {
        (self.ptr(), self.tag())
    }

    #[inline]
    pub const fn decompose_ptr(self) -> *mut T {
        self.0
    }

    #[inline]
    pub const fn decompose_tag(self) -> u64 {
        self.1
    }

    #[inline]
    pub fn ptr(self) -> *mut T {
        self.0
    }

    #[inline]
    pub fn tag(self) -> u64 {
        self.1
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
