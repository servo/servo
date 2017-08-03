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
    let match_body = synstructure::each_variant(&input, &style, |bindings, variant| {
        let mut identifier = to_css_identifier(variant.ident.as_ref());
        let mut css_attrs = variant.attrs.iter().filter(|attr| attr.name() == "css");
        let (is_function, use_comma) = css_attrs.next().map_or((false, false), |attr| {
            match attr.value {
                syn::MetaItem::List(ref ident, ref items) if ident.as_ref() == "css" => {
                    let mut nested = items.iter();
                    let mut is_function = false;
                    let mut use_comma = false;
                    for attr in nested.by_ref() {
                        match *attr {
                            syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref ident)) => {
                                match ident.as_ref() {
                                    "function" => {
                                        if is_function {
                                            panic!("repeated `#[css(function)]` attribute");
                                        }
                                        is_function = true;
                                    },
                                    "comma" => {
                                        if use_comma {
                                            panic!("repeated `#[css(comma)]` attribute");
                                        }
                                        use_comma = true;
                                    },
                                    _ => panic!("only `#[css(function | comma)]` is supported for now"),
                                }
                            },
                            _ => panic!("only `#[css(<ident...>)]` is supported for now"),
                        }
                    }
                    if nested.next().is_some() {
                        panic!("only `#[css()]` or `#[css(<ident>)]` is supported for now")
                    }
                    (is_function, use_comma)
                },
                _ => panic!("only `#[css(...)]` is supported for now"),
            }
        });
        if css_attrs.next().is_some() {
            panic!("only a single `#[css(...)]` attribute is supported for now");
        }
        let separator = if use_comma { ", " } else { " " };
        let mut expr = if !bindings.is_empty() {
            let mut expr = quote! {};
            for binding in bindings {
                if has_free_params(&binding.field.ty, &input.generics.ty_params) {
                    where_clause.predicates.push(where_predicate(binding.field.ty.clone()));
                }
                expr = quote! {
                    #expr
                    writer.item(#binding)?;
                };
            }
            quote! {{
                let mut writer = ::style_traits::values::SequenceWriter::new(&mut *dest, #separator);
                #expr
                Ok(())
            }}
        } else {
            quote! {
                ::std::fmt::Write::write_str(dest, #identifier)
            }
        };
        if is_function {
            identifier.push_str("(");
            expr = quote! {
                ::std::fmt::Write::write_str(dest, #identifier)?;
                #expr?;
                ::std::fmt::Write::write_str(dest, ")")
            }
        }
        Some(expr)
    });

    quote! {
        impl #impl_generics ::style_traits::ToCss for #name #ty_generics #where_clause {
            #[allow(unused_variables)]
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

/// Returns whether `ty` is parameterized by any parameter from `params`.
fn has_free_params(ty: &syn::Ty, params: &[syn::TyParam]) -> bool {
    use syn::visit::Visitor;

    struct HasFreeParams<'a> {
        params: &'a [syn::TyParam],
        has_free: bool,
    }

    impl<'a> Visitor for HasFreeParams<'a> {
        fn visit_path(&mut self, path: &syn::Path) {
            if !path.global && path.segments.len() == 1 {
                if self.params.iter().any(|param| param.ident == path.segments[0].ident) {
                    self.has_free = true;
                }
            }
            syn::visit::walk_path(self, path);
        }
    }

    let mut visitor = HasFreeParams { params: params, has_free: false };
    visitor.visit_ty(ty);
    visitor.has_free
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

/// Transforms "FooBar" to "foo-bar".
///
/// If the first Camel segment is "Moz" or "Webkit", the result string
/// is prepended with "-".
fn to_css_identifier(mut camel_case: &str) -> String {
    camel_case = camel_case.trim_right_matches('_');
    let mut first = true;
    let mut result = String::with_capacity(camel_case.len());
    while let Some(segment) = split_camel_segment(&mut camel_case) {
        if first {
            match segment {
                "Moz" | "Webkit" => first = false,
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
