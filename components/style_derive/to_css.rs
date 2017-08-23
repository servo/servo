/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote;
use syn;
use synstructure;

pub fn derive(input: syn::DeriveInput) -> quote::Tokens {
    let name = &input.ident;
    let trait_path = &["style_traits", "ToCss"];
    let (impl_generics, ty_generics, mut where_clause) =
        cg::trait_parts(&input, trait_path);

    let style = synstructure::BindStyle::Ref.into();
    let match_body = synstructure::each_variant(&input, &style, |bindings, variant| {
        let mut identifier = to_css_identifier(variant.ident.as_ref());
        let css_attrs = cg::parse_variant_attrs::<CssAttrs>(variant);
        let separator = if css_attrs.comma { ", " } else { " " };
        let mut expr = if !bindings.is_empty() {
            let mut expr = quote! {};
            for binding in bindings {
                if cg::is_parameterized(&binding.field.ty, &input.generics.ty_params) {
                    where_clause.predicates.push(
                        cg::where_predicate(binding.field.ty.clone(), trait_path),
                    );
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
        if css_attrs.function {
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

#[derive(Default, FromVariant)]
#[darling(attributes(css), default)]
struct CssAttrs {
    function: bool,
    comma: bool,
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
