/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote;
use syn;
use synstructure;

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause, computed_value_type) =
        cg::fmap_trait_parts(
            &input,
            &["values", "computed", "ToComputedValue"],
            "ComputedValue",
        );

    let to_body = match_body(&input, |field| {
        quote!(::values::computed::ToComputedValue::to_computed_value(#field, context))
    });
    let from_body = match_body(&input, |field| {
        quote!(::values::computed::ToComputedValue::from_computed_value(#field))
    });

    quote! {
        impl #impl_generics ::values::computed::ToComputedValue for #name #ty_generics #where_clause {
            type ComputedValue = #computed_value_type;

            #[allow(unused_variables)]
            #[inline]
            fn to_computed_value(&self, context: &::values::computed::Context) -> Self::ComputedValue {
                match *self {
                    #to_body
                }
            }

            #[inline]
            fn from_computed_value(computed: &Self::ComputedValue) -> Self {
                match *computed {
                    #from_body
                }
            }
        }
    }
}

fn match_body<F>(input: &syn::DeriveInput, f: F) -> quote::Tokens
    where F: Fn(&synstructure::BindingInfo) -> quote::Tokens,
{
    let by_ref = synstructure::BindStyle::Ref.into();
    let by_value = synstructure::BindStyle::Move.into();

    synstructure::each_variant(&input, &by_ref, |fields, variant| {
        let name = if let syn::Body::Enum(_) = input.body {
            format!("{}::{}", input.ident, variant.ident).into()
        } else {
            variant.ident.clone()
        };
        let (computed_value, computed_fields) = synstructure::match_pattern(&name, &variant.data, &by_value);
        let fields_pairs = fields.iter().zip(computed_fields.iter());
        let mut computations = quote!();
        computations.append_all(fields_pairs.map(|(field, computed_field)| {
            let expr = f(field);
            quote!(let #computed_field = #expr;)
        }));
        Some(quote!(
            #computations
            #computed_value
        ))
    })
}
