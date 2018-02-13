/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values for counter properties

#[cfg(feature = "servo")]
use computed_values::list_style_type::T as ListStyleType;
use cssparser::{self, Parser, Token};
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};
#[cfg(feature = "gecko")]
use values::generics::CounterStyleOrNone;
use values::generics::counters::CounterIncrement as GenericCounterIncrement;
use values::generics::counters::CounterReset as GenericCounterReset;
#[cfg(feature = "gecko")]
use values::specified::Attr;
#[cfg(feature = "gecko")]
use values::specified::url::SpecifiedUrl;
pub use values::specified::{Content, ContentItem};

/// A computed value for the `counter-increment` property.
pub type CounterIncrement = GenericCounterIncrement<i32>;

/// A computed value for the `counter-increment` property.
pub type CounterReset = GenericCounterReset<i32>;

impl Content {
    /// Set `content` property to `normal`.
    #[inline]
    pub fn normal() -> Self {
        Content::Normal
    }

    #[cfg(feature = "servo")]
    fn parse_counter_style(
        input: &mut Parser
    ) -> ListStyleType {
        input.try(|input| {
            input.expect_comma()?;
            ListStyleType::parse(input)
        }).unwrap_or(ListStyleType::Decimal)
    }

    #[cfg(feature = "gecko")]
    fn parse_counter_style(
        context: &ParserContext,
        input: &mut Parser
    ) -> CounterStyleOrNone {
        input.try(|input| {
            input.expect_comma()?;
            CounterStyleOrNone::parse(context, input)
        }).unwrap_or(CounterStyleOrNone::decimal())
    }
}

impl Parse for Content {
    // normal | none | [ <string> | <counter> | open-quote | close-quote | no-open-quote |
    // no-close-quote ]+
    // TODO: <uri>, attr(<identifier>)
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
            return Ok(Content::Normal);
        }
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(Content::None);
        }
        #[cfg(feature = "gecko")] {
            if input.try(|input| input.expect_ident_matching("-moz-alt-content")).is_ok() {
                return Ok(Content::MozAltContent);
            }
        }

        let mut content = vec![];
        loop {
            #[cfg(feature = "gecko")] {
                if let Ok(mut url) = input.try(|i| SpecifiedUrl::parse(_context, i)) {
                    url.build_image_value();
                    content.push(ContentItem::Url(url));
                    continue;
                }
            }
            // FIXME: remove clone() when lifetimes are non-lexical
            match input.next().map(|t| t.clone()) {
                Ok(Token::QuotedString(ref value)) => {
                    content.push(ContentItem::String(value.as_ref().to_owned().into_boxed_str()));
                }
                Ok(Token::Function(ref name)) => {
                    let result = match_ignore_ascii_case! { &name,
                        "counter" => Some(input.parse_nested_block(|input| {
                            let name = input.expect_ident()?.as_ref().to_owned().into_boxed_str();
                            #[cfg(feature = "servo")]
                            let style = Content::parse_counter_style(input);
                            #[cfg(feature = "gecko")]
                            let style = Content::parse_counter_style(_context, input);
                            Ok(ContentItem::Counter(name, style))
                        })),
                        "counters" => Some(input.parse_nested_block(|input| {
                            let name = input.expect_ident()?.as_ref().to_owned().into_boxed_str();
                            input.expect_comma()?;
                            let separator = input.expect_string()?.as_ref().to_owned().into_boxed_str();
                            #[cfg(feature = "servo")]
                            let style = Content::parse_counter_style(input);
                            #[cfg(feature = "gecko")]
                            let style = Content::parse_counter_style(_context, input);
                            Ok(ContentItem::Counters(name, separator, style))
                        })),
                        #[cfg(feature = "gecko")]
                        "attr" => Some(input.parse_nested_block(|input| {
                            Ok(ContentItem::Attr(Attr::parse_function(_context, input)?))
                        })),
                        _ => None
                    };
                    match result {
                        Some(result) => content.push(result?),
                        None => return Err(input.new_custom_error(
                            StyleParseErrorKind::UnexpectedFunction(name.clone())
                        ))
                    }
                }
                Ok(Token::Ident(ref ident)) => {
                    content.push(
                        match_ignore_ascii_case! { &ident,
                            "open-quote" => ContentItem::OpenQuote,
                            "close-quote" => ContentItem::CloseQuote,
                            "no-open-quote" => ContentItem::NoOpenQuote,
                            "no-close-quote" => ContentItem::NoCloseQuote,
                            _ => return Err(input.new_custom_error(
                                    SelectorParseErrorKind::UnexpectedIdent(ident.clone())))
                        }
                    );
                }
                Err(_) => break,
                Ok(t) => return Err(input.new_unexpected_token_error(t))
            }
        }
        if content.is_empty() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }
        Ok(Content::Items(content.into_boxed_slice()))
    }
}

impl ToCss for ContentItem {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
        where W: Write,
    {
        match *self {
            ContentItem::String(ref s) => s.to_css(dest),
            ContentItem::Counter(ref s, ref counter_style) => {
                dest.write_str("counter(")?;
                cssparser::serialize_identifier(&**s, dest)?;
                dest.write_str(", ")?;
                counter_style.to_css(dest)?;
                dest.write_str(")")
            }
            ContentItem::Counters(ref s, ref separator, ref counter_style) => {
                dest.write_str("counters(")?;
                cssparser::serialize_identifier(&**s, dest)?;
                dest.write_str(", ")?;
                separator.to_css(dest)?;
                dest.write_str(", ")?;
                counter_style.to_css(dest)?;
                dest.write_str(")")
            }
            ContentItem::OpenQuote => dest.write_str("open-quote"),
            ContentItem::CloseQuote => dest.write_str("close-quote"),
            ContentItem::NoOpenQuote => dest.write_str("no-open-quote"),
            ContentItem::NoCloseQuote => dest.write_str("no-close-quote"),
            #[cfg(feature = "gecko")]
            ContentItem::Attr(ref attr) => {
                attr.to_css(dest)
            }
            #[cfg(feature = "gecko")]
            ContentItem::Url(ref url) => url.to_css(dest),
        }
    }
}
