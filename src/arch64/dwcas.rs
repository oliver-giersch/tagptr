use core::arch::x86_64::cmpxchg16b;
use core::cell::UnsafeCell;
use core::mem;
use core::ptr;
use core::sync::atomic::Ordering;

////////////////////////////////////////////////////////////////////////////////////////////////////
// AtomicMarkedWidePtr
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AtomicTagPtr<T> {
    inner: UnsafeCell<TagPtr<T>>,
}

impl<T> AtomicTagPtr<T> {
    const NULL: TagPtr<T> = TagPtr::null();

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

pub struct TagPtr<T>(pub *mut T, pub u64);

impl<T> Clone for TagPtr<T> {
    fn clone(&self) -> Self {
        unimplemented!()
    }
}

impl<T> Copy for TagPtr<T> {}

impl<T> TagPtr<T> {
    pub const fn null() -> Self {
        Self(ptr::null_mut(), 0)
    }

    pub const fn new(ptr: *mut T) -> Self {
        Self(ptr, 0)
    }

    pub const fn compose(ptr: *mut T, tag: u64) -> Self {
        Self(ptr, tag)
    }
}
