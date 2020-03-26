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
    mut binding_attrs: impl FnMut(&BindingInfo) -> ToValueAttrs,
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
) -> TokenStream {
    let name = &input.ident;

    let mut where_clause = input.generics.where_clause.take();
    cg::propagate_clauses_to_output_type(
        &mut where_clause,
        &input.generics,
        &trait_path,
        &output_type_name,
    );

    let moves = match bind_style {
        BindStyle::Move | BindStyle::MoveMut => true,
        BindStyle::Ref | BindStyle::RefMut => false,
    };

    let params = input.generics.type_params().collect::<Vec<_>>();
    for param in &params {
        cg::add_predicate(&mut where_clause, parse_quote!(#param: #trait_path));
    }

    let mut add_field_bound = |binding: &BindingInfo| {
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
    };

    let (to_body, from_body) = if params.is_empty() {
        let mut s = synstructure::Structure::new(&input);
        s.variants_mut().iter_mut().for_each(|v| {
            v.bind_with(|_| bind_style);
        });

        for variant in s.variants() {
            for binding in variant.bindings() {
                let attrs = binding_attrs(&binding);
                assert!(
                    !attrs.field_bound,
                    "It is default on a non-generic implementation",
                );
                if !attrs.no_field_bound {
                    // Add field bounds to all bindings except the manually
                    // excluded. This ensures the correctness of the clone() /
                    // move based implementation.
                    add_field_bound(binding);
                }
            }
        }

        let to_body = if moves {
            quote! { self }
        } else {
            quote! { std::clone::Clone::clone(self) }
        };

        let from_body = if moves {
            quote! { from }
        } else {
            quote! { std::clone::Clone::clone(from) }
        };

        (to_body, from_body)
    } else {
        let to_body = cg::fmap_match(&input, bind_style, |binding| {
            let attrs = binding_attrs(&binding);
            assert!(!attrs.no_field_bound, "It doesn't make sense on a generic implementation");
            if attrs.field_bound {
                add_field_bound(&binding);
            }
            call_to(&binding)
        });

        let from_body = cg::fmap_match(&input, bind_style, |binding| call_from(&binding));

        let self_ = if moves { quote! { self } } else { quote! { *self } };
        let from_ = if moves { quote! { from } } else { quote! { *from } };

        let to_body = quote! {
            match #self_ {
                #to_body
            }
        };

        let from_body = quote! {
            match #from_ {
                #from_body
            }
        };

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
             fn from_computed_value(from: &Self::ComputedValue) -> Self {
                 #from_body
             }

             #[allow(unused_variables)]
             #[inline]
             fn to_computed_value(&self, context: &crate::values::computed::Context) -> Self::ComputedValue {
                 #to_body
             }
        }
    };

    derive_to_value(
        input,
        parse_quote!(crate::values::computed::ToComputedValue),
        parse_quote!(ComputedValue),
        BindStyle::Ref,
        |binding| {
            let attrs = cg::parse_field_attrs::<ComputedValueAttrs>(&binding.ast());
            ToValueAttrs {
                field_bound: attrs.field_bound,
                no_field_bound: attrs.no_field_bound,
            }
        },
        |binding| quote!(crate::values::computed::ToComputedValue::from_computed_value(#binding)),
        |binding| quote!(crate::values::computed::ToComputedValue::to_computed_value(#binding, context)),
        trait_impl,
    )
}

#[derive(Default)]
pub struct ToValueAttrs {
    pub field_bound: bool,
    pub no_field_bound: bool,
}

#[darling(attributes(compute), default)]
#[derive(Default, FromField)]
struct ComputedValueAttrs {
    field_bound: bool,
    no_field_bound: bool,
}
