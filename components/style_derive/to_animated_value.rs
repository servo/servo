/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote;
use syn;
use synstructure::BindStyle;

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause, animated_value_type) =
        cg::fmap_trait_parts(
            &input,
            &["values", "animated", "ToAnimatedValue"],
            "AnimatedValue",
        );

    let to_body = cg::fmap_match(&input, BindStyle::Move, |field| {
        quote!(::values::animated::ToAnimatedValue::to_animated_value(#field))
    });
    let from_body = cg::fmap_match(&input, BindStyle::Move, |field| {
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
