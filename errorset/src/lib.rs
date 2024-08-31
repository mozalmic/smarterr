#![recursion_limit = "512"]
#![doc = include_str!("../README.md")]

extern crate proc_macro;

use core::panic;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    token::PathSep,
    Ident, ImplItem, ImplItemFn, ItemFn, ItemImpl, PathArguments, PathSegment, ReturnType, Token, Type, TypePath,
    Visibility,
};

struct ErrorsetArgs {
    visibility: Visibility,
    module: Option<Ident>,
}

impl Parse for ErrorsetArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut module = None;

        // try parse Visibility of module
        let visibility: Visibility = input.parse()?;
        // try parse module definition like `mod "module_name"`
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![mod]) {
            input.parse::<Token![mod]>()?;
            let mod_name: Ident = input.parse()?;
            module = Some(mod_name);
        }

        Ok(ErrorsetArgs { visibility, module })
    }
}

#[proc_macro_attribute]
pub fn errorset(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ErrorsetArgs);
    let input = parse_macro_input!(item as syn::Item);

    match input {
        syn::Item::Fn(item_fn) => handle_function(&args, item_fn),
        syn::Item::Impl(item_impl) => handle_impl_block(&args, item_impl),
        _ => panic!("errorset can only be applied to functions or impl blocks"),
    }
}

struct Output {
    enum_def: proc_macro2::TokenStream,
    fn_def: proc_macro2::TokenStream,
}

fn process_fn(args: &ErrorsetArgs, item_fn: ItemFn) -> Output {
    // Extract the function name and convert it to camel-case for the enum name
    let fn_name = &item_fn.sig.ident;
    let enum_name = format_ident!("{}Errors", fn_name.to_string().to_case(Case::Pascal));

    // Extract the return type from the function signature
    let output_type = match &item_fn.sig.output {
        ReturnType::Type(_, ty) => ty,
        _ => panic!("Function must have a valid return type"),
    };
    // Extract the function attributes, visibility, generics, inputs, and body
    let fn_body = &item_fn.block;
    let fn_inputs = &item_fn.sig.inputs;
    let fn_vis = &item_fn.vis;
    let fn_attrs = &item_fn.attrs;
    let fn_generics = &item_fn.sig.generics;

    // Ensure the return type is a generic with 2 parameters and the last one is tuple,
    // like SomeObject<_, (Error1, Error2, ...)>
    let (new_return_type, err_types) = if let Type::Path(TypePath { path, .. }) = &**output_type {
        if let Some(last_segment) = path.segments.last() {
            if let PathArguments::AngleBracketed(ref params) = last_segment.arguments {
                if params.args.len() == 2 {
                    if let Some(syn::GenericArgument::Type(Type::Tuple(tuple))) = params.args.iter().nth(1) {
                        // Reconstruct path w/o generic arguments
                        let mut punctuated = Punctuated::<PathSegment, PathSep>::new();
                        for seg in path.segments.iter() {
                            punctuated.push_value(seg.ident.clone().into());
                            // Add separator if there are more segments
                            if punctuated.len() < path.segments.len() {
                                punctuated.push_punct(PathSep::default());
                            }
                        }
                        let new_path = syn::Path {
                            leading_colon: path.leading_colon.clone(),
                            segments: punctuated,
                        };

                        // Create new return type with the same name and the first generic parameter
                        // The second parameter is the enum with error types
                        let first_generic_arg = params.args.iter().next().unwrap();
                        let new_return_type = if let Some(module) = &args.module {
                            quote! {
                                #new_path<#first_generic_arg, #module::#enum_name>
                            }
                        } else {
                            quote! {
                                #new_path<#first_generic_arg, #enum_name>
                            }
                        };
                        let err_types = tuple.elems.clone();
                        (new_return_type, err_types)
                    } else {
                        panic!("Expected the second generic argument to be a tuple");
                    }
                } else {
                    panic!("Expected exactly 2 generic arguments");
                }
            } else {
                panic!("Expected angle-bracketed arguments in the generic type");
            }
        } else {
            panic!("Expected a valid type path for the generic type");
        }
    } else {
        panic!("Function must return a generic type with 2 parameters");
    };

    // Generate enum variants for each error type
    let enum_variants = err_types.iter().map(|ty| {
        // get the last part in `ty` path
        let ty_name = match ty {
            Type::Path(TypePath { path, .. }) => path.segments.last().unwrap().ident.clone(),
            _ => panic!("Expected a type path"),
        };
        quote! {
            #[error(transparent)]
            #ty_name(#[from] #ty),
        }
    });

    // Generate the enum definition
    let enum_vis = if args.module.is_some() {
        // use pub visibility for the enum if it's inside a module
        syn::Visibility::Public(Default::default())
    } else {
        fn_vis.clone()
    };
    let enum_def = quote! {
        #[derive(::thiserror::Error, Debug)]
        #enum_vis enum #enum_name {
            #(#enum_variants)*
        }
    };

    // Generate the modified function with the new return type
    let new_fn = quote! {
        #(#fn_attrs)*
        #fn_vis fn #fn_name #fn_generics(#fn_inputs) -> #new_return_type
        #fn_body
    };

    Output { enum_def: enum_def, fn_def: new_fn }
}

fn handle_function(args: &ErrorsetArgs, item_fn: ItemFn) -> TokenStream {
    let Output { enum_def, fn_def } = process_fn(args, item_fn);

    // Combine the enum definition and the modified function
    let output = if let Some(module) = &args.module {
        let vis = &args.visibility;
        quote! {
            #vis mod #module {
                use super::*;
                #enum_def
            }
            #fn_def
        }
    } else {
        quote! {
            #enum_def
            #fn_def
        }
    };

    output.into()
}

fn handle_impl_block(args: &ErrorsetArgs, item_impl: ItemImpl) -> TokenStream {
    let mut new_items: Vec<ImplItem> = Vec::new();
    let mut new_enums: Vec<proc_macro2::TokenStream> = Vec::new();

    for item in item_impl.items {
        if let syn::ImplItem::Fn(method) = &item {
            let mut new_attrs = vec![];
            let mut marked = false;
            // check if `#[errorset]` is applied to the function
            // collecting other to the new_attrs (so `errorset` will be removed)
            // if `#[errorset]` is not applied, just add the function to the new_items
            // otherwice, check if errorset does not contain module definition
            for attr in &method.attrs {
                if attr.path().is_ident("errorset") {
                    if attr.meta.require_path_only().is_err() {
                        panic!(
                            "errorset attribute must have no arguments when applied to a function inside an impl block"
                        );
                    }
                    marked = true;
                } else {
                    new_attrs.push(attr.clone());
                }
            }
            if !marked {
                new_items.push(item);
                continue;
            }

            let item_fn = ItemFn {
                attrs: new_attrs,
                vis: syn::Visibility::Inherited,
                sig: method.sig.clone(),
                block: Box::new(method.block.clone()),
            };
            let Output { enum_def, fn_def } = process_fn(args, item_fn);

            // Convert fn_def to ImplItem::Fn
            let impl_item = syn::parse2::<ImplItemFn>(fn_def.into()).unwrap();
            new_items.push(impl_item.into());
            new_enums.push(enum_def);
        } else {
            new_items.push(item);
        }
    }

    let new_impl_block = ItemImpl { items: new_items, ..item_impl };

    let output = if let Some(module) = &args.module {
        // create module if new_enums is not empty
        // otherwise, just add new_impl_block
        if new_enums.is_empty() {
            quote! {
                #new_impl_block
            }
        } else {
            let vis = &args.visibility;
            quote! {
                #vis mod #module {
                    use super::*;
                    #(#new_enums)*
                }
                #new_impl_block
            }
        }
    } else {
        quote! {
            #(#new_enums)*
            #new_impl_block
        }
    };

    output.into()
}
