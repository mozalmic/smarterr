# SmartErr

**_SmartErr_**, an error handling library, introduces several convenient aproaches to raise, gather and distibute domain-specific errors in libraries and/or applications.

With **_SmartErr_** you'll be able to

* raise errors with `raise` and `throw` methods on regular types (numbers, strings, boolean, _Option_, _Result_, etc) as an error source. Look at [Raising errors](#raising-errors) section to find out more details.
* define the exact set of errors emitted by the function or introduce global set for the public API.
* passthrough unhandled errors of the called functions or handle them completelly or partially with special `handle` method generated for every specific situation.
* attach _context_ to errors and define specific messages both for new and non-handled errors. [Defining errors](#defining-errors) section describes this approache.

## Quick overview

See [this](#fbs-example) example below.

## Raising errors

Some functions may return simple types instead of _Result_. This part of the library is devoted to the processing of this kind of results. Simple values are converted with `raise` (or `raise_with`) and `throw` (or `throw_with`) methods from _Throwable_ trait.
`raise` emits an error if source is NOT treated as failure and `throw` emits an error if it's already in a failure state. Here is a reference table for types that have an implementation of _Throwable_ trait:

| Source type                | `throw` condition for the error | `raise` condition |
| -------------------------- | ------------------------------- | ----------------- |
| numbers (i32, usize, etc)  | != 0                            | == 0              |
| bool                       | false                           | true              |
| strings (&str, String etc) | is_empty()                      | !is_empty()       |
| Option                     | None                            | Some              |
| Result                     | Err                             | OK                |

If the condition is not met, the original value will be returned.

Assume there is some numeric input.
To convert it into _Result_ using _Throwable_:
```rust
fn raw_throwable(val: i32) -> Result<i32, RawError<i32>> {
    val.throw()
    //val.throw_with("raw error")
}

#[test]
pub fn test_throwable()  {
    assert_eq!(raw_throwable(0).unwrap(), 0);
    assert_eq!(raw_throwable(10).is_err(), true);
    assert_eq!(format!("{}", raw_throwable(10).unwrap_err()), 
        "raw error { value: 10 }"
    );
}
```
To convert with _Erroneous_:

```rust
smarterr_fledged!(DomainErrors{
    DomainError<<i32>> -> "Domain error"
});

fn raw_erroneous(val: i32) -> Result<i32, RawError<i32>> {
    val.throw_err(RawError::new_with(val, "raw error"))
}

fn raw_erroneous_then(val: i32) -> Result<i32, RawError<i32>> {
    val.throw_then(|v| RawError::new_with(v, "raw error"))
}

fn raw_erroneous_ctx(val: i32) -> Result<i32, DomainErrors> {
    val.throw_ctx(DomainErrorCtx{})
}

#[test]
pub fn test_erroneous()  {
    assert_eq!(raw_erroneous(0).unwrap(), 0);
    assert_eq!(raw_erroneous_then(10).is_err(), true);
    assert_eq!(format!("{}", raw_erroneous_then(10).unwrap_err()), 
        "raw error { value: 10 }"
    );
    assert_eq!(format!("{}", raw_erroneous_ctx(10).unwrap_err()), 
        "Domain error, caused by: raw error { value: 10 }"
    );
}
```
Domain error processing is described in [Defining errors](#defining-errors) section.

`raise` alternative could be used instead of `throw` as well. The only difference is that the `raise` condition is the opposite of `throw`.

## Defining errors

There are 2 approaches to define errors:
* "_fledged_": domain errors are defined globally (within the selected visibility)
* _function-based_: error set is specific for the each function
  
Both shares the same sintax, with limited inheritance for the fledged style.

### Fledged style

Fledged style is mostly convenient for standalone doman-specific errors.
The following example demonstrates the usage of _smarterr_fledged_ macros which is designed to support fledged approach.
```rust
smarterr_fledged!(pub PlanetsError {
    MercuryError{} -> "Mercury error",
    pub MarsError{ind: usize} -> "Mars Error",
    SaturnError<<i32>> -> "Saturn error",
    EarthError<ParseIntError> -> "EarthError",
});
```
First it should be defined the name of the error set and (optionally) it's visibility. Then goes certain errors definition inside curly braces. It follows simple pattern:
```
    [visibility] name[<[< source error type >]>] [{ context struct }] -> "error message",
```
The following code will be generated under the hood (shown without minor details and cutted to _MarsError_ only):

```rust
#[derive(Debug)]
pub enum PlanetsError {
    MercuryError(MercuryError),
    MarsError(MarsError),
    SaturnError(SaturnError),
    EarthError(EarthError),
}

/* cutted: Error and Display implementations for PlanetsError */

#[derive(Debug)]
pub struct MarsError {
    ctx: MarsErrorCtx,
}

impl MarsError {
    pub fn new<ES>(_src: ES, ctx: MarsErrorCtx) -> Self {
        MarsError { ctx }
    }
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
    pub fn default_message(&self) -> &'static str {
        "Mars Error"
    }
}

/* cutted: Display implementation for MarsError */

#[derive(Debug)]
#[allow(dead_code)]
pub struct MarsErrorCtx {
    ind: usize,
}

impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<PlanetsError, ES> for MercuryErrorCtx {
    fn into_error(self, source: ES) -> PlanetsError {
        PlanetsError::MercuryError(MercuryError::new(source, self))
    }
}
impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<PlanetsError, ES> for MarsErrorCtx {
    fn into_error(self, source: ES) -> PlanetsError {
        PlanetsError::MarsError(MarsError::new(source, self))
    }
}
impl smarterr::IntoError<PlanetsError, i32> for SaturnErrorCtx {
    fn into_error(self, source: i32) -> PlanetsError {
        PlanetsError::SaturnError(SaturnError::new(source, self))
    }
}
impl smarterr::IntoError<PlanetsError, ParseIntError> for EarthErrorCtx {
    fn into_error(self, source: ParseIntError) -> PlanetsError {
        PlanetsError::EarthError(EarthError::new(source, self))
    }
}
```

Several key details for the generated code:

1. Domain error set is the enum.
2. For each error (enum value) additional structure is created, its name is the same as the name of the error.
3. If context has been defined, the corresponding structure will be created. Its name is the error name followed with the `Ctx` suffix.

The example above it pretty simple and does not demonstate source error definition. Usually you'd like to set up source error. There are several posibilites:

| source        | definition example                             |
| ------------- | ---------------------------------------------- |
| no source     | `MercuryError -> "Mercury error"`              |
| dyn Error     | `MercuryError<> -> "Mercury error"`            |
| certain error | `MercuryError<SourceError> -> "Mercury error"` |
| dyn Debug     | `MercuryError<<>> -> "Mercury error"`          |
| certain Debug | `MercuryError<<i32>> -> "Mercury error"`       |

Raising errors is pretty simple:
```rust
"z12".parse::<i32>().throw_ctx(EarthErrorCtx{})
```
Note that it's done with _*Ctx_ structure (EarthErrorCtx in this example) which has an implementation of _smarterr::IntoError_ trait.

## Function-based style

This is a common situation when there are several functions calling from each other. Usually each function returns its own error set and some unhandled errors from the called one. Generally it is possible to use one error set (enum) for all functions but that's not quite right. The functions' contracts are inaccurate since they return subset of the common enum and some errors will never happen. If some functions are public it might be a problem to hide unused errors from the internals.

The more precise solution is to define its own error set for each function. But besides being quite difficult, it creates another problem. Some errors may be defined several times for each error set and require mapping between them even that they are the same. _SmartErr_ solves this problem providing all necessary and optimized stuff behind the scenes. 

For this, 2 additional keywords were introduced:

* _from_ keyword. It should be used if some errors from the called function need to be rethrown.
* _handle_ keyword. It is intended to mark errors from the called function which will be handled.

Here's how it works:

#### FBS example
```rust
#[smarterr(
    AlfaError{ind: i32, ext: String} -> "Alfa error",
    BetaError<>{ind: i32} -> "Beta error",
    BetaWrappedError<ParseIntError> -> "Beta Wrapped Error",
    GammaError<<>>{ext: String} -> "Gamma error",
    GammaWrappedError<<i32>>{ext: String} -> "Gamma Wrapped error",
)]
pub fn greek_func(err_ind: usize) -> String {
    let ok_str = "All is ok".to_string();
    let err_str = "Error raised".to_string();
    let ext = "ext".to_string();
    match err_ind {
        0 => Ok(ok_str),
        1 => err_str.raise_ctx(AlfaErrorCtx { ind: -1, ext }),
        2 => "z12".parse::<i32>().throw_ctx(BetaErrorCtx { ind: -2 }).map(|_| ok_str),
        3 => "z12".parse::<i32>().throw_ctx(BetaWrappedErrorCtx {}).map(|_| ok_str),
        4 => err_str.raise_ctx(GammaErrorCtx { ext }),
        5 => 5000000.throw_ctx(GammaWrappedErrorCtx { ext }).map(|_| ok_str),
        _ => Ok(ok_str),
    }
}

#[smarterr(
    from GreekFuncError { 
        AlfaError, BetaError<>, BetaWrappedError<ParseIntError>, GammaError<<>>, 
        handled GammaWrappedError
    },
    XError{ind: i32, ext: String} -> "X error",
    YError{ind: i32} -> "Y error",
    pub ZError<<String>>{ind: usize} -> "Z Error",
)]
fn latin_func(err_ind: usize) {
    greek_func(err_ind).handle(|h| match h {
        GreekFuncErrorHandled::GammaWrappedError(data) => 
            data.ctx.ext.throw_ctx(ZErrorCtx { ind: err_ind }),
    })?;
    Ok(())
}

#[smarterr(
    from GreekFuncError {
        AlfaError -> "Imported Alfa error",
        BetaError<> -> "Imported Beta error",
        BetaWrappedError<std::num::ParseIntError> -> "Imported Beta Wrapped Error",
        handled GammaError,
        handled GammaWrappedError,
    },
    from LatinFuncError {
        AlfaError, BetaError<>, BetaWrappedError<ParseIntError>, ZError<<String>>, 
        handled { GammaError, XError, YError }
    },
    FirstError{ind: i32, ext: String} -> "First error",
    SecondError{ind: i32} -> "Second error",
    ThirdError{} -> "Third Error",
)]
pub fn numeric_func(err_ind: usize) -> String {
    let g = greek_func(err_ind).handle(|h| match h {
        GreekFuncErrorHandled::GammaWrappedError(e) => 
            e.ctx.ext.clone().raise_ctx(FirstErrorCtx{ind: err_ind as i32, ext: e.ctx.ext}),
        GreekFuncErrorHandled::GammaError(e) => 
            e.ctx.ext.raise_ctx(SecondErrorCtx{ ind: err_ind as i32 }), 
    })?;

    latin_func(err_ind).handle(|h| match h {
        LatinFuncErrorHandled::XError(e)=>
            ().raise_ctx(FirstErrorCtx{ ind: err_ind as i32, ext: e.ctx.ext }),
        LatinFuncErrorHandled::YError(e)=>
            ().raise_ctx(SecondErrorCtx{ ind: e.ctx.ind }),
        LatinFuncErrorHandled::GammaError(_) => Ok(())
    })?;

    let t = ().raise_ctx(MarsErrorCtx{ind: err_ind});
    t.throw_ctx(BetaErrorCtx{ ind: err_ind as i32 })?;

    Ok(g)
}
```

It is also possible to define errors for methods. The only difference is that theses errors must be defined outside the implementation block. `smarterr_mod` macro is intended to do this. It should be used as an attribute for the implementation block. The name of the module should be passed as an argument.

Here's an example:
```rust
#[smarterr_mod(test_err)]
impl Test {
    #[smarterr(InitFailed{pub a: String, pub b: String} -> "Init error")]
    pub fn new(a: &str, b: &str) -> Self {
        Ok(Self {
            a: a.parse()
                .throw_ctx(test_err::InitFailedCtx { a: a.to_string(), b: b.to_string() })?,
            b: b.parse()
                .throw_ctx(test_err::InitFailedCtx { a: a.to_string(), b: b.to_string() })?,
        })
    }
}
```