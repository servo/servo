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
        if bindings.is_empty() {
            let identifier = to_css_identifier(variant.ident.as_ref());
            return Some(quote! {
                ::std::fmt::Write::write_str(dest, #identifier)
            });
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
                ::std::fmt::Write::write_str(dest, " ")?;
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

/// Transforms "FooBar" to "foo-bar".
///
/// If the first Camel segment is "Moz"" or "Webkit", the result string
/// is prepended with "-".
fn to_css_identifier(mut camel_case: &str) -> String {
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
