/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use darling::FromVariant;
use quote::Tokens;
use std::borrow::Cow;
use std::iter;
use syn::{AngleBracketedParameterData, Body, DeriveInput, Ident, ImplGenerics};
use syn::{Path, PathParameters, PathSegment, PolyTraitRef, QSelf};
use syn::{TraitBoundModifier, Ty, TyGenerics, TyParam, TyParamBound, TypeBinding};
use syn::{Variant, WhereBoundPredicate, WhereClause, WherePredicate};
use syn::visit::{self, Visitor};
use synstructure::{self, BindOpts, BindStyle, BindingInfo};

pub fn fmap_match<F>(
    input: &DeriveInput,
    bind_style: BindStyle,
    mut f: F,
) -> Tokens
where
    F: FnMut(BindingInfo) -> Tokens,
{
    synstructure::each_variant(input, &bind_style.into(), |fields, variant| {
        let name = variant_ctor(input, variant);
        let (mapped, mapped_fields) = value(&name, variant, "mapped");
        let fields_pairs = fields.into_iter().zip(mapped_fields);
        let mut computations = quote!();
        computations.append_all(fields_pairs.map(|(field, mapped_field)| {
            let expr = f(field);
            quote! { let #mapped_field = #expr; }
        }));
        computations.append(mapped);
        Some(computations)
    })
}

pub fn fmap_trait_parts<'a>(
    input: &'a DeriveInput,
    trait_path: &[&str],
    trait_output: &str,
) -> (ImplGenerics<'a>, TyGenerics<'a>, WhereClause, Path) {
    let (impl_generics, ty_generics, where_clause) = trait_parts(input, trait_path);
    let output_ty = PathSegment {
        ident: input.ident.clone(),
        parameters: PathParameters::AngleBracketed(AngleBracketedParameterData {
            lifetimes: input.generics.lifetimes.iter().map(|l| l.lifetime.clone()).collect(),
            types: input.generics.ty_params.iter().map(|ty| {
                Ty::Path(
                    Some(QSelf {
                        ty: Box::new(Ty::Path(None, ty.ident.clone().into())),
                        position: trait_path.len(),
                    }),
                    path(trait_path.iter().chain(iter::once(&trait_output))),
                )
            }).collect(),
            .. Default::default()
        }),
    }.into();
    (impl_generics, ty_generics, where_clause, output_ty)
}

fn fmap_trait_where_predicate(
    bounded_ty: Ty,
    trait_path: &[&str],
    trait_output: Option<(&str, Ty)>,
) -> WherePredicate {
    WherePredicate::BoundPredicate(WhereBoundPredicate {
        bound_lifetimes: vec![],
        bounded_ty,
        bounds: vec![TyParamBound::Trait(
            PolyTraitRef {
                bound_lifetimes: vec![],
                trait_ref: fmap_trait_ref(trait_path, trait_output),
            },
            TraitBoundModifier::None
        )],
    })
}

fn fmap_trait_ref(path: &[&str], output: Option<(&str, Ty)>) -> Path {
    let (name, parent) = path.split_last().unwrap();
    let last_segment = PathSegment {
        ident: (*name).into(),
        parameters: PathParameters::AngleBracketed(
            AngleBracketedParameterData {
                bindings: output.into_iter().map(|(param, ty)| {
                    TypeBinding { ident: param.into(), ty }
                }).collect(),
                .. Default::default()
            }
        )
    };
    Path {
        global: true,
        segments: {
            parent
                .iter()
                .cloned()
                .map(Into::into)
                .chain(iter::once(last_segment))
                .collect()
        },
    }
}

pub fn is_parameterized(ty: &Ty, params: &[TyParam]) -> bool {
    struct IsParameterized<'a> {
        params: &'a [TyParam],
        has_free: bool,
    }

    impl<'a> Visitor for IsParameterized<'a> {
        fn visit_path(&mut self, path: &Path) {
            if !path.global && path.segments.len() == 1 {
                if self.params.iter().any(|param| param.ident == path.segments[0].ident) {
                    self.has_free = true;
                }
            }
            visit::walk_path(self, path);
        }
    }

    let mut visitor = IsParameterized { params: params, has_free: false };
    visitor.visit_ty(ty);
    visitor.has_free
}

pub fn path<S>(segments: S) -> Path
where
    S: IntoIterator,
    <S as IntoIterator>::Item: AsRef<str>,
{
    Path {
        global: true,
        segments: segments.into_iter().map(|s| s.as_ref().into()).collect(),
    }
}

pub fn parse_variant_attrs<A>(variant: &Variant) -> A
where
    A: FromVariant,
{
    match A::from_variant(variant) {
        Ok(attrs) => attrs,
        Err(e) => panic!("failed to parse attributes: {}", e),
    }
}

pub fn ref_pattern<'a>(
    name: &Ident,
    variant: &'a Variant,
    prefix: &str,
) -> (Tokens, Vec<BindingInfo<'a>>) {
    synstructure::match_pattern(
        &name,
        &variant.data,
        &BindOpts::with_prefix(BindStyle::Ref, prefix.to_owned()),
    )
}

pub fn trait_parts<'a>(
    input: &'a DeriveInput,
    trait_path: &[&str],
) -> (ImplGenerics<'a>, TyGenerics<'a>, WhereClause) {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.clone();
    for param in &input.generics.ty_params {
        where_clause.predicates.push(fmap_trait_where_predicate(
            Ty::Path(None, param.ident.clone().into()),
            trait_path,
            None,
        ));
    }
    (impl_generics, ty_generics, where_clause)
}

pub fn value<'a>(
    name: &Ident,
    variant: &'a Variant,
    prefix: &str,
) -> (Tokens, Vec<BindingInfo<'a>>) {
    synstructure::match_pattern(
        &name,
        &variant.data,
        &BindOpts::with_prefix(BindStyle::Move, prefix.to_owned()),
    )
}

pub fn variant_ctor<'a>(
    input: &'a DeriveInput,
    variant: &Variant,
) -> Cow<'a, Ident> {
    match input.body {
        Body::Struct(_) => Cow::Borrowed(&input.ident),
        Body::Enum(_) => {
            Cow::Owned(Ident::from(
                format!("{}::{}", input.ident, variant.ident),
            ))
        },
    }
}

pub fn variants(input: &DeriveInput) -> Cow<[Variant]> {
    match input.body {
        Body::Enum(ref variants) => (&**variants).into(),
        Body::Struct(ref data) => {
            vec![Variant {
                ident: input.ident.clone(),
                attrs: input.attrs.clone(),
                data: data.clone(),
                discriminant: None,
            }].into()
        },
    }
}

pub fn where_predicate(ty: Ty, segments: &[&str]) -> WherePredicate {
    WherePredicate::BoundPredicate(WhereBoundPredicate {
        bound_lifetimes: vec![],
        bounded_ty: ty,
        bounds: vec![TyParamBound::Trait(
            PolyTraitRef {
                bound_lifetimes: vec![],
                trait_ref: path(segments),
            },
            TraitBoundModifier::None,
        )],
    })
}
