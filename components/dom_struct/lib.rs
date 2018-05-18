/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(proc_macro)]

extern crate proc_macro;

use proc_macro::{TokenStream, quote};

#[proc_macro_attribute]
pub fn dom_struct(args: TokenStream, input: TokenStream) -> TokenStream {
    if !args.is_empty() {
        panic!("#[dom_struct] takes no arguments");
    }
    let attributes = quote! {
        #[derive(DenyPublicFields, DomObject, JSTraceable, MallocSizeOf)]
        #[must_root]
        #[repr(C)]
    };

    // Work around https://github.com/rust-lang/rust/issues/46489
    let attributes: TokenStream = attributes.to_string().parse().unwrap();

    attributes.into_iter().chain(input.into_iter()).collect()
}
