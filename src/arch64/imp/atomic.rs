use core::fmt;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::arch64::{AtomicMarkedPtr64, MarkedPtr64};

/********** impl Send + Sync **********************************************************************/

unsafe impl<T> Send for AtomicMarkedPtr64<T> {}
unsafe impl<T> Sync for AtomicMarkedPtr64<T> {}

/********** impl inherent (const) *****************************************************************/

impl<T> AtomicMarkedPtr64<T> {}

/********** impl inherent *************************************************************************/

impl<T> AtomicMarkedPtr64<T> {
    impl_constants!(
        tag_bits = crate::arch64::TAG_BITS,
        tag_type = u16,
        tag_mask = crate::arch64::TAG_MASK
    );

    impl_atomic_inherent_const!(ptr_type = MarkedPtr64<T>, ptr_ident = MarkedPtr64);

    impl_atomic_inherent!(
        ptr_type = MarkedPtr64<T>,
        ptr_ident = MarkedPtr64,
        tag_type = u16,
        example_atomic_path = conquer_pointer::arch64::AtomicMarkedPtr64<i32>,
        example_ptr_path = conquer_pointer::arch64::MarkedPtr64<i32>
    );
}

/********** impl Debug ****************************************************************************/

impl<T> fmt::Debug for AtomicMarkedPtr64<T> {
    impl_atomic_debug!("AtomicMarkedPtr64");
}

/********** impl Default **************************************************************************/

impl<T> Default for AtomicMarkedPtr64<T> {
    impl_default!();
}

/********** impl From (*mut T) ********************************************************************/

impl<T> From<*mut T> for AtomicMarkedPtr64<T> {
    impl_atomic_from_raw!(*mut T);
}

/********** impl From (MarkedPtr64<T>) ************************************************************/

impl<T> From<MarkedPtr64<T>> for AtomicMarkedPtr64<T> {
    impl_atomic_from_raw!(MarkedPtr64<T>);
}

/********** impl Pointer **************************************************************************/

impl<T> fmt::Pointer for AtomicMarkedPtr64<T> {
    impl_atomic_pointer!();
}
