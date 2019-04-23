/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::to_css::CssVariantAttrs;
use derive_common::cg;
use proc_macro2::TokenStream;
use syn::{self, DeriveInput, Path};
use synstructure::{Structure, VariantInfo};

#[darling(attributes(parse), default)]
#[derive(Default, FromVariant)]
pub struct ParseVariantAttrs {
    pub aliases: Option<String>,
    pub condition: Option<Path>,
}

fn parse_non_keyword_variant(
    name: &syn::Ident,
    variant: &VariantInfo,
    variant_attrs: &CssVariantAttrs,
    parse_attrs: &ParseVariantAttrs,
    skip_try: bool,
) -> TokenStream {
    let bindings = variant.bindings();
    assert!(parse_attrs.aliases.is_none());
    assert!(variant_attrs.function.is_none());
    assert!(variant_attrs.keyword.is_none());
    assert_eq!(
        bindings.len(),
        1,
        "We only support deriving parse for simple variants"
    );
    let variant_name = &variant.ast().ident;
    let ty = &bindings[0].ast().ty;
    let mut parse = if skip_try {
        quote! {
            let v = <#ty as crate::parser::Parse>::parse(context, input)?;
            return Ok(#name::#variant_name(v));
        }
    } else {
        quote! {
            if let Ok(v) = input.try(|i| <#ty as crate::parser::Parse>::parse(context, i)) {
                return Ok(#name::#variant_name(v));
            }
        }
    };

    if let Some(ref condition) = parse_attrs.condition {
        parse = quote! {
            if #condition(context) {
                #parse
            }
        };

        if skip_try {
            // We're the last variant and we can fail to parse due to the
            // condition clause. If that happens, we need to return an error.
            parse = quote! {
                #parse
                Err(input.new_custom_error(style_traits::StyleParseErrorKind::UnspecifiedError))
            };
        }
    }

    parse
}

pub fn derive(mut input: DeriveInput) -> TokenStream {
    {
        let mut where_clause = input.generics.where_clause.take();
        for param in input.generics.type_params() {
            cg::add_predicate(
                &mut where_clause,
                parse_quote!(#param: crate::parser::Parse),
            );
        }
        input.generics.where_clause = where_clause;
    }

    let name = &input.ident;
    let s = Structure::new(&input);

    let mut saw_condition = false;
    let mut match_keywords = quote! {};
    let mut non_keywords = vec![];

    let mut effective_variants = 0;
    for variant in s.variants().iter() {
        let css_variant_attrs = cg::parse_variant_attrs_from_ast::<CssVariantAttrs>(&variant.ast());
        let parse_attrs = cg::parse_variant_attrs_from_ast::<ParseVariantAttrs>(&variant.ast());
        if css_variant_attrs.skip {
            continue;
        }
        effective_variants += 1;

        saw_condition |= parse_attrs.condition.is_some();

        if !variant.bindings().is_empty() {
            non_keywords.push((variant, css_variant_attrs, parse_attrs));
            continue;
        }

        let identifier = cg::to_css_identifier(
            &css_variant_attrs
                .keyword
                .unwrap_or_else(|| variant.ast().ident.to_string()),
        );
        let ident = &variant.ast().ident;

        let condition = match parse_attrs.condition {
            Some(ref p) => quote! { if #p(context) },
            None => quote! {},
        };

        match_keywords.extend(quote! {
            #identifier #condition => Ok(#name::#ident),
        });

        let aliases = match parse_attrs.aliases {
            Some(aliases) => aliases,
            None => continue,
        };

        for alias in aliases.split(',') {
            match_keywords.extend(quote! {
                #alias #condition => Ok(#name::#ident),
            });
        }
    }

    let needs_context = saw_condition || !non_keywords.is_empty();

    let context_ident = if needs_context {
        quote! { context }
    } else {
        quote! { _ }
    };

    let has_keywords = non_keywords.len() != effective_variants;

    let mut parse_non_keywords = quote! {};
    for (i, (variant, css_attrs, parse_attrs)) in non_keywords.iter().enumerate() {
        let skip_try = !has_keywords && i == non_keywords.len() - 1;
        let parse_variant =
            parse_non_keyword_variant(name, variant, css_attrs, parse_attrs, skip_try);
        parse_non_keywords.extend(parse_variant);
    }

    let parse_body = if needs_context {
        let parse_keywords = if has_keywords {
            quote! {
                let location = input.current_source_location();
                let ident = input.expect_ident()?;
                match_ignore_ascii_case! { &ident,
                    #match_keywords
                    _ => Err(location.new_unexpected_token_error(
                        cssparser::Token::Ident(ident.clone())
                    ))
                }
            }
        } else {
            quote! {}
        };

        quote! {
            #parse_non_keywords
            #parse_keywords
        }
    } else {
        quote! { Self::parse(input) }
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let parse_trait_impl = quote! {
        impl #impl_generics crate::parser::Parse for #name #ty_generics #where_clause {
            #[inline]
            fn parse<'i, 't>(
                #context_ident: &crate::parser::ParserContext,
                input: &mut cssparser::Parser<'i, 't>,
            ) -> Result<Self, style_traits::ParseError<'i>> {
                #parse_body
            }
        }
    };

    if needs_context {
        return parse_trait_impl;
    }

    assert!(non_keywords.is_empty());

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
                    #match_keywords
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
