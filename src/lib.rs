//! # SmartErr
//!
//! SmartErr is a library to easily generate errors from any sources.
//! It is a part of SmartErr approach to raise, gather and distibute
//!  domain-specific errors in libraries and/or applications.
//!
//! Supported types:
//! * ()
//! * boolean
//! * numbers (i32, usize, etc)
//! * Option
//! * Result
//!
//! | Source type                | `throw` condition for the error | `raise` condition |
//! | -------------------------- | ------------------------------- | ----------------- |
//! | numbers (i32, usize, etc)  | != 0                            | == 0              |
//! | bool                       | false                           | true              |
//! | strings (&str, String etc) | is_empty()                      | !is_empty()       |
//! | Option                     | Some                            | None              |
//! | Result                     | Ok                              | Err               |
//!
//! For detailed information, please see the [`SmartErr`][] macro.
//!
//! ## Quick overview
//!
//! ```rust
//! smarterr_fledged!(DomainErrors{
//!     DomainError<<i32>> -> "Domain error"
//! });
//!
//! fn example(val: i32) -> Result<i32, RawError<i32>> {
//!     val.throw()?; // throw error if val != 0
//!     val.throw_with("raw error")?;
//!     val.raise()?; // throw error if val == 0
//!     val.raise_with("raw error")?;
//!
//!     val.throw_err(RawError::new_with(val, "raw error"))?;
//!     val.raise_err(RawError::new_with(val, "raw error"))?;
//!     val.throw_then(|v| RawError::new_with(v, "raw error"))?;
//!     val.raise_then(|v| RawError::new_with(v, "raw error"))?;
//!
//!     Ok(0)
//! }
//!
//! fn example_ctx(val: i32) -> Result<i32, DomainErrors> {
//!     val.throw_ctx(DomainErrorCtx{})?;
//!     val.raise_ctx(DomainErrorCtx{})
//! }
//! ```

use std::{error::Error, fmt::Debug, rc::Rc, sync::Arc};

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
            fn throw(self) -> Result<Self, RawError<Self>> {
                ternary!(($condition(&self)), self, RawError::new(self))
            }

            fn throw_with<S: AsRef<str>>(self, msg: S) -> Result<Self, RawError<Self>> {
                ternary!(($condition(&self)), self, RawError::new_with(self, msg))
            }

            fn raise(self) -> Result<Self, RawError<Self>> {
                ternary!(!($condition(&self)), self, RawError::new(self))
            }

            fn raise_with<S: AsRef<str>>(self, msg: S) -> Result<Self, RawError<Self>> {
                ternary!(!($condition(&self)), self, RawError::new_with(self, msg))
            }
        }
    };
}

macro_rules! simple_erroneous {
    ($t: ty, $condition: expr) => {
        impl<E> Erroneous<E> for $t
        where
            E: std::error::Error,
        {
            type T = Self;
            type ES = Self;

            fn raise_err(self, error: E) -> Result<Self::ES, E> {
                ternary!(!($condition(&self)), self, error)
            }

            fn throw_err(self, error: E) -> Result<Self::T, E> {
                ternary!($condition(&self), self, error)
            }

            fn raise_then<F: FnOnce(Self::T) -> E>(self, error: F) -> Result<Self::ES, E> {
                ternary!(!($condition(&self)), self, error(self))
            }

            fn throw_then<F: FnOnce(Self::ES) -> E>(self, error: F) -> Result<Self::T, E> {
                ternary!($condition(&self), self, error(self))
            }

            fn raise_ctx<C: IntoError<E, Self::T>>(self, ctx: C) -> Result<Self::ES, E> {
                ternary!(!($condition(&self)), self, ctx.into_error(self))
            }

            fn throw_ctx<C: IntoError<E, Self::ES>>(self, ctx: C) -> Result<Self::T, E> {
                ternary!($condition(&self), self, ctx.into_error(self))
            }
        }
    };
}

/// Wrapps error source which might implement _Debug_ trait only
///  into Error-compliant structure adding optional message to it.
/// The default error message is "raw error".
#[derive(Debug)]
pub struct RawError<T: std::fmt::Debug + 'static>
where
    T: std::fmt::Debug + 'static,
{
    value: T,
    msg: Option<String>,
}

impl<T> std::fmt::Display for RawError<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = if let Some(msg) = &self.msg {
            format!("{} {{ value: {:?} }}", msg, self.value)
        } else {
            format!("raw error {{ value: {:?} }}", self.value)
        };
        write!(f, "{}", t.replace("\"", "\'"))
    }
}

impl<T> Error for RawError<T>
where
    T: std::fmt::Debug + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl<T> RawError<T>
where
    T: std::fmt::Debug + 'static,
{
    /// Creates new RawError
    /// ```rust
    /// use smarterr::{RawError, Erroneous};
    /// fn raw_erroneous(val: i32) -> Result<i32, RawError<i32>> {
    ///     val.throw_err(RawError::new(val))
    /// }
    /// ```
    pub fn new(value: T) -> Self {
        Self { value, msg: None }
    }

    /// Creates new RawError with error message attached
    /// ```rust
    /// use smarterr::{RawError, Erroneous};
    /// fn raw_erroneous(val: i32) -> Result<i32, RawError<i32>> {
    ///     val.throw_err(RawError::new_with(val, "raw error"))
    /// }
    /// ```
    pub fn new_with<S: AsRef<str>>(value: T, msg: S) -> Self {
        Self { value, msg: Some(msg.as_ref().to_string()) }
    }

    /// Returns typed error source
    pub fn value(&self) -> &T {
        &self.value
    }
}

pub trait Throwable
where
    Self: std::fmt::Debug + Sized,
{
    /// Converts `self` into `Result` passing `self` as an error source.
    ///
    /// Returns Ok(self) if:
    /// * **numbers**: == 0
    /// * **bool**: true
    /// * **strings**: !is_empty()
    /// * **Option**: Some
    /// * **Result**: Ok
    ///
    /// Otherwise it returns Err(RawError<Self>)
    ///
    /// ```rust
    /// use smarterr::{RawError, Throwable};
    /// fn example(val: i32) -> Result<i32, RawError<i32>> {
    ///     val.throw()
    /// }
    /// ```
    fn throw(self) -> Result<Self, RawError<Self>>;

    /// Converts `self` into `Result` passing `self` as an error source.
    /// Attaches message to error source.
    ///
    /// Returns Ok(self) if:
    /// * **numbers**: == 0
    /// * **bool**: true
    /// * **strings**: !is_empty()
    /// * **Option**: Some
    /// * **Result**: Ok
    ///
    /// Otherwise it returns Err(RawError<Self>)
    ///
    /// ```rust
    /// use smarterr::{RawError, Throwable};
    /// fn example(val: i32) -> Result<i32, RawError<i32>> {
    ///     val.throw_with("error message")
    /// }
    /// ```
    fn throw_with<S: AsRef<str>>(self, msg: S) -> Result<Self, RawError<Self>>;

    /// Converts `self` into `Result` passing `self` as an error source.
    ///
    /// Returns Ok(self) if:
    /// * **numbers**: != 0
    /// * **bool**: false
    /// * **strings**: is_empty()
    /// * **Option**: None
    /// * **Result**: Err
    ///
    /// Otherwise it returns Err(RawError<Self>)
    ///
    /// ```rust
    /// use smarterr::{RawError, Throwable};
    /// fn example(val: i32) -> Result<i32, RawError<i32>> {
    ///     val.raise()
    /// }
    /// ```
    fn raise(self) -> Result<Self, RawError<Self>>;

    /// Converts `self` into `Result` passing `self` as an error source.
    /// Attaches message to error source.
    ///
    /// Returns Ok(self) if:
    /// * **numbers**: != 0
    /// * **bool**: false
    /// * **strings**: is_empty()
    /// * **Option**: None
    /// * **Result**: Err
    ///
    /// Otherwise it returns Err(RawError<Self>)
    ///
    /// ```rust
    /// use smarterr::{RawError, Throwable};
    /// fn example(val: i32) -> Result<i32, RawError<i32>> {
    ///     val.raise_with("error message")
    /// }
    /// ```
    fn raise_with<S: AsRef<str>>(self, msg: S) -> Result<Self, RawError<Self>>;
}

pub trait IntoError<E, ES>
where
    E: std::error::Error,
{
    fn into_error(self, source: ES) -> E;
}

pub trait Erroneous<E>
where
    E: std::error::Error,
{
    type T;
    type ES;

    /// Converts `self` into `Result` passing `self` unchanged
    ///  if error condition is **NOT met**:
    /// * **numbers**: != 0
    /// * **bool**: false
    /// * **strings**: is_empty()
    /// * **Option**: None
    /// * **Result**: Err
    ///
    /// Otherwise it returns `error`
    ///
    /// ```rust
    /// use smarterr::{RawError, Erroneous};
    /// fn example(val: i32) -> Result<i32, RawError<i32>> {
    ///     val.raise_err(RawError::new_with(val, "raw error"))
    /// }
    /// ```
    fn raise_err(self, error: E) -> Result<Self::ES, E>;

    /// Converts `self` into `Result` passing `self` unchanged
    ///  if error condition is **met**:
    /// * **numbers**: == 0
    /// * **bool**: true
    /// * **strings**: !is_empty()
    /// * **Option**: Some
    /// * **Result**: Ok
    ///
    /// Otherwise it returns `error`
    ///
    /// ```rust
    /// use smarterr::{RawError, Erroneous};
    /// fn example(val: i32) -> Result<i32, RawError<i32>> {
    ///     val.throw_err(RawError::new_with(val, "raw error"))
    /// }
    /// ```
    fn throw_err(self, error: E) -> Result<Self::T, E>;

    /// Converts `self` into `Result` passing `self` unchanged
    ///  if error condition is **NOT met**:
    /// * **numbers**: != 0
    /// * **bool**: false
    /// * **strings**: is_empty()
    /// * **Option**: None
    /// * **Result**: Err
    ///
    /// Otherwise it returns `error` from the supplier function
    ///
    /// ```rust
    /// use smarterr::{RawError, Erroneous};
    /// fn example(val: i32) -> Result<i32, RawError<i32>> {
    ///     val.raise_then(|v| RawError::new_with(v, "raw error"))
    /// }
    /// ```
    fn raise_then<F: FnOnce(Self::T) -> E>(self, error: F) -> Result<Self::ES, E>;

    /// Converts `self` into `Result` passing `self` unchanged
    ///  if error condition is **met**:
    /// * **numbers**: == 0
    /// * **bool**: true
    /// * **strings**: !is_empty()
    /// * **Option**: Some
    /// * **Result**: Ok
    ///
    /// Otherwise it returns `error` from the supplier function
    ///
    /// ```rust
    /// use smarterr::{RawError, Erroneous};
    /// fn example(val: i32) -> Result<i32, RawError<i32>> {
    ///     val.throw_then(|v| RawError::new_with(v, "raw error"))
    /// }
    /// ```
    fn throw_then<F: FnOnce(Self::ES) -> E>(self, error: F) -> Result<Self::T, E>;

    /// Converts `self` into `Result` passing `self` unchanged
    ///  if error condition is **NOT met**:
    /// * **numbers**: != 0
    /// * **bool**: false
    /// * **strings**: is_empty()
    /// * **Option**: None
    /// * **Result**: Err
    ///
    /// Otherwise it returns error converted from error context
    ///
    /// ```rust
    /// use smarterr::{RawError, Erroneous};
    /// fn example(val: &str) -> Result<i32, EarthError<ParseIntError>> {
    ///     val.parse::<i32>().raise_ctx(EarthErrorCtx{})
    /// }
    /// ```
    fn raise_ctx<C: IntoError<E, Self::T>>(self, ctx: C) -> Result<Self::ES, E>;

    /// Converts `self` into `Result` passing `self` unchanged
    ///  if error condition is **met**:
    /// * **numbers**: == 0
    /// * **bool**: true
    /// * **strings**: !is_empty()
    /// * **Option**: Some
    /// * **Result**: Ok
    ///
    /// Otherwise it returns error converted from error context
    ///
    /// ```rust
    /// use smarterr::{RawError, Erroneous};
    /// fn example(val: i32) -> Result<i32, EarthError<ParseIntError>> {
    ///     val.parse::<i32>().throw_ctx(EarthErrorCtx{})
    /// }
    /// ```
    fn throw_ctx<C: IntoError<E, Self::ES>>(self, ctx: C) -> Result<Self::T, E>;
}

simple_erroneous!((), |_| true);

simple_erroneous!(bool, |&b| b);
simple_throwable!(bool, |&b| b);

simple_erroneous!(usize, |&x| x == 0);
simple_erroneous!(isize, |&x| x == 0);
simple_throwable!(usize, |&x| x == 0);
simple_throwable!(isize, |&x| x == 0);

simple_erroneous!(u128, |&x| x == 0);
simple_erroneous!(i128, |&x| x == 0);
simple_throwable!(u128, |&x| x == 0);
simple_throwable!(i128, |&x| x == 0);

simple_erroneous!(u64, |&x| x == 0);
simple_erroneous!(i64, |&x| x == 0);
simple_throwable!(u64, |&x| x == 0);
simple_throwable!(i64, |&x| x == 0);

simple_erroneous!(u32, |&x| x == 0);
simple_erroneous!(i32, |&x| x == 0);
simple_throwable!(u32, |&x| x == 0);
simple_throwable!(i32, |&x| x == 0);

simple_erroneous!(u16, |&x| x == 0);
simple_erroneous!(i16, |&x| x == 0);
simple_throwable!(u16, |&x| x == 0);
simple_throwable!(i16, |&x| x == 0);

simple_erroneous!(u8, |&x| x == 0);
simple_erroneous!(i8, |&x| x == 0);
simple_throwable!(u8, |&x| x == 0);
simple_throwable!(i8, |&x| x == 0);

simple_erroneous!(&str, |s: &str| !s.is_empty());
simple_throwable!(&str, |s: &str| !s.is_empty());
simple_erroneous!(String, |s: &String| !s.is_empty());
simple_throwable!(String, |s: &String| !s.is_empty());
simple_erroneous!(Box<String>, |s: &Box<String>| !s.is_empty());
simple_throwable!(Box<String>, |s: &Box<String>| !s.is_empty());
simple_erroneous!(Rc<String>, |s: &Rc<String>| !s.is_empty());
simple_throwable!(Rc<String>, |s: &Rc<String>| !s.is_empty());
simple_erroneous!(Arc<String>, |s: &Arc<String>| !s.is_empty());
simple_throwable!(Arc<String>, |s: &Arc<String>| !s.is_empty());

fn swap_opt<T>(opt: Option<T>) -> Result<(), T> {
    match opt {
        Some(v) => Err(v),
        None => Ok(()),
    }
}

impl<T, E> Erroneous<E> for Option<T>
where
    E: std::error::Error,
{
    type T = T;
    type ES = ();

    fn raise_err(self, error: E) -> Result<Self::ES, E> {
        swap_opt(self).map_err(|_| error)
    }

    fn throw_err(self, error: E) -> Result<Self::T, E> {
        self.ok_or(error)
    }

    fn raise_then<F: FnOnce(Self::T) -> E>(self, error: F) -> Result<Self::ES, E> {
        swap_opt(self).map_err(error)
    }

    fn throw_then<F: FnOnce(Self::ES) -> E>(self, error: F) -> Result<Self::T, E> {
        self.ok_or(error(()))
    }

    fn raise_ctx<C: IntoError<E, Self::T>>(self, ctx: C) -> Result<Self::ES, E> {
        swap_opt(self).map_err(|v| ctx.into_error(v))
    }

    fn throw_ctx<C: IntoError<E, Self::ES>>(self, ctx: C) -> Result<Self::T, E> {
        self.ok_or(ctx.into_error(()))
    }
}

fn swap_res<T, E>(res: Result<T, E>) -> Result<E, T> {
    match res {
        Ok(v) => Err(v),
        Err(e) => Ok(e),
    }
}

impl<T, E, ES> Erroneous<E> for Result<T, ES>
where
    E: std::error::Error,
{
    type T = T;
    type ES = ES;

    fn raise_err(self, error: E) -> Result<Self::ES, E> {
        swap_res(self).map_err(|_| error)
    }

    fn throw_err(self, error: E) -> Result<Self::T, E> {
        self.map_err(|_| error)
    }

    fn raise_then<F: FnOnce(Self::T) -> E>(self, error: F) -> Result<Self::ES, E> {
        swap_res(self).map_err(error)
    }

    fn throw_then<F: FnOnce(Self::ES) -> E>(self, error: F) -> Result<Self::T, E> {
        self.map_err(error)
    }

    fn raise_ctx<C: IntoError<E, Self::T>>(self, ctx: C) -> Result<Self::ES, E> {
        swap_res(self).map_err(|v| ctx.into_error(v))
    }

    fn throw_ctx<C: IntoError<E, Self::ES>>(self, ctx: C) -> Result<Self::T, E> {
        self.map_err(|v| ctx.into_error(v))
    }
}
