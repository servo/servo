/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! `list` specified values.

use cssparser::{Parser, Token, serialize_string};
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::{ParseError, StyleParseErrorKind, ToCss};

/// Specified and computed `quote` property
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct Quotes(pub Vec<(String, String)>);

impl ToCss for Quotes {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if self.0.is_empty() {
            return dest.write_str("none")
        }

        let mut first = true;
        for pair in &self.0 {
            if !first {
                dest.write_str(" ")?;
            }
            first = false;
            serialize_string(&*pair.0, dest)?;
            dest.write_str(" ")?;
            serialize_string(&*pair.1, dest)?;
        }
        Ok(())
    }
}

impl Parse for Quotes {
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Quotes, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(Quotes(Vec::new()))
        }

        let mut quotes = Vec::new();
        loop {
            let location = input.current_source_location();
            let first = match input.next() {
                Ok(&Token::QuotedString(ref value)) => value.as_ref().to_owned(),
                Ok(t) => return Err(location.new_unexpected_token_error(t.clone())),
                Err(_) => break,
            };
            let location = input.current_source_location();
            let second = match input.next() {
                Ok(&Token::QuotedString(ref value)) => value.as_ref().to_owned(),
                Ok(t) => return Err(location.new_unexpected_token_error(t.clone())),
                Err(e) => return Err(e.into()),
            };
            quotes.push((first, second))
        }
        if !quotes.is_empty() {
            Ok(Quotes(quotes))
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}
