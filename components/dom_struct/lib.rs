/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(proc_macro)]

extern crate proc_macro;

use proc_macro::{TokenStream, quote};
use std::iter;

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
    iter::once(attributes).chain(iter::once(input)).collect()
}
