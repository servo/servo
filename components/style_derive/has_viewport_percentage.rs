/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote;
use syn;
use synstructure;

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let trait_path = &["style_traits", "HasViewportPercentage"];
    let (impl_generics, ty_generics, mut where_clause) =
        cg::trait_parts(&input, trait_path);

    let style = synstructure::BindStyle::Ref.into();
    let match_body = synstructure::each_variant(&input, &style, |bindings, _| {
        let (first, rest) = match bindings.split_first() {
            None => return Some(quote!(false)),
            Some(pair) => pair,
        };
        let mut expr = quote!(::style_traits::HasViewportPercentage::has_viewport_percentage(#first));
        for binding in rest {
            where_clause.predicates.push(
                cg::where_predicate(binding.field.ty.clone(), trait_path),
            );
            expr = quote!(#expr || ::style_traits::HasViewportPercentage::has_viewport_percentage(#binding));
        }
        Some(expr)
    });

    quote! {
        impl #impl_generics ::style_traits::HasViewportPercentage for #name #ty_generics #where_clause {
            #[allow(unused_variables, unused_imports)]
            #[inline]
            fn has_viewport_percentage(&self) -> bool {
                match *self {
                    #match_body
                }
            }
        }
    }
}
