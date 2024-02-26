/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use darling::util::PathList;
use derive_common::cg;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use syn::{DeriveInput, WhereClause};
use synstructure::{Structure, VariantInfo};

pub fn derive(mut input: DeriveInput) -> TokenStream {
    let animation_input_attrs = cg::parse_input_attrs::<AnimationInputAttrs>(&input);

    let no_bound = animation_input_attrs.no_bound.unwrap_or_default();
    let mut where_clause = input.generics.where_clause.take();
    for param in input.generics.type_params() {
        if !no_bound.iter().any(|name| name.is_ident(&param.ident)) {
            cg::add_predicate(
                &mut where_clause,
                parse_quote!(#param: crate::values::animated::Animate),
            );
        }
    }
    let (mut match_body, needs_catchall_branch) = {
        let s = Structure::new(&input);
        let needs_catchall_branch = s.variants().len() > 1;
        let match_body = s.variants().iter().fold(quote!(), |body, variant| {
            let arm = derive_variant_arm(variant, &mut where_clause);
            quote! { #body #arm }
        });
        (match_body, needs_catchall_branch)
    };

    input.generics.where_clause = where_clause;

    if needs_catchall_branch {
        // This ideally shouldn't be needed, but see
        // https://github.com/rust-lang/rust/issues/68867
        match_body.append_all(quote! { _ => unsafe { debug_unreachable!() } });
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics crate::values::animated::Animate for #name #ty_generics #where_clause {
            #[allow(unused_variables, unused_imports)]
            #[inline]
            fn animate(
                &self,
                other: &Self,
                procedure: crate::values::animated::Procedure,
            ) -> Result<Self, ()> {
                if std::mem::discriminant(self) != std::mem::discriminant(other) {
                    return Err(());
                }
                match (self, other) {
                    #match_body
                }
            }
        }
    }
}

fn derive_variant_arm(
    variant: &VariantInfo,
    where_clause: &mut Option<WhereClause>,
) -> TokenStream {
    let variant_attrs = cg::parse_variant_attrs_from_ast::<AnimationVariantAttrs>(&variant.ast());
    let (this_pattern, this_info) = cg::ref_pattern(&variant, "this");
    let (other_pattern, other_info) = cg::ref_pattern(&variant, "other");

    if variant_attrs.error {
        return quote! {
            (&#this_pattern, &#other_pattern) => Err(()),
        };
    }

    let (result_value, result_info) = cg::value(&variant, "result");
    let mut computations = quote!();
    let iter = result_info.iter().zip(this_info.iter().zip(&other_info));
    computations.append_all(iter.map(|(result, (this, other))| {
        let field_attrs = cg::parse_field_attrs::<AnimationFieldAttrs>(&result.ast());
        if field_attrs.field_bound {
            let ty = &this.ast().ty;
            cg::add_predicate(
                where_clause,
                parse_quote!(#ty: crate::values::animated::Animate),
            );
        }
        if field_attrs.constant {
            quote! {
                if #this != #other {
                    return Err(());
                }
                let #result = std::clone::Clone::clone(#this);
            }
        } else {
            quote! {
                let #result =
                    crate::values::animated::Animate::animate(#this, #other, procedure)?;
            }
        }
    }));

    quote! {
        (&#this_pattern, &#other_pattern) => {
            #computations
            Ok(#result_value)
        }
    }
}

#[derive(Default, FromDeriveInput)]
#[darling(attributes(animation), default)]
pub struct AnimationInputAttrs {
    pub no_bound: Option<PathList>,
}

#[derive(Default, FromVariant)]
#[darling(attributes(animation), default)]
pub struct AnimationVariantAttrs {
    pub error: bool,
    // Only here because of structs, where the struct definition acts as a
    // variant itself.
    pub no_bound: Option<PathList>,
}

#[derive(Default, FromField)]
#[darling(attributes(animation), default)]
pub struct AnimationFieldAttrs {
    pub constant: bool,
    pub field_bound: bool,
}
