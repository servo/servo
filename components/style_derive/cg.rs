/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use darling::{FromField, FromVariant};
use quote::{ToTokens, Tokens};
use std::borrow::Cow;
use std::collections::HashSet;
use std::iter;
use syn::{self, AngleBracketedParameterData, Body, DeriveInput, Field, Ident};
use syn::{ImplGenerics, Path, PathParameters, PathSegment, PolyTraitRef};
use syn::{QSelf, TraitBoundModifier, Ty, TyGenerics, TyParam, TyParamBound};
use syn::{Variant, WhereBoundPredicate, WherePredicate};
use syn::visit::{self, Visitor};
use synstructure::{self, BindOpts, BindStyle, BindingInfo};

pub struct WhereClause<'input, 'path> {
    pub inner: syn::WhereClause,
    pub params: &'input [TyParam],
    trait_path: &'path [&'path str],
    bounded_types: HashSet<Ty>,
}

impl<'input, 'path> ToTokens for WhereClause<'input, 'path> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        self.inner.to_tokens(tokens);
    }
}

impl<'input, 'path> WhereClause<'input, 'path> {
    pub fn add_trait_bound(&mut self, ty: Ty) {
        if is_parameterized(&ty, self.params) && !self.bounded_types.contains(&ty) {
            self.bounded_types.insert(ty.clone());
            self.inner.predicates.push(where_predicate(ty, self.trait_path));
        }
    }
}

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

pub fn fmap_trait_parts<'input, 'path>(
    input: &'input DeriveInput,
    trait_path: &'path [&'path str],
    trait_output: &str,
) -> (ImplGenerics<'input>, TyGenerics<'input>, WhereClause<'input, 'path>, Path) {
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

pub fn parse_field_attrs<A>(field: &Field) -> A
where
    A: FromField,
{
    match A::from_field(field) {
        Ok(attrs) => attrs,
        Err(e) => panic!("failed to parse field attributes: {}", e),
    }
}

pub fn parse_variant_attrs<A>(variant: &Variant) -> A
where
    A: FromVariant,
{
    match A::from_variant(variant) {
        Ok(attrs) => attrs,
        Err(e) => panic!("failed to parse variant attributes: {}", e),
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

pub fn trait_parts<'input, 'path>(
    input: &'input DeriveInput,
    trait_path: &'path [&'path str],
) -> (ImplGenerics<'input>, TyGenerics<'input>, WhereClause<'input, 'path>) {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let where_clause = WhereClause {
        inner: where_clause.clone(),
        params: &input.generics.ty_params,
        trait_path,
        bounded_types: HashSet::new()
    };
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
