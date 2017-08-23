/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote;
use syn;

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let trait_path = &["values", "animated", "Animate"];
    let (impl_generics, ty_generics, mut where_clause) =
        cg::trait_parts(&input, trait_path);

    let variants = cg::variants(&input);
    let mut match_body = quote!();
    match_body.append_all(variants.iter().map(|variant| {
        let name = cg::variant_ctor(&input, variant);
        let (this_pattern, this_info) = cg::ref_pattern(&name, variant, "this");
        let (other_pattern, other_info) = cg::ref_pattern(&name, variant, "other");
        let (result_value, result_info) = cg::value(&name, variant, "result");
        let mut computations = quote!();
        let iter = result_info.iter().zip(this_info.iter().zip(&other_info));
        computations.append_all(iter.map(|(result, (this, other))| {
            where_clause.predicates.push(
                cg::where_predicate(this.field.ty.clone(), trait_path),
            );
            quote! {
                let #result = ::values::animated::Animate::animate(#this, #other, procedure)?;
            }
        }));
        quote! {
            (&#this_pattern, &#other_pattern) => {
                #computations
                Ok(#result_value)
            }
        }
    }));

    if variants.len() > 1 {
        match_body = quote! { #match_body, _ => Err(()), };
    }

    quote! {
        impl #impl_generics ::values::animated::Animate for #name #ty_generics #where_clause {
            #[allow(unused_variables, unused_imports)]
            #[inline]
            fn animate(
                &self,
                other: &Self,
                procedure: ::values::animated::Procedure,
            ) -> Result<Self, ()> {
                match (self, other) {
                    #match_body
                }
            }
        }
    }
}
