use core::fmt;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicUsize, Ordering};

use typenum::Unsigned;

use crate::{AtomicMarkedPtr, MarkedPtr};

/********** impl Send + Sync **********************************************************************/

unsafe impl<T, N> Send for AtomicMarkedPtr<T, N> {}
unsafe impl<T, N> Sync for AtomicMarkedPtr<T, N> {}

/********** impl inherent (const) *****************************************************************/

impl<T, N> AtomicMarkedPtr<T, N> {
    impl_atomic_inherent_const!(ptr_type = MarkedPtr<T, N>, ptr_ident = MarkedPtr);
}

/********** impl inherent *************************************************************************/

impl<T, N: Unsigned> AtomicMarkedPtr<T, N> {
    impl_constants!(
        tag_bits = N::USIZE,
        tag_type = usize,
        tag_mask = crate::mark_mask::<T>(N::USIZE)
    );

    impl_atomic_inherent!(
        ptr_type = MarkedPtr<T, N>,
        ptr_ident = MarkedPtr,
        tag_type = usize,
        example_atomic_path = conquer_pointer::AtomicMarkedPtr<i32, conquer_pointer::typenum::U2>,
        example_ptr_path = conquer_pointer::MarkedPtr<i32, conquer_pointer::typenum::U2>
    );
}

/********** impl Debug ****************************************************************************/

impl<T, N: Unsigned> fmt::Debug for AtomicMarkedPtr<T, N> {
    impl_atomic_debug!("AtomicMarkedPtr");
}

/********** impl Default **************************************************************************/

impl<T, N> Default for AtomicMarkedPtr<T, N> {
    impl_default!();
}

/********** impl From (*mut T) ********************************************************************/

impl<T, N> From<*mut T> for AtomicMarkedPtr<T, N> {
    impl_atomic_from_raw!(*mut T);
}

/********** impl From (MarkedPtr<T, N>) ***********************************************************/

impl<T, N> From<MarkedPtr<T, N>> for AtomicMarkedPtr<T, N> {
    impl_atomic_from_raw!(MarkedPtr<T, N>);
}

/********** impl Pointer **************************************************************************/

impl<T, N: Unsigned> fmt::Pointer for AtomicMarkedPtr<T, N> {
    impl_atomic_pointer!();
}
