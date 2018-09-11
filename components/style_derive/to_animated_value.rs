/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote;
use syn::DeriveInput;
use synstructure::BindStyle;

pub fn derive(mut input: DeriveInput) -> quote::Tokens {
    let mut where_clause = input.generics.where_clause.take();
    for param in input.generics.type_params() {
        cg::add_predicate(
            &mut where_clause,
            parse_quote!(#param: ::values::animated::ToAnimatedValue),
        );
    }

    let to_body = cg::fmap_match(
        &input,
        BindStyle::Move,
        |binding| quote!(::values::animated::ToAnimatedValue::to_animated_value(#binding)),
    );
    let from_body = cg::fmap_match(
        &input,
        BindStyle::Move,
        |binding| quote!(::values::animated::ToAnimatedValue::from_animated_value(#binding)),
    );

    input.generics.where_clause = where_clause;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let animated_value_type = cg::fmap_trait_output(
        &input,
        &parse_quote!(values::animated::ToAnimatedValue),
        "AnimatedValue".into(),
    );

    quote! {
        impl #impl_generics ::values::animated::ToAnimatedValue for #name #ty_generics #where_clause {
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
