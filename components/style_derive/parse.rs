/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use cg;
use quote::Tokens;
use syn::DeriveInput;
use synstructure;

pub fn derive(input: DeriveInput) -> Tokens {
    let name = &input.ident;

    let mut match_body = quote! {};

    let style = synstructure::BindStyle::Ref.into();
    synstructure::each_variant(&input, &style, |bindings, variant| {
        assert!(
            bindings.is_empty(),
            "Parse is only supported for single-variant enums for now"
        );

        let variant_attrs = cg::parse_variant_attrs::<ParseVariantAttrs>(variant);
        let identifier = cg::to_css_identifier(variant.ident.as_ref());
        let ident = &variant.ident;

        match_body = quote! {
            #match_body
            #identifier => Ok(#name::#ident),
        };

        let aliases = match variant_attrs.aliases {
            Some(aliases) => aliases,
            None => return,
        };

        for alias in aliases.split(",") {
            match_body = quote! {
                #match_body
                #alias => Ok(#name::#ident),
            };
        }
    });

    let parse_trait_impl = quote! {
        impl ::parser::Parse for #name {
            #[inline]
            fn parse<'i, 't>(
                _: &::parser::ParserContext,
                input: &mut ::cssparser::Parser<'i, 't>,
            ) -> Result<Self, ::style_traits::ParseError<'i>> {
                Self::parse(input)
            }
        }
    };

    // TODO(emilio): It'd be nice to get rid of these, but that makes the
    // conversion harder...
    let methods_impl = quote! {
        impl #name {
            /// Parse this keyword.
            #[inline]
            pub fn parse<'i, 't>(
                input: &mut ::cssparser::Parser<'i, 't>,
            ) -> Result<Self, ::style_traits::ParseError<'i>> {
                let location = input.current_source_location();
                let ident = input.expect_ident()?;
                Self::from_ident(ident.as_ref()).map_err(|()| {
                    location.new_unexpected_token_error(
                        ::cssparser::Token::Ident(ident.clone())
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

#[darling(attributes(parse), default)]
#[derive(Default, FromVariant)]
struct ParseVariantAttrs {
    /// The comma-separated list of aliases this variant should be aliased to at
    /// parse time.
    aliases: Option<String>,
}
