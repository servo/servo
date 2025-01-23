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

    let structure_name = &ast.ident;
    quote! {
        impl #structure_name {
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
        }
    }
}
