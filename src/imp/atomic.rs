use core::fmt;
use core::sync::atomic::Ordering;

use typenum::Unsigned;

use crate::{AtomicMarkedPtr, MarkedPtr};

/********** impl Send + Sync **********************************************************************/

unsafe impl<T, N> Send for AtomicMarkedPtr<T, N> {}
unsafe impl<T, N> Sync for AtomicMarkedPtr<T, N> {}

/********** impl inherent (const) *****************************************************************/

impl<T, N> AtomicMarkedPtr<T, N> {}

/********** impl inherent *************************************************************************/

impl<T, N: Unsigned> AtomicMarkedPtr<T, N> {
    impl_constants!(
        tag_bits = N::USIZE,
        tag_type = usize,
        tag_mask = crate::mark_mask::<T>(N::USIZE)
    );
}

/********** impl Debug ****************************************************************************/

impl<T, N: Unsigned> fmt::Debug for AtomicMarkedPtr<T, N> {
    impl_atomic_debug!("AtomicMarkedPtr");
}
