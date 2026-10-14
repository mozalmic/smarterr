#![recursion_limit = "512"]
#![doc = include_str!("../README.md")]

extern crate proc_macro;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use std::{
    collections::{HashMap, HashSet},
    sync::atomic::{AtomicUsize, Ordering},
};
use syn::{
    braced, bracketed, parenthesized,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
    token::PathSep,
    Ident, ImplItemFn, ItemFn, ItemImpl, ItemType, LitInt, Path, PathArguments, PathSegment, ReturnType, Token, Type, TypePath,
    Visibility,
};

#[derive(Clone)]
struct Source {
    ty: TypePath,
    descriptor: Path,
    mappings: Vec<(Ident, Ident)>,
}

enum DescriptorExport {
    Default,
    Named(Ident),
}

struct ErrorsetArgs {
    visibility: Visibility,
    module: Option<Ident>,
    export: Option<DescriptorExport>,
    sources: Vec<Source>,
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

        let mut export = None;
        let mut sources = Vec::new();
        while !input.is_empty() {
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
            let option: Ident = input.parse()?;
            match option.to_string().as_str() {
                "export" => {
                    if export.is_some() {
                        return Err(syn::Error::new_spanned(option, "duplicate export option"));
                    }
                    export = Some(if input.peek(syn::token::Paren) {
                        let content;
                        parenthesized!(content in input);
                        let name = content.parse()?;
                        if !content.is_empty() {
                            return Err(content.error("expected one descriptor name"));
                        }
                        DescriptorExport::Named(name)
                    } else {
                        DescriptorExport::Default
                    });
                }
                "from" => {
                    let content;
                    parenthesized!(content in input);
                    if content.is_empty() {
                        return Err(content.error("from requires at least one source set"));
                    }
                    while !content.is_empty() {
                        let ty: TypePath = content.parse()?;
                        let descriptor = if content.peek(Token![=>]) {
                            content.parse::<Token![=>]>()?;
                            content.parse()?
                        } else {
                            let mut path = ty.path.clone();
                            let last = path.segments.last_mut().unwrap();
                            last.arguments = PathArguments::None;
                            path
                        };
                        let mut mappings = Vec::new();
                        if content.peek(syn::token::Brace) {
                            let map;
                            braced!(map in content);
                            while !map.is_empty() {
                                let source = map.parse()?;
                                map.parse::<Token![=>]>()?;
                                mappings.push((source, map.parse()?));
                                if !map.is_empty() {
                                    map.parse::<Token![,]>()?;
                                }
                            }
                        }
                        sources.push(Source { ty, descriptor, mappings });
                        if !content.is_empty() {
                            content.parse::<Token![,]>()?;
                        }
                    }
                }
                _ => return Err(syn::Error::new_spanned(option, "expected export or from(...)")),
            }
        }

        Ok(ErrorsetArgs { visibility, module, export, sources })
    }
}

#[proc_macro_attribute]
pub fn errorset(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ErrorsetArgs);
    let input = parse_macro_input!(item as syn::Item);

    match input {
        syn::Item::Fn(item_fn) => handle_function(&args, item_fn),
        syn::Item::Impl(item_impl) => handle_impl_block(&args, item_impl),
        syn::Item::Type(item_type) => match process_type(&args, item_type) {
            Ok(output) => output.into(),
            Err(error) => error.to_compile_error().into(),
        },
        _ => syn::Error::new_spanned(input, "errorset can only be applied to functions, impl blocks, or type declarations")
            .to_compile_error()
            .into(),
    }
}

struct Output {
    enum_def: proc_macro2::TokenStream,
    fn_def: proc_macro2::TokenStream,
}

fn process_fn(args: &ErrorsetArgs, item_fn: &ItemFn, impl_generics: Option<&syn::Generics>) -> Result<Option<Output>> {
    // Extract the function name and convert it to camel-case for the enum name
    let fn_name = &item_fn.sig.ident;
    let enum_name = Ident::new(
        &format!("{}Errors", fn_name.to_string().to_case(Case::Pascal)),
        Span::call_site(),
    );

    // Extract the return type from the function signature
    let output_type = match &item_fn.sig.output {
        ReturnType::Type(_, ty) => ty,
        _ => {
            return Err(syn::Error::new_spanned(
                &item_fn.sig.output,
                "Function must have a valid return type",
            ))
        }
    };

    let (new_return_type, err_types) = if let Type::Path(TypePath { path, .. }) = &**output_type {
        if let Some(last_segment) = path.segments.last() {
            if let PathArguments::AngleBracketed(ref params) = last_segment.arguments {
                if params.args.len() != 2 {
                    return Err(syn::Error::new_spanned(
                        &params.args,
                        "Expected exactly 2 generic arguments",
                    ));
                }

                match params.args.iter().nth(1).unwrap() {
                    syn::GenericArgument::Type(Type::Tuple(tuple)) => {
                        let mut punctuated = Punctuated::<PathSegment, PathSep>::new();
                        for seg in path.segments.iter() {
                            punctuated.push_value(seg.ident.clone().into());
                            // Add separator if there are more segments
                            if punctuated.len() < path.segments.len() {
                                punctuated.push_punct(PathSep::default());
                            }
                        }
                        let new_path = syn::Path {
                            leading_colon: path.leading_colon,
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
                    }
                    argument @ (syn::GenericArgument::Type(Type::Paren(_)) | syn::GenericArgument::Type(Type::Path(_))) => {
                        if cfg!(feature = "runtime") || args.export.is_some() || !args.sources.is_empty() {
                            return Err(syn::Error::new_spanned(
                                argument,
                                "errorset requires an explicit tuple of error types; remove #[errorset] to return the atom directly, or use (Atom,) for a single-error set",
                            ));
                        }
                        // If the second argument is defined as `(Error1)`, it does not determined as a tuple, just leave it as is
                        // The same if the second argument is a regular type
                        return Ok(None);
                    }
                    other => {
                        return Err(syn::Error::new_spanned(
                            other,
                            "Expected the second generic argument to be a tuple",
                        ));
                    }
                }
            } else {
                return Err(syn::Error::new_spanned(
                    last_segment,
                    "Expected angle-bracketed generic arguments",
                ));
            }
        } else {
            return Err(syn::Error::new_spanned(
                path,
                "Expected a valid type path for the generic type",
            ));
        }
    } else {
        return Err(syn::Error::new_spanned(
            output_type,
            "Function must return a generic type with 2 parameters",
        ));
    };

    let members = members(&err_types)?;
    if args.export.is_some() || !args.sources.is_empty() {
        reject_generic_errors(&item_fn.sig.generics, &members)?;
        if let Some(generics) = impl_generics {
            reject_generic_errors(generics, &members)?;
        }
    }

    let enum_vis = if args.module.is_some() {
        syn::Visibility::Public(Default::default())
    } else {
        item_fn.vis.clone()
    };
    let attrs: Vec<_> = item_fn.attrs.iter().filter(|a| a.path().is_ident("cfg")).cloned().collect();
    let enum_def = generate_set(args, &enum_name, &enum_vis, &members, &attrs)?;

    let fn_sig = &item_fn.sig;
    let fn_attrs = &item_fn.attrs;
    let fn_vis = &item_fn.vis;
    let fn_body = &item_fn.block;

    let mut new_sig = fn_sig.clone();
    new_sig.output = syn::parse2(quote! { -> #new_return_type }).unwrap();

    // Generate the modified function with the new return type
    let new_fn = quote! {
        #(#fn_attrs)*
        #fn_vis #new_sig
        #fn_body
    };

    Ok(Some(Output { enum_def, fn_def: new_fn }))
}

fn process_type(args: &ErrorsetArgs, item: ItemType) -> Result<proc_macro2::TokenStream> {
    if args.module.is_some() || !matches!(args.visibility, Visibility::Inherited) {
        return Err(syn::Error::new_spanned(
            &item,
            "on type declarations, use the declaration's visibility and place it in the desired module; module/visibility options are not supported",
        ));
    }
    if !item.generics.params.is_empty() || item.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            &item.generics,
            "errorset type declarations do not support generics or where clauses; use concrete error types",
        ));
    }
    let Type::Tuple(tuple) = &*item.ty else {
        return Err(syn::Error::new_spanned(
            &item.ty,
            "errorset type declarations require an explicit tuple of error types; use (Atom,) for a single-error set",
        ));
    };
    let mut seen = HashMap::new();
    for (index, ty) in tuple.elems.iter().enumerate() {
        if let Some((position, first)) = seen.insert(ty.to_token_stream().to_string(), (index + 1, ty)) {
            let mut error = syn::Error::new_spanned(
                ty,
                format!("duplicate error atom at position {}; first declared at position {position}", index + 1),
            );
            error.combine(syn::Error::new_spanned(first, "first declaration of this error atom"));
            return Err(error);
        }
    }
    let members = members(&tuple.elems)?;
    reject_generic_errors(&item.generics, &members)?;
    generate_set(args, &item.ident, &item.vis, &members, &item.attrs)
}

fn generate_set(
    args: &ErrorsetArgs,
    enum_name: &Ident,
    enum_vis: &Visibility,
    members: &[(Ident, Type)],
    attrs: &[syn::Attribute],
) -> Result<proc_macro2::TokenStream> {
    let mut sources = HashSet::new();
    for source in &args.sources {
        if source.ty.qself.is_some()
            || source.ty.path.segments.iter().any(|segment| !matches!(segment.arguments, PathArguments::None))
        {
            return Err(syn::Error::new_spanned(&source.ty, "generic and associated source sets are not supported"));
        }
        if !sources.insert(source.ty.to_token_stream().to_string()) {
            return Err(syn::Error::new_spanned(&source.ty, "duplicate source set"));
        }
    }
    let cfgs: Vec<_> = attrs.iter().filter(|a| a.path().is_ident("cfg")).collect();
    let enum_variants = members.iter().map(|(name, ty)| {
        quote! {
            #[error(transparent)]
            #name(#[from] #ty),
        }
    });
    let metadata = metadata(args, enum_name, enum_vis, members, &cfgs);
    let remainders = remainder_names(&args.sources, enum_name)?;
    let conversions = args.sources.iter().zip(&remainders).map(|(source, remainder)| {
        let descriptor = &source.descriptor;
        let ty = &source.ty;
        let mappings = source.mappings.iter().map(|(s, t)| quote! { #s => #t, });
        let targets = members.iter().map(|(name, ty)| quote! { #name : #ty, });
        quote! {
            #(#cfgs)*
            #descriptor!(::smarterr::errorset::__compose, [
                #ty; #enum_name; #enum_vis; #remainder;
                [#(#targets)*] [#(#mappings)*]
            ]);
        }
    });
    Ok(quote! {
        #(#attrs)*
        #[derive(::thiserror::Error, Debug)]
        #enum_vis enum #enum_name {
            #(#enum_variants)*
        }
        #metadata
        #(#conversions)*
    })
}

fn remainder_names(sources: &[Source], target: &Ident) -> Result<Vec<Ident>> {
    let mut counts = HashMap::new();
    for source in sources {
        let name = source.ty.path.segments.last().unwrap().ident.to_string();
        *counts.entry(name).or_insert(0usize) += 1;
    }
    let mut seen = HashSet::new();
    sources.iter().map(|source| {
        let name = source.ty.path.segments.last().unwrap().ident.to_string();
        let prefix = if counts[&name] > 1 {
            source.ty.path.segments.iter()
                .map(|segment| segment.ident.to_string().to_case(Case::Pascal))
                .collect::<String>()
        } else {
            name
        };
        let remainder = format_ident!("{}For{}", prefix, target);
        if !seen.insert(remainder.to_string()) {
            return Err(syn::Error::new_spanned(
                &source.ty,
                "ambiguous remainder name; give these source sets distinct named aliases",
            ));
        }
        Ok(remainder)
    }).collect()
}

fn members(types: &Punctuated<Type, Token![,]>) -> Result<Vec<(Ident, Type)>> {
    let mut seen = HashSet::new();
    let mut entries = Vec::new();
    let mut counts = HashMap::new();
    for ty in types {
        let Type::Path(TypePath { qself: None, path }) = ty else {
            return Err(syn::Error::new_spanned(ty, "error members must be named type paths"));
        };
        if seen.insert(ty.to_token_stream().to_string()) {
            let name = path.segments.last().unwrap().ident.clone();
            *counts.entry(name.to_string()).or_insert(0usize) += 1;
            entries.push((name, ty.clone()));
        }
    }
    let mut names = HashSet::new();
    for (name, ty) in &mut entries {
        if counts[&name.to_string()] > 1 {
            let Type::Path(TypePath { path, .. }) = ty else {
                unreachable!()
            };
            let qualified = path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string().to_case(Case::Pascal))
                .collect::<String>();
            *name = Ident::new(&qualified, name.span());
        }
        if !names.insert(name.to_string()) {
            return Err(syn::Error::new_spanned(
                ty,
                "ambiguous variant name; give these types distinct named aliases",
            ));
        }
    }
    Ok(entries)
}

fn reject_generic_errors(generics: &syn::Generics, members: &[(Ident, Type)]) -> Result<()> {
    let mut parameters: HashSet<String> = generics
        .params
        .iter()
        .map(|p| match p {
            syn::GenericParam::Type(p) => p.ident.to_string(),
            syn::GenericParam::Lifetime(p) => p.lifetime.ident.to_string(),
            syn::GenericParam::Const(p) => p.ident.to_string(),
        })
        .collect();
    parameters.insert("Self".into());
    struct GenericUse<'a> {
        parameters: &'a HashSet<String>,
        found: bool,
    }
    impl<'ast> syn::visit::Visit<'ast> for GenericUse<'_> {
        fn visit_path(&mut self, path: &'ast Path) {
            if path.leading_colon.is_none()
                && path
                    .segments
                    .first()
                    .is_some_and(|s| self.parameters.contains(&s.ident.to_string()))
            {
                self.found = true;
            }
            syn::visit::visit_path(self, path);
        }

        fn visit_lifetime(&mut self, lifetime: &'ast syn::Lifetime) {
            if self.parameters.contains(&lifetime.ident.to_string()) {
                self.found = true;
            }
        }
    }
    for (_, ty) in members {
        let mut usage = GenericUse { parameters: &parameters, found: false };
        syn::visit::Visit::visit_type(&mut usage, ty);
        if usage.found {
            return Err(syn::Error::new_spanned(ty, "composition requires concrete error types; function/impl generic parameters and Self are not supported"));
        }
    }
    Ok(())
}

fn metadata(
    args: &ErrorsetArgs,
    name: &Ident,
    vis: &Visibility,
    members: &[(Ident, Type)],
    cfgs: &[&syn::Attribute],
) -> proc_macro2::TokenStream {
    // Only internal exported macro names need crate-wide uniqueness.
    static NEXT_DESCRIPTOR: AtomicUsize = AtomicUsize::new(0);
    let internal = format_ident!("__errorset_{}_{}", name, NEXT_DESCRIPTOR.fetch_add(1, Ordering::Relaxed));
    let descriptor = args.export.as_ref().map(|export| match export {
        DescriptorExport::Default => format_ident!("{}Descriptor", name),
        DescriptorExport::Named(name) => name.clone(),
    });
    let descriptor_alias = descriptor.filter(|descriptor| descriptor != name).map(|descriptor| {
        quote! {
            #(#cfgs)*
            #[doc(hidden)]
            #[allow(unused_imports)]
            #vis use #internal as #descriptor;
        }
    });
    let runtime = cfg!(feature = "runtime") || args.export.is_some() || !args.sources.is_empty();
    let member_impls = members.iter().enumerate().filter(|_| runtime).map(|(index, (_, ty))| {
        quote! {
            #(#cfgs)*
            impl ::smarterr::ErrorSetMember<#index> for #name {
                type Error = #ty;
            }
        }
    });
    let variants = members
        .iter()
        .enumerate()
        .map(|(index, (variant, _))| quote! { #variant #index, });
    let atom = runtime.then(|| error_set_impl(name, members.iter().map(|(variant, _)| variant), cfgs));
    quote! {
        #(#member_impls)*
        #atom
        #(#cfgs)*
        #[doc(hidden)]
        #[macro_export]
        macro_rules! #internal {
            ($callback:path, [$($context:tt)*]) => {
                $callback! { [$($context)*] [#(#variants)*] }
            };
        }
        #(#cfgs)*
        #[doc(hidden)]
        #[allow(unused_imports)]
        #vis use #internal as #name;
        #descriptor_alias
    }
}

fn error_set_impl<'a>(
    name: &Ident,
    variants: impl Iterator<Item = &'a Ident>,
    cfgs: &[&syn::Attribute],
) -> proc_macro2::TokenStream {
    let arms: Vec<_> = variants.map(|variant| quote! { Self::#variant(error) => error, }).collect();
    let body = if arms.is_empty() {
        quote! { match *self {} }
    } else {
        quote! { match self { #(#arms)* } }
    };
    quote! {
        #(#cfgs)*
        impl ::smarterr::ErrorSet for #name {
            fn as_error(&self) -> &(dyn ::std::error::Error + 'static) {
                #body
            }
        }
    }
}

struct Composition {
    source: TypePath,
    target: Ident,
    visibility: Visibility,
    remainder: Ident,
    targets: Vec<(Ident, Type)>,
    mappings: Vec<(Ident, Ident)>,
    variants: Vec<(Ident, LitInt)>,
}

impl Parse for Composition {
    fn parse(input: ParseStream) -> Result<Self> {
        let context;
        bracketed!(context in input);
        let source = context.parse()?;
        context.parse::<Token![;]>()?;
        let target = context.parse()?;
        context.parse::<Token![;]>()?;
        let visibility = context.parse()?;
        context.parse::<Token![;]>()?;
        let remainder = context.parse()?;
        context.parse::<Token![;]>()?;
        let target_list;
        bracketed!(target_list in context);
        let mut targets = Vec::new();
        while !target_list.is_empty() {
            let name = target_list.parse()?;
            target_list.parse::<Token![:]>()?;
            targets.push((name, target_list.parse()?));
            target_list.parse::<Token![,]>()?;
        }
        let mapping_list;
        bracketed!(mapping_list in context);
        let mut mappings = Vec::new();
        while !mapping_list.is_empty() {
            let name = mapping_list.parse()?;
            mapping_list.parse::<Token![=>]>()?;
            mappings.push((name, mapping_list.parse()?));
            mapping_list.parse::<Token![,]>()?;
        }
        if !context.is_empty() {
            return Err(context.error("unexpected composition metadata"));
        }
        let variant_list;
        bracketed!(variant_list in input);
        let mut variants = Vec::new();
        while !variant_list.is_empty() {
            variants.push((variant_list.parse()?, variant_list.parse()?));
            variant_list.parse::<Token![,]>()?;
        }
        Ok(Self {
            source,
            target,
            visibility,
            remainder,
            targets,
            mappings,
            variants,
        })
    }
}

/// Callback for the version-one descriptor protocol.
#[doc(hidden)]
#[proc_macro]
pub fn __compose(input: TokenStream) -> TokenStream {
    let composition = parse_macro_input!(input as Composition);
    match compose(composition) {
        Ok(output) => output,
        Err(error) => error.to_compile_error(),
    }
    .into()
}

fn compose(input: Composition) -> Result<proc_macro2::TokenStream> {
    let Composition {
        source,
        target,
        visibility,
        remainder,
        targets,
        mappings,
        variants,
    } = input;
    if source.qself.is_none() && source.path.get_ident() == Some(&target) {
        return Err(syn::Error::new_spanned(source, "a set cannot convert from itself"));
    }
    let target_names: HashSet<_> = targets.iter().map(|(name, _)| name.to_string()).collect();
    let source_names: HashSet<_> = variants.iter().map(|(name, _)| name.to_string()).collect();
    let mut explicit = HashMap::new();
    for (s, t) in mappings {
        if !source_names.contains(&s.to_string()) {
            return Err(syn::Error::new_spanned(s, "mapping names an unknown source variant"));
        }
        if !target_names.contains(&t.to_string()) {
            return Err(syn::Error::new_spanned(t, "mapping names an unknown target variant"));
        }
        if explicit.insert(s.to_string(), t).is_some() {
            return Err(syn::Error::new_spanned(s, "duplicate source variant mapping"));
        }
    }
    let mut remainder_variants = Vec::new();
    let mut remainder_names = Vec::new();
    let mut split_arms = Vec::new();
    let mut from_arms = Vec::new();
    for (variant, index) in variants {
        let destination = explicit
            .get(&variant.to_string())
            .cloned()
            .or_else(|| target_names.contains(&variant.to_string()).then(|| variant.clone()));
        if let Some(destination) = destination {
            // Constructing the target variant makes rustc check actual type equality.
            split_arms.push(quote! { Self::#variant(e) => Err(#target::#destination(e)), });
            from_arms.push(quote! { #source::#variant(e) => Self::#destination(e), });
        } else {
            remainder_names.push(variant.clone());
            remainder_variants.push(quote! {
                #[error(transparent)]
                #variant(<#source as ::smarterr::ErrorSetMember<#index>>::Error),
            });
            split_arms.push(quote! { Self::#variant(e) => Ok(#remainder::#variant(e)), });
        }
    }
    let from = if remainder_variants.is_empty() {
        quote! {
            impl ::core::convert::From<#source> for #target {
                fn from(error: #source) -> Self {
                    match error { #(#from_arms)* }
                }
            }
        }
    } else {
        quote! {}
    };
    let atom = error_set_impl(&remainder, remainder_names.iter(), &[]);
    Ok(quote! {
        #[derive(::thiserror::Error, Debug)]
        #visibility enum #remainder {
            #(#remainder_variants)*
        }
        #atom
        impl ::smarterr::SplitInto<#target> for #source {
            type Remainder = #remainder;
            fn split_into(self) -> ::core::result::Result<Self::Remainder, #target> {
                match self { #(#split_arms)* }
            }
        }
        #from
    })
}

fn handle_function(args: &ErrorsetArgs, item_fn: ItemFn) -> TokenStream {
    match process_fn(args, &item_fn, None) {
        Ok(Some(Output { enum_def, fn_def })) => {
            if let Some(module) = &args.module {
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
            }
        }
        Ok(None) => quote! { #item_fn },
        Err(e) => e.to_compile_error(),
    }
    .into()
}

fn handle_impl_block(args: &ErrorsetArgs, item_impl: ItemImpl) -> TokenStream {
    if args.export.is_some() || !args.sources.is_empty() {
        return syn::Error::new_spanned(
            &item_impl,
            "place export/from on individual methods; keep the module option on the impl block",
        )
        .to_compile_error()
        .into();
    }
    if item_impl.trait_.is_some()
        && item_impl.items.iter().any(|item| {
            matches!(item, syn::ImplItem::Fn(method) if method.attrs.iter().any(|a| {
                a.path().is_ident("errorset") && !matches!(a.meta, syn::Meta::Path(_))
            }))
        })
    {
        return syn::Error::new_spanned(&item_impl, "composition on trait implementations is not supported")
            .to_compile_error()
            .into();
    }
    let mut new_items = Vec::new();
    let mut new_enums = Vec::new();

    for item in item_impl.items {
        if let syn::ImplItem::Fn(method) = &item {
            let mut new_attrs = Vec::new();
            let mut marked = false;
            let mut method_args = ErrorsetArgs {
                visibility: args.visibility.clone(),
                module: args.module.clone(),
                export: None,
                sources: Vec::new(),
            };

            for attr in &method.attrs {
                if attr.path().is_ident("errorset") {
                    if marked {
                        return syn::Error::new_spanned(attr, "duplicate errorset attribute")
                            .to_compile_error()
                            .into();
                    }
                    if let syn::Meta::List(_) = &attr.meta {
                        let options = match attr.parse_args::<ErrorsetArgs>() {
                            Ok(options) => options,
                            Err(error) => return error.to_compile_error().into(),
                        };
                        if options.module.is_some() || !matches!(options.visibility, Visibility::Inherited) {
                            return syn::Error::new_spanned(attr, "place module/visibility options on the impl block")
                                .to_compile_error()
                                .into();
                        }
                        method_args.export = options.export;
                        method_args.sources = options.sources;
                    } else if !matches!(attr.meta, syn::Meta::Path(_)) {
                        return syn::Error::new_spanned(attr, "expected errorset or errorset(...)")
                            .to_compile_error()
                            .into();
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
                vis: method.vis.clone(),
                sig: method.sig.clone(),
                block: Box::new(method.block.clone()),
            };

            match process_fn(&method_args, &item_fn, Some(&item_impl.generics)) {
                Ok(Some(Output { enum_def, fn_def })) => {
                    let impl_item = syn::parse2::<ImplItemFn>(fn_def).expect("Invalid method reparse");
                    new_items.push(impl_item.into());
                    new_enums.push(enum_def);
                }
                Ok(None) => new_items.push(item),
                Err(e) => return e.to_compile_error().into(),
            }
        } else {
            new_items.push(item);
        }
    }

    let new_impl_block = ItemImpl { items: new_items, ..item_impl };

    if let Some(module) = &args.module {
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
    }
    .into()
}
