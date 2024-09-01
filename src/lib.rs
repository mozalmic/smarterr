#![doc = include_str!("../README.md")]

use std::{rc::Rc, sync::Arc};

#[cfg(feature = "errorset")]
pub use errorset;

#[cfg(feature = "atomic_error")]
#[macro_export]
macro_rules! error {
    ($evis:vis $name:ident<$source:ty> { $($vis:vis $field:ident: $ty:ty),* $(,)? } => $msg:literal) => {
        #[derive(thiserror::Error, Debug)]
        #[error($msg)]
        $evis struct $name {
            $(
                $vis $field: $ty,
            )*
            #[source]
            pub source: $source,
        }
    };
    ($evis:vis $name:ident { $($vis:vis $field:ident: $ty:ty),* $(,)? } => $msg:literal) => {
        #[derive(thiserror::Error, Debug)]
        #[error($msg)]
        $evis struct $name {
            $(
                $vis $field: $ty,
            )*
        }
    };
}

macro_rules! ternary {
    ($condition: expr, $_true: expr, $_false: expr) => {
        if $condition {
            Ok($_true)
        } else {
            Err($_false)
        }
    };
}

macro_rules! simple_throwable {
    ($t: ty, $condition: expr) => {
        impl Throwable for $t {
            type T = Self;
            type ES = Self;

            fn throw<E, F: FnOnce(Self::ES) -> E>(self, err_map: F) -> Result<Self::T, E> {
                ternary!(($condition(&self)), self, err_map(self))
            }

            fn raise<E, F: FnOnce(Self::T) -> E>(self, ok_to_err_map: F) -> Result<Self::ES, E> {
                ternary!(!($condition(&self)), self, ok_to_err_map(self))
            }
        }
    };
}

pub trait Throwable
where
    Self: Sized,
{
    type T;
    type ES;

    /// Returns new Result keeping error state of the source unchanged.
    /// Error state is defined by the condition:
    /// * **numbers**: != 0
    /// * **bool**:    false
    /// * **strings**: is_empty()
    /// * **Option**:  None
    /// * **Result**:  Err
    ///
    /// If error state is NOT detected, it returns `Ok(Self::T)``
    /// Otherwise it calls `err_map` function and returns `Err(err_map(Self::ES))`
    ///
    /// ```rust
    /// use smarterr::Throwable;
    /// fn example(val: i32) -> Result<i32, String> {
    ///     val.throw(|v| format!("Value {} is not zero, for integers it means error state", v))
    /// }
    /// ```
    fn throw<E, F: FnOnce(Self::ES) -> E>(self, err_map: F) -> Result<Self::T, E>;

    /// Returns new Result INVERTING error state of the source.
    /// Error state is defined by the condition:
    /// * **numbers**: != 0
    /// * **bool**:    false
    /// * **strings**: is_empty()
    /// * **Option**:  None
    /// * **Result**:  Err
    ///
    /// If error state IS detected, it returns `Ok(Self::ES)`
    /// Otherwise it calls `ok_to_err_map` function and returns `Err(ok_to_err_map(Self::T))`
    ///
    /// ```rust
    /// use smarterr::Throwable;
    /// fn example(val: i32) -> Result<i32, String> {
    ///     val.throw(|v| format!("Value {} is not zero, for integers it means error state", v))
    /// }
    /// ```
    fn raise<E, F: FnOnce(Self::T) -> E>(self, ok_to_err_map: F) -> Result<Self::ES, E>;
}

simple_throwable!((), |_| true);

simple_throwable!(bool, |&b| b);
simple_throwable!(usize, |&x| x == 0);
simple_throwable!(isize, |&x| x == 0);
simple_throwable!(u128, |&x| x == 0);
simple_throwable!(i128, |&x| x == 0);
simple_throwable!(u64, |&x| x == 0);
simple_throwable!(i64, |&x| x == 0);
simple_throwable!(u32, |&x| x == 0);
simple_throwable!(i32, |&x| x == 0);
simple_throwable!(u16, |&x| x == 0);
simple_throwable!(i16, |&x| x == 0);
simple_throwable!(u8, |&x| x == 0);
simple_throwable!(i8, |&x| x == 0);

simple_throwable!(&str, |s: &str| !s.is_empty());
simple_throwable!(String, |s: &String| !s.is_empty());
simple_throwable!(Box<String>, |s: &Box<String>| !s.is_empty());
simple_throwable!(Rc<String>, |s: &Rc<String>| !s.is_empty());
simple_throwable!(Arc<String>, |s: &Arc<String>| !s.is_empty());

impl<T> Throwable for Option<T> {
    type T = T;
    type ES = Self;

    fn throw<E, F: FnOnce(Self::ES) -> E>(self, err_map: F) -> Result<Self::T, E> {
        match self {
            Some(v) => Ok(v),
            None => Err(err_map(self)),
        }
    }

    fn raise<E, F: FnOnce(Self::T) -> E>(self, ok_to_err_map: F) -> Result<Self::ES, E> {
        match self {
            Some(v) => Err(ok_to_err_map(v)),
            None => Ok(self),
        }
    }
}

impl<T, ES> Throwable for Result<T, ES> {
    type T = T;
    type ES = ES;

    fn throw<E, F: FnOnce(Self::ES) -> E>(self, err_map: F) -> Result<Self::T, E> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(err_map(e)),
        }
    }

    fn raise<E, F: FnOnce(Self::T) -> E>(self, ok_to_err_map: F) -> Result<Self::ES, E> {
        match self {
            Ok(v) => Err(ok_to_err_map(v)),
            Err(e) => Ok(e),
        }
    }
}
