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

    let computed_value_type = syn::Path::from(syn::PathSegment {
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
                            "computed".into(),
                            "ToComputedValue".into(),
                            "ComputedValue".into(),
                        ],
                    },
                )
            }).collect(),
            .. Default::default()
        }),
    });

    let to_body = match_body(&input, |field| {
        quote!(::values::computed::ToComputedValue::to_computed_value(#field, context))
    });
    let from_body = match_body(&input, |field| {
        quote!(::values::computed::ToComputedValue::from_computed_value(#field))
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

fn match_body<F>(input: &syn::DeriveInput, f: F) -> quote::Tokens
    where F: Fn(&synstructure::BindingInfo) -> quote::Tokens,
{
    let by_ref = synstructure::BindStyle::Ref.into();
    let by_value = synstructure::BindStyle::Move.into();

    synstructure::each_variant(&input, &by_ref, |fields, variant| {
        let name = if let syn::Body::Enum(_) = input.body {
            format!("{}::{}", input.ident, variant.ident).into()
        } else {
            variant.ident.clone()
        };
        let (computed_value, computed_fields) = synstructure::match_pattern(&name, &variant.data, &by_value);
        let fields_pairs = fields.iter().zip(computed_fields.iter());
        let mut computations = quote!();
        computations.append_all(fields_pairs.map(|(field, computed_field)| {
            let expr = f(field);
            quote!(let #computed_field = #expr;)
        }));
        Some(quote!(
            #computations
            #computed_value
        ))
    })
}

/// `#ty: ::values::computed::ToComputedValue<ComputedValue = #computed_value,>`
fn where_predicate(ty: syn::Ty, computed_value: Option<syn::Ty>) -> syn::WherePredicate {
    syn::WherePredicate::BoundPredicate(syn::WhereBoundPredicate {
        bound_lifetimes: vec![],
        bounded_ty: ty,
        bounds: vec![syn::TyParamBound::Trait(
            syn::PolyTraitRef {
                bound_lifetimes: vec![],
                trait_ref: trait_ref(computed_value),
            },
            syn::TraitBoundModifier::None
        )],
    })
}

/// `::values::computed::ToComputedValue<ComputedValue = #computed_value,>`
fn trait_ref(computed_value: Option<syn::Ty>) -> syn::Path {
    syn::Path {
        global: true,
        segments: vec![
            "values".into(),
            "computed".into(),
            syn::PathSegment {
                ident: "ToComputedValue".into(),
                parameters: syn::PathParameters::AngleBracketed(
                    syn::AngleBracketedParameterData {
                        bindings: trait_bindings(computed_value),
                        .. Default::default()
                    }
                ),
            }
        ],
    }
}

/// `ComputedValue = #computed_value,`
fn trait_bindings(computed_value: Option<syn::Ty>) -> Vec<syn::TypeBinding> {
    computed_value.into_iter().map(|ty| {
        syn::TypeBinding {
            ident: "ComputedValue".into(),
            ty: ty,
        }
    }).collect()
}
