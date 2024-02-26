/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::animate::{AnimationFieldAttrs, AnimationInputAttrs, AnimationVariantAttrs};
use derive_common::cg;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use syn::{DeriveInput, WhereClause};
use synstructure;

pub fn derive(mut input: DeriveInput) -> TokenStream {
    let animation_input_attrs = cg::parse_input_attrs::<AnimationInputAttrs>(&input);
    let no_bound = animation_input_attrs.no_bound.unwrap_or_default();
    let mut where_clause = input.generics.where_clause.take();
    for param in input.generics.type_params() {
        if !no_bound.iter().any(|name| name.is_ident(&param.ident)) {
            cg::add_predicate(
                &mut where_clause,
                parse_quote!(#param: crate::values::distance::ComputeSquaredDistance),
            );
        }
    }

    let (mut match_body, needs_catchall_branch) = {
        let s = synstructure::Structure::new(&input);
        let needs_catchall_branch = s.variants().len() > 1;

        let match_body = s.variants().iter().fold(quote!(), |body, variant| {
            let arm = derive_variant_arm(variant, &mut where_clause);
            quote! { #body #arm }
        });

        (match_body, needs_catchall_branch)
    };

    input.generics.where_clause = where_clause;

    if needs_catchall_branch {
        // This ideally shouldn't be needed, but see:
        // https://github.com/rust-lang/rust/issues/68867
        match_body.append_all(quote! { _ => unsafe { debug_unreachable!() } });
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics crate::values::distance::ComputeSquaredDistance for #name #ty_generics #where_clause {
            #[allow(unused_variables, unused_imports)]
            #[inline]
            fn compute_squared_distance(
                &self,
                other: &Self,
            ) -> Result<crate::values::distance::SquaredDistance, ()> {
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
    variant: &synstructure::VariantInfo,
    mut where_clause: &mut Option<WhereClause>,
) -> TokenStream {
    let variant_attrs = cg::parse_variant_attrs_from_ast::<AnimationVariantAttrs>(&variant.ast());
    let (this_pattern, this_info) = cg::ref_pattern(&variant, "this");
    let (other_pattern, other_info) = cg::ref_pattern(&variant, "other");

    if variant_attrs.error {
        return quote! {
            (&#this_pattern, &#other_pattern) => Err(()),
        };
    }

    let sum = if this_info.is_empty() {
        quote! { crate::values::distance::SquaredDistance::from_sqrt(0.) }
    } else {
        let mut sum = quote!();
        sum.append_separated(this_info.iter().zip(&other_info).map(|(this, other)| {
            let field_attrs = cg::parse_field_attrs::<DistanceFieldAttrs>(&this.ast());
            if field_attrs.field_bound {
                let ty = &this.ast().ty;
                cg::add_predicate(
                    &mut where_clause,
                    parse_quote!(#ty: crate::values::distance::ComputeSquaredDistance),
                );
            }

            let animation_field_attrs =
                cg::parse_field_attrs::<AnimationFieldAttrs>(&this.ast());

            if animation_field_attrs.constant {
                quote! {
                    {
                        if #this != #other {
                            return Err(());
                        }
                        crate::values::distance::SquaredDistance::from_sqrt(0.)
                    }
                }
            } else {
                quote! {
                    crate::values::distance::ComputeSquaredDistance::compute_squared_distance(#this, #other)?
                }
            }
        }), quote!(+));
        sum
    };

    return quote! {
        (&#this_pattern, &#other_pattern) => Ok(#sum),
    };
}

#[derive(Default, FromField)]
#[darling(attributes(distance), default)]
struct DistanceFieldAttrs {
    field_bound: bool,
}
