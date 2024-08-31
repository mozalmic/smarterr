# ErrorSet

This library introduces simple approach of managing errors in applications and libraries.
It is based on `thiserror` and `anyhow` and extends their functionality with `errorset` macro.
Here is a simple example of how to use it:

```rust

// define new errors

#[derive(Error, Debug)]
#[error("The first error [data={data}]")]
pub struct Error1 {
    pub data: u32,
    #[source]
    pub source: ParseIntError,
}

#[derive(Error, Debug)]
#[error("The second error [data={data}]")]
pub struct Error2 {
    pub data: u32,
    #[source]
    pub source: ParseIntError,
}

// use them in the code
struct SomeStruct {}

#[errorset(pub(crate) mod errors)]
impl SomeStruct {
    #[errorset]
    fn method_one(&self) -> Result<(), (Error1, Error2)> {
        todo!()
    }
    #[errorset]
    fn method_two(&self) -> Result<(), (Error1, Error2)> {
        todo!()
    }
    fn method_wo_erros(&self) -> u32 {
        todo!()
    }
}

// or just for the regular function
#[errorset(pub mod fn_errors)]
pub fn error_set() -> Result<(), (Error1, Error2)> {
    todo!()
}

```

The `errorset` macro generates a new error type that contains all the errors from the function signature. Here is how the generated code looks like:

```rust
pub(crate) mod errors {
    use super::*;
    #[derive(::thiserror::Error, Debug)]
    pub enum MethodOneErrors {
        #[error(transparent)]
        Error1(#[from] Error1),
        #[error(transparent)]
        Error2(#[from] Error2),
    }
    #[derive(::thiserror::Error, Debug)]
    pub enum MethodTwoErrors {
        #[error(transparent)]
        Error1(#[from] Error1),
        #[error(transparent)]
        Error2(#[from] Error2),
    }
}
impl SomeStruct {
    fn method_one(&self) -> Result<(), errors::MethodOneErrors> {
        $crate::panicking::panic("not yet implemented")
    }
    fn method_two(&self) -> Result<(), errors::MethodTwoErrors> {
        $crate::panicking::panic("not yet implemented")
    }
    fn method_wo_erros(&self) -> u32 {
        $crate::panicking::panic("not yet implemented")
    }
}

pub mod fn_errors {
    use super::*;
    #[derive(::thiserror::Error, Debug)]
    pub enum ErrorSetErrors {
        #[error(transparent)]
        Error1(#[from] Error1),
        #[error(transparent)]
        Error2(#[from] Error2),
    }
}
fn error_set() -> Result<(), fn_errors::ErrorSetErrors> {
    $crate::panicking::panic("not yet implemented")
}
```

So, there are few steps `errorset` macro does:
1. First it looks for the `#[errorset]` attribute and gathers all the errors from the function signature. Function signature must return some generic type with 2 parameters. The second parameter mush be a tuple of errors. You may use any type that meets these requirements. For example, `Result<(), (Error1, Error2)>` or `MyIncredibleObject<String, (Error1, Error2, Error3)>`.
2. Then macro generates a new enum error type that contains all the errors from the function signature placing the generated error type in the module if it was defined in macro.
3. Finally original error tuple is replaced with the generated error type.

Generated error enum is just a transparent wrapper around the original error types. This allows to define erors only once and reuse them across the code without any additional boilerplate.