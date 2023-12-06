/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified angles.

use crate::parser::{Parse, ParserContext};
use crate::values::computed::angle::Angle as ComputedAngle;
use crate::values::computed::{Context, ToComputedValue};
use crate::values::specified::calc::CalcNode;
use crate::values::CSSFloat;
use crate::Zero;
use cssparser::{Parser, Token};
use std::f32::consts::PI;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, SpecifiedValueInfo, ToCss};

/// A specified angle dimension.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, PartialOrd, ToCss, ToShmem)]
pub enum AngleDimension {
    /// An angle with degree unit.
    #[css(dimension)]
    Deg(CSSFloat),
    /// An angle with gradian unit.
    #[css(dimension)]
    Grad(CSSFloat),
    /// An angle with radian unit.
    #[css(dimension)]
    Rad(CSSFloat),
    /// An angle with turn unit.
    #[css(dimension)]
    Turn(CSSFloat),
}

impl Zero for AngleDimension {
    fn zero() -> Self {
        AngleDimension::Deg(0.)
    }

    fn is_zero(&self) -> bool {
        self.unitless_value() == 0.0
    }
}

impl AngleDimension {
    /// Returns the amount of degrees this angle represents.
    #[inline]
    fn degrees(&self) -> CSSFloat {
        const DEG_PER_RAD: f32 = 180.0 / PI;
        const DEG_PER_TURN: f32 = 360.0;
        const DEG_PER_GRAD: f32 = 180.0 / 200.0;

        match *self {
            AngleDimension::Deg(d) => d,
            AngleDimension::Rad(rad) => rad * DEG_PER_RAD,
            AngleDimension::Turn(turns) => turns * DEG_PER_TURN,
            AngleDimension::Grad(gradians) => gradians * DEG_PER_GRAD,
        }
    }

    fn unitless_value(&self) -> CSSFloat {
        match *self {
            AngleDimension::Deg(v) |
            AngleDimension::Rad(v) |
            AngleDimension::Turn(v) |
            AngleDimension::Grad(v) => v,
        }
    }

    fn unit(&self) -> &'static str {
        match *self {
            AngleDimension::Deg(_) => "deg",
            AngleDimension::Rad(_) => "rad",
            AngleDimension::Turn(_) => "turn",
            AngleDimension::Grad(_) => "grad",
        }
    }
}

/// A specified Angle value, which is just the angle dimension, plus whether it
/// was specified as `calc()` or not.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub struct Angle {
    value: AngleDimension,
    was_calc: bool,
}

impl Zero for Angle {
    fn zero() -> Self {
        Self {
            value: Zero::zero(),
            was_calc: false,
        }
    }

    fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
}

impl ToCss for Angle {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        crate::values::serialize_specified_dimension(
            self.value.unitless_value(),
            self.value.unit(),
            self.was_calc,
            dest,
        )
    }
}

impl ToComputedValue for Angle {
    type ComputedValue = ComputedAngle;

    #[inline]
    fn to_computed_value(&self, _context: &Context) -> Self::ComputedValue {
        let degrees = self.degrees();

        // NaN and +-infinity should degenerate to 0: https://github.com/w3c/csswg-drafts/issues/6105
        ComputedAngle::from_degrees(if degrees.is_finite() { degrees } else { 0.0 })
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Angle {
            value: AngleDimension::Deg(computed.degrees()),
            was_calc: false,
        }
    }
}

impl Angle {
    /// Creates an angle with the given value in degrees.
    #[inline]
    pub fn from_degrees(value: CSSFloat, was_calc: bool) -> Self {
        Angle {
            value: AngleDimension::Deg(value),
            was_calc,
        }
    }

    /// Creates an angle with the given value in radians.
    #[inline]
    pub fn from_radians(value: CSSFloat) -> Self {
        Angle {
            value: AngleDimension::Rad(value),
            was_calc: false,
        }
    }

    /// Return `0deg`.
    pub fn zero() -> Self {
        Self::from_degrees(0.0, false)
    }

    /// Returns the value of the angle in degrees, mostly for `calc()`.
    #[inline]
    pub fn degrees(&self) -> CSSFloat {
        self.value.degrees()
    }

    /// Returns the value of the angle in radians.
    #[inline]
    pub fn radians(&self) -> CSSFloat {
        const RAD_PER_DEG: f32 = PI / 180.0;
        self.value.degrees() * RAD_PER_DEG
    }

    /// Whether this specified angle came from a `calc()` expression.
    #[inline]
    pub fn was_calc(&self) -> bool {
        self.was_calc
    }

    /// Returns an `Angle` parsed from a `calc()` expression.
    pub fn from_calc(degrees: CSSFloat) -> Self {
        Angle {
            value: AngleDimension::Deg(degrees),
            was_calc: true,
        }
    }

    /// Returns the unit of the angle.
    #[inline]
    pub fn unit(&self) -> &'static str {
        self.value.unit()
    }
}

/// Whether to allow parsing an unitless zero as a valid angle.
///
/// This should always be `No`, except for exceptions like:
///
///   https://github.com/w3c/fxtf-drafts/issues/228
///
/// See also: https://github.com/w3c/csswg-drafts/issues/1162.
#[allow(missing_docs)]
pub enum AllowUnitlessZeroAngle {
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
    pub fn parse_dimension(value: CSSFloat, unit: &str, was_calc: bool) -> Result<Angle, ()> {
        let value = match_ignore_ascii_case! { unit,
            "deg" => AngleDimension::Deg(value),
            "grad" => AngleDimension::Grad(value),
            "turn" => AngleDimension::Turn(value),
            "rad" => AngleDimension::Rad(value),
             _ => return Err(())
        };

        Ok(Self { value, was_calc })
    }

    /// Parse an `<angle>` allowing unitless zero to represent a zero angle.
    ///
    /// See the comment in `AllowUnitlessZeroAngle` for why.
    #[inline]
    pub fn parse_with_unitless<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Self::parse_internal(context, input, AllowUnitlessZeroAngle::Yes)
    }

    pub(super) fn parse_internal<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        allow_unitless_zero: AllowUnitlessZeroAngle,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let t = input.next()?;
        let allow_unitless_zero = matches!(allow_unitless_zero, AllowUnitlessZeroAngle::Yes);
        match *t {
            Token::Dimension {
                value, ref unit, ..
            } => {
                match Angle::parse_dimension(value, unit, /* from_calc = */ false) {
                    Ok(angle) => Ok(angle),
                    Err(()) => {
                        let t = t.clone();
                        Err(input.new_unexpected_token_error(t))
                    },
                }
            },
            Token::Function(ref name) => {
                let function = CalcNode::math_function(context, name, location)?;
                CalcNode::parse_angle(context, input, function)
            },
            Token::Number { value, .. } if value == 0. && allow_unitless_zero => Ok(Angle::zero()),
            ref t => {
                let t = t.clone();
                Err(input.new_unexpected_token_error(t))
            },
        }
    }
}

impl SpecifiedValueInfo for Angle {}
