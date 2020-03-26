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
             fn from_resolved_value(from: Self::ResolvedValue) -> Self {
                 #from_body
             }

             #[inline]
             fn to_resolved_value(
                 self,
                 context: &crate::values::resolved::Context,
             ) -> Self::ResolvedValue {
                 #to_body
             }
        }
    };

    to_computed_value::derive_to_value(
        input,
        parse_quote!(crate::values::resolved::ToResolvedValue),
        parse_quote!(ResolvedValue),
        BindStyle::Move,
        |binding| {
            let attrs = cg::parse_field_attrs::<ResolvedValueAttrs>(&binding.ast());
            to_computed_value::ToValueAttrs {
                field_bound: attrs.field_bound,
                no_field_bound: attrs.no_field_bound,
            }
        },
        |binding| quote!(crate::values::resolved::ToResolvedValue::from_resolved_value(#binding)),
        |binding| quote!(crate::values::resolved::ToResolvedValue::to_resolved_value(#binding, context)),
        trait_impl,
    )
}

#[darling(attributes(resolve), default)]
#[derive(Default, FromField)]
struct ResolvedValueAttrs {
    field_bound: bool,
    no_field_bound: bool,
}
