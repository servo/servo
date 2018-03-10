/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use animate::AnimationVariantAttrs;
use cg;
use quote::Tokens;
use syn::{DeriveInput, Path};
use synstructure;

pub fn derive(input: DeriveInput) -> Tokens {
    let name = &input.ident;
    let trait_path = parse_quote!(values::distance::ComputeSquaredDistance);
    let (impl_generics, ty_generics, mut where_clause) =
        cg::trait_parts(&input, &trait_path);

    let input_attrs = cg::parse_input_attrs::<DistanceInputAttrs>(&input);
    let s = synstructure::Structure::new(&input);
    let mut append_error_clause = s.variants().len() > 1;

    let mut match_body = s.variants().iter().fold(quote!(), |body, variant| {
        let attrs = cg::parse_variant_attrs::<AnimationVariantAttrs>(&variant.ast());
        if attrs.error {
            append_error_clause = true;
            return body;
        }

        let (this_pattern, this_info) = cg::ref_pattern(&variant, "this");
        let (other_pattern, other_info) = cg::ref_pattern(&variant, "other");
        let sum = if this_info.is_empty() {
            quote! { ::values::distance::SquaredDistance::from_sqrt(0.) }
        } else {
            let mut sum = quote!();
            sum.append_separated(this_info.iter().zip(&other_info).map(|(this, other)| {
                where_clause.add_trait_bound(&this.ast().ty);
                quote! {
                    ::values::distance::ComputeSquaredDistance::compute_squared_distance(#this, #other)?
                }
            }), quote!(+));
            sum
        };
        quote! {
            #body
            (&#this_pattern, &#other_pattern) => {
                Ok(#sum)
            }
        }
    });

    if append_error_clause {
        if let Some(fallback) = input_attrs.fallback {
            match_body.append_all(quote! {
                (this, other) => #fallback(this, other)
            });
        } else {
            match_body.append_all(quote! { _ => Err(()) });
        }
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

#[darling(attributes(distance), default)]
#[derive(Default, FromDeriveInput)]
struct DistanceInputAttrs {
    fallback: Option<Path>,
}
