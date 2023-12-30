#![recursion_limit = "512"]

//! # SmartErr
//!
//! SmartErr, an error handling library, introduces several convenient aproaches
//!  to raise, gather and distibute domain-specific errors in libraries and/or
//!  applications.
//!
//! With **_SmartErr_** you'll be able to
//!
//! * raise errors with `raise` and `throw` methods on regular types (numbers,
//!  strings, boolean, _Option_, _Result_, etc) as an error source.
//!  Look at [Raising errors](#raising-errors) section to find out more details.
//! * define the exact set of errors emitted by the function or introduce
//!  global set for the public API.
//! * passthrough unhandled errors of the called functions or handle them
//!  completelly or partially with special `handle` method generated
//!  for every specific situation.
//! * attach _context_ to errors and define specific messages both for new and
//!  non-handled errors. [Defining errors](#defining-errors) section describes
//!  this approache.
//!
//! ## Quick overview
//!
//! See [this](#fbs-example) example below.
//!
//! ## Raising errors
//!
//! Some functions may return simple types instead of _Result_. This part of
//!  the library is devoted to the processing of this kind of results. Simple
//!  values are converted with `raise` (or `raise_with`) and `throw` (or
//!  `throw_with`) methods from _Throwable_ trait.
//! `raise` emits an error if source is NOT treated as failure and `throw` emits
//!  an error if it's already in a failure state. Here is a reference table for
//!  types that have an implementation of _Throwable_ trait:
//!
//! | Source type                | `throw` condition for the error | `raise` condition |
//! | -------------------------- | ------------------------------- | ----------------- |
//! | numbers (i32, usize, etc)  | != 0                            | == 0              |
//! | bool                       | false                           | true              |
//! | strings (&str, String etc) | is_empty()                      | !is_empty()       |
//! | Option                     | Some                            | None              |
//! | Result                     | Ok                              | Err               |
//!
//! If the condition is not met, the original value will be returned.
//!
//! Assume there is some numeric input.
//! To convert it into _Result_ using _Throwable_:
//! ```rust
//! fn raw_throwable(val: i32) -> Result<i32, RawError<i32>> {
//!     val.throw()
//!     //val.throw_with("raw error")
//! }
//!
//! #[test]
//! pub fn test_throwable()  {
//!     assert_eq!(raw_throwable(0).unwrap(), 0);
//!     assert_eq!(raw_throwable(10).is_err(), true);
//!     assert_eq!(format!("{}", raw_throwable(10).unwrap_err()),
//!         "raw error { value: 10 }"
//!     );
//! }
//! ```
//! To convert with _Erroneous_:
//!
//! ```rust
//! smarterr_fledged!(DomainErrors{
//!     DomainError<<i32>> -> "Domain error"
//! });
//!
//! fn raw_erroneous(val: i32) -> Result<i32, RawError<i32>> {
//!     val.throw_err(RawError::new_with(val, "raw error"))
//! }
//!
//! fn raw_erroneous_then(val: i32) -> Result<i32, RawError<i32>> {
//!     val.throw_then(|v| RawError::new_with(v, "raw error"))
//! }
//!
//! fn raw_erroneous_ctx(val: i32) -> Result<i32, DomainErrors> {
//!     val.throw_ctx(DomainErrorCtx{})
//! }
//!
//! #[test]
//! pub fn test_erroneous()  {
//!     assert_eq!(raw_erroneous(0).unwrap(), 0);
//!     assert_eq!(raw_erroneous_then(10).is_err(), true);
//!     assert_eq!(format!("{}", raw_erroneous_then(10).unwrap_err()),
//!         "raw error { value: 10 }"
//!     );
//!     assert_eq!(format!("{}", raw_erroneous_ctx(10).unwrap_err()),
//!         "Domain error, caused by: raw error { value: 10 }"
//!     );
//! }
//! ```
//! Domain error processing is described in
//!  [Defining errors](#definig_errors) section.
//!
//! `raise` alternative could be used instead of `throw` as well. The only
//!  difference is that the `raise` condition is the opposite of `throw`.
//!
//! ## Defining errors
//!
//! There are 2 approaches to define errors:
//! * "_fledged_": domain errors are defined globally (within the selected
//!  visibility)
//! * _function-based_: error set is specific for the each function
//!   
//! Both shares the same sintax, with limited inheritance for the fledged style.
//!
//! ### Fledged style
//!
//! Fledged style is mostly convenient for standalone doman-specific errors.
//! The following example demonstrates the usage of _smarterr_fledged_ macros
//!  which is designed to support fledged approach.
//! ```rust
//! smarterr_fledged!(pub PlanetsError {
//!     MercuryError{} -> "Mercury error",
//!     pub MarsError{ind: usize} -> "Mars Error",
//!     SaturnError<<i32>> -> "Saturn error",
//!     EarthError<ParseIntError> -> "EarthError",
//! });
//! ```
//! First it should be defined the name of the error set and (optionally) it's
//!  visibility. Then goes certain errors definition inside curly braces.
//!  It follows simple pattern:
//! ```
//!     [visibility] name[<[< source error type >]>] [{ context struct }] -> "error message",
//! ```
//! The following code will be generated under the hood (shown without minor
//!  details and cutted to _MarsError_ only):
//!
//! ```rust
//! #[derive(Debug)]
//! pub enum PlanetsError {
//!     MercuryError(MercuryError),
//!     MarsError(MarsError),
//!     SaturnError(SaturnError),
//!     EarthError(EarthError),
//! }
//!
//! /* cutted: Error and Display implementations for PlanetsError */
//!
//! #[derive(Debug)]
//! pub struct MarsError {
//!     ctx: MarsErrorCtx,
//! }
//!
//! impl MarsError {
//!     pub fn new<ES>(_src: ES, ctx: MarsErrorCtx) -> Self {
//!         MarsError { ctx }
//!     }
//!     pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//!         None
//!     }
//!     pub fn default_message(&self) -> &'static str {
//!         "Mars Error"
//!     }
//! }
//!
//! /* cutted: Display implementation for MarsError */
//!
//! #[derive(Debug)]
//! #[allow(dead_code)]
//! pub struct MarsErrorCtx {
//!     ind: usize,
//! }
//!
//! impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<PlanetsError, ES> for MercuryErrorCtx {
//!     fn into_error(self, source: ES) -> PlanetsError {
//!         PlanetsError::MercuryError(MercuryError::new(source, self))
//!     }
//! }
//! impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<PlanetsError, ES> for MarsErrorCtx {
//!     fn into_error(self, source: ES) -> PlanetsError {
//!         PlanetsError::MarsError(MarsError::new(source, self))
//!     }
//! }
//! impl smarterr::IntoError<PlanetsError, i32> for SaturnErrorCtx {
//!     fn into_error(self, source: i32) -> PlanetsError {
//!         PlanetsError::SaturnError(SaturnError::new(source, self))
//!     }
//! }
//! impl smarterr::IntoError<PlanetsError, ParseIntError> for EarthErrorCtx {
//!     fn into_error(self, source: ParseIntError) -> PlanetsError {
//!         PlanetsError::EarthError(EarthError::new(source, self))
//!     }
//! }
//! ```
//!
//! Several key details for the generated code:
//!
//! 1. Domain error set is the enum.
//! 2. For each error (enum value) additional structure is created, its name is
//!  the same as the name of the error.
//! 3. If context has been defined, the corresponding structure will be created.
//!  Its name is the error name followed with the `Ctx` suffix.
//!
//! The example above it pretty simple and does not demonstate source error
//!  definition. Usually you'd like to set up source error. There are several
//!  posibilites:
//!
//! | source        | definition example                             |
//! | ------------- | ---------------------------------------------- |
//! | no source     | `MercuryError -> "Mercury error"`              |
//! | dyn Error     | `MercuryError<> -> "Mercury error"`            |
//! | certain error | `MercuryError<SourceError> -> "Mercury error"` |
//! | dyn Debug     | `MercuryError<<>> -> "Mercury error"`          |
//! | certain Debug | `MercuryError<<i32>> -> "Mercury error"`       |
//!
//! Raising errors is pretty simple:
//! ```rust
//! "z12".parse::<i32>().throw_ctx(EarthErrorCtx{})
//! ```
//! Note that it's done with _*Ctx_ structure (EarthErrorCtx in this example)
//!  which has an implementation of _smarterr::IntoError_ trait.
//!
//! ## Function-based style
//!
//! This is a common situation when there are several functions calling from each
//!  other. Usually each function returns its own error set and some unhandled
//!  errors from the called one. Generally it is possible to use one error set
//!  (enum) for all functions but that's not quite right. The functions' contracts
//!  are inaccurate since they return subset of the common enum and some errors
//!  will never happen. If some functions are public it might be a problem to hide
//!  unused errors from the internals.
//!
//! The more precise solution is to define its own error set for each function.
//!  But besides being quite difficult, it creates another problem. Some errors
//!  may be defined several times for each error set and require mapping between
//!  them even that they are the same. _SmartErr_ solves this problem providing
//!  all necessary and optimized stuff behind the scenes.
//!
//! For this, 2 additional keywords were introduced:
//!
//! * _from_ keyword. It should be used if some errors from the called function
//!  need to be rethrown.
//! * _handle_ keyword. It is intended to mark errors from the called function
//!  which will be handled.
//!
//! Here's how it works:
//!
//! #### FBS example
//! ```rust
//! #[smarterr(
//!     AlfaError{ind: i32, ext: String} -> "Alfa error",
//!     BetaError<>{ind: i32} -> "Beta error",
//!     BetaWrappedError<ParseIntError> -> "Beta Wrapped Error",
//!     GammaError<<>>{ext: String} -> "Gamma error",
//!     GammaWrappedError<<i32>>{ext: String} -> "Gamma Wrapped error",
//! )]
//! pub fn greek_func(err_ind: usize) -> String {
//!     let ok_str = "All is ok".to_string();
//!     let err_str = "Error raised".to_string();
//!     let ext = "ext".to_string();
//!     match err_ind {
//!         0 => Ok(ok_str),
//!         1 => err_str.raise_ctx(AlfaErrorCtx { ind: -1, ext }),
//!         2 => "z12".parse::<i32>().throw_ctx(BetaErrorCtx { ind: -2 }).map(|_| ok_str),
//!         3 => "z12".parse::<i32>().throw_ctx(BetaWrappedErrorCtx {}).map(|_| ok_str),
//!         4 => err_str.raise_ctx(GammaErrorCtx { ext }),
//!         5 => 5000000.throw_ctx(GammaWrappedErrorCtx { ext }).map(|_| ok_str),
//!         _ => Ok(ok_str),
//!     }
//! }
//!
//! #[smarterr(
//!     from GreekFuncError {
//!         AlfaError, BetaError<>, BetaWrappedError<ParseIntError>, GammaError<<>>,
//!         handled GammaWrappedError
//!     },
//!     XError{ind: i32, ext: String} -> "X error",
//!     YError{ind: i32} -> "Y error",
//!     pub ZError<<String>>{ind: usize} -> "Z Error",
//! )]
//! fn latin_func(err_ind: usize) {
//!     greek_func(err_ind).handle(|h| match h {
//!         GreekFuncErrorHandled::GammaWrappedError(data) =>
//!             data.ctx.ext.throw_ctx(ZErrorCtx { ind: err_ind }),
//!     })?;
//!     Ok(())
//! }
//!
//! #[smarterr(
//!     from GreekFuncError {
//!         AlfaError -> "Imported Alfa error",
//!         BetaError<> -> "Imported Beta error",
//!         BetaWrappedError<std::num::ParseIntError> -> "Imported Beta Wrapped Error",
//!         handled GammaError,
//!         handled GammaWrappedError,
//!     },
//!     from LatinFuncError {
//!         AlfaError, BetaError<>, BetaWrappedError<ParseIntError>, ZError<<String>>,
//!         handled { GammaError, XError, YError }
//!     },
//!     FirstError{ind: i32, ext: String} -> "First error",
//!     SecondError{ind: i32} -> "Second error",
//!     ThirdError{} -> "Third Error",
//! )]
//! pub fn numeric_func(err_ind: usize) -> String {
//!     let g = greek_func(err_ind).handle(|h| match h {
//!         GreekFuncErrorHandled::GammaWrappedError(e) =>
//!             e.ctx.ext.clone().raise_ctx(FirstErrorCtx{ind: err_ind as i32, ext: e.ctx.ext}),
//!         GreekFuncErrorHandled::GammaError(e) =>
//!             e.ctx.ext.raise_ctx(SecondErrorCtx{ ind: err_ind as i32 }),
//!     })?;
//!
//!     latin_func(err_ind).handle(|h| match h {
//!         LatinFuncErrorHandled::XError(e)=>
//!             ().raise_ctx(FirstErrorCtx{ ind: err_ind as i32, ext: e.ctx.ext }),
//!         LatinFuncErrorHandled::YError(e)=>
//!             ().raise_ctx(SecondErrorCtx{ ind: e.ctx.ind }),
//!         LatinFuncErrorHandled::GammaError(_) => Ok(())
//!     })?;
//!
//!     let t = ().raise_ctx(MarsErrorCtx{ind: err_ind});
//!     t.throw_ctx(BetaErrorCtx{ ind: err_ind as i32 })?;
//!
//!     Ok(g)
//! }
//! ```
//! It is also possible to define errors for methods. The only difference is that
//! theses errors must be defined outside the implementation block. `smarterr_mod`
//! macro is intended to do this. It should be used as an attribute for the
//! implementation block. The name of the module should be passed as an argument.
//! Here's an example:
//! ```rust
//! #[smarterr_mod(test_err)]
//! impl Test {
//!     #[smarterr(InitFailed{pub a: String, pub b: String} -> "Init error")]
//!     pub fn new(a: &str, b: &str) -> Self {
//!         Ok(Self {
//!             a: a.parse()
//!                 .throw_ctx(test_err::InitFailedCtx { a: a.to_string(), b: b.to_string() })?,
//!             b: b.parse()
//!                 .throw_ctx(test_err::InitFailedCtx { a: a.to_string(), b: b.to_string() })?,
//!         })
//!     }
//! }
//! ```

use std::collections::BTreeSet;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, TokenStreamExt};
use syn::{
    parse::Parse,
    parse_macro_input,
    punctuated::Punctuated,
    token::{Brace, Comma, Gt, Lt, Pub, RArrow, Shl, Shr},
    FieldsNamed, ItemFn, ItemImpl, LitStr, ReturnType, Token, Type, Visibility,
};

mod keywords {
    syn::custom_keyword![from];
    syn::custom_keyword![handled];
}

struct ErrorNs {
    visibility: Visibility,
    name: Ident,
}

struct FledgedError {
    visibility: Visibility,
    name: Ident,
    definition: Punctuated<OwnError, Comma>,
}

struct SmartErrors {
    errors: Option<Punctuated<ErrorDef, Comma>>,
}

enum ErrorDef {
    Own(OwnError),
    Inherited(InheritedErrors),
}

struct OwnError {
    visibility: Option<Visibility>,
    name: Ident,
    definition: Option<FieldsNamed>,
    source: Option<Type>,
    is_typed_source: bool,
    is_boxed_source: bool,
    msg: Option<LitStr>,
}

struct InheritedErrors {
    source: Ident,
    errors: Option<Punctuated<InheritedErrorDef, Comma>>,
}

enum InheritedErrorDef {
    Unhandled(UnhandledError),
    Handled(HandledError),
}

type UnhandledError = OwnError;

struct HandledError {
    names: Vec<Ident>,
}

impl Parse for ErrorNs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(ErrorNs {
            visibility: input.parse::<Visibility>().unwrap_or(Visibility::Inherited {}),
            name: input.parse()?,
        })
    }
}

impl Parse for FledgedError {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(FledgedError {
            visibility: input.parse::<Visibility>().unwrap_or(Visibility::Inherited {}),
            name: input.parse()?,
            definition: {
                _ = syn::braced!(content in input);
                content.parse_terminated(OwnError::parse, Token![,])?
            },
        })
    }
}

impl Parse for SmartErrors {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let t: Option<Punctuated<ErrorDef, Comma>> = input.parse_terminated(ErrorDef::parse, Token![,]).ok();
        Ok(SmartErrors { errors: t })
    }
}

impl Parse for ErrorDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(keywords::from) {
            Ok(ErrorDef::Inherited(InheritedErrors::parse(input)?))
        } else {
            Ok(ErrorDef::Own(OwnError::parse(input)?))
        }
    }
}

impl Parse for OwnError {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(OwnError {
            visibility: input.parse::<Visibility>().ok().and_then(|v| match v {
                Visibility::Inherited => None,
                _ => Some(v),
            }),
            name: input.parse()?,
            is_typed_source: input.peek(Lt),
            is_boxed_source: input.peek(Shl),
            source: {
                if input.peek(Shl) {
                    _ = input.parse::<Shl>()?;
                    let e = input.parse::<Type>().ok();
                    _ = input.parse::<Shr>()?;
                    e
                } else if input.peek(Lt) {
                    _ = input.parse::<Lt>()?;
                    let e = input.parse::<Type>().ok();
                    _ = input.parse::<Gt>()?;
                    e
                } else {
                    None
                }
            },
            definition: input.parse().ok(),
            msg: {
                if input.parse::<RArrow>().is_ok() {
                    Some(input.parse()?)
                } else {
                    None
                }
            },
        })
    }
}

impl Parse for InheritedErrors {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<keywords::from>()?;
        Ok(InheritedErrors {
            source: input.parse()?,
            errors: {
                let content;
                let _ = syn::braced!(content in input);
                content.parse_terminated(InheritedErrorDef::parse, Token![,]).ok()
            },
        })
    }
}

impl Parse for InheritedErrorDef {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(keywords::handled) {
            Ok(InheritedErrorDef::Handled(HandledError::parse(input)?))
        } else {
            let own = OwnError::parse(input)?;
            if own.definition.is_some() {
                Err(syn::Error::new(
                    input.span(),
                    "Inherited error cannot contain its own definition",
                ))
            } else {
                Ok(InheritedErrorDef::Unhandled(own))
            }
        }
    }
}

impl Parse for HandledError {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<keywords::handled>()?;

        let mut names = vec![];
        if input.peek(Brace) {
            let content;
            let _ = syn::braced!(content in input);
            for ident in content.parse_terminated(Ident::parse, Token![,])? {
                names.push(ident);
            }
        } else {
            names.push(input.parse()?);
        }
        Ok(HandledError { names })
    }
}

impl OwnError {
    fn to_enum_error_item(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        tokens.append_all(quote!(#name (#name),));
    }

    fn to_ctx(&self, visibility: &Visibility, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let visibility = self.visibility.as_ref().unwrap_or(visibility);
        let msg = self.msg.as_ref().map(|l| l.value()).unwrap_or("".to_string());
        let ctx_name_str = format!("{}Ctx", self.name);
        let ctx_name: Ident = Ident::new(&ctx_name_str, self.name.span());
        let definition = self.definition.clone().unwrap_or(FieldsNamed {
            brace_token: Brace::default().into(),
            named: Punctuated::new(),
        });

        let mut is_default = false;
        let ts = if let Some(st) = &self.source {
            let (struct_st, new_src) = if !self.is_boxed_source {
                (quote! { #st }, quote! { src })
            } else {
                (
                    quote! { smarterr::RawError<#st> },
                    quote! { smarterr::RawError::new(src) },
                )
            };
            quote! {
                #[derive(std::fmt::Debug)]
                #visibility struct #name {
                    src: #struct_st,
                    ctx: #ctx_name,
                }
                impl #name {
                    pub fn new(src: #st, ctx: #ctx_name) -> Self {
                        #name { src: #new_src, ctx }
                    }
                    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                        Some(&self.src as _)
                    }
                    pub fn default_message(&self) -> &'static str {
                        #msg
                    }
                }
            }
        } else if self.is_typed_source {
            let (st, struct_st, new_src, src) = if !self.is_boxed_source {
                (
                    quote! { std::error::Error + 'static },
                    quote! { Box<dyn std::error::Error + 'static> },
                    quote! { Box::new(src) },
                    quote! { Some(&*self.src) },
                )
            } else {
                (
                    quote! { std::fmt::Debug + 'static },
                    quote! { smarterr::RawError<Box<dyn std::fmt::Debug + 'static>> },
                    quote! { smarterr::RawError::new (Box::new(src)) },
                    quote! { Some(&self.src as _) },
                )
            };
            quote! {
                #[derive(std::fmt::Debug)]
                #visibility struct #name {
                    src: #struct_st,
                    ctx: #ctx_name,
                }

                impl #name {
                    pub fn new<ES: #st>(src: ES, ctx: #ctx_name) -> Self {
                        #name { src: #new_src, ctx }
                    }

                    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                        #src
                    }

                    pub fn default_message(&self) -> &'static str {
                        #msg
                    }
                }
            }
        } else {
            is_default = true;
            quote! {
                #[derive(std::fmt::Debug)]
                #visibility struct #name {
                    ctx: #ctx_name,
                }

                impl #name {
                    pub fn new<ES>(_src: ES, ctx: #ctx_name) -> Self {
                        #name { ctx }
                    }

                    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                        None
                    }

                    pub fn default_message(&self) -> &'static str {
                        #msg
                    }
                }
            }
        };
        tokens.append_all(ts);

        let write = if is_default {
            quote!(write!(f, "{}", x)?;)
        } else {
            quote!(write!(f, "{}, caused by: {}", x, self.src)?;)
        };
        tokens.append_all(quote! {
            impl std::fmt::Display for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let x = format!("{:?}", self.ctx).replace("\"", "\'");
                    let x = x.strip_prefix(#ctx_name_str).unwrap_or("");
                    #write
                    Ok(())
                }
            }

            #[allow(dead_code)]
            #[derive(std::fmt::Debug)]
            #visibility struct #ctx_name #definition
        });
    }

    fn to_into_error_impl(&self, others: &BTreeSet<String>, err_enum: &Ident, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        if others.contains(&name.to_string()) {
            return;
        }
        let ctx_name: Ident = Ident::new(&format!("{}Ctx", self.name), self.name.span());

        let ts = if let Some(st) = &self.source {
            quote! {
                impl smarterr::IntoError<#err_enum, #st> for #ctx_name {
                    fn into_error(self, source: #st) -> #err_enum {
                        #err_enum::#name(#name::new(source, self))
                    }
                }
            }
        } else if self.is_boxed_source {
            quote! {
                impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<#err_enum, ES> for #ctx_name {
                    fn into_error(self, source: ES) -> #err_enum {
                        #err_enum::#name(#name::new(source, self))
                    }
                }
            }
        } else if self.is_typed_source {
            quote! {
                impl<ES: std::error::Error + 'static> smarterr::IntoError<#err_enum, ES> for #ctx_name {
                    fn into_error(self, source: ES) -> #err_enum {
                        #err_enum::#name(#name::new(source, self))
                    }
                }
            }
        } else {
            quote! {
                impl<ES: std::fmt::Debug + 'static> smarterr::IntoError<#err_enum, ES> for #ctx_name {
                    fn into_error(self, source: ES) -> #err_enum {
                        #err_enum::#name(#name::new(source, self))
                    }
                }
            }
        };
        tokens.append_all(ts);
    }

    fn to_errors_sources(&self, others: &BTreeSet<String>, err_enum: &Ident, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        if others.contains(&name.to_string()) || self.source.is_none() && !self.is_typed_source {
            return;
        }
        tokens.append_all(quote! {
            #err_enum::#name(err) => err.source(),
        });
    }

    fn to_display(&self, others: &BTreeSet<String>, err_enum: &Ident, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        if others.contains(&name.to_string()) {
            return;
        }

        tokens.append_all(quote! {
            #err_enum::#name(err) => {
                write!(f, "{}{}", err.default_message(), err)?;
            }
        });
    }
}

impl UnhandledError {
    fn to_unhandled_enum_error_item(&self, others: &BTreeSet<String>, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        if !others.contains(&name.to_string()) {
            tokens.append_all(quote!(#name (#name),));
        }
    }

    fn to_handler_action(
        &self,
        err_enum: &Ident,
        from_err_enum: &Ident,
        module: &Option<Ident>,
        from: &mut proc_macro2::TokenStream,
        handles: &mut proc_macro2::TokenStream,
    ) {
        let name = &self.name;
        if let Some(mod_name) = module {
            handles.append_all(quote!(#mod_name::#from_err_enum::#name(e) => Err(#mod_name::#err_enum::#name(e)),));
            from.append_all(quote!(#mod_name::#from_err_enum::#name(ctx) => #mod_name::#err_enum::#name(ctx),));
        } else {
            handles.append_all(quote!(#from_err_enum::#name(e) => Err(#err_enum::#name(e)),));
            from.append_all(quote!(#from_err_enum::#name(ctx) => #err_enum::#name(ctx),));
        }
    }
}

impl HandledError {
    fn to_handled_enum_error_item(&self, handled_enum_errors: &mut proc_macro2::TokenStream) {
        for name in &self.names {
            handled_enum_errors.append_all(quote!(#name(#name),));
        }
    }

    fn to_handler_action(
        &self,
        from_err_enum: &Ident,
        handled_err_enum: &Ident,
        handles: &mut proc_macro2::TokenStream,
    ) {
        for name in &self.names {
            handles.append_all(quote!(
                #from_err_enum::#name(e) => handler(#handled_err_enum::#name(e)),
            ));
        }
    }
}

#[proc_macro_attribute]
pub fn smarterr_mod(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemImpl);
    let meta = parse_macro_input!(metadata as ErrorNs);

    let mod_name: Ident = meta.name.clone();
    let mod_visibility = meta.visibility.clone();

    let mut mod_content = proc_macro2::TokenStream::new();
    for item in &mut input.items {
        if let syn::ImplItem::Fn(method) = item {
            let func: ItemFn = syn::parse2(quote! {
                #method
            })
            .unwrap();
            for attr in &method.attrs {
                if attr.path().segments.len() == 1 && attr.path().segments[0].ident == "smarterr" {
                    if let Ok(smart_errors) = attr.parse_args::<SmartErrors>() {
                        let r = _smarterr(func, smart_errors, Some(mod_name.clone()));
                        let r0 = r.0;
                        let func_out: ItemFn = syn::parse2(quote! {
                            #r0
                        })
                        .unwrap();
                        method.sig.output = func_out.sig.output;
                        method.block = *func_out.block;
                        mod_content.extend(r.1.into_iter());
                        break;
                    }
                }
            }
            // remove only smarterr attributes
            method
                .attrs
                .retain(|attr| !(attr.path().segments.len() == 1 && attr.path().segments[0].ident == "smarterr"));
        } else if let syn::ImplItem::Macro(macros) = item {
            // check it is `smarterr_fledged` macro
            if macros.mac.path.segments.len() == 1 && macros.mac.path.segments[0].ident == "smarterr_fledged" {
                // copy it to the mod
                mod_content.extend(quote! {
                    #macros
                });
            }
        }
    }

    // remove `smarterr_fledged` macroses
    input.items.retain(|item| {
        if let syn::ImplItem::Macro(macros) = item {
            // check it is `smarterr_fledged` macro
            if macros.mac.path.segments.len() == 1 && macros.mac.path.segments[0].ident == "smarterr_fledged" {
                return false;
            }
        }
        true
    });

    let output: TokenStream = quote! {
        #input
        #mod_visibility mod #mod_name {
            use smarterr_macro::smarterr_fledged;
            #mod_content
        }
    }
    .into();
    output
}

#[proc_macro_attribute]
pub fn smarterr(metadata: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let meta = parse_macro_input!(metadata as SmartErrors);

    let (mut output, ts) = _smarterr(input, meta, None);

    output.extend(ts.into_iter());
    output.into()
}

fn _smarterr(
    mut input: ItemFn,
    meta: SmartErrors,
    module: Option<Ident>,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    // if module is defined, visibility should be public
    let visibility = if module.is_some() {
        Visibility::Public(Pub::default())
    } else {
        input.vis.clone()
    };
    let err_enum: Ident = Ident::new(
        &format!("{}Error", input.sig.ident).to_case(Case::Pascal),
        input.sig.ident.span(),
    );

    input.sig.output = match input.sig.output {
        ReturnType::Default => {
            let spans = [input.sig.ident.span().clone(), input.sig.ident.span().clone()];
            let rt = if let Some(mod_name) = &module {
                syn::Type::Verbatim(quote! { std::result::Result<(), #mod_name::#err_enum> })
            } else {
                syn::Type::Verbatim(quote! { std::result::Result<(), #err_enum> })
            };
            ReturnType::Type(RArrow { spans }, Box::<Type>::new(rt))
        }
        ReturnType::Type(arrow, tt) => {
            let spans = arrow.spans.clone();
            let rt = if let Some(mod_name) = &module {
                syn::Type::Verbatim(quote! { std::result::Result<#tt, #mod_name::#err_enum> })
            } else {
                syn::Type::Verbatim(quote! { std::result::Result<#tt, #err_enum> })
            };
            ReturnType::Type(RArrow { spans }, Box::<Type>::new(rt))
        }
    };

    let mut dedup = BTreeSet::<String>::new();
    let mut enum_errors = proc_macro2::TokenStream::new();
    let mut errors_ctx = proc_macro2::TokenStream::new();
    let mut errors_ctx_into_error_impl = proc_macro2::TokenStream::new();
    let mut errors_sources = proc_macro2::TokenStream::new();
    let mut errors_display = proc_macro2::TokenStream::new();

    let mut handlers: Vec<syn::Stmt> = vec![syn::parse2(quote! {
        trait ErrorHandler<T, EH, ER> {
            fn handle<F: FnOnce(EH) -> Result<T, ER>>(self, handler: F) -> Result<T, ER>;
        }
    })
    .unwrap()];

    meta.errors.iter().flat_map(|p| p.iter()).for_each(|ed| match ed {
        ErrorDef::Own(oe) => {
            oe.to_enum_error_item(&mut enum_errors);
            oe.to_ctx(&visibility, &mut errors_ctx);
            oe.to_into_error_impl(&dedup, &err_enum, &mut errors_ctx_into_error_impl);
            oe.to_errors_sources(&dedup, &err_enum, &mut errors_sources);
            oe.to_display(&dedup, &err_enum, &mut errors_display);
        }
        ErrorDef::Inherited(ie) => {
            let mut has_handled = false;
            let mut handles = proc_macro2::TokenStream::new();
            let mut unhandled_from = proc_macro2::TokenStream::new();
            let mut handled_enum_errors = proc_macro2::TokenStream::new();

            let source_err_enum = ie.source.clone();
            let handled_err_enum: Ident = Ident::new(&format!("{}Handled", source_err_enum), ie.source.span());
            ie.errors.iter().flat_map(|p| p.iter()).for_each(|ed| match ed {
                InheritedErrorDef::Unhandled(ue) => {
                    ue.to_unhandled_enum_error_item(&dedup, &mut enum_errors);
                    ue.to_handler_action(&err_enum, &source_err_enum, &module, &mut unhandled_from, &mut handles);
                    ue.to_into_error_impl(&dedup, &err_enum, &mut errors_ctx_into_error_impl);
                    ue.to_errors_sources(&dedup, &err_enum, &mut errors_sources);
                    ue.to_display(&dedup, &err_enum, &mut errors_display);
                    dedup.insert(ue.name.to_string());
                }
                InheritedErrorDef::Handled(he) => {
                    has_handled = true;
                    he.to_handled_enum_error_item(&mut handled_enum_errors);
                    he.to_handler_action(&source_err_enum, &handled_err_enum, &mut handles);
                    for name in &he.names {
                        dedup.insert(name.to_string());
                    }
                }
            });

            if has_handled {
                let stmt = syn::parse2::<syn::Stmt>(quote! {
                    enum #handled_err_enum {
                        #handled_enum_errors
                    }
                });
                handlers.push(stmt.unwrap());
                let stmt = syn::parse2::<syn::Stmt>(quote! {
                    impl<T> ErrorHandler<T, #handled_err_enum, #err_enum> for Result<T, #source_err_enum> {
                        fn handle<F: FnOnce(#handled_err_enum) -> Result<T, #err_enum>>(
                            self,
                            handler: F,
                        ) -> Result<T, #err_enum> {
                            match self {
                                Ok(v) => Ok(v),
                                Err(e) => match e {
                                    #handles
                                },
                            }
                        }
                    }
                });
                handlers.push(stmt.unwrap());
            } else {
                let stmt = syn::parse2::<syn::Stmt>(if let Some(mod_name) = &module {
                    quote! {
                        impl From<#mod_name::#source_err_enum> for #mod_name::#err_enum {
                            fn from(source: #mod_name::#source_err_enum) -> Self {
                                match source {
                                    #unhandled_from
                                }
                            }
                        }
                    }
                } else {
                    quote! {
                        impl From<#source_err_enum> for #err_enum {
                            fn from(source: #source_err_enum) -> Self {
                                match source {
                                    #unhandled_from
                                }
                            }
                        }
                    }
                });
                handlers.push(stmt.unwrap());
            }
        }
    });

    if handlers.len() > 1 {
        handlers.extend_from_slice(&input.block.stmts);
        input.block.stmts = handlers;
    }

    let output = quote! { #input };

    let ts = quote! {
        #[derive(std::fmt::Debug)]
        #visibility enum #err_enum {
            #enum_errors
        }
        impl std::error::Error for #err_enum {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    #errors_sources
                    _ => None,
                }
            }
        }
        impl std::fmt::Display for #err_enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #errors_display
                }
                Ok(())
            }
        }
        #errors_ctx
        #errors_ctx_into_error_impl
    };

    (output, ts)
}

#[proc_macro]
pub fn smarterr_fledged(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as FledgedError);

    let visibility = input.visibility.clone();
    let err_enum: Ident = input.name.clone();

    let dedup = BTreeSet::<String>::new();
    let mut enum_errors = proc_macro2::TokenStream::new();
    let mut errors_ctx = proc_macro2::TokenStream::new();
    let mut errors_ctx_into_error_impl = proc_macro2::TokenStream::new();
    let mut errors_sources = proc_macro2::TokenStream::new();
    let mut errors_display = proc_macro2::TokenStream::new();

    input.definition.iter().for_each(|oe| {
        oe.to_enum_error_item(&mut enum_errors);
        oe.to_ctx(&visibility, &mut errors_ctx);
        oe.to_into_error_impl(&dedup, &err_enum, &mut errors_ctx_into_error_impl);
        oe.to_errors_sources(&dedup, &err_enum, &mut errors_sources);
        oe.to_display(&dedup, &err_enum, &mut errors_display);
    });

    quote! {
        #[derive(std::fmt::Debug)]
        #visibility enum #err_enum {
            #enum_errors
        }
        impl std::error::Error for #err_enum {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    #errors_sources
                    _ => None,
                }
            }
        }
        impl std::fmt::Display for #err_enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #errors_display
                }
                Ok(())
            }
        }
        #errors_ctx
        #errors_ctx_into_error_impl
    }
    .into()
}
