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
pub fn dom_struct(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        panic!("#[dom_struct] takes no arguments");
    }
    let attributes = quote! {
        #[derive(DenyPublicFields, DomObject, JSTraceable, MallocSizeOf)]
        #[unrooted_must_root_lint::must_root]
        #[repr(C)]
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
        if let Fields::Named(ref f) = s.fields {
            let f = f.named.first().expect("Must have at least one field");
            let ident = f.ident.as_ref().expect("Must have named fields");
            let name = &s.ident;
            let ty = &f.ty;

            quote! (
                #s2

                impl crate::dom::bindings::inheritance::HasParent for #name {
                    type Parent = #ty;
                    /// This is used in a type assertion to ensure that
                    /// the source and webidls agree as to what the parent type is
                    fn as_parent(&self) -> &#ty {
                        &self.#ident
                    }
                }
            )
            .into()
        } else {
            panic!("#[dom_struct] only applies to structs with named fields");
        }
    } else {
        panic!("#[dom_struct] only applies to structs");
    }
}
