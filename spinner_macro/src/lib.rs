use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro_attribute]
pub fn fn_spinner_wrap(arg: TokenStream, item: TokenStream) -> TokenStream {
    let vfn = venial::parse_declaration(item.clone().into()).unwrap();
    let mut out = item;

    if let venial::Declaration::Function(f) = vfn {
        out.extend(generate_wrapping_fn(f, arg.clone().to_string()));
    }

    return out;
}

fn generate_wrapping_fn(org_fn: venial::Function, conv: String) -> TokenStream {
    let org_ident = org_fn.name;
    let vis = org_fn.vis_marker;
    let ident = format_ident!("{org_ident}_spinner");
    let inputs = org_fn.params.clone();
    let params = extract_params(inputs.clone());
    let converter = if conv.is_empty() {
        quote! { |s| s }
    } else {
        let ident = format_ident!("{conv}",);
        quote! { |r| #ident(r) }
    };
    (quote! {
        #vis fn #ident(#inputs) {
            crate::commands::wrap_in_spinner(
                || #org_ident(#params),
                #converter
            )
        }
    })
    .into()
}

fn extract_params(args: venial::Punctuated<venial::FnParam>) -> venial::Punctuated<proc_macro2::Ident> {
    let mut params: venial::Punctuated<proc_macro2::Ident> = venial::Punctuated::new();

    for p in args.items() {
        if let venial::FnParam::Typed(p) = p {
            params.push(p.name.clone(), None);
        }
    }

    params
}
