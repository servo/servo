/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use darling::util::IdentList;
use quote::Tokens;
use syn::{DeriveInput, Path};
use synstructure::{Structure, VariantInfo};

pub fn derive(mut input: DeriveInput) -> Tokens {
    let animation_input_attrs = cg::parse_input_attrs::<AnimationInputAttrs>(&input);
    let no_bound = animation_input_attrs.no_bound.unwrap_or_default();
    let mut where_clause = input.generics.where_clause.take();
    for param in input.generics.type_params() {
        if !no_bound.contains(&param.ident) {
            cg::add_predicate(
                &mut where_clause,
                parse_quote!(#param: ::values::animated::Animate),
            );
        }
    }
    input.generics.where_clause = where_clause;

    let s = Structure::new(&input);
    let mut append_error_clause = s.variants().len() > 1;

    let mut match_body = s.variants().iter().fold(quote!(), |body, variant| {
        let arm = match derive_variant_arm(variant) {
            Ok(arm) => arm,
            Err(()) => {
                append_error_clause = true;
                return body;
            }
        };
        quote! { #body #arm }
    });

    if append_error_clause {
        let input_attrs = cg::parse_input_attrs::<AnimateInputAttrs>(&input);
        if let Some(fallback) = input_attrs.fallback {
            match_body.append_all(quote! {
                (this, other) => #fallback(this, other, procedure)
            });
        } else {
            match_body.append_all(quote! { _ => Err(()) });
        }
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

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

fn derive_variant_arm(variant: &VariantInfo) -> Result<Tokens, ()> {
    let variant_attrs = cg::parse_variant_attrs_from_ast::<AnimationVariantAttrs>(&variant.ast());
    if variant_attrs.error {
        return Err(());
    }
    let (this_pattern, this_info) = cg::ref_pattern(&variant, "this");
    let (other_pattern, other_info) = cg::ref_pattern(&variant, "other");
    let (result_value, result_info) = cg::value(&variant, "result");
    let mut computations = quote!();
    let iter = result_info.iter().zip(this_info.iter().zip(&other_info));
    computations.append_all(iter.map(|(result, (this, other))| {
        let field_attrs = cg::parse_field_attrs::<AnimationFieldAttrs>(&result.ast());
        if field_attrs.constant {
            quote! {
                if #this != #other {
                    return Err(());
                }
                let #result = ::std::clone::Clone::clone(#this);
            }
        } else {
            quote! {
                let #result =
                    ::values::animated::Animate::animate(#this, #other, procedure)?;
            }
        }
    }));
    Ok(quote! {
        (&#this_pattern, &#other_pattern) => {
            #computations
            Ok(#result_value)
        }
    })
}

#[darling(attributes(animate), default)]
#[derive(Default, FromDeriveInput)]
struct AnimateInputAttrs {
    fallback: Option<Path>,
}

#[darling(attributes(animation), default)]
#[derive(Default, FromDeriveInput)]
pub struct AnimationInputAttrs {
    pub no_bound: Option<IdentList>,
}

#[darling(attributes(animation), default)]
#[derive(Default, FromVariant)]
pub struct AnimationVariantAttrs {
    pub error: bool,
    // Only here because of structs, where the struct definition acts as a
    // variant itself.
    pub no_bound: Option<IdentList>,
}

#[darling(attributes(animation), default)]
#[derive(Default, FromField)]
pub struct AnimationFieldAttrs {
    pub constant: bool,
}
