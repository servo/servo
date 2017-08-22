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
        where_clause.predicates.push(
            where_predicate(syn::Ty::Path(None, param.ident.clone().into())),
        );
    }

    let to_body = match_body(&input);

    quote! {
        impl #impl_generics ::values::animated::ToAnimatedZero for #name #ty_generics #where_clause {
            #[allow(unused_variables)]
            #[inline]
            fn to_animated_zero(&self) -> Result<Self, ()> {
                match *self {
                    #to_body
                }
            }
        }
    }
}

fn match_body(input: &syn::DeriveInput) -> quote::Tokens {
    synstructure::each_variant(&input, &synstructure::BindStyle::Ref.into(), |fields, variant| {
        let name = if let syn::Body::Enum(_) = input.body {
            format!("{}::{}", input.ident, variant.ident).into()
        } else {
            variant.ident.clone()
        };
        let (zero, computed_fields) = synstructure::match_pattern(
            &name,
            &variant.data,
            &synstructure::BindStyle::Move.into(),
        );
        let fields_pairs = fields.iter().zip(computed_fields.iter());
        let mut computations = quote!();
        computations.append_all(fields_pairs.map(|(field, computed_field)| {
            quote! {
                let #computed_field = ::values::animated::ToAnimatedZero::to_animated_zero(#field)?;
            }
        }));
        Some(quote!(
            #computations
            Ok(#zero)
        ))
    })
}

fn where_predicate(ty: syn::Ty) -> syn::WherePredicate {
    syn::WherePredicate::BoundPredicate(syn::WhereBoundPredicate {
        bound_lifetimes: vec![],
        bounded_ty: ty,
        bounds: vec![syn::TyParamBound::Trait(
            syn::PolyTraitRef {
                bound_lifetimes: vec![],
                trait_ref: syn::Path {
                    global: true,
                    segments: vec![
                        "values".into(),
                        "animated".into(),
                        "ToAnimatedZero".into(),
                    ],
                },
            },
            syn::TraitBoundModifier::None,
        )],
    })
}
