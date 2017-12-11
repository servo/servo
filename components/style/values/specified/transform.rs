/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values that are related to transformations.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseErrorKind;
use style_traits::{ParseError, StyleParseErrorKind};
use values::computed::{Context, LengthOrPercentage as ComputedLengthOrPercentage};
use values::computed::{Percentage as ComputedPercentage, ToComputedValue};
use values::computed::transform::TimingFunction as ComputedTimingFunction;
use values::generics::transform::{Matrix3D, Transform as GenericTransform};
use values::generics::transform::{StepPosition, TimingFunction as GenericTimingFunction, Matrix};
use values::generics::transform::{TimingKeyword, TransformOrigin as GenericTransformOrigin};
use values::generics::transform::TransformOperation as GenericTransformOperation;
use values::specified::{self, Angle, Number, Length, Integer};
use values::specified::{LengthOrNumber, LengthOrPercentage, LengthOrPercentageOrNumber};
use values::specified::position::{Side, X, Y};

/// A single operation in a specified CSS `transform`
pub type TransformOperation = GenericTransformOperation<
    Angle,
    Number,
    Length,
    Integer,
    LengthOrNumber,
    LengthOrPercentage,
    LengthOrPercentageOrNumber,
>;

/// A specified CSS `transform`
pub type Transform = GenericTransform<TransformOperation>;

/// The specified value of a CSS `<transform-origin>`
pub type TransformOrigin = GenericTransformOrigin<OriginComponent<X>, OriginComponent<Y>, Length>;

impl Transform {
    /// Internal parse function for deciding if we wish to accept prefixed values or not
    ///
    /// `transform` allows unitless zero angles as an exception, see:
    /// https://github.com/w3c/csswg-drafts/issues/1162
    fn parse_internal<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        prefixed: bool,
    ) -> Result<Self, ParseError<'i>> {
        use style_traits::{Separator, Space};

        if input
            .try(|input| input.expect_ident_matching("none"))
            .is_ok()
        {
            return Ok(GenericTransform(Vec::new()));
        }

        Ok(GenericTransform(Space::parse(input, |input| {
            let function = input.expect_function()?.clone();
            input.parse_nested_block(|input| {
                let location = input.current_source_location();
                let result =
                    match_ignore_ascii_case! { &function,
                    "matrix" => {
                        let a = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let b = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let c = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let d = Number::parse(context, input)?;
                        input.expect_comma()?;
                        if !prefixed {
                            // Standard matrix parsing.
                            let e = Number::parse(context, input)?;
                            input.expect_comma()?;
                            let f = Number::parse(context, input)?;
                            Ok(GenericTransformOperation::Matrix(Matrix { a, b, c, d, e, f }))
                        } else {
                            // Non-standard prefixed matrix parsing for -moz-transform.
                            let e = LengthOrPercentageOrNumber::parse(context, input)?;
                            input.expect_comma()?;
                            let f = LengthOrPercentageOrNumber::parse(context, input)?;
                            Ok(GenericTransformOperation::PrefixedMatrix(Matrix { a, b, c, d, e, f }))
                        }
                    },
                    "matrix3d" => {
                        let m11 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m12 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m13 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m14 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m21 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m22 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m23 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m24 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m31 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m32 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m33 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m34 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        if !prefixed {
                            // Standard matrix3d parsing.
                            let m41 = Number::parse(context, input)?;
                            input.expect_comma()?;
                            let m42 = Number::parse(context, input)?;
                            input.expect_comma()?;
                            let m43 = Number::parse(context, input)?;
                            input.expect_comma()?;
                            let m44 = Number::parse(context, input)?;
                            Ok(GenericTransformOperation::Matrix3D(Matrix3D {
                                m11, m12, m13, m14,
                                m21, m22, m23, m24,
                                m31, m32, m33, m34,
                                m41, m42, m43, m44,
                            }))
                        } else {
                            // Non-standard prefixed matrix parsing for -moz-transform.
                            let m41 = LengthOrPercentageOrNumber::parse(context, input)?;
                            input.expect_comma()?;
                            let m42 = LengthOrPercentageOrNumber::parse(context, input)?;
                            input.expect_comma()?;
                            let m43 = LengthOrNumber::parse(context, input)?;
                            input.expect_comma()?;
                            let m44 = Number::parse(context, input)?;
                            Ok(GenericTransformOperation::PrefixedMatrix3D(Matrix3D {
                                m11, m12, m13, m14,
                                m21, m22, m23, m24,
                                m31, m32, m33, m34,
                                m41, m42, m43, m44,
                            }))
                        }
                    },
                    "translate" => {
                        let sx = specified::LengthOrPercentage::parse(context, input)?;
                        if input.try(|input| input.expect_comma()).is_ok() {
                            let sy = specified::LengthOrPercentage::parse(context, input)?;
                            Ok(GenericTransformOperation::Translate(sx, Some(sy)))
                        } else {
                            Ok(GenericTransformOperation::Translate(sx, None))
                        }
                    },
                    "translatex" => {
                        let tx = specified::LengthOrPercentage::parse(context, input)?;
                        Ok(GenericTransformOperation::TranslateX(tx))
                    },
                    "translatey" => {
                        let ty = specified::LengthOrPercentage::parse(context, input)?;
                        Ok(GenericTransformOperation::TranslateY(ty))
                    },
                    "translatez" => {
                        let tz = specified::Length::parse(context, input)?;
                        Ok(GenericTransformOperation::TranslateZ(tz))
                    },
                    "translate3d" => {
                        let tx = specified::LengthOrPercentage::parse(context, input)?;
                        input.expect_comma()?;
                        let ty = specified::LengthOrPercentage::parse(context, input)?;
                        input.expect_comma()?;
                        let tz = specified::Length::parse(context, input)?;
                        Ok(GenericTransformOperation::Translate3D(tx, ty, tz))
                    },
                    "scale" => {
                        let sx = Number::parse(context, input)?;
                        if input.try(|input| input.expect_comma()).is_ok() {
                            let sy = Number::parse(context, input)?;
                            Ok(GenericTransformOperation::Scale(sx, Some(sy)))
                        } else {
                            Ok(GenericTransformOperation::Scale(sx, None))
                        }
                    },
                    "scalex" => {
                        let sx = Number::parse(context, input)?;
                        Ok(GenericTransformOperation::ScaleX(sx))
                    },
                    "scaley" => {
                        let sy = Number::parse(context, input)?;
                        Ok(GenericTransformOperation::ScaleY(sy))
                    },
                    "scalez" => {
                        let sz = Number::parse(context, input)?;
                        Ok(GenericTransformOperation::ScaleZ(sz))
                    },
                    "scale3d" => {
                        let sx = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let sy = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let sz = Number::parse(context, input)?;
                        Ok(GenericTransformOperation::Scale3D(sx, sy, sz))
                    },
                    "rotate" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(GenericTransformOperation::Rotate(theta))
                    },
                    "rotatex" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(GenericTransformOperation::RotateX(theta))
                    },
                    "rotatey" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(GenericTransformOperation::RotateY(theta))
                    },
                    "rotatez" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(GenericTransformOperation::RotateZ(theta))
                    },
                    "rotate3d" => {
                        let ax = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let ay = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let az = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        // TODO(gw): Check that the axis can be normalized.
                        Ok(GenericTransformOperation::Rotate3D(ax, ay, az, theta))
                    },
                    "skew" => {
                        let ax = specified::Angle::parse_with_unitless(context, input)?;
                        if input.try(|input| input.expect_comma()).is_ok() {
                            let ay = specified::Angle::parse_with_unitless(context, input)?;
                            Ok(GenericTransformOperation::Skew(ax, Some(ay)))
                        } else {
                            Ok(GenericTransformOperation::Skew(ax, None))
                        }
                    },
                    "skewx" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(GenericTransformOperation::SkewX(theta))
                    },
                    "skewy" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(GenericTransformOperation::SkewY(theta))
                    },
                    "perspective" => {
                        let d = specified::Length::parse_non_negative(context, input)?;
                        Ok(GenericTransformOperation::Perspective(d))
                    },
                    _ => Err(()),
                };
                result
                    .map_err(|()| location.new_custom_error(StyleParseErrorKind::UnexpectedFunction(function.clone())))
            })
        })?))
    }

    /// Parses `-moz-transform` property. This prefixed property also accepts LengthOrPercentage
    /// in the nondiagonal homogeneous components of matrix and matrix3d.
    #[inline]
    pub fn parse_prefixed<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Transform::parse_internal(context, input, true)
    }
}

impl Parse for Transform {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        Transform::parse_internal(context, input, false)
    }
}

/// The specified value of a component of a CSS `<transform-origin>`.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
pub enum OriginComponent<S> {
    /// `center`
    Center,
    /// `<lop>`
    Length(LengthOrPercentage),
    /// `<side>`
    Side(S),
}

/// A specified timing function.
pub type TimingFunction = GenericTimingFunction<Integer, Number>;

impl Parse for TransformOrigin {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        let parse_depth = |input: &mut Parser| {
            input.try(|i| Length::parse(context, i)).unwrap_or(
                Length::from_px(0.),
            )
        };
        match input.try(|i| OriginComponent::parse(context, i)) {
            Ok(x_origin @ OriginComponent::Center) => {
                if let Ok(y_origin) = input.try(|i| OriginComponent::parse(context, i)) {
                    let depth = parse_depth(input);
                    return Ok(Self::new(x_origin, y_origin, depth));
                }
                let y_origin = OriginComponent::Center;
                if let Ok(x_keyword) = input.try(X::parse) {
                    let x_origin = OriginComponent::Side(x_keyword);
                    let depth = parse_depth(input);
                    return Ok(Self::new(x_origin, y_origin, depth));
                }
                let depth = Length::from_px(0.);
                return Ok(Self::new(x_origin, y_origin, depth));
            },
            Ok(x_origin) => {
                if let Ok(y_origin) = input.try(|i| OriginComponent::parse(context, i)) {
                    let depth = parse_depth(input);
                    return Ok(Self::new(x_origin, y_origin, depth));
                }
                let y_origin = OriginComponent::Center;
                let depth = Length::from_px(0.);
                return Ok(Self::new(x_origin, y_origin, depth));
            },
            Err(_) => {},
        }
        let y_keyword = Y::parse(input)?;
        let y_origin = OriginComponent::Side(y_keyword);
        if let Ok(x_keyword) = input.try(X::parse) {
            let x_origin = OriginComponent::Side(x_keyword);
            let depth = parse_depth(input);
            return Ok(Self::new(x_origin, y_origin, depth));
        }
        if input.try(|i| i.expect_ident_matching("center")).is_ok() {
            let x_origin = OriginComponent::Center;
            let depth = parse_depth(input);
            return Ok(Self::new(x_origin, y_origin, depth));
        }
        let x_origin = OriginComponent::Center;
        let depth = Length::from_px(0.);
        Ok(Self::new(x_origin, y_origin, depth))
    }
}

impl<S> Parse for OriginComponent<S>
where
    S: Parse,
{
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("center")).is_ok() {
            return Ok(OriginComponent::Center);
        }
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse(context, i)) {
            return Ok(OriginComponent::Length(lop));
        }
        let keyword = S::parse(context, input)?;
        Ok(OriginComponent::Side(keyword))
    }
}

impl<S> ToComputedValue for OriginComponent<S>
where
    S: Side,
{
    type ComputedValue = ComputedLengthOrPercentage;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            OriginComponent::Center => ComputedLengthOrPercentage::Percentage(ComputedPercentage(0.5)),
            OriginComponent::Length(ref length) => length.to_computed_value(context),
            OriginComponent::Side(ref keyword) => {
                let p = ComputedPercentage(if keyword.is_start() { 0. } else { 1. });
                ComputedLengthOrPercentage::Percentage(p)
            },
        }
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        OriginComponent::Length(ToComputedValue::from_computed_value(computed))
    }
}

impl<S> OriginComponent<S> {
    /// `0%`
    pub fn zero() -> Self {
        OriginComponent::Length(LengthOrPercentage::Percentage(ComputedPercentage::zero()))
    }
}

#[cfg(feature = "gecko")]
#[inline]
fn allow_frames_timing() -> bool {
    use gecko_bindings::structs::mozilla;
    unsafe { mozilla::StylePrefs_sFramesTimingFunctionEnabled }
}

#[cfg(feature = "servo")]
#[inline]
fn allow_frames_timing() -> bool {
    true
}

impl Parse for TimingFunction {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(keyword) = input.try(TimingKeyword::parse) {
            return Ok(GenericTimingFunction::Keyword(keyword));
        }
        if let Ok(ident) = input.try(|i| i.expect_ident_cloned()) {
            let position =
                match_ignore_ascii_case! { &ident,
                "step-start" => StepPosition::Start,
                "step-end" => StepPosition::End,
                _ => return Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone()))),
            };
            return Ok(GenericTimingFunction::Steps(Integer::new(1), position));
        }
        let location = input.current_source_location();
        let function = input.expect_function()?.clone();
        input.parse_nested_block(move |i| {
            (match_ignore_ascii_case! { &function,
                "cubic-bezier" => {
                    let x1 = Number::parse(context, i)?;
                    i.expect_comma()?;
                    let y1 = Number::parse(context, i)?;
                    i.expect_comma()?;
                    let x2 = Number::parse(context, i)?;
                    i.expect_comma()?;
                    let y2 = Number::parse(context, i)?;

                    if x1.get() < 0.0 || x1.get() > 1.0 || x2.get() < 0.0 || x2.get() > 1.0 {
                        return Err(i.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                    }

                    Ok(GenericTimingFunction::CubicBezier { x1, y1, x2, y2 })
                },
                "steps" => {
                    let steps = Integer::parse_positive(context, i)?;
                    let position = i.try(|i| {
                        i.expect_comma()?;
                        StepPosition::parse(i)
                    }).unwrap_or(StepPosition::End);
                    Ok(GenericTimingFunction::Steps(steps, position))
                },
                "frames" => {
                    if allow_frames_timing() {
                        let frames = Integer::parse_with_minimum(context, i, 2)?;
                        Ok(GenericTimingFunction::Frames(frames))
                    } else {
                        Err(())
                    }
                },
                _ => Err(()),
            }).map_err(|()| location.new_custom_error(StyleParseErrorKind::UnexpectedFunction(function.clone())))
        })
    }
}

impl ToComputedValue for TimingFunction {
    type ComputedValue = ComputedTimingFunction;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            GenericTimingFunction::Keyword(keyword) => GenericTimingFunction::Keyword(keyword),
            GenericTimingFunction::CubicBezier {
                x1,
                y1,
                x2,
                y2,
            } => {
                GenericTimingFunction::CubicBezier {
                    x1: x1.to_computed_value(context),
                    y1: y1.to_computed_value(context),
                    x2: x2.to_computed_value(context),
                    y2: y2.to_computed_value(context),
                }
            },
            GenericTimingFunction::Steps(steps, position) => {
                GenericTimingFunction::Steps(steps.to_computed_value(context) as u32, position)
            },
            GenericTimingFunction::Frames(frames) => {
                GenericTimingFunction::Frames(frames.to_computed_value(context) as u32)
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            GenericTimingFunction::Keyword(keyword) => GenericTimingFunction::Keyword(keyword),
            GenericTimingFunction::CubicBezier {
                ref x1,
                ref y1,
                ref x2,
                ref y2,
            } => {
                GenericTimingFunction::CubicBezier {
                    x1: Number::from_computed_value(x1),
                    y1: Number::from_computed_value(y1),
                    x2: Number::from_computed_value(x2),
                    y2: Number::from_computed_value(y2),
                }
            },
            GenericTimingFunction::Steps(steps, position) => {
                GenericTimingFunction::Steps(Integer::from_computed_value(&(steps as i32)), position)
            },
            GenericTimingFunction::Frames(frames) => {
                GenericTimingFunction::Frames(Integer::from_computed_value(&(frames as i32)))
            },
        }
    }
}
