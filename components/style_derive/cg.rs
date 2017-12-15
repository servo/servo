/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use darling::{FromDeriveInput, FromField, FromVariant};
use quote::{ToTokens, Tokens};
use std::borrow::Cow;
use std::collections::HashSet;
use std::iter;
use syn::{self, AngleBracketedParameterData, Body, DeriveInput, Field, Ident};
use syn::{ImplGenerics, Path, PathParameters, PathSegment, PolyTraitRef};
use syn::{QSelf, TraitBoundModifier, Ty, TyGenerics, TyParam, TyParamBound};
use syn::{TypeBinding, Variant, WhereBoundPredicate, WherePredicate};
use syn::visit::{self, Visitor};
use synstructure::{self, BindOpts, BindStyle, BindingInfo};

pub struct WhereClause<'input, 'path> {
    pub inner: syn::WhereClause,
    pub params: &'input [TyParam],
    trait_path: &'path [&'path str],
    trait_output: Option<&'path str>,
    bounded_types: HashSet<Ty>,
}

impl<'input, 'path> ToTokens for WhereClause<'input, 'path> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        self.inner.to_tokens(tokens);
    }
}

impl<'input, 'path> WhereClause<'input, 'path> {
    pub fn add_trait_bound(&mut self, ty: &Ty) {
        let trait_path = self.trait_path;
        let params = self.params;
        let mut found = self.trait_output.map(|_| HashSet::new());
        if self.bounded_types.contains(&ty) {
            return;
        }
        if !is_parameterized(&ty, params, found.as_mut()) {
            return;
        }
        self.bounded_types.insert(ty.clone());

        let output = if let Some(output) = self.trait_output {
            output
        } else {
            self.inner.predicates.push(where_predicate(ty.clone(), trait_path, None));
            return;
        };

        if let Ty::Path(None, ref path) = *ty {
            if path_to_ident(path).is_some() {
                self.inner.predicates.push(where_predicate(ty.clone(), trait_path, None));
                return;
            }
        }

        let output_type = map_type_params(ty, params, &mut |ident| {
            let ty = Ty::Path(None, ident.clone().into());
            fmap_output_type(ty, trait_path, output)
        });

        let pred = where_predicate(
            ty.clone(),
            trait_path,
            Some((output, output_type)),
        );

        self.inner.predicates.push(pred);

        if let Some(found) = found {
            for ident in found {
                let ty = Ty::Path(None, ident.into());
                if !self.bounded_types.contains(&ty) {
                    self.bounded_types.insert(ty.clone());
                    self.inner.predicates.push(
                        where_predicate(ty, trait_path, None),
                    );
                };
            }
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

fn fmap_output_type(
    ty: Ty,
    trait_path: &[&str],
    trait_output: &str,
) -> Ty {
    Ty::Path(
        Some(QSelf {
            ty: Box::new(ty),
            position: trait_path.len(),
        }),
        path(trait_path.iter().chain(iter::once(&trait_output))),
    )
}

pub fn fmap_trait_parts<'input, 'path>(
    input: &'input DeriveInput,
    trait_path: &'path [&'path str],
    trait_output: &'path str,
) -> (ImplGenerics<'input>, TyGenerics<'input>, WhereClause<'input, 'path>, Path) {
    let (impl_generics, ty_generics, mut where_clause) = trait_parts(input, trait_path);
    where_clause.trait_output = Some(trait_output);
    let output_ty = PathSegment {
        ident: input.ident.clone(),
        parameters: PathParameters::AngleBracketed(AngleBracketedParameterData {
            lifetimes: input.generics.lifetimes.iter().map(|l| l.lifetime.clone()).collect(),
            types: input.generics.ty_params.iter().map(|ty| {
                fmap_output_type(
                    Ty::Path(None, ty.ident.clone().into()),
                    trait_path,
                    trait_output,
                )
            }).collect(),
            .. Default::default()
        }),
    }.into();
    (impl_generics, ty_generics, where_clause, output_ty)
}

pub fn is_parameterized(
    ty: &Ty,
    params: &[TyParam],
    found: Option<&mut HashSet<Ident>>,
) -> bool {
    struct IsParameterized<'a, 'b> {
        params: &'a [TyParam],
        has_free: bool,
        found: Option<&'b mut HashSet<Ident>>,
    }

    impl<'a, 'b> Visitor for IsParameterized<'a, 'b> {
        fn visit_path(&mut self, path: &Path) {
            if let Some(ident) = path_to_ident(path) {
                if self.params.iter().any(|param| param.ident == ident) {
                    self.has_free = true;
                    if let Some(ref mut found) = self.found {
                        found.insert(ident.clone());
                    }
                }
            }
            visit::walk_path(self, path);
        }
    }

    let mut visitor = IsParameterized { params, has_free: false, found };
    visitor.visit_ty(ty);
    visitor.has_free
}

pub fn map_type_params<F>(ty: &Ty, params: &[TyParam], f: &mut F) -> Ty
where
    F: FnMut(&Ident) -> Ty,
{
    match *ty {
        Ty::Slice(ref ty) => Ty::Slice(Box::new(map_type_params(ty, params, f))),
        Ty::Array(ref ty, ref expr) => {
            Ty::Array(Box::new(map_type_params(ty, params, f)), expr.clone())
        },
        Ty::Never => Ty::Never,
        Ty::Tup(ref items) => {
            Ty::Tup(items.iter().map(|ty| map_type_params(ty, params, f)).collect())
        },
        Ty::Path(None, ref path) => {
            if let Some(ident) = path_to_ident(path) {
                if params.iter().any(|param| param.ident == ident) {
                    return f(ident);
                }
            }
            Ty::Path(None, map_type_params_in_path(path, params, f))
        }
        Ty::Path(ref qself, ref path) => {
            Ty::Path(
                qself.as_ref().map(|qself| {
                    QSelf {
                        ty: Box::new(map_type_params(&qself.ty, params, f)),
                        position: qself.position,
                    }
                }),
                map_type_params_in_path(path, params, f),
            )
        },
        Ty::Paren(ref ty) => Ty::Paren(Box::new(map_type_params(ty, params, f))),
        ref ty => panic!("type {:?} cannot be mapped yet", ty),
    }
}

fn map_type_params_in_path<F>(path: &Path, params: &[TyParam], f: &mut F) -> Path
where
    F: FnMut(&Ident) -> Ty,
{
    Path {
        global: path.global,
        segments: path.segments.iter().map(|segment| {
            PathSegment {
                ident: segment.ident.clone(),
                parameters: match segment.parameters {
                    PathParameters::AngleBracketed(ref data) => {
                        PathParameters::AngleBracketed(AngleBracketedParameterData {
                            lifetimes: data.lifetimes.clone(),
                            types: data.types.iter().map(|ty| {
                                map_type_params(ty, params, f)
                            }).collect(),
                            bindings: data.bindings.iter().map(|binding| {
                                TypeBinding {
                                    ident: binding.ident.clone(),
                                    ty: map_type_params(&binding.ty, params, f),
                                }
                            }).collect(),
                        })
                    },
                    ref parameters => {
                        panic!("parameters {:?} cannot be mapped yet", parameters)
                    },
                },
            }
        }).collect(),
    }
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

fn path_to_ident(path: &Path) -> Option<&Ident> {
    match *path {
        Path { global: false, ref segments } if segments.len() == 1 => {
            if segments[0].parameters.is_empty() {
                Some(&segments[0].ident)
            } else {
                None
            }
        },
        _ => None,
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

pub fn parse_input_attrs<A>(input: &DeriveInput) -> A
where
    A: FromDeriveInput,
{
    match A::from_derive_input(input) {
        Ok(attrs) => attrs,
        Err(e) => panic!("failed to parse input attributes: {}", e),
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
        trait_output: None,
        bounded_types: HashSet::new()
    };
    (impl_generics, ty_generics, where_clause)
}

fn trait_ref(path: &[&str], output: Option<(&str, Ty)>) -> Path {
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

pub fn where_predicate(
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
                trait_ref: trait_ref(trait_path, trait_output),
            },
            TraitBoundModifier::None
        )],
    })
}

/// Transforms "FooBar" to "foo-bar".
///
/// If the first Camel segment is "Moz", "Webkit", or "Servo", the result string
/// is prepended with "-".
pub fn to_css_identifier(mut camel_case: &str) -> String {
    camel_case = camel_case.trim_right_matches('_');
    let mut first = true;
    let mut result = String::with_capacity(camel_case.len());
    while let Some(segment) = split_camel_segment(&mut camel_case) {
        if first {
            match segment {
                "Moz" | "Webkit" | "Servo" => first = false,
                _ => {},
            }
        }
        if !first {
            result.push_str("-");
        }
        first = false;
        result.push_str(&segment.to_lowercase());
    }
    result
}

/// Given "FooBar", returns "Foo" and sets `camel_case` to "Bar".
fn split_camel_segment<'input>(camel_case: &mut &'input str) -> Option<&'input str> {
    let index = match camel_case.chars().next() {
        None => return None,
        Some(ch) => ch.len_utf8(),
    };
    let end_position = camel_case[index..]
        .find(char::is_uppercase)
        .map_or(camel_case.len(), |pos| index + pos);
    let result = &camel_case[..end_position];
    *camel_case = &camel_case[end_position..];
    Some(result)
}
