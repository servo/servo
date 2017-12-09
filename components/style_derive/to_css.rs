/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote::Tokens;
use syn::DeriveInput;
use synstructure;

pub fn derive(input: DeriveInput) -> Tokens {
    let name = &input.ident;
    let trait_path = &["style_traits", "ToCss"];
    let (impl_generics, ty_generics, mut where_clause) =
        cg::trait_parts(&input, trait_path);

    let input_attrs = cg::parse_input_attrs::<CssInputAttrs>(&input);
    let style = synstructure::BindStyle::Ref.into();
    let match_body = synstructure::each_variant(&input, &style, |bindings, variant| {
        let mut identifier = to_css_identifier(variant.ident.as_ref());
        let variant_attrs = cg::parse_variant_attrs::<CssVariantAttrs>(variant);
        let separator = if variant_attrs.comma { ", " } else { " " };

        if variant_attrs.dimension {
            assert_eq!(bindings.len(), 1);
            assert!(!variant_attrs.function, "That makes no sense");
        }

        let mut expr = if !bindings.is_empty() {
            let mut expr = quote! {};
            if variant_attrs.function && variant_attrs.iterable {
                assert_eq!(bindings.len(), 1);
                let binding = &bindings[0];
                expr = quote! {
                    #expr

                    for item in #binding.iter() {
                        writer.item(item)?;
                    }
                };
            } else {
                for binding in bindings {
                    where_clause.add_trait_bound(&binding.field.ty);
                    expr = quote! {
                        #expr
                        writer.item(#binding)?;
                    };
                }
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

        if variant_attrs.dimension {
            expr = quote! {
                #expr?;
                ::std::fmt::Write::write_str(dest, #identifier)
            }
        } else if variant_attrs.function {
            identifier.push_str("(");
            expr = quote! {
                ::std::fmt::Write::write_str(dest, #identifier)?;
                #expr?;
                ::std::fmt::Write::write_str(dest, ")")
            }
        }
        Some(expr)
    });

    let mut impls = quote! {
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
    };

    if input_attrs.derive_debug {
        impls.append(quote! {
            impl #impl_generics ::std::fmt::Debug for #name #ty_generics #where_clause {
                fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    ::style_traits::ToCss::to_css(self, f)
                }
            }
        });
    }

    impls
}

#[darling(attributes(css), default)]
#[derive(Default, FromDeriveInput)]
struct CssInputAttrs {
    derive_debug: bool,
    function: bool,
    comma: bool,
}

#[darling(attributes(css), default)]
#[derive(Default, FromVariant)]
struct CssVariantAttrs {
    function: bool,
    iterable: bool,
    comma: bool,
    dimension: bool,
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
    let index = camel_case.chars().next()?.len_utf8();
    let end_position = camel_case[index..]
        .find(char::is_uppercase)
        .map_or(camel_case.len(), |pos| index + pos);
    let result = &camel_case[..end_position];
    *camel_case = &camel_case[end_position..];
    Some(result)
}
