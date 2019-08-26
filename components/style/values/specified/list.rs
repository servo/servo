/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! `list` specified values.

use crate::parser::{Parse, ParserContext};
#[cfg(feature = "gecko")]
use crate::values::generics::CounterStyleOrNone;
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
    /// <counter-style> | none
    CounterStyle(CounterStyleOrNone),
    /// <string>
    String(String),
}

#[cfg(feature = "gecko")]
impl ListStyleType {
    /// Initial specified value for `list-style-type`.
    #[inline]
    pub fn disc() -> Self {
        ListStyleType::CounterStyle(CounterStyleOrNone::disc())
    }

    /// Convert from gecko keyword to list-style-type.
    ///
    /// This should only be used for mapping type attribute to
    /// list-style-type, and thus only values possible in that
    /// attribute is considered here.
    pub fn from_gecko_keyword(value: u32) -> Self {
        use crate::gecko_bindings::structs;

        if value == structs::NS_STYLE_LIST_STYLE_NONE {
            return ListStyleType::CounterStyle(CounterStyleOrNone::None);
        }

        ListStyleType::CounterStyle(CounterStyleOrNone::Name(CustomIdent(match value {
            structs::NS_STYLE_LIST_STYLE_DISC => atom!("disc"),
            structs::NS_STYLE_LIST_STYLE_CIRCLE => atom!("circle"),
            structs::NS_STYLE_LIST_STYLE_SQUARE => atom!("square"),
            structs::NS_STYLE_LIST_STYLE_DECIMAL => atom!("decimal"),
            structs::NS_STYLE_LIST_STYLE_LOWER_ROMAN => atom!("lower-roman"),
            structs::NS_STYLE_LIST_STYLE_UPPER_ROMAN => atom!("upper-roman"),
            structs::NS_STYLE_LIST_STYLE_LOWER_ALPHA => atom!("lower-alpha"),
            structs::NS_STYLE_LIST_STYLE_UPPER_ALPHA => atom!("upper-alpha"),
            _ => unreachable!("Unknown counter style keyword value"),
        })))
    }
}

#[cfg(feature = "gecko")]
impl Parse for ListStyleType {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(style) = input.try(|i| CounterStyleOrNone::parse(context, i)) {
            return Ok(ListStyleType::CounterStyle(style));
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
            .try(|input| input.expect_ident_matching("auto"))
            .is_ok()
        {
            return Ok(Quotes::Auto);
        }

        if input
            .try(|input| input.expect_ident_matching("none"))
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

/// Specified and computed `-moz-list-reversed` property (for UA sheets only).
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum MozListReversed {
    /// the initial value
    False,
    /// exclusively used for <ol reversed> in our html.css UA sheet
    True,
}
