# SmartErr

**_SmartErr_**, an error handling library, introduces several convenient aproaches to define, raise and handle domain-specific errors in libraries and/or applications.

With **_SmartErr_** it is possible to:

* raise errors with `raise` and `throw` methods on regular types (numbers, strings, boolean, _Option_, _Result_, etc) as an error source. Look into [Raising errors](#raising-or-throwing-errors) section to find out more details.
* define atomic errors with `error!` macro (requires `atomic_error` feature).
* define the exact set of errors emitted by the function or introduce global set for the public API (requires `errorset` feature).

## Installation

Features are disabled by default. To use `error!`, `#[errorset]`, and conversions
between error sets, enable `errorset`:

```toml
[dependencies]
smarterr = { version = "1.2", features = ["errorset"] }
thiserror = "2.0"
```

The `errorset` feature also enables `atomic_error`. A direct `thiserror`
dependency is needed because the generated code uses its derive macro.
For `throw` and `raise` alone, no features are required.

## Defining atomic errors

With `atomic_error` feature enabled (it is also included in the `errorset` feature), it is possible to define atomic errors, which are structs, annotated with `thiserror::Error` attribute. For instance, the following definition:

```rust,ignore
error!(pub InvalidData<ParseIntError> { pub data: String } => "Invalid data provided [data={data}]");
```

will generate the code:
```rust,ignore
#[derive(thiserror::Error, Debug)]
#[error("Invalid data provided [data={data}]")]
pub struct InvalidData {
    pub data: String,
    #[source]
    pub source: ParseIntError,
}
```

## Raising or throwing errors

Sometimes functions may return simple types instead of _Result_. Library provides a set of methods to convert these types into _Result_ based on the convention what values should be treated as an error:

| Source type                | error state for the type |
| -------------------------- | ------------------------ |
| numbers (i32, usize, etc)  | != 0                     |
| bool                       | false                    |
| strings (&str, String etc) | is_empty()               |
| Option                     | None                     |
| Result                     | Err                      |

Further actions depend on the function used to convert the value into _Result_:

**throw** - does NOT change error context. When the error state is detected, the value is mapped with the provided `err_map` function and returned as `Err(err_map(original_value))`. Otherwise, the original value is just wrapped into _Result::Ok_.

**raise** - CHANGES error context. When the error state is detected, the original value is wrapped into _Result::Ok_. Otherwise, the value is mapped with the provided `ok_to_err_map` function and returned as `Err(ok_to_err_map(original_value))`.

## Defining output error set

With `errorset` feature enabled, it is possible to define a set of errors emitted by the function. The `errorset` macro generates a new error type that contains all the errors from the function signature. 

You may find more details in the [ErrorSet crate](https://docs.rs/errorset/1.2.0/errorset/) documentation.

`SmartErr` crate reexports `ErrorSet` crate, so you may use it functionality directly.

For instance, the following code:

```rust,ignore
#[errorset]
pub fn parse_u32(data: &str) -> Result<u32, (InvalidData, OutOfBounds)> {
    ...
}
```

will produce the code:
```rust,ignore
#[derive(::thiserror::Error, Debug)]
pub enum ParseU32Errors {
    #[error(transparent)]
    InvalidData(#[from] InvalidData),
    #[error(transparent)]
    OutOfBounds(#[from] OutOfBounds),
}
impl _SomeStruct {
    fn parse_u32(data: &str) -> Result<u32, ParseU32Errors> {
        ...
    }
}
```

## Converting between method error sets

The target method still lists its atomic errors explicitly. `from(...)` generates
conversions; it does not add errors to that list.

```rust
use smarterr::{error, errorset::errorset, ErrorSet};

error!(pub FileError {} => "File error");
error!(pub VersionError {} => "Unsupported version");
error!(pub CryptoError {} => "Crypto error");

#[errorset]
pub fn read() -> Result<(), (FileError, VersionError, CryptoError)> {
    Err(VersionError {}.into())
}

#[errorset(from(ReadErrors))]
pub fn load() -> Result<(), (CryptoError,)> {
    match read() {
        Ok(()) => {}
        Err(error) => match error.split::<LoadErrors>()? {
            ReadErrorsForLoadErrors::FileError(e) => {
                let _ = e; // handle locally
            }
            ReadErrorsForLoadErrors::VersionError(e) => {
                let _ = e; // handle locally
            }
        },
    }
    Ok(())
}
```

`split::<LoadErrors>()` returns `Result<ReadErrorsForLoadErrors, LoadErrors>`.
Errors in the target contract go into `Err` and propagate through `?`. Excluded
errors go into `Ok`, in an enum containing only those variants. Rust checks that
the `match` handles all of them. There is no boxing, type erasure, or panic.

If every source variant is included in the target, `From<Source>` is also
generated, so the target can simply call `read()?`. For a partial conversion,
that call deliberately does not compile: use `split` and handle the remainder.

The macro checks conversions and exhaustive handling of excluded errors. It does
not analyze method bodies to detect unused or unreachable errors in the declared
tuple; keeping that contract precise remains the author's responsibility.

Source sets automatically expose hidden metadata in their own module. No
`export` annotation is required for `from(path::SourceErrors)`, including across
crates. Methods in different modules can have the same name. Reexporting a set
with `pub use module::SourceErrors as PublicErrors` also carries its metadata.
The existing `export` and `export(custom_descriptor)` options remain available
for explicitly named descriptor aliases.

## Classifying by atomic error type

Import `smarterr::ErrorSet` to borrow the active atom without listing every
source set or inspecting transparent wrappers:

```rust
use smarterr::{error, errorset::errorset, ErrorSet};

error!(pub ChatClosed {} => "Chat closed");
error!(pub FileError { pub path: String } => "File error: {path}");

#[errorset]
fn send() -> Result<(), (ChatClosed, FileError)> {
    Err(ChatClosed {}.into())
}

let error = send().unwrap_err();
assert!(error.downcast_ref::<ChatClosed>().is_some());
assert!(error.downcast_ref::<FileError>().is_none());
```

`downcast_ref::<Atom>()` works on generated sets and `split` remainders. It
returns a reference to the stored atom, with no allocation or cloning. It checks
only that atom's exact type (including type aliases), never nested causes.
`Display` and `std::error::Error::source()` retain their transparent behavior;
use `source()` to inspect the atom's cause, not to classify the atom itself.
`ErrorSet::as_error()` also exposes the atom as `&dyn std::error::Error`.
A generic classifier needs only an `E: ErrorSet` bound, not a list of source
sets.

**Compatibility:** `ErrorSet` is now implemented for generated sets, rather
than blanket-implemented for every type. Generated sets retain `split()`.
User-defined types with manual `SplitInto` implementations must implement
`ErrorSet::as_error()` to retain the extension method, or call
`SplitInto::split_into()` directly.

See the [errorset documentation](https://docs.rs/errorset/1.2.0/errorset/#conversions-and-partial-handling)
for modules, crates, aliases, and restrictions.