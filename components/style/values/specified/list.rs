/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! `list` specified values.

use crate::parser::{Parse, ParserContext};
#[cfg(feature = "gecko")]
use crate::values::generics::CounterStyle;
#[cfg(feature = "gecko")]
use crate::values::CustomIdent;
use cssparser::{Parser, Token};
use style_traits::{ParseError, StyleParseErrorKind};

/// Specified and computed `list-style-type` property.
#[cfg(feature = "gecko")]
#[derive(
    Clone,
    Debug,
    Eq,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
pub enum ListStyleType {
    /// `none`
    None,
    /// <counter-style>
    CounterStyle(CounterStyle),
    /// <string>
    String(String),
}

#[cfg(feature = "gecko")]
impl ListStyleType {
    /// Initial specified value for `list-style-type`.
    #[inline]
    pub fn disc() -> Self {
        ListStyleType::CounterStyle(CounterStyle::disc())
    }

    /// Convert from gecko keyword to list-style-type.
    ///
    /// This should only be used for mapping type attribute to
    /// list-style-type, and thus only values possible in that
    /// attribute is considered here.
    pub fn from_gecko_keyword(value: u32) -> Self {
        use crate::gecko_bindings::structs;
        let v8 = value as u8;

        if v8 == structs::ListStyle_None {
            return ListStyleType::None;
        }

        ListStyleType::CounterStyle(CounterStyle::Name(CustomIdent(match v8 {
            structs::ListStyle_Disc => atom!("disc"),
            structs::ListStyle_Circle => atom!("circle"),
            structs::ListStyle_Square => atom!("square"),
            structs::ListStyle_Decimal => atom!("decimal"),
            structs::ListStyle_LowerRoman => atom!("lower-roman"),
            structs::ListStyle_UpperRoman => atom!("upper-roman"),
            structs::ListStyle_LowerAlpha => atom!("lower-alpha"),
            structs::ListStyle_UpperAlpha => atom!("upper-alpha"),
            _ => unreachable!("Unknown counter style keyword value"),
        })))
    }

    /// Is this a bullet? (i.e. `list-style-type: disc|circle|square|disclosure-closed|disclosure-open`)
    #[inline]
    pub fn is_bullet(&self) -> bool {
        match self {
            ListStyleType::CounterStyle(ref style) => style.is_bullet(),
            _ => false,
        }
    }
}

#[cfg(feature = "gecko")]
impl Parse for ListStyleType {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(style) = input.try_parse(|i| CounterStyle::parse(context, i)) {
            return Ok(ListStyleType::CounterStyle(style));
        }
        if input.try_parse(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(ListStyleType::None);
        }
        Ok(ListStyleType::String(
            input.expect_string()?.as_ref().to_owned(),
        ))
    }
}

/// A quote pair.
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct QuotePair {
    /// The opening quote.
    pub opening: crate::OwnedStr,

    /// The closing quote.
    pub closing: crate::OwnedStr,
}

/// List of quote pairs for the specified/computed value of `quotes` property.
#[derive(
    Clone,
    Debug,
    Default,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
pub struct QuoteList(
    #[css(iterable, if_empty = "none")]
    #[ignore_malloc_size_of = "Arc"]
    pub crate::ArcSlice<QuotePair>,
);

/// Specified and computed `quotes` property: `auto`, `none`, or a list
/// of characters.
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub enum Quotes {
    /// list of quote pairs
    QuoteList(QuoteList),
    /// auto (use lang-dependent quote marks)
    Auto,
}

impl Parse for Quotes {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Quotes, ParseError<'i>> {
        if input
            .try_parse(|input| input.expect_ident_matching("auto"))
            .is_ok()
        {
            return Ok(Quotes::Auto);
        }

        if input
            .try_parse(|input| input.expect_ident_matching("none"))
            .is_ok()
        {
            return Ok(Quotes::QuoteList(QuoteList::default()));
        }

        let mut quotes = Vec::new();
        loop {
            let location = input.current_source_location();
            let opening = match input.next() {
                Ok(&Token::QuotedString(ref value)) => value.as_ref().to_owned().into(),
                Ok(t) => return Err(location.new_unexpected_token_error(t.clone())),
                Err(_) => break,
            };

            let closing = input.expect_string()?.as_ref().to_owned().into();
            quotes.push(QuotePair { opening, closing });
        }

        if !quotes.is_empty() {
            Ok(Quotes::QuoteList(QuoteList(crate::ArcSlice::from_iter(
                quotes.into_iter(),
            ))))
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}
