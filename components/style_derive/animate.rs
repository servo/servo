/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg::{self, WhereClause};
use quote::Tokens;
use syn::{DeriveInput, Path};
use synstructure::{Structure, VariantInfo};

pub fn derive(input: DeriveInput) -> Tokens {
    let name = &input.ident;
    let trait_path = parse_quote!(values::animated::Animate);
    let (impl_generics, ty_generics, mut where_clause) =
        cg::trait_parts(&input, &trait_path);

    let input_attrs = cg::parse_input_attrs::<AnimateInputAttrs>(&input);
    let s = Structure::new(&input);
    let mut append_error_clause = s.variants().len() > 1;

    let mut match_body = s.variants().iter().fold(quote!(), |body, variant| {
        let arm = match derive_variant_arm(variant, &mut where_clause) {
            Ok(arm) => arm,
            Err(()) => {
                append_error_clause = true;
                return body;
            }
        };
        quote! { #body #arm }
    });

    if append_error_clause {
        if let Some(fallback) = input_attrs.fallback {
            match_body.append_all(quote! {
                (this, other) => #fallback(this, other, procedure)
            });
        } else {
            match_body.append_all(quote! { _ => Err(()) });
        }
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

fn derive_variant_arm(
    variant: &VariantInfo,
    where_clause: &mut WhereClause,
) -> Result<Tokens, ()> {
    let variant_attrs = cg::parse_variant_attrs::<AnimationVariantAttrs>(&variant.ast());
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
            where_clause.add_trait_bound(&result.ast().ty);
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
#[derive(Default, FromVariant)]
pub struct AnimationVariantAttrs {
    pub error: bool,
}

#[darling(attributes(animation), default)]
#[derive(Default, FromField)]
pub struct AnimationFieldAttrs {
    pub constant: bool,
}
