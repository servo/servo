/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![recursion_limit = "128"]

#[macro_use]
extern crate darling;
extern crate derive_common;
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
extern crate synstructure;

use proc_macro::TokenStream;

mod to_shmem;

#[proc_macro_derive(ToShmem, attributes(shmem))]
pub fn derive_to_shmem(stream: TokenStream) -> TokenStream {
    let input = syn::parse(stream).unwrap();
    to_shmem::derive(input).into()
}
