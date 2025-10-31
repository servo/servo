/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, Fields};
use synstructure::decl_derive;

decl_derive!([ServoPreferences] => servo_preferences_derive);

/// A derive macro that adds string-based getter and setter for each field of this struct
/// (enums and other types are not supported). Each field must be able to be convertable
/// (with `into()`) into a `PrefValue`.
fn servo_preferences_derive(input: synstructure::Structure) -> TokenStream {
    let ast = input.ast();

    let Data::Struct(ref data) = ast.data else {
        unimplemented!();
    };
    let Fields::Named(ref named_fields) = data.fields else {
        unimplemented!()
    };

    let mut exists_match_cases = quote!();
    for field in named_fields.named.iter() {
        let name = field.ident.as_ref().unwrap();
        exists_match_cases.extend(quote!(stringify!(#name) => true,))
    }

    let mut get_match_cases = quote!();
    for field in named_fields.named.iter() {
        let name = field.ident.as_ref().unwrap();
        get_match_cases.extend(quote!(stringify!(#name) => self.#name.clone().into(),))
    }

    let mut set_match_cases = quote!();
    for field in named_fields.named.iter() {
        let name = field.ident.as_ref().unwrap();
        set_match_cases.extend(quote!(stringify!(#name) => self.#name = value.try_into().unwrap(),))
    }

    let mut type_of_match_cases = quote!();
    for field in named_fields.named.iter() {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;
        type_of_match_cases.extend(quote!(stringify!(#name) => std::any::type_name::<#ty>(),))
    }

    let mut comparisons = quote!();
    for field in named_fields.named.iter() {
        let name = field.ident.as_ref().unwrap();
        comparisons.extend(quote!(if self.#name != other.#name { changes.push((stringify!(#name), self.#name.clone().into(),)) }))
    }

    let mut all_fields = quote!();
    for field in named_fields.named.iter() {
        let name = field.ident.as_ref().unwrap();
        all_fields.extend(quote!(stringify!(#name),));
    }

    let structure_name = &ast.ident;
    quote! {
        impl #structure_name {
            pub fn exists(name: &str) -> bool {
                match name {
                    #exists_match_cases
                    _ => { false }
                }
            }

            pub fn get_value(&self, name: &str) -> PrefValue {
                match name {
                    #get_match_cases
                    _ => { panic!("Unknown preference: {:?}", name); }
                }
            }

            pub fn set_value(&mut self, name: &str, value: PrefValue) {
                match name {
                    #set_match_cases
                    _ => { panic!("Unknown preference: {:?}", name); }
                }
            }

            pub fn type_of(name: &str) -> &'static str {
                match name {
                    #type_of_match_cases
                    _ => { panic!("Unknown preference: {:?}", name); }
                }
            }

            pub fn diff(&self, other: &Self) -> Vec<(&'static str, PrefValue)> {
                let mut changes = vec![];
                #comparisons
                changes
            }

            pub fn all_fields() -> Vec<&'static str> {
                vec![
                    #all_fields
                ]
            }
        }
    }
}
