/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values for counter properties

#[cfg(feature = "servo")]
use computed_values::list_style_type::T as ListStyleType;
use cssparser::{Parser, Token};
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseErrorKind;
use style_traits::{ParseError, StyleParseErrorKind};
use values::CustomIdent;
#[cfg(feature = "gecko")]
use values::generics::CounterStyleOrNone;
use values::generics::counters::CounterIncrement as GenericCounterIncrement;
use values::generics::counters::CounterReset as GenericCounterReset;
#[cfg(feature = "gecko")]
use values::specified::Attr;
#[cfg(feature = "gecko")]
use values::specified::url::SpecifiedImageUrl;
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
                if let Ok(url) = input.try(|i| SpecifiedImageUrl::parse(_context, i)) {
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
                            let location = input.current_source_location();
                            let name = CustomIdent::from_ident(location, input.expect_ident()?, &[])?;
                            #[cfg(feature = "servo")]
                            let style = Content::parse_counter_style(input);
                            #[cfg(feature = "gecko")]
                            let style = Content::parse_counter_style(_context, input);
                            Ok(ContentItem::Counter(name, style))
                        })),
                        "counters" => Some(input.parse_nested_block(|input| {
                            let location = input.current_source_location();
                            let name = CustomIdent::from_ident(location, input.expect_ident()?, &[])?;
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
