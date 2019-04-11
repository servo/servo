/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use derive_common::cg;
use proc_macro2::TokenStream;
use syn::{DeriveInput, Ident, Path};
use synstructure::{BindStyle, BindingInfo};

pub fn derive_to_value(
    mut input: DeriveInput,
    trait_path: Path,
    output_type_name: Ident,
    bind_style: BindStyle,
    // Returns whether to apply the field bound for a given item.
    mut field_bound: impl FnMut(&BindingInfo) -> bool,
    // Returns a token stream of the form: trait_path::from_foo(#binding)
    mut call_from: impl FnMut(&BindingInfo) -> TokenStream,
    mut call_to: impl FnMut(&BindingInfo) -> TokenStream,
    // Returns a tokenstream of the form:
    // fn from_function_syntax(foobar) -> Baz {
    //     #first_arg
    // }
    //
    // fn to_function_syntax(foobar) -> Baz {
    //     #second_arg
    // }
    mut trait_impl: impl FnMut(TokenStream, TokenStream) -> TokenStream,
    // if this is provided, the derive for non-generic types will be simplified
    // to this token stream, which should be the body of the impl block.
    non_generic_implementation: impl FnOnce() -> Option<TokenStream>,
) -> TokenStream {
    let name = &input.ident;

    if input.generics.type_params().next().is_none() {
        if let Some(non_generic_implementation) = non_generic_implementation() {
            let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
            return quote! {
                impl #impl_generics #trait_path for #name #ty_generics
                #where_clause
                {
                    #non_generic_implementation
                }
            };
        }
    }

    let mut where_clause = input.generics.where_clause.take();
    cg::propagate_clauses_to_output_type(
        &mut where_clause,
        &input.generics,
        &trait_path,
        &output_type_name,
    );
    let (to_body, from_body) = {
        let params = input.generics.type_params().collect::<Vec<_>>();
        for param in &params {
            cg::add_predicate(&mut where_clause, parse_quote!(#param: #trait_path));
        }

        let to_body = cg::fmap_match(&input, bind_style, |binding| {
            if field_bound(&binding) {
                let ty = &binding.ast().ty;

                let output_type = cg::map_type_params(
                    ty,
                    &params,
                    &mut |ident| parse_quote!(<#ident as #trait_path>::#output_type_name),
                );

                cg::add_predicate(
                    &mut where_clause,
                    parse_quote!(
                        #ty: #trait_path<#output_type_name = #output_type>
                    ),
                );
            }
            call_to(&binding)
        });
        let from_body = cg::fmap_match(&input, bind_style, |binding| call_from(&binding));

        (to_body, from_body)
    };

    input.generics.where_clause = where_clause;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let computed_value_type = cg::fmap_trait_output(&input, &trait_path, &output_type_name);

    let impl_ = trait_impl(from_body, to_body);

    quote! {
        impl #impl_generics #trait_path for #name #ty_generics #where_clause {
            type #output_type_name = #computed_value_type;

            #impl_
        }
    }
}

pub fn derive(input: DeriveInput) -> TokenStream {
    let trait_impl = |from_body, to_body| {
        quote! {
             #[inline]
             fn from_computed_value(computed: &Self::ComputedValue) -> Self {
                 match *computed {
                     #from_body
                 }
             }

             #[allow(unused_variables)]
             #[inline]
             fn to_computed_value(&self, context: &crate::values::computed::Context) -> Self::ComputedValue {
                 match *self {
                     #to_body
                 }
             }
        }
    };

    let non_generic_implementation = || {
        Some(quote! {
            type ComputedValue = Self;

            #[inline]
            fn to_computed_value(&self, _: &crate::values::computed::Context) -> Self::ComputedValue {
                std::clone::Clone::clone(self)
            }

            #[inline]
            fn from_computed_value(computed: &Self::ComputedValue) -> Self {
                std::clone::Clone::clone(computed)
            }
        })
    };

    derive_to_value(
        input,
        parse_quote!(crate::values::computed::ToComputedValue),
        parse_quote!(ComputedValue),
        BindStyle::Ref,
        |binding| cg::parse_field_attrs::<ComputedValueAttrs>(&binding.ast()).field_bound,
        |binding| quote!(crate::values::computed::ToComputedValue::from_computed_value(#binding)),
        |binding| quote!(crate::values::computed::ToComputedValue::to_computed_value(#binding, context)),
        trait_impl,
        non_generic_implementation,
    )
}

#[darling(attributes(compute), default)]
#[derive(Default, FromField)]
struct ComputedValueAttrs {
    field_bound: bool,
}
