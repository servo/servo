/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for counter properties.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::{ParseError, ToCss};
use values::CustomIdent;
use values::computed::{Context, CounterReset as ComputedValue, ToComputedValue};
use values::specified::Integer;

/// A specified value for the `counter-reset` property.
#[cfg_attr(feature = "gecko", derive(MallocSizeOf))]
#[derive(Clone, Debug, PartialEq)]
pub struct CounterReset(pub Vec<(CustomIdent, super::Integer)>);

impl Parse for CounterReset {
     fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<CounterReset, ParseError<'i>> {
        use cssparser::Token;
        use style_traits::StyleParseErrorKind;

        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(CounterReset(Vec::new()))
        }

        let mut counters: Vec<(CustomIdent, super::Integer)> = Vec::new();
        loop {
            let location = input.current_source_location();
            let counter_name = match input.next() {
                Ok(&Token::Ident(ref ident)) => CustomIdent::from_ident(location, ident, &["none"])?,
                Ok(t) => return Err(location.new_unexpected_token_error(t.clone())),
                Err(_) => break,
            };
            let counter_delta = input.try(|input| Integer::parse(context, input))
                                     .unwrap_or(Integer::new(0));
            counters.push((counter_name, counter_delta))
        }

        if !counters.is_empty() {
            Ok(CounterReset(counters))
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

impl ToComputedValue for CounterReset {
    type ComputedValue = ComputedValue;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        ComputedValue(self.0.iter().map(|&(ref name, ref value)| {
            (name.clone(), value.to_computed_value(context))
        }).collect::<Vec<_>>())
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        CounterReset(computed.0.iter().map(|&(ref name, ref value)| {
            (name.clone(), Integer::from_computed_value(&value))
        }).collect::<Vec<_>>())
    }
}

impl ToCss for CounterReset {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
                where W: fmt::Write,
    {
        if self.0.is_empty() {
            return dest.write_str("none")
        }

        let mut first = true;
        for &(ref name, value) in &self.0 {
            if !first {
                dest.write_str(" ")?;
            }
            first = false;
            name.to_css(dest)?;
            dest.write_str(" ")?;
            value.to_css(dest)?;
        }
        Ok(())
    }
}
