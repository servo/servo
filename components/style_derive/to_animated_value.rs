/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cg;
use proc_macro2::{Span, TokenStream};
use syn::{DeriveInput, Ident};
use synstructure::BindStyle;

pub fn derive(mut input: DeriveInput) -> TokenStream {
    let mut where_clause = input.generics.where_clause.take();
    cg::propagate_clauses_to_output_type(
        &mut where_clause,
        &input.generics,
        parse_quote!(crate::values::animated::ToAnimatedValue),
        parse_quote!(AnimatedValue),
    );
    for param in input.generics.type_params() {
        cg::add_predicate(
            &mut where_clause,
            parse_quote!(#param: crate::values::animated::ToAnimatedValue),
        );
    }

    let to_body = cg::fmap_match(
        &input,
        BindStyle::Move,
        |binding| quote!(crate::values::animated::ToAnimatedValue::to_animated_value(#binding)),
    );
    let from_body = cg::fmap_match(
        &input,
        BindStyle::Move,
        |binding| quote!(crate::values::animated::ToAnimatedValue::from_animated_value(#binding)),
    );

    input.generics.where_clause = where_clause;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let animated_value_type = cg::fmap_trait_output(
        &input,
        &parse_quote!(crate::values::animated::ToAnimatedValue),
        Ident::new("AnimatedValue", Span::call_site()),
    );

    quote! {
        impl #impl_generics crate::values::animated::ToAnimatedValue for #name #ty_generics #where_clause {
            type AnimatedValue = #animated_value_type;

            #[allow(unused_variables)]
            #[inline]
            fn to_animated_value(self) -> Self::AnimatedValue {
                match self {
                    #to_body
                }
            }

            #[inline]
            fn from_animated_value(animated: Self::AnimatedValue) -> Self {
                match animated {
                    #from_body
                }
            }
        }
    }
}
