/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(proc_macro)]

extern crate proc_macro;
#[macro_use] extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn dom_struct(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.to_string().is_empty() {
        panic!("#[dom_struct] takes no arguments");
    }
    expand_string(&input.to_string()).parse().unwrap()
}

fn expand_string(input: &str) -> String {
    let mut tokens = quote! {
        #[derive(DenyPublicFields, DomObject, HeapSizeOf, JSTraceable)]
        #[must_root]
        #[repr(C)]
    };
    tokens.append(input);
    tokens.to_string()
}
