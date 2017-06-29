/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use quote;
use syn;
use synstructure;

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.clone();
    for param in &input.generics.ty_params {
        where_clause.predicates.push(where_predicate(syn::Ty::Path(None, param.ident.clone().into()), None));
    }

    let animated_value_type = syn::Path::from(syn::PathSegment {
        ident: name.clone(),
        parameters: syn::PathParameters::AngleBracketed(syn::AngleBracketedParameterData {
            lifetimes: input.generics.lifetimes.iter().map(|l| {
                l.lifetime.clone()
            }).collect(),
            types: input.generics.ty_params.iter().map(|ty| {
                syn::Ty::Path(
                    Some(syn::QSelf {
                        ty: Box::new(syn::Ty::Path(None, ty.ident.clone().into())),
                        position: 3,
                    }),
                    syn::Path {
                        global: true,
                        segments: vec![
                            "values".into(),
                            "animated".into(),
                            "ToAnimatedValue".into(),
                            "AnimatedValue".into(),
                        ],
                    },
                )
            }).collect(),
            .. Default::default()
        }),
    });

    let to_body = match_body(&input, |field| {
        quote!(::values::animated::ToAnimatedValue::to_animated_value(#field))
    });
    let from_body = match_body(&input, |field| {
        quote!(::values::animated::ToAnimatedValue::from_animated_value(#field))
    });

    quote! {
        impl #impl_generics ::values::animated::ToAnimatedValue for #name #ty_generics #where_clause {
            type AnimatedValue = #animated_value_type;

            #[allow(unused_variables)]
            #[inline]
            fn to_animated_value(self) -> Self::AnimatedValue {
                match self {
                    #to_body
                }
            }

            #[inline]
            fn from_animated_value(animated: Self::AnimatedValue) -> Self {
                match animated {
                    #from_body
                }
            }
        }
    }
}

fn match_body<F>(input: &syn::DeriveInput, f: F) -> quote::Tokens
where
    F: Fn(&synstructure::BindingInfo) -> quote::Tokens,
{
    let by_value = synstructure::BindStyle::Move.into();
    synstructure::each_variant(&input, &by_value, |fields, variant| {
        let name = if let syn::Body::Enum(_) = input.body {
            format!("{}::{}", input.ident, variant.ident).into()
        } else {
            variant.ident.clone()
        };
        let (animated_value, computed_fields) = synstructure::match_pattern(&name, &variant.data, &by_value);
        let fields_pairs = fields.iter().zip(computed_fields.iter());
        let mut computations = quote!();
        computations.append_all(fields_pairs.map(|(field, computed_field)| {
            let expr = f(field);
            quote!(let #computed_field = #expr;)
        }));
        Some(quote!(
            #computations
            #animated_value
        ))
    })
}

/// `#ty: ::values::animated::ToAnimatedValue<AnimatedValue = #animated_value,>`
fn where_predicate(ty: syn::Ty, animated_value: Option<syn::Ty>) -> syn::WherePredicate {
    syn::WherePredicate::BoundPredicate(syn::WhereBoundPredicate {
        bound_lifetimes: vec![],
        bounded_ty: ty,
        bounds: vec![syn::TyParamBound::Trait(
            syn::PolyTraitRef {
                bound_lifetimes: vec![],
                trait_ref: trait_ref(animated_value),
            },
            syn::TraitBoundModifier::None
        )],
    })
}

/// `::values::animated::ToAnimatedValue<AnimatedValue = #animated_value,>`
fn trait_ref(animated_value: Option<syn::Ty>) -> syn::Path {
    syn::Path {
        global: true,
        segments: vec![
            "values".into(),
            "animated".into(),
            syn::PathSegment {
                ident: "ToAnimatedValue".into(),
                parameters: syn::PathParameters::AngleBracketed(
                    syn::AngleBracketedParameterData {
                        bindings: trait_bindings(animated_value),
                        .. Default::default()
                    }
                ),
            }
        ],
    }
}

/// `AnimatedValue = #animated_value,`
fn trait_bindings(animated_value: Option<syn::Ty>) -> Vec<syn::TypeBinding> {
    animated_value.into_iter().map(|ty| {
        syn::TypeBinding {
            ident: "AnimatedValue".into(),
            ty: ty,
        }
    }).collect()
}
