/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use darling::{FromDeriveInput, FromField, FromVariant};
use quote::{ToTokens, Tokens};
use std::collections::HashSet;
use syn::{self, AngleBracketedGenericArguments, Binding, DeriveInput, Field};
use syn::{GenericArgument, GenericParam, Ident, ImplGenerics, Path};
use syn::{PathArguments, PathSegment, QSelf, Type, TypeArray, TypeGenerics};
use syn::{TypeParam, TypeParen, TypePath, TypeSlice, TypeTuple};
use syn::{Variant, WherePredicate};
use syn::visit::{self, Visit};
use synstructure::{self, BindingInfo, BindStyle, VariantAst, VariantInfo};

pub struct WhereClause<'input, 'path> {
    pub inner: Option<syn::WhereClause>,
    pub params: Vec<&'input TypeParam>,
    trait_path: &'path Path,
    trait_output: Option<Ident>,
    bounded_types: HashSet<Type>,
}

impl<'input, 'path> ToTokens for WhereClause<'input, 'path> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        self.inner.to_tokens(tokens);
    }
}

impl<'input, 'path> WhereClause<'input, 'path> {
    pub fn add_trait_bound(&mut self, ty: &Type) {
        let trait_path = self.trait_path;
        let mut found = self.trait_output.map(|_| HashSet::new());
        if self.bounded_types.contains(&ty) {
            return;
        }
        if !is_parameterized(&ty, &self.params, found.as_mut()) {
            return;
        }
        self.bounded_types.insert(ty.clone());

        let output = if let Some(output) = self.trait_output {
            output
        } else {
            add_predicate(&mut self.inner, where_predicate(ty.clone(), trait_path, None));
            return;
        };

        if let Type::Path(syn::TypePath { ref path, .. }) = *ty {
            if path_to_ident(path).is_some() {
                add_predicate(&mut self.inner, where_predicate(ty.clone(), trait_path, None));
                return;
            }
        }

        let output_type = map_type_params(ty, &self.params, &mut |ident| {
            parse_quote!(<#ident as ::#trait_path>::#output)
        });

        let pred = where_predicate(
            ty.clone(),
            trait_path,
            Some((output, output_type)),
        );

        add_predicate(&mut self.inner, pred);

        if let Some(found) = found {
            for ident in found {
                let ty = Type::Path(syn::TypePath { qself: None, path: ident.into() });
                if !self.bounded_types.contains(&ty) {
                    self.bounded_types.insert(ty.clone());
                    add_predicate(
                        &mut self.inner,
                        where_predicate(ty, trait_path, None),
                    );
                };
            }
        }
    }
}

pub fn add_predicate(
    where_clause: &mut Option<syn::WhereClause>,
    pred: WherePredicate,
) {
    where_clause.get_or_insert(parse_quote!(where)).predicates.push(pred);
}

pub fn fmap_match<F>(
    input: &DeriveInput,
    bind_style: BindStyle,
    mut f: F,
) -> Tokens
where
    F: FnMut(BindingInfo) -> Tokens,
{
    let mut s = synstructure::Structure::new(input);
    s.variants_mut().iter_mut().for_each(|v| { v.bind_with(|_| bind_style); });
    s.each_variant(|variant| {
        let (mapped, mapped_fields) = value(variant, "mapped");
        let fields_pairs = variant.bindings().into_iter().zip(mapped_fields);
        let mut computations = quote!();
        computations.append_all(fields_pairs.map(|(field, mapped_field)| {
            let expr = f(field.clone());
            quote! { let #mapped_field = #expr; }
        }));
        computations.append_all(mapped);
        Some(computations)
    })
}

pub fn fmap_trait_output(
    input: &DeriveInput,
    trait_path: &Path,
    trait_output: Ident,
) -> Path {
    let segment = PathSegment {
        ident: input.ident.clone(),
        arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            args: input.generics.params.iter().map(|arg| {
                match arg {
                    &GenericParam::Lifetime(ref data) => GenericArgument::Lifetime(data.lifetime.clone()),
                    &GenericParam::Type(ref data) => {
                        let ident = data.ident;
                        GenericArgument::Type(
                            parse_quote!(<#ident as ::#trait_path>::#trait_output),
                        )
                    },
                    ref arg => panic!("arguments {:?} cannot be mapped yet", arg)
                }
            }).collect(),
            colon2_token: Default::default(),
            gt_token: Default::default(),
            lt_token: Default::default(),

        })
    };
    segment.into()
}

pub fn is_parameterized(
    ty: &Type,
    params: &[&TypeParam],
    found: Option<&mut HashSet<Ident>>,
) -> bool {
    struct IsParameterized<'a, 'b> {
        params: &'a [&'a TypeParam],
        has_free: bool,
        found: Option<&'b mut HashSet<Ident>>,
    }

    impl<'a, 'b, 'ast> Visit<'ast> for IsParameterized<'a, 'b> {
        fn visit_path(&mut self, path: &'ast Path) {
            if let Some(ident) = path_to_ident(path) {
                if self.params.iter().any(|param| param.ident == ident) {
                    self.has_free = true;
                    if let Some(ref mut found) = self.found {
                        found.insert(ident.clone());
                    }
                }
            }
            visit::visit_path(self, path);
        }
    }

    let mut visitor = IsParameterized { params, has_free: false, found };
    visitor.visit_type(ty);
    visitor.has_free
}

pub fn map_type_params<F>(ty: &Type, params: &[&TypeParam], f: &mut F) -> Type
where
    F: FnMut(&Ident) -> Type,
{
    match *ty {
        Type::Slice(ref inner) => {
            Type::from(TypeSlice { elem: Box::new(map_type_params(&inner.elem, params, f)), ..inner.clone() })
        },
        Type::Array(ref inner) => { //ref ty, ref expr) => {
            Type::from(TypeArray { elem: Box::new(map_type_params(&inner.elem, params, f)), ..inner.clone() })
        },
        ref ty @ Type::Never(_) => ty.clone(),
        Type::Tuple(ref inner) => {
            Type::from(
                TypeTuple {
                    elems: inner.elems.iter().map(|ty| map_type_params(&ty, params, f)).collect(),
                    ..inner.clone()
                }
            )
        },
        Type::Path(TypePath { qself: None, ref path }) => {
            if let Some(ident) = path_to_ident(path) {
                if params.iter().any(|param| param.ident == ident) {
                    return f(ident);
                }
            }
            Type::from(TypePath { qself: None, path: map_type_params_in_path(path, params, f) })
        }
        Type::Path(TypePath { ref qself, ref path }) => {
            Type::from(TypePath {
                qself: qself.as_ref().map(|qself| {
                    QSelf {
                        ty: Box::new(map_type_params(&qself.ty, params, f)),
                        position: qself.position,
                        ..qself.clone()
                    }
                }),
                path: map_type_params_in_path(path, params, f),
            })
        },
        Type::Paren(ref inner) => {
            Type::from(TypeParen { elem: Box::new(map_type_params(&inner.elem, params, f)), ..inner.clone() })
        },
        ref ty => panic!("type {:?} cannot be mapped yet", ty),
    }
}

fn map_type_params_in_path<F>(path: &Path, params: &[&TypeParam], f: &mut F) -> Path
where
    F: FnMut(&Ident) -> Type,
{
    Path {
        leading_colon: path.leading_colon,
        segments: path.segments.iter().map(|segment| {
            PathSegment {
                ident: segment.ident.clone(),
                arguments: match segment.arguments {
                    PathArguments::AngleBracketed(ref data) => {
                        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            args: data.args.iter().map(|arg| {
                                match arg {
                                    ty @ &GenericArgument::Lifetime(_) => ty.clone(),
                                    &GenericArgument::Type(ref data) => {
                                        GenericArgument::Type(map_type_params(data, params, f))
                                    },
                                    &GenericArgument::Binding(ref data) => GenericArgument::Binding(Binding {
                                        ty: map_type_params(&data.ty, params, f),
                                        ..data.clone()
                                    }),
                                    ref arg => panic!("arguments {:?} cannot be mapped yet", arg)
                                }
                            }).collect(),
                            ..data.clone()
                        })
                    },
                    ref arg @ PathArguments::None => arg.clone(),
                    ref parameters => {
                        panic!("parameters {:?} cannot be mapped yet", parameters)
                    }
                },
            }
        }).collect(),
    }
}

fn path_to_ident(path: &Path) -> Option<&Ident> {
    match *path {
        Path { leading_colon: None, ref segments } if segments.len() == 1 => {
            if segments[0].arguments.is_empty() {
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

pub fn parse_variant_attrs<A>(variant: &VariantAst) -> A
where
    A: FromVariant,
{
    let v = Variant {
        ident: *variant.ident,
        attrs: variant.attrs.to_vec(),
        fields: variant.fields.clone(),
        discriminant: variant.discriminant.clone(),
    };
    match A::from_variant(&v) {
        Ok(attrs) => attrs,
        Err(e) => panic!("failed to parse variant attributes: {}", e),
    }
}


pub fn ref_pattern<'a>(
    variant: &'a VariantInfo,
    prefix: &str,
) -> (Tokens, Vec<BindingInfo<'a>>) {
    let mut v = variant.clone();
    v.bind_with(|_| BindStyle::Ref);
    v.bindings_mut().iter_mut().for_each(|b| { b.binding = Ident::from(format!("{}_{}", b.binding, prefix)) });
    (v.pat(), v.bindings().iter().cloned().collect())
}

pub fn trait_parts<'input, 'path>(
    input: &'input DeriveInput,
    trait_path: &'path Path,
) -> (ImplGenerics<'input>, TypeGenerics<'input>, WhereClause<'input, 'path>) {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let where_clause = WhereClause {
        inner: where_clause.cloned(),
        params: input.generics.type_params().into_iter().collect::<Vec<&TypeParam>>(),
        trait_path,
        trait_output: None,
        bounded_types: HashSet::new()
    };
    (impl_generics, ty_generics, where_clause)
}

fn trait_ref(path: &Path, output: Option<(Ident, Type)>) -> Path {
    let segments = path.segments.iter().collect::<Vec<&PathSegment>>();
    let (name, parent) = segments.split_last().unwrap();

    let last_segment: PathSegment = if let Some((param, ty)) = output {
        parse_quote!(#name<#param = #ty>)
    } else {
        parse_quote!(#name)
    };
    parse_quote!(::#(#parent::)*#last_segment)
}

pub fn value<'a>(
    variant: &'a VariantInfo,
    prefix: &str,
) -> (Tokens, Vec<BindingInfo<'a>>) {
    let mut v = variant.clone();
    v.bindings_mut().iter_mut().for_each(|b| { b.binding = Ident::from(format!("{}_{}", b.binding, prefix)) });
    v.bind_with(|_| BindStyle::Move);
    (v.pat(), v.bindings().iter().cloned().collect())
}

pub fn where_predicate(
    bounded_ty: Type,
    trait_path: &Path,
    trait_output: Option<(Ident, Type)>,
) -> WherePredicate {
    let trait_ref = trait_ref(trait_path, trait_output);
    parse_quote!(#bounded_ty: #trait_ref)
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
