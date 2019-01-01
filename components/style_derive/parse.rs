/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::cg;
use crate::to_css::CssVariantAttrs;
use proc_macro2::TokenStream;
use syn::{DeriveInput, Path};
use synstructure;

#[darling(attributes(parse), default)]
#[derive(Default, FromVariant)]
pub struct ParseVariantAttrs {
    pub aliases: Option<String>,
    pub condition: Option<Path>,
}

pub fn derive(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let s = synstructure::Structure::new(&input);

    let mut saw_condition = false;
    let match_body = s.variants().iter().fold(quote!(), |match_body, variant| {
        let bindings = variant.bindings();
        assert!(
            bindings.is_empty(),
            "Parse is only supported for single-variant enums for now"
        );

        let css_variant_attrs = cg::parse_variant_attrs_from_ast::<CssVariantAttrs>(&variant.ast());
        let parse_attrs = cg::parse_variant_attrs_from_ast::<ParseVariantAttrs>(&variant.ast());
        if css_variant_attrs.skip {
            return match_body;
        }

        let identifier = cg::to_css_identifier(
            &css_variant_attrs
                .keyword
                .unwrap_or_else(|| variant.ast().ident.to_string()),
        );
        let ident = &variant.ast().ident;

        saw_condition |= parse_attrs.condition.is_some();
        let condition = match parse_attrs.condition {
            Some(ref p) => quote! { if #p(context) },
            None => quote!{},
        };

        let mut body = quote! {
            #match_body
            #identifier #condition => Ok(#name::#ident),
        };

        let aliases = match parse_attrs.aliases {
            Some(aliases) => aliases,
            None => return body,
        };

        for alias in aliases.split(',') {
            body = quote! {
                #body
                #alias #condition => Ok(#name::#ident),
            };
        }

        body
    });

    let context_ident = if saw_condition {
        quote! { context }
    } else {
        quote! { _ }
    };

    let parse_body = if saw_condition {
        quote! {
            let location = input.current_source_location();
            let ident = input.expect_ident()?;
            match_ignore_ascii_case! { &ident,
                #match_body
                _ => Err(location.new_unexpected_token_error(
                    cssparser::Token::Ident(ident.clone())
                ))
            }
        }
    } else {
        quote! { Self::parse(input) }
    };

    let parse_trait_impl = quote! {
        impl crate::parser::Parse for #name {
            #[inline]
            fn parse<'i, 't>(
                #context_ident: &crate::parser::ParserContext,
                input: &mut cssparser::Parser<'i, 't>,
            ) -> Result<Self, style_traits::ParseError<'i>> {
                #parse_body
            }
        }
    };

    if saw_condition {
        return parse_trait_impl;
    }

    // TODO(emilio): It'd be nice to get rid of these, but that makes the
    // conversion harder...
    let methods_impl = quote! {
        impl #name {
            /// Parse this keyword.
            #[inline]
            pub fn parse<'i, 't>(
                input: &mut cssparser::Parser<'i, 't>,
            ) -> Result<Self, style_traits::ParseError<'i>> {
                let location = input.current_source_location();
                let ident = input.expect_ident()?;
                Self::from_ident(ident.as_ref()).map_err(|()| {
                    location.new_unexpected_token_error(
                        cssparser::Token::Ident(ident.clone())
                    )
                })
            }

            /// Parse this keyword from a string slice.
            #[inline]
            pub fn from_ident(ident: &str) -> Result<Self, ()> {
                match_ignore_ascii_case! { ident,
                    #match_body
                    _ => Err(()),
                }
            }
        }
    };

    quote! {
        #parse_trait_impl
        #methods_impl
    }
}
