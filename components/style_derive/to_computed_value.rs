/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote::Tokens;
use syn::DeriveInput;
use synstructure::BindStyle;

pub fn derive(input: DeriveInput) -> Tokens {
    let name = &input.ident;
    let trait_path = &["values", "computed", "ToComputedValue"];
    let (impl_generics, ty_generics, mut where_clause, computed_value_type) =
        cg::fmap_trait_parts(&input, trait_path, "ComputedValue");

    let to_body = cg::fmap_match(&input, BindStyle::Ref, |binding| {
        let attrs = cg::parse_field_attrs::<ComputedValueAttrs>(&binding.field);
        if attrs.clone {
            if cg::is_parameterized(&binding.field.ty, where_clause.params, None) {
                where_clause.inner.predicates.push(cg::where_predicate(
                    binding.field.ty.clone(),
                    &["std", "clone", "Clone"],
                    None,
                ));
            }
            quote! { ::std::clone::Clone::clone(#binding) }
        } else {
            where_clause.add_trait_bound(&binding.field.ty);
            quote! {
                ::values::computed::ToComputedValue::to_computed_value(#binding, context)
            }
        }
    });
    let from_body = cg::fmap_match(&input, BindStyle::Ref, |binding| {
        let attrs = cg::parse_field_attrs::<ComputedValueAttrs>(&binding.field);
        if attrs.clone {
            quote! { ::std::clone::Clone::clone(#binding) }
        } else {
            quote! {
                ::values::computed::ToComputedValue::from_computed_value(#binding)
            }
        }
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

#[darling(attributes(compute), default)]
#[derive(Default, FromField)]
struct ComputedValueAttrs {
    clone: bool,
}
