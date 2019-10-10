use core::mem;
use core::ptr;

use typenum::Unsigned;

use crate::{
    MarkedPtr,
    MarkedNonNull,
    MarkedOption::{self, Null, Value},
    NonNullable,
};

/********** impl Default **************************************************************************/

impl<T: NonNullable> Default for MarkedOption<T> {
    #[inline]
    fn default() -> Self {
        Null(0)
    }
}

/********** impl inherent *************************************************************************/

impl<P: NonNullable> MarkedOption<P> {
    /// Returns `true` if `self` contains a [`Null`] variant.
    #[inline]
    pub fn is_null(&self) -> bool {
        match self {
            Value(_) => false,
            Null(_) => true,
        }
    }

    /// Returns `true` if self contains a [`Value`] variant.
    #[inline]
    pub fn is_value(&self) -> bool {
        match self {
            Value(_) => true,
            Null(_) => false,
        }
    }

    /// Converts from `MarkedOption<T>` to `MarkedOption<&T>`.
    #[inline]
    pub fn as_ref(&self) -> MarkedOption<&P> {
        match self {
            Value(ptr) => Value(ptr),
            Null(tag) => Null(*tag),
        }
    }

    /// Converts from `MarkedOption<T>` to `MarkedOption<&mut T>`.
    #[inline]
    pub fn as_mut(&mut self) -> MarkedOption<&mut P> {
        match self {
            Value(ptr) => Value(ptr),
            Null(tag) => Null(*tag),
        }
    }

    /// Unwraps the [`MarkedOption`], yielding the content of a [`Value`].
    ///
    /// # Panics
    ///
    /// Panics, if the value is a [`Null`] with a custom panic message provided
    /// by `msg`.
    #[inline]
    pub fn expect(self, msg: &str) -> P {
        self.value().expect(msg)
    }

    /// Moves the pointer out of the [`MarkedOption`] if it is
    /// [`Value(ptr)`][Value].
    ///
    /// # Panics
    ///
    /// This method panics, if the [`MarkedOption`] contains a [`Null`] variant.
    #[inline]
    pub fn unwrap(self) -> P {
        match self {
            Value(ptr) => ptr,
            _ => panic!("called `unwrap` on `Null` variant"),
        }
    }

    /// Returns the contained value or the result of the given `func`.
    #[inline]
    pub fn unwrap_or_else(self, func: impl (FnOnce(usize) -> P)) -> P {
        match self {
            Value(ptr) => ptr,
            Null(tag) => func(tag),
        }
    }

    /// Extracts the tag out of the `MarkedOption`, if it is
    /// [`Null(tag)`][Null].
    ///
    /// # Panics
    ///
    /// This method panics, if the [`MarkedOption`] contains a [`Value`]
    /// variant.
    #[inline]
    pub fn unwrap_null(self) -> usize {
        match self {
            Null(tag) => tag,
            _ => panic!("called `unwrap_tag()` on a `Value`"),
        }
    }

    /// Maps a `MarkedOption<T>` to `MarkedOption<U>` by applying a function to
    /// a contained value.
    #[inline]
    pub fn map<U: NonNullable>(self, func: impl FnOnce(P) -> U) -> MarkedOption<U> {
        match self {
            Value(ptr) => Value(func(ptr)),
            Null(tag) => Null(tag),
        }
    }

    /// Applies a function `func` to the contained pointer (if any) or returns
    /// the provided `default` value.
    #[inline]
    pub fn map_or<U, F>(self, default: U, func: impl FnOnce(P) -> U) -> U {
        match self {
            Value(ptr) => func(ptr),
            Null(_) => default,
        }
    }

    /// Applies a function to the contained value (if any), or computes a
    /// default value using `func`, if no value is contained.
    #[inline]
    pub fn map_or_else<U: NonNullable>(
        self,
        default: impl FnOnce(usize) -> U,
        func: impl FnOnce(P) -> U,
    ) -> U {
        match self {
            Value(ptr) => func(ptr),
            Null(tag) => default(tag),
        }
    }

    /// Converts `self` from `MarkedOption<T>` to [`Option<T>`][Option].
    #[inline]
    pub fn value(self) -> Option<P> {
        match self {
            Value(ptr) => Some(ptr),
            _ => None,
        }
    }

    /// Takes the value of the [`MarkedOption`], leaving a [`Null`] variant in
    /// its place.
    #[inline]
    pub fn take(&mut self) -> Self {
        mem::replace(self, Null(0))
    }

    /// Replaces the actual value in the [`MarkedOption`] with the given
    /// `value`, returning the old value.
    #[inline]
    pub fn replace(&mut self, value: P) -> Self {
        mem::replace(self, Value(value))
    }

    #[inline]
    pub fn decompose(&self) -> (*mut P::Item, usize) {
        match self {
            Value(ptr) => (ptr.as_mut_ptr(), ptr.tag()),
            Null(tag) => (ptr::null_mut(), *tag),
        }
    }

    #[inline]
    pub fn decompose_ptr(&self) -> *mut P::Item {
        match self {
            Value(ptr) => ptr.as_mut_ptr(),
            Null(_) => ptr::null_mut()
        }
    }

    /// Returns the [`MarkedOption`]'s tag.
    #[inline]
    pub fn decompose_tag(&self) -> usize {
        match self {
            Value(ptr) => ptr.tag(),
            Null(tag) => *tag
        }
    }
}

/*********** impl From ****************************************************************************/

impl<T, N: Unsigned> From<MarkedPtr<T, N>> for MarkedOption<MarkedNonNull<T, N>> {
    #[inline]
    fn from(marked_ptr: MarkedPtr<T, N>) -> Self {
        match marked_ptr.decompose() {
            (ptr, _) if !ptr.is_null() => Value(unsafe { MarkedNonNull::new_unchecked(marked_ptr) }),
            (_, tag) => Null(tag),
        }
    }
}

impl<T: NonNullable> From<Option<T>> for MarkedOption<T> {
    #[inline]
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(ptr) => Value(ptr),
            None => Null(0),
        }
    }
}

impl<'a, T: NonNullable> From<&'a MarkedOption<T>> for MarkedOption<&'a T> {
    #[inline]
    fn from(reference: &'a MarkedOption<T>) -> Self {
        match reference {
            Value(val) => Value(val),
            Null(tag) => Null(*tag),
        }
    }
}

impl<'a, T: NonNullable> From<&'a mut MarkedOption<T>> for MarkedOption<&'a mut T> {
    #[inline]
    fn from(reference: &'a mut MarkedOption<T>) -> Self {
        match reference {
            Value(val) => Value(val),
            Null(tag) => Null(*tag),
        }
    }
}
