use core::mem;

use crate::{MarkedOption::{self, Value, Null}, NonNullable};

/********** impl Default **************************************************************************/

impl<T: NonNullable> Default for MarkedOption<T> {
    #[inline]
    fn default() -> Self {
        Null(0)
    }
}

/********** impl inherent *************************************************************************/

impl<T: NonNullable> MarkedOption<T> {
    #[inline]
    pub fn is_null(&self) -> bool {
        match self {
            Value(_) => false,
            Null(_) => true,
        }
    }

    #[inline]
    pub fn is_value(&self) -> bool {
        match self {
            Value(_) => true,
            Null(_) => false,
        }
    }

    #[inline]
    pub fn map<U: NonNullable>(self, func: impl FnOnce(T) -> U) -> MarkedOption<U> {
        match self {
            Value(ptr) => Value(func(ptr)),
            Null(tag) => Null(tag),
        }
    }

    #[inline]
    pub fn unwrap(self) -> T {
        match self {
            Value(value) => value,
            _ => panic!("called `unwrap` on `Null` variant")
        }
    }

    #[inline]
    pub fn take(&mut self) -> Self {
        mem::replace(self, Null(0))
    }
}
