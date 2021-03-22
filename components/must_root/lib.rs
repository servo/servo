/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use syn::*;

#[proc_macro_attribute]
pub fn must_root(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        panic!("#[must_root] takes no arguments");
    }
    let attributes = quote! {
        #[unrooted_must_root_lint::must_root_type]
    };

    // Work around https://github.com/rust-lang/rust/issues/46489
    let attributes: TokenStream = attributes.to_string().parse().unwrap();

    let output: TokenStream = attributes.into_iter().chain(input.into_iter()).collect();

    let item: Item = syn::parse(output).unwrap();

    if let Item::Struct(s) = item {
        let s2 = s.clone();
        if s.generics.params.len() > 0 {
            return quote!(#s2).into();
        }
        if let Fields::Named(_) = s.fields {
            let name = &s.ident;

            quote! (
                #s2

                impl Drop for #name {
                    fn drop(&mut self) {}
                }
            )
            .into()
        } else {
            panic!("#[must_root] only applies to structs with named fields");
        }
    } else {
        panic!("#[must_root] only applies to structs");
    }
}
