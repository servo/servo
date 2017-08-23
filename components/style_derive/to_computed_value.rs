/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote;
use syn;
use synstructure::BindStyle;

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause, computed_value_type) =
        cg::fmap_trait_parts(
            &input,
            &["values", "computed", "ToComputedValue"],
            "ComputedValue",
        );

    let to_body = cg::fmap_match(&input, BindStyle::Ref, |field| {
        quote!(::values::computed::ToComputedValue::to_computed_value(#field, context))
    });
    let from_body = cg::fmap_match(&input, BindStyle::Ref, |field| {
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
