use core::mem;

use crate::nightly::{AtomicMarkedPtr, MarkedPtr};

/********** impl inherent *************************************************************************/

impl<T, const N: usize> AtomicMarkedPtr<T, {N}> {
    #[inline]
    pub fn get_mut(&mut self) -> &mut MarkedPtr<T, {N}> {
        unsafe { mem::transmute(self.ptr.get_mut()) }
    }

    #[inline]
    pub fn into_inner(self) -> MarkedPtr<T, {N}> {
        MarkedPtr::from_usize(self.ptr.into_inner())
    }
}
