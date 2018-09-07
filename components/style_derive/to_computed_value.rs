/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote::Tokens;
use syn::DeriveInput;
use synstructure::BindStyle;

pub fn derive(mut input: DeriveInput) -> Tokens {
    let mut where_clause = input.generics.where_clause.take();
    let (to_body, from_body) = {
        let params = input.generics.type_params().collect::<Vec<_>>();
        for param in &params {
            cg::add_predicate(
                &mut where_clause,
                parse_quote!(#param: ::values::computed::ToComputedValue),
            );
        }

        let to_body = cg::fmap_match(&input, BindStyle::Ref, |binding| {
            let attrs = cg::parse_field_attrs::<ComputedValueAttrs>(&binding.ast());
            if attrs.field_bound {
                let ty = &binding.ast().ty;

                let output_type = cg::map_type_params(
                    ty,
                    &params,
                    &mut |ident| parse_quote!(<#ident as ::values::computed::ToComputedValue>::ComputedValue),
                );

                cg::add_predicate(
                    &mut where_clause,
                    parse_quote!(
                        #ty: ::values::computed::ToComputedValue<ComputedValue = #output_type>
                    ),
                );
            }
            quote! {
                ::values::computed::ToComputedValue::to_computed_value(#binding, context)
            }
        });
        let from_body = cg::fmap_match(&input, BindStyle::Ref, |binding| {
            quote! {
                ::values::computed::ToComputedValue::from_computed_value(#binding)
            }
        });

        (to_body, from_body)
    };

    input.generics.where_clause = where_clause;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    if input.generics.type_params().next().is_none() {
        return quote! {
            impl #impl_generics ::values::computed::ToComputedValue for #name #ty_generics
            #where_clause
            {
                type ComputedValue = Self;

                #[inline]
                fn to_computed_value(
                    &self,
                    _context: &::values::computed::Context,
                ) -> Self::ComputedValue {
                    ::std::clone::Clone::clone(self)
                }

                #[inline]
                fn from_computed_value(computed: &Self::ComputedValue) -> Self {
                    ::std::clone::Clone::clone(computed)
                }
            }
        };
    }

    let computed_value_type = cg::fmap_trait_output(
        &input,
        &parse_quote!(values::computed::ToComputedValue),
        "ComputedValue".into(),
    );

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

#[darling(attributes(compute), default)]
#[derive(Default, FromField)]
struct ComputedValueAttrs {
    field_bound: bool,
}
