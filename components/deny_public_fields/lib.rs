/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate proc_macro;
extern crate syn;
extern crate synstructure;

#[proc_macro_derive(DenyPublicFields)]
pub fn expand_token_stream(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_string(&input.to_string()).parse().unwrap()
}

fn expand_string(input: &str) -> String {
    let type_ = syn::parse_macro_input(input).unwrap();

    let style = synstructure::BindStyle::Ref.into();
    synstructure::each_field(&type_, &style, |binding| {
        if binding.field.vis != syn::Visibility::Inherited {
            panic!("Field `{}` should not be public",
                   binding.field.ident.as_ref().unwrap_or(&binding.ident));
        }
        "".to_owned()
    });

    "".to_owned()
}
