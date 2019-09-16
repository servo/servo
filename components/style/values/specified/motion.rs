/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values that are related to motion path.

use crate::parser::{Parse, ParserContext};
use crate::values::computed::motion::OffsetRotate as ComputedOffsetRotate;
use crate::values::computed::{Context, ToComputedValue};
use crate::values::generics::motion::{GenericOffsetPath, RayFunction, RaySize};
use crate::values::specified::{Angle, SVGPathData};
use crate::Zero;
use cssparser::Parser;
use style_traits::{ParseError, StyleParseErrorKind};

/// The specified value of `offset-path`.
pub type OffsetPath = GenericOffsetPath<Angle>;

impl Parse for RayFunction<Angle> {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut angle = None;
        let mut size = None;
        let mut contain = false;
        loop {
            if angle.is_none() {
                angle = input.try(|i| Angle::parse(context, i)).ok();
            }

            if size.is_none() {
                size = input.try(RaySize::parse).ok();
                if size.is_some() {
                    continue;
                }
            }

            if !contain {
                contain = input.try(|i| i.expect_ident_matching("contain")).is_ok();
                if contain {
                    continue;
                }
            }
            break;
        }

        if angle.is_none() || size.is_none() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(RayFunction {
            angle: angle.unwrap(),
            size: size.unwrap(),
            contain,
        })
    }
}

impl Parse for OffsetPath {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // Parse none.
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(OffsetPath::none());
        }

        // Parse possible functions.
        let location = input.current_source_location();
        let function = input.expect_function()?.clone();
        input.parse_nested_block(move |i| {
            match_ignore_ascii_case! { &function,
                // Bug 1186329: Implement the parser for <basic-shape>, <geometry-box>,
                // and <url>.
                "path" => SVGPathData::parse(context, i).map(GenericOffsetPath::Path),
                "ray" => RayFunction::parse(context, i).map(GenericOffsetPath::Ray),
                _ => {
                    Err(location.new_custom_error(
                        StyleParseErrorKind::UnexpectedFunction(function.clone())
                    ))
                },
            }
        })
    }
}

/// The direction of offset-rotate.
#[derive(Clone, Copy, Debug, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
#[repr(u8)]
pub enum OffsetRotateDirection {
    /// Unspecified direction keyword.
    #[css(skip)]
    None,
    /// 0deg offset (face forward).
    Auto,
    /// 180deg offset (face backward).
    Reverse,
}

impl OffsetRotateDirection {
    /// Returns true if it is none (i.e. the keyword is not specified).
    #[inline]
    fn is_none(&self) -> bool {
        *self == OffsetRotateDirection::None
    }
}

#[inline]
fn direction_specified_and_angle_is_zero(direction: &OffsetRotateDirection, angle: &Angle) -> bool {
    !direction.is_none() && angle.is_zero()
}

/// The specified offset-rotate.
/// The syntax is: "[ auto | reverse ] || <angle>"
///
/// https://drafts.fxtf.org/motion-1/#offset-rotate-property
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub struct OffsetRotate {
    /// [auto | reverse].
    #[css(skip_if = "OffsetRotateDirection::is_none")]
    direction: OffsetRotateDirection,
    /// <angle>.
    /// If direction is None, this is a fixed angle which indicates a
    /// constant clockwise rotation transformation applied to it by this
    /// specified rotation angle. Otherwise, the angle will be added to
    /// the angle of the direction in layout.
    #[css(contextual_skip_if = "direction_specified_and_angle_is_zero")]
    angle: Angle,
}

impl Parse for OffsetRotate {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let mut direction = input.try(OffsetRotateDirection::parse);
        let angle = input.try(|i| Angle::parse(context, i));
        if direction.is_err() {
            // The direction and angle could be any order, so give it a change to parse
            // direction again.
            direction = input.try(OffsetRotateDirection::parse);
        }

        if direction.is_err() && angle.is_err() {
            return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(OffsetRotate {
            direction: direction.unwrap_or(OffsetRotateDirection::None),
            angle: angle.unwrap_or(Zero::zero()),
        })
    }
}

impl ToComputedValue for OffsetRotate {
    type ComputedValue = ComputedOffsetRotate;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        use crate::values::computed::Angle as ComputedAngle;

        ComputedOffsetRotate {
            auto: !self.direction.is_none(),
            angle: if self.direction == OffsetRotateDirection::Reverse {
                // The computed value should always convert "reverse" into "auto".
                // e.g. "reverse calc(20deg + 10deg)" => "auto 210deg"
                self.angle.to_computed_value(context) + ComputedAngle::from_degrees(180.0)
            } else {
                self.angle.to_computed_value(context)
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        OffsetRotate {
            direction: if computed.auto {
                OffsetRotateDirection::Auto
            } else {
                OffsetRotateDirection::None
            },
            angle: ToComputedValue::from_computed_value(&computed.angle),
        }
    }
}
