//! Implementation for the [`MarkedOption`] type.

use core::fmt;
use core::mem;
use core::ptr;

use typenum::Unsigned;

use crate::traits::{MarkedNonNullable, NonNullable};
use crate::{
    MarkedNonNull, MarkedPtr,
    MaybeNull::{self, NotNull, Null},
};

/********** impl inherent *************************************************************************/

impl<P: NonNullable> MaybeNull<P> {
    /// Returns `true` if `self` contains a [`Null`] variant.
    #[inline]
    pub fn is_null(&self) -> bool {
        match self {
            NotNull(_) => false,
            Null(_) => true,
        }
    }

    /// Returns `true` if self contains a [`Pointer`] variant.
    #[inline]
    pub fn is_not_null(&self) -> bool {
        match self {
            NotNull(_) => true,
            Null(_) => false,
        }
    }

    /// Converts from [`&Marked<T>`][Marked] to [`Marked<&T>`][Marked].
    #[inline]
    pub fn as_ref(&self) -> MaybeNull<&P> {
        match self {
            NotNull(ptr) => NotNull(ptr),
            Null(tag) => Null(*tag),
        }
    }

    /// Converts from [`Marked<T>`][Marked] to `MarkedOption<&mut T>`.
    #[inline]
    pub fn as_mut(&mut self) -> MaybeNull<&mut P> {
        match self {
            NotNull(ptr) => NotNull(ptr),
            Null(tag) => Null(*tag),
        }
    }

    /// Unwraps the [`Marked`], yielding the content of a [`Pointer`].
    ///
    /// # Panics
    ///
    /// Panics, if the value is a [`Null`] with a custom panic message provided
    /// by `msg`.
    #[inline]
    pub fn expect(self, msg: &str) -> P {
        self.not_null().expect(msg)
    }

    /// Moves the pointer out of the [`Marked`] if it is
    /// [`Pointer(ptr)`][Value].
    ///
    /// # Panics
    ///
    /// This method panics, if the [`Marked`] contains a [`Null`] variant.
    #[inline]
    pub fn unwrap(self) -> P {
        match self {
            NotNull(ptr) => ptr,
            _ => panic!("called `unwrap` on `Null` variant"),
        }
    }

    /// Returns the contained value or the result of the given `func`.
    #[inline]
    pub fn unwrap_or_else(self, func: impl (FnOnce(usize) -> P)) -> P {
        match self {
            NotNull(ptr) => ptr,
            Null(tag) => func(tag),
        }
    }

    /// Extracts the tag out of the [`Marked`], if it is [`Null(tag)`][Null].
    ///
    /// # Panics
    ///
    /// This method panics, if the [`Marked`] contains a [`Value`]
    /// variant.
    #[inline]
    pub fn unwrap_null(self) -> usize {
        match self {
            Null(tag) => tag,
            _ => panic!("called `unwrap_tag()` on a `Value`"),
        }
    }

    /// Maps a [`Marked<T>`][Marked] to [`Marked<U>`][Marked] by applying a
    /// function to a contained value.
    #[inline]
    pub fn map<U: NonNullable>(self, func: impl FnOnce(P) -> U) -> MaybeNull<U> {
        match self {
            NotNull(ptr) => NotNull(func(ptr)),
            Null(tag) => Null(tag),
        }
    }

    /// Applies a function `func` to the contained pointer (if any) or returns
    /// the provided `default` value.
    #[inline]
    pub fn map_or<U, F>(self, default: U, func: impl FnOnce(P) -> U) -> U {
        match self {
            NotNull(ptr) => func(ptr),
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
            NotNull(ptr) => func(ptr),
            Null(tag) => default(tag),
        }
    }

    /// Converts `self` from [`Marked<T>`][Marked] to [`Option<T>`][Option].
    #[inline]
    pub fn not_null(self) -> Option<P> {
        match self {
            NotNull(ptr) => Some(ptr),
            _ => None,
        }
    }

    /// Takes the value of the [`Marked`], leaving a [`Null`] variant in
    /// its place.
    #[inline]
    pub fn take(&mut self) -> Self {
        mem::replace(self, Null(0))
    }

    /// Replaces the actual value in the [`Marked`] with the given `value`,
    /// returning the old value.
    #[inline]
    pub fn replace(&mut self, value: P) -> Self {
        mem::replace(self, NotNull(value))
    }
}

impl<P: MarkedNonNullable> MaybeNull<P> {
    #[inline]
    pub fn as_marked_ptr(&self) -> MarkedPtr<P::Item, P::MarkBits> {
        match self {
            NotNull(ptr) => P::as_marked_ptr(ptr),
            Null(tag) => MarkedPtr::compose(ptr::null_mut(), *tag),
        }
    }

    #[inline]
    pub fn into_marked_ptr(self) -> MarkedPtr<P::Item, P::MarkBits> {
        match self {
            NotNull(ptr) => P::into_marked_ptr(ptr),
            Null(tag) => MarkedPtr::compose(ptr::null_mut(), tag),
        }
    }

    #[inline]
    pub fn clear_tag(self) -> Self {
        match self {
            NotNull(ptr) => NotNull(P::clear_tag(ptr)),
            Null(_) => Null(0),
        }
    }

    #[inline]
    pub fn split_tag(self) -> (Self, usize) {
        match self {
            NotNull(ptr) => {
                let (ptr, tag) = P::split_tag(ptr);
                (NotNull(ptr), tag)
            }
            Null(tag) => (Null(0), tag),
        }
    }

    #[inline]
    pub fn set_tag(self, tag: usize) -> Self {
        match self {
            NotNull(ptr) => NotNull(P::set_tag(ptr, tag)),
            Null(_) => Null(tag),
        }
    }

    #[inline]
    pub fn decompose(self) -> (Option<P>, usize) {
        match self {
            NotNull(ptr) => {
                let (ptr, tag) = P::split_tag(ptr);
                (Some(ptr), tag)
            }
            Null(tag) => (None, tag),
        }
    }

    #[inline]
    pub fn decompose_ptr(&self) -> *mut P::Item {
        match self {
            NotNull(ptr) => P::decompose_ptr(ptr),
            Null(_) => ptr::null_mut(),
        }
    }

    /// Returns the [`Marked`]'s tag.
    #[inline]
    pub fn decompose_tag(&self) -> usize {
        match self {
            NotNull(ptr) => P::decompose_tag(ptr),
            Null(tag) => *tag,
        }
    }
}

/********** impl Debug ****************************************************************************/

impl<T: NonNullable + fmt::Debug> fmt::Debug for MaybeNull<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotNull(ptr) => write!(f, "Value({:?})", ptr),
            Null(tag) => write!(f, "Null({})", *tag),
        }
    }
}

/********** impl Default **************************************************************************/

impl<T: NonNullable> Default for MaybeNull<T> {
    #[inline]
    fn default() -> Self {
        Null(0)
    }
}

/*********** impl From ****************************************************************************/

impl<T, N: Unsigned> From<MarkedPtr<T, N>> for MaybeNull<MarkedNonNull<T, N>> {
    #[inline]
    fn from(marked_ptr: MarkedPtr<T, N>) -> Self {
        match marked_ptr.decompose() {
            (ptr, _) if !ptr.is_null() => {
                NotNull(unsafe { MarkedNonNull::new_unchecked(marked_ptr) })
            }
            (_, tag) => Null(tag),
        }
    }
}

impl<T: NonNullable> From<MaybeNull<T>> for Option<T> {
    #[inline]
    fn from(marked: MaybeNull<T>) -> Self {
        marked.not_null()
    }
}

impl<T: NonNullable> From<Option<T>> for MaybeNull<T> {
    #[inline]
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(ptr) => NotNull(ptr),
            None => Null(0),
        }
    }
}

impl<'a, T: NonNullable> From<&'a MaybeNull<T>> for MaybeNull<&'a T> {
    #[inline]
    fn from(reference: &'a MaybeNull<T>) -> Self {
        match reference {
            NotNull(val) => NotNull(val),
            Null(tag) => Null(*tag),
        }
    }
}

impl<'a, T: NonNullable> From<&'a mut MaybeNull<T>> for MaybeNull<&'a mut T> {
    #[inline]
    fn from(reference: &'a mut MaybeNull<T>) -> Self {
        match reference {
            NotNull(val) => NotNull(val),
            Null(tag) => Null(*tag),
        }
    }
}
