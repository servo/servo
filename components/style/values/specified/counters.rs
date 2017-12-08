/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for counter properties.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use style_traits::ParseError;
use values::CustomIdent;
use values::generics::counters::CounterIntegerList;
use values::specified::Integer;

/// A specified value for the `counter-increment` and `counter-reset` property.
type SpecifiedIntegerList = CounterIntegerList<Integer>;

impl SpecifiedIntegerList {
    fn parse_with_default<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        default_value: i32
    ) -> Result<SpecifiedIntegerList, ParseError<'i>> {
        use cssparser::Token;
        use style_traits::StyleParseErrorKind;

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(CounterIntegerList::new(Vec::new()))
        }

        let mut counters: Vec<(CustomIdent, Integer)> = Vec::new();
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
            Ok(CounterIntegerList::new(counters))
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

/// A specified value for the `counter-increment` property.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq, ToCss)]
pub struct CounterIncrement(pub SpecifiedIntegerList);

impl CounterIncrement {
    /// Returns a new specified `counter-increment` object with the given values.
    pub fn new(vec: Vec<(CustomIdent, Integer)>) -> CounterIncrement {
        CounterIncrement(SpecifiedIntegerList::new(vec))
    }

    /// Returns the values of the specified `counter-increment` object.
    pub fn get_values(&self) -> &[(CustomIdent, Integer)] {
        self.0.get_values()
    }
}

impl Parse for CounterIncrement {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<CounterIncrement, ParseError<'i>> {
        Ok(CounterIncrement(SpecifiedIntegerList::parse_with_default(context, input, 1)?))
    }
}

/// A specified value for the `counter-reset` property.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq, ToCss)]
pub struct CounterReset(pub SpecifiedIntegerList);

impl CounterReset {
    /// Returns a new specified `counter-reset` object with the given values.
    pub fn new(vec: Vec<(CustomIdent, Integer)>) -> CounterReset {
        CounterReset(SpecifiedIntegerList::new(vec))
    }

    /// Returns the values of the specified `counter-reset` object.
    pub fn get_values(&self) -> &[(CustomIdent, Integer)] {
        self.0.get_values()
    }
}

impl Parse for CounterReset {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<CounterReset, ParseError<'i>> {
        Ok(CounterReset(SpecifiedIntegerList::parse_with_default(context, input, 0)?))
    }
}
