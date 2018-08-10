/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate proc_macro;
extern crate syn;
#[macro_use] extern crate quote;

use proc_macro::{TokenStream};
use quote::__rt::Span;
use syn::*;

#[proc_macro_attribute]
pub fn dom_struct(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut base = None;
    let mut crater = None;
    if !args.is_empty() {
        use quote::ToTokens;

        let args = args.to_string();
        let crate_name = args.trim_matches(&['(', ')', ' '][..]);
        crater = Some(syn::Ident::new(crate_name, Span::call_site()));
        base = Some(quote! {#[base = #crate_name]});
        println!("{}", base.clone().unwrap());
    }
    let attributes = quote! {
        #[derive(DomObject, DenyPublicFields, JSTraceable, MallocSizeOf)]
        #base
        #[must_root]
        #[repr(C)]
    };

    // Work around https://github.com/rust-lang/rust/issues/46489
    let attributes: TokenStream = attributes.to_string().parse().unwrap();


    let output: TokenStream = attributes.into_iter().chain(input.into_iter()).collect();

    let item: Item = syn::parse(output).unwrap();

    if let Item::Struct(s) = item {
        let s2 = s.clone();
        if let Fields::Named(ref f) = s.fields {
            let f = f.named.first().expect("Must have at least one field").into_value();
            let ident = f.ident.as_ref().expect("Must have named fields");
            let name = &s.ident;
            let (impl_generics, ty_generics, _) = &s.generics.split_for_impl();
            let ty = &f.ty;

            quote! (
                #s2

                impl#impl_generics #crater::dom::bindings::inheritance::HasParent for #name#ty_generics {
                    type Parent = #ty;
                    /// This is used in a type assertion to ensure that
                    /// the source and webidls agree as to what the parent type is
                    fn as_parent(&self) -> &#ty {
                        &self.#ident
                    }
                }
            ).into()
        } else {
            panic!("#[dom_struct] only applies to structs with named fields");
        }
    } else {
        panic!("#[dom_struct] only applies to structs");
    }
}
