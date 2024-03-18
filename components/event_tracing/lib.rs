/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::{braced, parenthesized, parse_macro_input, token, Ident, Token};

#[proc_macro]
pub fn log_target(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    log_target_real(parse_macro_input!(input as LogTarget)).into()
}

struct LogTarget {
    value: MatchValue,
    variants: Vec<TopLevelVariant>,
}

enum MatchValue {
    SelfArgument,
    Other { value: Ident, r#type: Ident },
}

struct TopLevelVariant {
    us: Ident,
    direction: Direction,
    them: Ident,
    variant: Variant,
}

struct Variant {
    name: Ident,
    fields: Option<MatchFields>,
    nested_match: Option<NestedMatch>,
}

enum Direction {
    Received,
    Sent,
}

enum MatchFields {
    Struct(TokenStream),
    Tuple(TokenStream),
}

struct NestedMatch {
    value: MatchValue,
    variants: Vec<Variant>,
}

impl Parse for LogTarget {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse MatchValue, the enum value to match on.
        let value = input.parse::<MatchValue>()?;

        // Parse list of TopLevelVariant in braces.
        let braces;
        braced!(braces in input);
        let variants = braces.parse_terminated(TopLevelVariant::parse, Token![,])?;

        Ok(Self {
            value,
            variants: variants.into_iter().collect(),
        })
    }
}

impl Parse for MatchValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse “self” or “value as Type”.
        let value = input.lookahead1();
        if value.peek(Token![self]) {
            input.parse::<Token![self]>()?;
            Ok(Self::SelfArgument)
        } else if value.peek(Ident) {
            let value = input.parse::<Ident>()?;
            input.parse::<Token![as]>()?;
            let r#type = input.parse::<Ident>()?;
            Ok(Self::Other { value, r#type })
        } else {
            Err(value.error())
        }
    }
}

impl Parse for NestedMatch {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse MatchValue, the enum value to match on.
        let value = input.parse::<MatchValue>()?;

        // Parse list of Variant in braces.
        let braces;
        braced!(braces in input);
        let variants = braces.parse_terminated(Variant::parse, Token![,])?;

        Ok(Self {
            value,
            variants: variants.into_iter().collect(),
        })
    }
}

impl Parse for TopLevelVariant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse “us<them@” or “us>them@”.
        let us = input.parse::<Ident>()?;
        let direction = input.lookahead1();
        let direction = if direction.peek(Token![<]) {
            input.parse::<Token![<]>()?;
            Direction::Received
        } else if direction.peek(Token![>]) {
            input.parse::<Token![>]>()?;
            Direction::Sent
        } else {
            return Err(direction.error());
        };
        let them = input.parse::<Ident>()?;
        input.parse::<Token![@]>()?;

        // Parse the rest of the Variant.
        let variant = input.parse::<Variant>()?;

        Ok(Self {
            us,
            direction,
            them,
            variant,
        })
    }
}

impl Parse for Variant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse the variant name.
        let name = input.parse::<Ident>()?;

        // Parse optional “{struct pattern}” or “(tuple pattern)”.
        let group = input.lookahead1();
        let fields = if group.peek(token::Brace) {
            let braces;
            braced!(braces in input);
            Some(MatchFields::Struct(braces.parse::<TokenStream>()?))
        } else if group.peek(token::Paren) {
            let parens;
            parenthesized!(parens in input);
            Some(MatchFields::Tuple(parens.parse::<TokenStream>()?))
        } else if group.peek(Token![,]) || group.peek(Token![=>]) {
            None
        } else {
            return Err(group.error());
        };

        // Parse optional “=> NestedMatch”.
        let nested_match = input.lookahead1();
        let nested_match = if nested_match.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            Some(input.parse::<NestedMatch>()?)
        } else if nested_match.peek(Token![,]) {
            None
        } else {
            return Err(group.error());
        };

        Ok(Self {
            name,
            fields,
            nested_match,
        })
    }
}

fn log_target_real(input: LogTarget) -> TokenStream {
    let LogTarget { value, variants } = input;
    let r#type = value.r#type();
    let value = value.value();
    let variants = variants
        .into_iter()
        .map(|v| v.variant.generate(r#type.clone(), &v, &mut vec![]));

    quote! {
        match #value {
            #(#variants)*
        }
    }
}

impl Variant {
    fn generate(
        &self,
        r#type: TokenStream,
        top_level_variant: &TopLevelVariant,
        outer_idents: &mut Vec<Ident>,
    ) -> TokenStream {
        let name = &self.name;
        let fields = self.fields.iter().map(|fields| fields.quote());
        let us = &top_level_variant.us;
        let direction = top_level_variant.direction.quote();
        let them = &top_level_variant.them;

        match &self.nested_match {
            Some(nested_match) => {
                outer_idents.push(name.clone());
                let value = nested_match.value.value();
                let variants = nested_match
                    .variants
                    .iter()
                    .map(|v| {
                        v.generate(nested_match.value.r#type(), top_level_variant, outer_idents)
                    })
                    .collect::<Vec<_>>();
                outer_idents.pop();
                quote! {
                    #r#type::#name #(#fields)* => match #value {
                        #(#variants)*
                    },
                }
            },
            None => {
                let close_parens = outer_idents.iter().map(|_| quote! { ")" });
                quote! {
                    #r#type::#name #(#fields)* => concat!(
                        stringify!(#us),
                        stringify!(#direction),
                        stringify!(#them),
                        stringify!(@),
                        #(stringify!(#outer_idents), "(",)*
                        stringify!(#name),
                        #(#close_parens,)*
                    ),
                }
            },
        }
    }
}

impl MatchValue {
    fn value(&self) -> TokenStream {
        match self {
            MatchValue::SelfArgument => quote! { self },
            MatchValue::Other { value, .. } => quote! { #value },
        }
    }

    fn r#type(&self) -> TokenStream {
        match self {
            MatchValue::SelfArgument => quote! { Self },
            MatchValue::Other { r#type, .. } => quote! { #r#type },
        }
    }
}

impl MatchFields {
    fn quote(&self) -> TokenStream {
        match self {
            MatchFields::Struct(tokens) => quote! { { #tokens } },
            MatchFields::Tuple(tokens) => quote! { ( #tokens ) },
        }
    }
}

impl Direction {
    fn quote(&self) -> TokenStream {
        match self {
            Direction::Received => quote! { < },
            Direction::Sent => quote! { > },
        }
    }
}
