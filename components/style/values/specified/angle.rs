/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified angles.

use cssparser::{Parser, Token};
use parser::{ParserContext, Parse};
#[allow(unused_imports)] use std::ascii::AsciiExt;
use std::fmt;
use style_traits::{ToCss, ParseError};
use values::CSSFloat;
use values::computed::{Context, ToComputedValue};
use values::computed::angle::Angle as ComputedAngle;
use values::specified::calc::CalcNode;

/// A specified angle.
///
/// Computed angles are essentially same as specified ones except for `calc()`
/// value serialization. Therefore we are storing a computed angle inside
/// to hold the actual value and its unit.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq)]
pub struct Angle {
    value: ComputedAngle,
    was_calc: bool,
}

impl ToCss for Angle {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if self.was_calc {
            dest.write_str("calc(")?;
        }
        self.value.to_css(dest)?;
        if self.was_calc {
            dest.write_str(")")?;
        }
        Ok(())
    }
}

impl ToComputedValue for Angle {
    type ComputedValue = ComputedAngle;

    fn to_computed_value(&self, _context: &Context) -> Self::ComputedValue {
        self.value
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Angle {
            value: *computed,
            was_calc: false,
        }
    }
}

impl Angle {
    /// Creates an angle with the given value in degrees.
    pub fn from_degrees(value: CSSFloat, was_calc: bool) -> Self {
        Angle { value: ComputedAngle::Degree(value), was_calc }
    }

    /// Creates an angle with the given value in gradians.
    pub fn from_gradians(value: CSSFloat, was_calc: bool) -> Self {
        Angle { value: ComputedAngle::Gradian(value), was_calc }
    }

    /// Creates an angle with the given value in turns.
    pub fn from_turns(value: CSSFloat, was_calc: bool) -> Self {
        Angle { value: ComputedAngle::Turn(value), was_calc }
    }

    /// Creates an angle with the given value in radians.
    pub fn from_radians(value: CSSFloat, was_calc: bool) -> Self {
        Angle { value: ComputedAngle::Radian(value), was_calc }
    }

    /// Returns the amount of radians this angle represents.
    #[inline]
    pub fn radians(self) -> f32 {
        self.value.radians()
    }

    /// Returns `0deg`.
    pub fn zero() -> Self {
        Self::from_degrees(0.0, false)
    }

    /// Returns an `Angle` parsed from a `calc()` expression.
    pub fn from_calc(radians: CSSFloat) -> Self {
        Angle {
            value: ComputedAngle::Radian(radians),
            was_calc: true,
        }
    }
}

impl Parse for Angle {
    /// Parses an angle according to CSS-VALUES § 6.1.
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // FIXME: remove clone() when lifetimes are non-lexical
        let token = input.next()?.clone();
        match token {
            Token::Dimension { value, ref unit, .. } => {
                Angle::parse_dimension(value, unit, /* from_calc = */ false)
            }
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                return input.parse_nested_block(|i| CalcNode::parse_angle(context, i))
            }
            _ => Err(())
        }.map_err(|()| input.new_unexpected_token_error(token.clone()))
    }
}

impl Angle {
    /// Parse an `<angle>` value given a value and an unit.
    pub fn parse_dimension(
        value: CSSFloat,
        unit: &str,
        from_calc: bool)
        -> Result<Angle, ()>
    {
        let angle = match_ignore_ascii_case! { unit,
            "deg" => Angle::from_degrees(value, from_calc),
            "grad" => Angle::from_gradians(value, from_calc),
            "turn" => Angle::from_turns(value, from_calc),
            "rad" => Angle::from_radians(value, from_calc),
             _ => return Err(())
        };
        Ok(angle)
    }

    /// Parse an angle, including unitless 0 degree.
    ///
    /// Note that numbers without any AngleUnit, including unitless 0 angle,
    /// should be invalid. However, some properties still accept unitless 0
    /// angle and stores it as '0deg'.
    ///
    /// We can remove this and get back to the unified version Angle::parse once
    /// https://github.com/w3c/csswg-drafts/issues/1162 is resolved.
    pub fn parse_with_unitless<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                       -> Result<Self, ParseError<'i>> {
        // FIXME: remove clone() when lifetimes are non-lexical
        let token = input.next()?.clone();
        match token {
            Token::Dimension { value, ref unit, .. } => {
                Angle::parse_dimension(value, unit, /* from_calc = */ false)
            }
            Token::Number { value, .. } if value == 0. => Ok(Angle::zero()),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                return input.parse_nested_block(|i| CalcNode::parse_angle(context, i))
            }
            _ => Err(())
        }.map_err(|()| input.new_unexpected_token_error(token.clone()))
    }
}
