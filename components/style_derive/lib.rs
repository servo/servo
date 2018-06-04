/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![recursion_limit = "128"]

#[macro_use] extern crate darling;
extern crate proc_macro;
#[macro_use] extern crate quote;
#[macro_use] extern crate syn;
extern crate synstructure;

use proc_macro::TokenStream;

mod animate;
mod cg;
mod compute_squared_distance;
mod parse;
mod specified_value_info;
mod to_animated_value;
mod to_animated_zero;
mod to_computed_value;
mod to_css;

#[proc_macro_derive(Animate, attributes(animate, animation))]
pub fn derive_animate(stream: TokenStream) -> TokenStream {
    let input = syn::parse(stream).unwrap();
    animate::derive(input).into()
}

#[proc_macro_derive(ComputeSquaredDistance, attributes(animation, distance))]
pub fn derive_compute_squared_distance(stream: TokenStream) -> TokenStream {
    let input = syn::parse(stream).unwrap();
    compute_squared_distance::derive(input).into()
}

#[proc_macro_derive(ToAnimatedValue)]
pub fn derive_to_animated_value(stream: TokenStream) -> TokenStream {
    let input = syn::parse(stream).unwrap();
    to_animated_value::derive(input).into()
}

#[proc_macro_derive(Parse, attributes(css, parse))]
pub fn derive_parse(stream: TokenStream) -> TokenStream {
    let input = syn::parse(stream).unwrap();
    parse::derive(input).into()
}

#[proc_macro_derive(ToAnimatedZero, attributes(animation, zero))]
pub fn derive_to_animated_zero(stream: TokenStream) -> TokenStream {
    let input = syn::parse(stream).unwrap();
    to_animated_zero::derive(input).into()
}

#[proc_macro_derive(ToComputedValue, attributes(compute))]
pub fn derive_to_computed_value(stream: TokenStream) -> TokenStream {
    let input = syn::parse(stream).unwrap();
    to_computed_value::derive(input).into()
}

#[proc_macro_derive(ToCss, attributes(css))]
pub fn derive_to_css(stream: TokenStream) -> TokenStream {
    let input = syn::parse(stream).unwrap();
    to_css::derive(input).into()
}

#[proc_macro_derive(SpecifiedValueInfo, attributes(css, parse, value_info))]
pub fn derive_specified_value_info(stream: TokenStream) -> TokenStream {
    let input = syn::parse(stream).unwrap();
    specified_value_info::derive(input).into()
}
