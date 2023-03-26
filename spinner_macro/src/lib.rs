use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, punctuated::Punctuated, token::Comma, Expr};

#[proc_macro_attribute]
pub fn spinner_wrap(arg: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(arg as syn::AttributeArgs);
    let org_fn = parse_macro_input!(item as syn::ItemFn);
    let name = org_fn.sig.ident.clone();
    let inputs = org_fn.sig.inputs.clone();
    let transformed_params = transform_params(inputs.clone());
    let converter = match args.first() {
        Some(f) => quote! { |r| #f(r) },
        None => quote! { |s| s },
    };

    let name_spinner = format_ident!("{}_spinner", name);
    let expanded = quote! {
        #org_fn

        pub fn #name_spinner(#inputs) {
            crate::commands::wrap_in_spinner(
                || #name #transformed_params,
                #converter
            )
        }
    };

    TokenStream::from(expanded)
}

// See https://stackoverflow.com/questions/71480280/how-do-i-pass-arguments-from-a-generated-function-to-another-function-in-a-proce
fn transform_params(params: Punctuated<syn::FnArg, syn::token::Comma>) -> Expr {
    // 1. Filter the params, so that only typed arguments remain
    // 2. Extract the ident (in case the pattern type is ident)
    let idents = params.iter().filter_map(|param| {
        if let syn::FnArg::Typed(pat_type) = param {
            if let syn::Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                return Some(pat_ident.ident);
            }
        }
        None
    });

    // Add all idents to a Punctuated => param1, param2, ...
    let mut punctuated: Punctuated<syn::Ident, Comma> = Punctuated::new();
    idents.for_each(|ident| punctuated.push(ident));

    // Generate expression from Punctuated (and wrap with parentheses)
    let transformed_params = parse_quote!((#punctuated));
    transformed_params
}
