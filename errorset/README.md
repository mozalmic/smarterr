# ErrorSet

This library introduces simple approach of managing errors in applications and libraries.
It is based on `thiserror` and `anyhow` and extends their functionality with `errorset` macro.
Here is a simple example of how to use it:

```rust,ignore

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

```rust,ignore
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

For one error, write `Result<T, (Atom,)>`: the trailing comma makes a tuple.
With runtime support or `export`/`from`, `Result<T, Atom>` and
`Result<T, (Atom)>` are diagnosed with a suggestion to remove `#[errorset]`
or use `(Atom,)`. Standalone function attributes without runtime or composition
retain the legacy behavior of leaving non-tuple error types unchanged.

## Named sets

Declare a set without a placeholder function:

```rust,ignore
/// Errors exposed by the API.
#[errorset(from(ReadErrors, SendErrors))]
pub type ApiErrors = (FileError, VersionError, CryptoError);

#[errorset(from(ApiErrors))]
pub type StorageErrors = (FileError,);
```

This syntax generates a **new enum `ApiErrors`**, not a Rust type alias or
tuple. It uses the declared name and visibility and transfers doc comments and
other attributes to the enum. The tuple is the complete explicit contract:
`from(...)` adds conversions, not members. Full conversions implement `From`;
partial ones provide `split` with an exhaustive remainder, exactly as for
function-generated sets.

Named sets also expose their same-named hidden descriptor in the declaration's
module and, with runtime support, implement `ErrorSet` and `ErrorSetMember`.
They can be sources for other declarations or methods, including through
renamed reexports and across crates. `export` and custom descriptor aliases
work as on functions. Place declarations in the desired module and use their
own visibility; attribute module/visibility options are not supported here.

Use `(Atom,)` for a single atom and `()` for an empty set. Generic declarations
(`type ApiErrors<T> = ...`, including lifetime/const parameters and where
clauses) are rejected. Concrete generic atom types remain supported.
Repeated identical atom syntax is rejected with the duplicate and original
positions; function signatures keep their existing deduplication behavior.
Different aliases of the same type remain subject to rustc's overlapping
`From` checks. Nested sets are not automatically flattened.

The attribute cannot inspect neighboring declarations. Name collisions, such
as `SendErrors` declared here and generated by a `send` method, are reported
by rustc (E0428) with both declaration sites, without a global macro registry.

## Conversions and partial handling

Composition and atom introspection require the `smarterr` runtime and `thiserror`
dependencies. Using `smarterr` with its `errorset` feature enables the proc
macro's `runtime` feature automatically. Direct users of the `errorset` crate
should enable its `runtime` feature and depend on `smarterr` to use these APIs.
Standalone uses without `runtime`, `export`, or `from` still need no runtime
traits.

Every generated enum exposes a hidden descriptor macro with the **same name**
in the enum's module; Rust's type and macro namespaces are separate. No `export`
annotation is needed on the source. This macro carries variant names and
indices, not resolved Rust types. Generated
`smarterr::ErrorSetMember<INDEX>` implementations expose the actual payload types
to the Rust compiler.

On the target method, `#[errorset(from(ReadErrors))]` requests a conversion from
`ReadErrors`, using its same-named metadata macro. Multiple sources are supported:

```rust,ignore
#[errorset(from(ReadErrors, DecryptErrors))]
fn load() -> Result<(), (FileError, VersionError, CryptoError, InvalidKey)> {
    read()?;
    decrypt()?;
    Ok(())
}
```

The explicit tuple remains the entire contract. For each source, the macro
generates `SplitInto<Target>` and a `<SourceName>For<TargetName>` remainder enum.
If multiple sources have the same final name, their remainder names use the
source paths in PascalCase to avoid collisions (for example,
`ApiReadErrorsForLoadErrors` and `StorageReadErrorsForLoadErrors`).
Import `smarterr::ErrorSet` to call:

```rust,ignore
match error.split::<LoadErrors>()? {
    ReadErrorsForLoadErrors::FileError(e) => { /* handle */ }
    ReadErrorsForLoadErrors::VersionError(e) => { /* handle */ }
}
```

`split` places propagated errors in `Err(Target)` and excluded errors in
`Ok(Remainder)`. The remainder holds the original atomic payloads, and the match
is exhaustive. If the remainder is empty, `From<Source> for Target` is also
generated. Otherwise there is no full `From`, no implicit fallback, and no panic.
An error with a `source` field remains atomic; its source is never expanded.

### Modules and crates

The default descriptor path follows the source type path:

```rust,ignore
// In a producer crate:
pub mod api {
    #[errorset]
    pub fn read() -> Result<(), (FileError, VersionError)> { /* ... */ }
}

// In a consumer crate; `reader` may be a renamed Cargo dependency:
#[errorset(from(reader::api::ReadErrors))]
pub fn load() -> Result<(), (reader::FileError,)> {
    match reader::api::read() {
        Ok(()) => {}
        Err(e) => match e.split::<LoadErrors>()? {
            ReadErrorsForLoadErrors::VersionError(e) => { /* handle */ }
        },
    }
    Ok(())
}
```

`export` additionally exposes the legacy `<EnumName>Descriptor` alias.
`export(custom_descriptor)` exposes an explicitly named descriptor alias.
`from(Source => path::custom_descriptor)` specifies its path explicitly.
For a producer built with an older macro version, use this explicit form with
its existing descriptor alias, or rebuild it to expose same-named metadata.
With `#[errorset(pub mod errors)]`, the descriptor is in `errors` alongside the
enum. On inherent impls, place `export`/`from` on individual marked methods and
the module option on the impl block.

Without a generated module, the enum, metadata macro, descriptor alias, and remainder have the
same visibility as the function or method (`pub`, `pub(crate)`, `pub(super)`,
`pub(in ...)`, or private). With a generated module, enums and descriptor aliases
are `pub` inside that module, as in the original API; the module's visibility
restricts access from outside. Function/method visibility itself is unchanged.
Private source types and modules cannot be used to generate conversions outside
their Rust visibility scope.

Descriptor macros use automatically unique exported internal names. Two
modules may declare identically named enums or descriptor aliases without
renaming methods or configuring custom exports.
The hidden internal macro is exported at crate root even for a private source;
only its module alias follows the declared visibility. It contains variant
names and indices, not payload values, and does not bypass Rust type privacy.
Declaration order is not significant; no global registry of source sets is
used. A counter allocates internal macro names only; composition resolves the
module-scoped metadata through Rust's name resolution. Descriptors are a
version-one internal protocol, not a general-purpose type-reflection facility.

### Type identity, aliases, and variant names

* Repeated **identical type syntax** in one tuple is deduplicated.
* Distinct paths are not assumed equal. If their last identifiers collide
  (`a::Error`, `b::Error`), variants are named from their paths (`AError`,
  `BError`). An unresolved naming collision is an error.
* Between sets, matching variant names propose a conversion. Rust then checks
  that the payload types are actually equal. A different type with the same
  variant name causes a type error, never a silent merge.
* When aliases/reexports change variant names, map them explicitly:

  ```rust,ignore
  #[errorset(from(reader::api::ReadErrors {
      FileError => LocalFileError
  }))]
  fn load() -> Result<(), (LocalFileError,)> { /* ... */ }
  ```

  Mappings name enum variants, not types. Unknown names and duplicate source
  mappings are diagnosed. Without a mapping, an unmatched source variant stays
  in the remainder, even if rustc would consider its payload equal to a target
  payload. The macro does not resolve aliases.
* Reexporting an enum also reexports its same-named metadata macro. Renamed
  reexports work directly with `from(PublicErrors)`. Separately named descriptor
  aliases still need their own reexport or an explicit path with `=>`.
* Listing two aliases of the same type as separate atoms is unsupported: rustc
  rejects overlapping `From` implementations. It is not silently deduplicated.

### Limits

Concrete generic payloads such as `ParseError<u32>` are supported. Payloads
depending on function/impl generic parameters, lifetimes, const parameters, or
`Self` are not supported for composition. Generic source sets, associated source
types, and composition on trait impls are unsupported. Success types and method
bodies may still be generic.

Use `cfg` on whole declarations, and ensure source/target declarations exist
under the same configuration. Rust evaluates `cfg_attr` before the proc macro;
the macro processes only the resulting declaration, not all possible
configurations. Per-member conditional contracts are unsupported. No automatic
removal of locally handled errors and no body-based contract lint is implemented.

The emitted code currently expects dependency names `smarterr` and `thiserror`.
Producer crates containing the atomic errors may be renamed freely.

### Atomic introspection

With runtime support enabled, generated enums and remainder enums support
`error.downcast_ref::<Atom>()` via `smarterr::ErrorSet`. This borrows the active
atomic payload by its actual Rust type, independently of variant names and
aliases. An absent atom returns `None`; nested causes are not searched.
Transparent formatting and `source()` are unchanged. Runtime-supported atoms
must implement `std::error::Error + 'static`.
`ErrorSet::as_error()` borrows the same atom as `&dyn std::error::Error`.
Generic classifiers need only an `E: ErrorSet` bound. The previous blanket
implementation for all types is removed: manual `SplitInto` types must also
implement `ErrorSet::as_error()` to use the `split()` extension.

Run the two-crate regression fixture with:

```sh
cargo test --manifest-path tests/cross-crate/Cargo.toml --workspace
```

from the repository root. The root test suite also checks that partial direct
`?`, incomplete handling, and unsupported declarations fail to compile.