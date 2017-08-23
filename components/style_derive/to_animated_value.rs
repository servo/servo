/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote;
use syn;
use synstructure;

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause, animated_value_type) =
        cg::fmap_trait_parts(
            &input,
            &["values", "animated", "ToAnimatedValue"],
            "AnimatedValue",
        );

    let to_body = match_body(&input, |field| {
        quote!(::values::animated::ToAnimatedValue::to_animated_value(#field))
    });
    let from_body = match_body(&input, |field| {
        quote!(::values::animated::ToAnimatedValue::from_animated_value(#field))
    });

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

fn match_body<F>(input: &syn::DeriveInput, f: F) -> quote::Tokens
where
    F: Fn(&synstructure::BindingInfo) -> quote::Tokens,
{
    let by_value = synstructure::BindStyle::Move.into();
    synstructure::each_variant(&input, &by_value, |fields, variant| {
        let name = if let syn::Body::Enum(_) = input.body {
            format!("{}::{}", input.ident, variant.ident).into()
        } else {
            variant.ident.clone()
        };
        let (animated_value, computed_fields) = synstructure::match_pattern(&name, &variant.data, &by_value);
        let fields_pairs = fields.iter().zip(computed_fields.iter());
        let mut computations = quote!();
        computations.append_all(fields_pairs.map(|(field, computed_field)| {
            let expr = f(field);
            quote!(let #computed_field = #expr;)
        }));
        Some(quote!(
            #computations
            #animated_value
        ))
    })
}
