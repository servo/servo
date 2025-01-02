/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use proc_macro2::Span;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{braced, token, Attribute, Ident, Path, Token, Type};

#[allow(non_camel_case_types)]
mod kw {
    syn::custom_keyword!(accessor_type);
    syn::custom_keyword!(gen_accessors);
    syn::custom_keyword!(gen_types);
}

pub struct MacroInput {
    pub type_def: RootTypeDef,
    pub gen_accessors: Ident,
    pub accessor_type: Path,
}

enum MacroArg {
    GenAccessors(ArgInner<kw::gen_accessors, Ident>),
    AccessorType(ArgInner<kw::accessor_type, Path>),
    Types(ArgInner<kw::gen_types, RootTypeDef>),
}

struct ArgInner<K, V> {
    _field_kw: K,
    _equals: Token![=],
    value: V,
}

pub struct Field {
    pub attributes: Vec<Attribute>,
    pub name: Ident,
    _colon: Token![:],
    pub field_type: FieldType,
}

pub enum FieldType {
    Existing(Type),
    NewTypeDef(NewTypeDef),
}

pub struct NewTypeDef {
    _braces: token::Brace,
    pub fields: Punctuated<Field, Token![, ]>,
}

pub struct RootTypeDef {
    pub type_name: Ident,
    pub type_def: NewTypeDef,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let fields: Punctuated<MacroArg, Token![, ]> =
            Punctuated::parse_terminated_with(input, MacroArg::parse)?;
        let mut gen_accessors = None;
        let mut type_def = None;
        let mut accessor_type = None;
        for arg in fields.into_iter() {
            match arg {
                MacroArg::GenAccessors(ArgInner { value, .. }) => gen_accessors = Some(value),
                MacroArg::AccessorType(ArgInner { value, .. }) => accessor_type = Some(value),
                MacroArg::Types(ArgInner { value, .. }) => type_def = Some(value),
            }
        }

        fn missing_attr(att_name: &str) -> syn::Error {
            syn::Error::new(
                Span::call_site(),
                format!("Expected `{}` attribute", att_name),
            )
        }

        Ok(MacroInput {
            type_def: type_def.ok_or_else(|| missing_attr("gen_types"))?,
            gen_accessors: gen_accessors.ok_or_else(|| missing_attr("gen_accessors"))?,
            accessor_type: accessor_type.ok_or_else(|| missing_attr("accessor_type"))?,
        })
    }
}

impl Parse for MacroArg {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::gen_types) {
            Ok(MacroArg::Types(input.parse()?))
        } else if lookahead.peek(kw::gen_accessors) {
            Ok(MacroArg::GenAccessors(input.parse()?))
        } else if lookahead.peek(kw::accessor_type) {
            Ok(MacroArg::AccessorType(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl<K: Parse, V: Parse> Parse for ArgInner<K, V> {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(ArgInner {
            _field_kw: input.parse()?,
            _equals: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for Field {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Field {
            attributes: input.call(Attribute::parse_outer)?,
            name: input.parse()?,
            _colon: input.parse()?,
            field_type: input.parse()?,
        })
    }
}

impl Parse for RootTypeDef {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(RootTypeDef {
            type_name: input.parse()?,
            type_def: input.parse()?,
        })
    }
}

impl Parse for NewTypeDef {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let content;
        #[allow(clippy::mixed_read_write_in_expression)]
        Ok(NewTypeDef {
            _braces: braced!(content in input),
            fields: Punctuated::parse_terminated_with(&content, Field::parse)?,
        })
    }
}

impl Parse for FieldType {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(token::Brace) {
            Ok(FieldType::NewTypeDef(input.parse()?))
        } else {
            Ok(FieldType::Existing(input.parse()?))
        }
    }
}
