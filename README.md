# SmartErr

**_SmartErr_**, an error handling library, introduces several convenient aproaches to define, raise and handle domain-specific errors in libraries and/or applications.

With **_SmartErr_** it is possible to:

* raise errors with `raise` and `throw` methods on regular types (numbers, strings, boolean, _Option_, _Result_, etc) as an error source. Look into [Raising errors](#raising-errors) section to find out more details.
* define atomic errors with `error!` macro (requires `atomic_error` feature).
* define the exact set of errors emitted by the function or introduce global set for the public API (requires `erorrset` feature).

## Defining atomic errors

With `atomic_error` feature enabled (it is also included into `smarterr` feature), it is possible to define atomic errors, which are structs, annotated with `thiserror::Error` attribute. For instance, the following definition:

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

You may find more details in the [ErrorSet crate](https://crates.io/crates/errorset) documentation.

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