/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for counter properties.

use cssparser::{Token, Parser};
use parser::{Parse, ParserContext};
use style_traits::{ParseError, StyleParseErrorKind};
use values::CustomIdent;
use values::generics::counters::CounterIncrement as GenericCounterIncrement;
use values::generics::counters::CounterReset as GenericCounterReset;
use values::specified::Integer;

/// A specified value for the `counter-increment` property.
pub type CounterIncrement = GenericCounterIncrement<Integer>;

impl Parse for CounterIncrement {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        Ok(Self::new(parse_counters(context, input, 1)?))
    }
}

/// A specified value for the `counter-increment` property.
pub type CounterReset = GenericCounterReset<Integer>;

impl Parse for CounterReset {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        Ok(Self::new(parse_counters(context, input, 0)?))
    }
}

fn parse_counters<'i, 't>(
    context: &ParserContext,
    input: &mut Parser<'i, 't>,
    default_value: i32,
) -> Result<Vec<(CustomIdent, Integer)>, ParseError<'i>> {
    if input.try(|input| input.expect_ident_matching("none")).is_ok() {
        return Ok(vec![]);
    }

    let mut counters = Vec::new();
    loop {
        let location = input.current_source_location();
        let counter_name = match input.next() {
            Ok(&Token::Ident(ref ident)) => CustomIdent::from_ident(location, ident, &["none"])?,
            Ok(t) => return Err(location.new_unexpected_token_error(t.clone())),
            Err(_) => break,
        };

        let counter_delta = input.try(|input| Integer::parse(context, input))
                                    .unwrap_or(Integer::new(default_value));
        counters.push((counter_name, counter_delta))
    }

    if !counters.is_empty() {
        Ok(counters)
    } else {
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}
