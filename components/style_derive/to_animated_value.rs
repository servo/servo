/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote;
use syn;
use synstructure::BindStyle;

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let trait_path = &["values", "animated", "ToAnimatedValue"];
    let (impl_generics, ty_generics, mut where_clause, animated_value_type) =
        cg::fmap_trait_parts(&input, trait_path, "AnimatedValue");

    let to_body = cg::fmap_match(&input, BindStyle::Move, |binding| {
        where_clause.add_trait_bound(&binding.field.ty);
        quote!(::values::animated::ToAnimatedValue::to_animated_value(#binding))
    });
    let from_body = cg::fmap_match(&input, BindStyle::Move, |binding| {
        quote!(::values::animated::ToAnimatedValue::from_animated_value(#binding))
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
