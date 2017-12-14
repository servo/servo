/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! `list` specified values.

use cssparser::{Parser, Token};
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::{ParseError, StyleParseErrorKind, ToCss};

/// Specified and computed `quote` property.
///
/// FIXME(emilio): It's a shame that this allocates all the time it's computed,
/// probably should just be refcounted.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct Quotes(pub Box<[(Box<str>, Box<str>)]>);

impl ToCss for Quotes {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut iter = self.0.iter();

        match iter.next() {
            Some(&(ref l, ref r)) => {
                l.to_css(dest)?;
                dest.write_char(' ')?;
                r.to_css(dest)?;
            }
            None => return dest.write_str("none"),
        }

        for &(ref l, ref r) in iter {
            dest.write_char(' ')?;
            l.to_css(dest)?;
            dest.write_char(' ')?;
            r.to_css(dest)?;
        }

        Ok(())
    }
}

impl Parse for Quotes {
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Quotes, ParseError<'i>> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(Quotes(Vec::new().into_boxed_slice()))
        }

        let mut quotes = Vec::new();
        loop {
            let location = input.current_source_location();
            let first = match input.next() {
                Ok(&Token::QuotedString(ref value)) => {
                    value.as_ref().to_owned().into_boxed_str()
                },
                Ok(t) => return Err(location.new_unexpected_token_error(t.clone())),
                Err(_) => break,
            };

            let second =
                input.expect_string()?.as_ref().to_owned().into_boxed_str();
            quotes.push((first, second))
        }

        if !quotes.is_empty() {
            Ok(Quotes(quotes.into_boxed_slice()))
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}
