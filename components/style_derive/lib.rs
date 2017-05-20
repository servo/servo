/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

extern crate proc_macro;
#[macro_use] extern crate quote;
extern crate syn;
extern crate synstructure;

use proc_macro::TokenStream;

mod has_viewport_percentage;

#[proc_macro_derive(HasViewportPercentage)]
pub fn derive_has_viewport_percentage(stream: TokenStream) -> TokenStream {
    let input = syn::parse_derive_input(&stream.to_string()).unwrap();
    has_viewport_percentage::derive(input).to_string().parse().unwrap()
}
