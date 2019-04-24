/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use derive_common::cg;
use proc_macro2::TokenStream;
use syn::DeriveInput;
use synstructure::BindStyle;
use to_computed_value;

pub fn derive(input: DeriveInput) -> TokenStream {
    let trait_impl = |from_body, to_body| {
        quote! {
             #[inline]
             fn from_resolved_value(resolved: Self::ResolvedValue) -> Self {
                 match resolved {
                     #from_body
                 }
             }

             #[inline]
             fn to_resolved_value(
                 self,
                 context: &crate::values::resolved::Context,
             ) -> Self::ResolvedValue {
                 match self {
                     #to_body
                 }
             }
        }
    };

    let non_generic_implementation = || {
        Some(quote! {
            type ResolvedValue = Self;

            #[inline]
            fn from_resolved_value(resolved: Self::ResolvedValue) -> Self {
                resolved
            }

            #[inline]
            fn to_resolved_value(
                self,
                context: &crate::values::resolved::Context,
            ) -> Self {
                self
            }
        })
    };

    to_computed_value::derive_to_value(
        input,
        parse_quote!(crate::values::resolved::ToResolvedValue),
        parse_quote!(ResolvedValue),
        BindStyle::Move,
        |binding| cg::parse_field_attrs::<ResolvedValueAttrs>(&binding.ast()).field_bound,
        |binding| quote!(crate::values::resolved::ToResolvedValue::from_resolved_value(#binding)),
        |binding| quote!(crate::values::resolved::ToResolvedValue::to_resolved_value(#binding, context)),
        trait_impl,
        non_generic_implementation,
    )
}

#[darling(attributes(resolve), default)]
#[derive(Default, FromField)]
struct ResolvedValueAttrs {
    field_bound: bool,
}
