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
        where_clause.predicates.push(where_predicate(syn::Ty::Path(None, param.ident.clone().into())))
    }

    let style = synstructure::BindStyle::Ref.into();
    let match_body = synstructure::each_variant(&input, &style, |bindings, _| {
        if bindings.is_empty() {
            panic!("unit variants are not yet supported");
        }
        let (first, rest) = bindings.split_first().expect("unit variants are not yet supported");
        where_clause.predicates.push(where_predicate(first.field.ty.clone()));
        let mut expr = quote! {
            ::style_traits::ToCss::to_css(#first, dest)
        };
        for binding in rest {
            where_clause.predicates.push(where_predicate(binding.field.ty.clone()));
            expr = quote! {
                #expr?;
                dest.write_str(" ")?;
                ::style_traits::ToCss::to_css(#binding, dest)
            };
        }
        Some(expr)
    });

    quote! {
        impl #impl_generics ::style_traits::ToCss for #name #ty_generics #where_clause {
            #[allow(unused_variables, unused_imports)]
            #[inline]
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
            where
                W: ::std::fmt::Write
            {
                match *self {
                    #match_body
                }
            }
        }
    }
}

/// `#ty: ::style_traits::ToCss`
fn where_predicate(ty: syn::Ty) -> syn::WherePredicate {
    syn::WherePredicate::BoundPredicate(syn::WhereBoundPredicate {
        bound_lifetimes: vec![],
        bounded_ty: ty,
        bounds: vec![syn::TyParamBound::Trait(
            syn::PolyTraitRef {
                bound_lifetimes: vec![],
                trait_ref: syn::Path {
                    global: true,
                    segments: vec!["style_traits".into(), "ToCss".into()],
                },
            },
            syn::TraitBoundModifier::None
        )],
    })
}
