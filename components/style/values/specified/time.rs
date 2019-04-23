/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified time values.

use crate::parser::{Parse, ParserContext};
use crate::values::computed::time::Time as ComputedTime;
use crate::values::computed::{Context, ToComputedValue};
use crate::values::specified::calc::CalcNode;
use crate::values::CSSFloat;
use cssparser::{Parser, Token};
use std::fmt::{self, Write};
use style_traits::values::specified::AllowedNumericType;
use style_traits::{CssWriter, ParseError, SpecifiedValueInfo, StyleParseErrorKind, ToCss};

/// A time value according to CSS-VALUES § 6.2.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub struct Time {
    seconds: CSSFloat,
    unit: TimeUnit,
    was_calc: bool,
}

/// A time unit.
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToShmem)]
pub enum TimeUnit {
    /// `s`
    Second,
    /// `ms`
    Millisecond,
}

impl Time {
    /// Returns a time value that represents `seconds` seconds.
    pub fn from_seconds(seconds: CSSFloat) -> Self {
        Time {
            seconds,
            unit: TimeUnit::Second,
            was_calc: false,
        }
    }

    /// Returns `0s`.
    pub fn zero() -> Self {
        Self::from_seconds(0.0)
    }

    /// Returns the time in fractional seconds.
    pub fn seconds(self) -> CSSFloat {
        self.seconds
    }

    /// Parses a time according to CSS-VALUES § 6.2.
    pub fn parse_dimension(value: CSSFloat, unit: &str, was_calc: bool) -> Result<Time, ()> {
        let (seconds, unit) = match_ignore_ascii_case! { unit,
            "s" => (value, TimeUnit::Second),
            "ms" => (value / 1000.0, TimeUnit::Millisecond),
            _ => return Err(())
        };

        Ok(Time {
            seconds,
            unit,
            was_calc,
        })
    }

    /// Returns a `Time` value from a CSS `calc()` expression.
    pub fn from_calc(seconds: CSSFloat) -> Self {
        Time {
            seconds: seconds,
            unit: TimeUnit::Second,
            was_calc: true,
        }
    }

    fn parse_with_clamping_mode<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        clamping_mode: AllowedNumericType,
    ) -> Result<Self, ParseError<'i>> {
        use style_traits::ParsingMode;

        let location = input.current_source_location();
        // FIXME: remove early returns when lifetimes are non-lexical
        match input.next() {
            // Note that we generally pass ParserContext to is_ok() to check
            // that the ParserMode of the ParserContext allows all numeric
            // values for SMIL regardless of clamping_mode, but in this Time
            // value case, the value does not animate for SMIL at all, so we use
            // ParsingMode::DEFAULT directly.
            Ok(&Token::Dimension {
                value, ref unit, ..
            }) if clamping_mode.is_ok(ParsingMode::DEFAULT, value) => {
                return Time::parse_dimension(value, unit, /* from_calc = */ false).map_err(|()| {
                    location.new_custom_error(StyleParseErrorKind::UnspecifiedError)
                });
            },
            Ok(&Token::Function(ref name)) if name.eq_ignore_ascii_case("calc") => {},
            Ok(t) => return Err(location.new_unexpected_token_error(t.clone())),
            Err(e) => return Err(e.into()),
        }
        match input.parse_nested_block(|i| CalcNode::parse_time(context, i)) {
            Ok(time) if clamping_mode.is_ok(ParsingMode::DEFAULT, time.seconds) => Ok(time),
            _ => Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
        }
    }

    /// Parses a non-negative time value.
    pub fn parse_non_negative<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_with_clamping_mode(context, input, AllowedNumericType::NonNegative)
    }
}

impl ToComputedValue for Time {
    type ComputedValue = ComputedTime;

    fn to_computed_value(&self, _context: &Context) -> Self::ComputedValue {
        ComputedTime::from_seconds(self.seconds())
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Time {
            seconds: computed.seconds(),
            unit: TimeUnit::Second,
            was_calc: false,
        }
    }
}

impl Parse for Time {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_with_clamping_mode(context, input, AllowedNumericType::All)
    }
}

impl ToCss for Time {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.was_calc {
            dest.write_str("calc(")?;
        }
        match self.unit {
            TimeUnit::Second => {
                self.seconds.to_css(dest)?;
                dest.write_str("s")?;
            },
            TimeUnit::Millisecond => {
                (self.seconds * 1000.).to_css(dest)?;
                dest.write_str("ms")?;
            },
        }
        if self.was_calc {
            dest.write_str(")")?;
        }
        Ok(())
    }
}

impl SpecifiedValueInfo for Time {}
