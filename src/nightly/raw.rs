use core::cmp;
use core::fmt;
use core::ptr::{self, NonNull};

use crate::nightly::MarkedPtr;

/********** impl Clone ****************************************************************************/

impl<T, const N: usize> Clone for MarkedPtr<T, {N}> {
    #[inline]
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

/********** impl Copy *****************************************************************************/

impl<T, const N: usize> Copy for MarkedPtr<T, {N}> {}

/********** impl Default **************************************************************************/

impl<T, const N: usize> Default for MarkedPtr<T, {N}> {
    #[inline]
    fn default() -> Self {
        Self::new(ptr::null_mut())
    }
}

/********** impl inherent *************************************************************************/

impl<T, const N: usize> MarkedPtr<T, {N}> {
    pub const MARK_BITS: usize = N;
    pub const MARK_MASK: usize = crate::mark_mask::<T>(Self::MARK_BITS);
    pub const POINTER_MASK: usize = !Self::MARK_MASK;

    #[inline]
    pub const fn new(ptr: *mut T) -> Self {
        Self { ptr }
    }

    #[inline]
    pub const fn null() -> Self {
        Self { ptr: ptr::null_mut() }
    }

    #[inline]
    pub const fn cast<U>(self) -> MarkedPtr<U, {N}> {
        MarkedPtr { ptr: self.ptr.cast() }
    }

    #[inline]
    pub const fn from_usize(val: usize) -> Self {
        Self { ptr: val as *mut _ }
    }

    #[inline]
    pub fn into_usize(self) -> usize {
        self.ptr as usize
    }

    #[inline]
    pub fn into_ptr(self) -> *mut T {
        self.ptr
    }

    #[inline]
    pub fn compose(ptr: *mut T, tag: usize) -> Self {
        Self { ptr: crate::compose::<T, {N}>(N, ptr, tag) }
    }

    #[inline]
    pub fn clear_tag(self) -> Self {
        Self::new(self.decompose_ptr())
    }

    #[inline]
    pub fn decompose(self) -> (*mut T, usize) {
        crate::decompose::<T>(self.ptr as usize, N)
    }

    #[inline]
    pub fn decompose_ptr(self) -> *mut T {
        crate::decompose_ptr::<T>(self.ptr as usize, N)
    }

    #[inline]
    pub fn decompose_tag(self) -> usize {
        crate::decompose_tag::<T>(self.ptr as usize, N)
    }

    #[inline]
    pub unsafe fn as_ref<'a>(self) -> Option<&'a T> {
        self.decompose_ptr().as_ref()
    }

    #[inline]
    pub unsafe fn as_mut<'a>(self) -> Option<&'a mut T> {
        self.decompose_ptr().as_mut()
    }

    #[inline]
    pub unsafe fn decompose_ref<'a>(self) -> (Option<&'a T>, usize) {
        (self.decompose_ptr().as_ref(), self.decompose_tag())
    }

    #[inline]
    pub unsafe fn decompose_mut<'a>(self) -> (Option<&'a mut T>, usize) {
        (self.decompose_ptr().as_mut(), self.decompose_tag())
    }

    #[inline]
    pub fn is_null(self) -> bool {
        self.decompose_ptr().is_null()
    }
}

/********** impl From *****************************************************************************/

impl<T, const N: usize> From<*const T> for MarkedPtr<T, {N}> {
    #[inline]
    fn from(ptr: *const T) -> Self {
        Self::new(ptr as *mut _)
    }
}

impl<T, const N: usize> From<*mut T> for MarkedPtr<T, {N}> {
    #[inline]
    fn from(ptr: *mut T) -> Self {
        Self::new(ptr)
    }
}

impl<'a, T, const N: usize> From<&'a T> for MarkedPtr<T, {N}> {
    #[inline]
    fn from(reference: &'a T) -> Self {
        Self::new(reference as *const _ as *mut _)
    }
}

impl<'a, T, const N: usize> From<&'a mut T> for MarkedPtr<T, {N}> {
    #[inline]
    fn from(reference: &'a mut T) -> Self {
        Self::new(reference)
    }
}

impl<T, const N: usize> From<NonNull<T>> for MarkedPtr<T, {N}> {
    #[inline]
    fn from(ptr: NonNull<T>) -> Self {
        Self::new(ptr.as_ptr())
    }
}

impl<T, const N: usize> From<(*mut T, usize)> for MarkedPtr<T, {N}> {
    #[inline]
    fn from((ptr, tag): (*mut T, usize)) -> Self {
        Self::compose(ptr, tag)
    }
}

impl<T, const N: usize> From<(*const T, usize)> for MarkedPtr<T, {N}> {
    #[inline]
    fn from((ptr, tag): (*const T, usize)) -> Self {
        Self::compose(ptr as *mut _, tag)
    }
}

/********** impl Debug ****************************************************************************/

impl<T, const N: usize> fmt::Debug for MarkedPtr<T, {N}> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MarkedPtr")
            .field("ptr", &self.decompose_ptr())
            .field("tag", &self.decompose_tag())
            .finish()
    }
}

/********** impl Pointer **************************************************************************/

impl<T, const N: usize> fmt::Pointer for MarkedPtr<T, {N}> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.decompose_ptr(), f)
    }
}

/********** impl PartialEq ************************************************************************/

impl<T, const N: usize> PartialEq for MarkedPtr<T, {N}> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.ptr.eq(&other.ptr)
    }
}

/********** impl PartialOrd ***********************************************************************/

impl<T, const N: usize> PartialOrd for MarkedPtr<T, {N}> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.ptr.partial_cmp(&other.ptr)
    }
}

/********** impl Eq *******************************************************************************/

impl<T, const N: usize> Eq for MarkedPtr<T, {N}> {}

/********** impl Ord ******************************************************************************/

impl<T, const N: usize> Ord for MarkedPtr<T, {N}> {
    #[inline]
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.ptr.cmp(&other.ptr)
    }
}
