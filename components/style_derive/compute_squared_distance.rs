/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use animate::AnimateAttrs;
use cg;
use quote;
use syn;

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let trait_path = &["values", "distance", "ComputeSquaredDistance"];
    let (impl_generics, ty_generics, mut where_clause) =
        cg::trait_parts(&input, trait_path);

    let variants = cg::variants(&input);
    let mut match_body = quote!();
    let mut append_error_clause = variants.len() > 1;
    match_body.append_all(variants.iter().map(|variant| {
        let attrs = cg::parse_variant_attrs::<AnimateAttrs>(variant);
        if attrs.error {
            append_error_clause = true;
            return None;
        }
        let name = cg::variant_ctor(&input, variant);
        let (this_pattern, this_info) = cg::ref_pattern(&name, &variant, "this");
        let (other_pattern, other_info) = cg::ref_pattern(&name, &variant, "other");
        let sum = if this_info.is_empty() {
            quote! { ::values::distance::SquaredDistance::Value(0.) }
        } else {
            let mut sum = quote!();
            sum.append_separated(this_info.iter().zip(&other_info).map(|(this, other)| {
                where_clause.predicates.push(
                    cg::where_predicate(this.field.ty.clone(), trait_path),
                );
                quote! {
                    ::values::distance::ComputeSquaredDistance::compute_squared_distance(#this, #other)?
                }
            }), "+");
            sum
        };
        Some(quote! {
            (&#this_pattern, &#other_pattern) => {
                Ok(#sum)
            }
        })
    }));

    if append_error_clause {
        match_body = quote! { #match_body, _ => Err(()), };
    }

    quote! {
        impl #impl_generics ::values::distance::ComputeSquaredDistance for #name #ty_generics #where_clause {
            #[allow(unused_variables, unused_imports)]
            #[inline]
            fn compute_squared_distance(
                &self,
                other: &Self,
            ) -> Result<::values::distance::SquaredDistance, ()> {
                match (self, other) {
                    #match_body
                }
            }
        }
    }
}
