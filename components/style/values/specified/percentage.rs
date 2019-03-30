/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified percentages.

use crate::parser::{Parse, ParserContext};
use crate::values::computed::percentage::Percentage as ComputedPercentage;
use crate::values::computed::{Context, ToComputedValue};
use crate::values::specified::calc::CalcNode;
use crate::values::{serialize_percentage, CSSFloat};
use cssparser::{Parser, Token};
use std::fmt::{self, Write};
use style_traits::values::specified::AllowedNumericType;
use style_traits::{CssWriter, ParseError, SpecifiedValueInfo, ToCss};

/// A percentage value.
#[derive(Clone, Copy, Debug, Default, MallocSizeOf, PartialEq, ToShmem)]
pub struct Percentage {
    /// The percentage value as a float.
    ///
    /// [0 .. 100%] maps to [0.0 .. 1.0]
    value: CSSFloat,
    /// If this percentage came from a calc() expression, this tells how
    /// clamping should be done on the value.
    calc_clamping_mode: Option<AllowedNumericType>,
}

impl ToCss for Percentage {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.calc_clamping_mode.is_some() {
            dest.write_str("calc(")?;
        }

        serialize_percentage(self.value, dest)?;

        if self.calc_clamping_mode.is_some() {
            dest.write_str(")")?;
        }
        Ok(())
    }
}

impl Percentage {
    /// Creates a percentage from a numeric value.
    pub fn new(value: CSSFloat) -> Self {
        Self {
            value,
            calc_clamping_mode: None,
        }
    }

    /// `0%`
    #[inline]
    pub fn zero() -> Self {
        Percentage {
            value: 0.,
            calc_clamping_mode: None,
        }
    }

    /// `100%`
    #[inline]
    pub fn hundred() -> Self {
        Percentage {
            value: 1.,
            calc_clamping_mode: None,
        }
    }
    /// Gets the underlying value for this float.
    pub fn get(&self) -> CSSFloat {
        self.calc_clamping_mode
            .map_or(self.value, |mode| mode.clamp(self.value))
    }

    /// Returns whether this percentage is a `calc()` value.
    pub fn is_calc(&self) -> bool {
        self.calc_clamping_mode.is_some()
    }

    /// Reverses this percentage, preserving calc-ness.
    ///
    /// For example: If it was 20%, convert it into 80%.
    pub fn reverse(&mut self) {
        let new_value = 1. - self.value;
        self.value = new_value;
    }

    /// Parses a specific kind of percentage.
    pub fn parse_with_clamping_mode<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        num_context: AllowedNumericType,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        // FIXME: remove early returns when lifetimes are non-lexical
        match *input.next()? {
            Token::Percentage { unit_value, .. }
                if num_context.is_ok(context.parsing_mode, unit_value) =>
            {
                return Ok(Percentage::new(unit_value));
            }
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {},
            ref t => return Err(location.new_unexpected_token_error(t.clone())),
        }

        let result = input.parse_nested_block(|i| CalcNode::parse_percentage(context, i))?;

        // TODO(emilio): -moz-image-rect is the only thing that uses
        // the clamping mode... I guess we could disallow it...
        Ok(Percentage {
            value: result,
            calc_clamping_mode: Some(num_context),
        })
    }

    /// Parses a percentage token, but rejects it if it's negative.
    pub fn parse_non_negative<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_with_clamping_mode(context, input, AllowedNumericType::NonNegative)
    }

    /// Clamp to 100% if the value is over 100%.
    #[inline]
    pub fn clamp_to_hundred(self) -> Self {
        Percentage {
            value: self.value.min(1.),
            calc_clamping_mode: self.calc_clamping_mode,
        }
    }
}

impl Parse for Percentage {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_with_clamping_mode(context, input, AllowedNumericType::All)
    }
}

impl ToComputedValue for Percentage {
    type ComputedValue = ComputedPercentage;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> Self::ComputedValue {
        ComputedPercentage(self.get())
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Percentage::new(computed.0)
    }
}

impl SpecifiedValueInfo for Percentage {}
