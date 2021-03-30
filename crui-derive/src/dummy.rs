use proc_macro2::{Ident, TokenStream};
use quote::format_ident;

use crate::r#try;

pub fn wrap_in_const(
    serde_path: Option<&syn::Path>,
    crui_path: Option<&syn::Path>,
    trait_: &str,
    ty: &Ident,
    code: TokenStream,
) -> TokenStream {
    let try_replacement = r#try::replacement();

    let dummy_const = if cfg!(underscore_consts) {
        format_ident!("_")
    } else {
        format_ident!("_IMPL_{}_FOR_{}", trait_, unraw(ty))
    };

    let use_serde = match serde_path {
        Some(path) => quote! {
            use #path as _serde;
        },
        None => quote! {
            #[allow(rust_2018_idioms, clippy::useless_attribute)]
            extern crate serde as _serde;
        },
    };

    let use_crui = match crui_path {
        Some(path) => quote! {
            use #path as _crui;
        },
        None => quote! {
            #[allow(rust_2018_idioms, clippy::useless_attribute)]
            extern crate crui as _crui;
        },
    };

    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            #use_serde
            #use_crui
            #try_replacement
            #code
        };
    }
}

#[allow(deprecated)]
fn unraw(ident: &Ident) -> String {
    // str::trim_start_matches was added in 1.30, trim_left_matches deprecated
    // in 1.33. We currently support rustc back to 1.15 so we need to continue
    // to use the deprecated one.
    ident.to_string().trim_left_matches("r#").to_owned()
}
