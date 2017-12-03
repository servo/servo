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
        Angle { value: ComputedAngle::Deg(value), was_calc }
    }

    /// Creates an angle with the given value in gradians.
    pub fn from_gradians(value: CSSFloat, was_calc: bool) -> Self {
        Angle { value: ComputedAngle::Grad(value), was_calc }
    }

    /// Creates an angle with the given value in turns.
    pub fn from_turns(value: CSSFloat, was_calc: bool) -> Self {
        Angle { value: ComputedAngle::Turn(value), was_calc }
    }

    /// Creates an angle with the given value in radians.
    pub fn from_radians(value: CSSFloat, was_calc: bool) -> Self {
        Angle { value: ComputedAngle::Rad(value), was_calc }
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
            value: ComputedAngle::Rad(radians),
            was_calc: true,
        }
    }
}

impl AsRef<ComputedAngle> for Angle {
    #[inline]
    fn as_ref(&self) -> &ComputedAngle {
        &self.value
    }
}

/// Whether to allow parsing an unitless zero as a valid angle.
///
/// This should always be `No`, except for exceptions like:
///
///   https://github.com/w3c/fxtf-drafts/issues/228
///
/// See also: https://github.com/w3c/csswg-drafts/issues/1162.
enum AllowUnitlessZeroAngle {
    Yes,
    No,
}

impl Parse for Angle {
    /// Parses an angle according to CSS-VALUES § 6.1.
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, AllowUnitlessZeroAngle::No)
    }
}

impl Angle {
    /// Parse an `<angle>` value given a value and an unit.
    pub fn parse_dimension(
        value: CSSFloat,
        unit: &str,
        from_calc: bool,
    ) -> Result<Angle, ()> {
        let angle = match_ignore_ascii_case! { unit,
            "deg" => Angle::from_degrees(value, from_calc),
            "grad" => Angle::from_gradians(value, from_calc),
            "turn" => Angle::from_turns(value, from_calc),
            "rad" => Angle::from_radians(value, from_calc),
             _ => return Err(())
        };
        Ok(angle)
    }

    /// Parse an `<angle>` allowing unitless zero to represent a zero angle.
    ///
    /// See the comment in `AllowUnitlessZeroAngle` for why.
    pub fn parse_with_unitless<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, AllowUnitlessZeroAngle::Yes)
    }

    fn parse_internal<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_unitless_zero: AllowUnitlessZeroAngle,
    ) -> Result<Self, ParseError<'i>> {
        // FIXME: remove clone() when lifetimes are non-lexical
        let token = input.next()?.clone();
        match token {
            Token::Dimension { value, ref unit, .. } => {
                Angle::parse_dimension(value, unit, /* from_calc = */ false)
            }
            Token::Number { value, .. } if value == 0. => {
                match allow_unitless_zero {
                    AllowUnitlessZeroAngle::Yes => Ok(Angle::zero()),
                    AllowUnitlessZeroAngle::No => Err(()),
                }
            },
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                return input.parse_nested_block(|i| CalcNode::parse_angle(context, i))
            }
            _ => Err(())
        }.map_err(|()| input.new_unexpected_token_error(token.clone()))
    }
}
