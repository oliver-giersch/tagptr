use crate::arch64::AtomicMarkedPtr64;

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
}
