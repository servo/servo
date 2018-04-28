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
use values::generics::transform as generic;
use values::generics::transform::{Matrix, Matrix3D, StepPosition, TimingKeyword};
use values::specified::{self, Angle, Integer, Length, LengthOrPercentage, Number};
use values::specified::position::{Side, X, Y};

pub use values::generics::transform::TransformStyle;

/// A single operation in a specified CSS `transform`
pub type TransformOperation =
    generic::TransformOperation<Angle, Number, Length, Integer, LengthOrPercentage>;

/// A specified CSS `transform`
pub type Transform = generic::Transform<TransformOperation>;

/// The specified value of a CSS `<transform-origin>`
pub type TransformOrigin = generic::TransformOrigin<OriginComponent<X>, OriginComponent<Y>, Length>;

impl Transform {
    /// Internal parse function for deciding if we wish to accept prefixed values or not
    ///
    /// `transform` allows unitless zero angles as an exception, see:
    /// https://github.com/w3c/csswg-drafts/issues/1162
    fn parse_internal<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        use style_traits::{Separator, Space};

        if input
            .try(|input| input.expect_ident_matching("none"))
            .is_ok()
        {
            return Ok(generic::Transform(Vec::new()));
        }

        Ok(generic::Transform(Space::parse(input, |input| {
            let function = input.expect_function()?.clone();
            input.parse_nested_block(|input| {
                let location = input.current_source_location();
                let result = match_ignore_ascii_case! { &function,
                    "matrix" => {
                        let a = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let b = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let c = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let d = Number::parse(context, input)?;
                        input.expect_comma()?;
                        // Standard matrix parsing.
                        let e = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let f = Number::parse(context, input)?;
                        Ok(generic::TransformOperation::Matrix(Matrix { a, b, c, d, e, f }))
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
                        // Standard matrix3d parsing.
                        let m41 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m42 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m43 = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let m44 = Number::parse(context, input)?;
                        Ok(generic::TransformOperation::Matrix3D(Matrix3D {
                            m11, m12, m13, m14,
                            m21, m22, m23, m24,
                            m31, m32, m33, m34,
                            m41, m42, m43, m44,
                        }))
                    },
                    "translate" => {
                        let sx = specified::LengthOrPercentage::parse(context, input)?;
                        if input.try(|input| input.expect_comma()).is_ok() {
                            let sy = specified::LengthOrPercentage::parse(context, input)?;
                            Ok(generic::TransformOperation::Translate(sx, Some(sy)))
                        } else {
                            Ok(generic::TransformOperation::Translate(sx, None))
                        }
                    },
                    "translatex" => {
                        let tx = specified::LengthOrPercentage::parse(context, input)?;
                        Ok(generic::TransformOperation::TranslateX(tx))
                    },
                    "translatey" => {
                        let ty = specified::LengthOrPercentage::parse(context, input)?;
                        Ok(generic::TransformOperation::TranslateY(ty))
                    },
                    "translatez" => {
                        let tz = specified::Length::parse(context, input)?;
                        Ok(generic::TransformOperation::TranslateZ(tz))
                    },
                    "translate3d" => {
                        let tx = specified::LengthOrPercentage::parse(context, input)?;
                        input.expect_comma()?;
                        let ty = specified::LengthOrPercentage::parse(context, input)?;
                        input.expect_comma()?;
                        let tz = specified::Length::parse(context, input)?;
                        Ok(generic::TransformOperation::Translate3D(tx, ty, tz))
                    },
                    "scale" => {
                        let sx = Number::parse(context, input)?;
                        if input.try(|input| input.expect_comma()).is_ok() {
                            let sy = Number::parse(context, input)?;
                            Ok(generic::TransformOperation::Scale(sx, Some(sy)))
                        } else {
                            Ok(generic::TransformOperation::Scale(sx, None))
                        }
                    },
                    "scalex" => {
                        let sx = Number::parse(context, input)?;
                        Ok(generic::TransformOperation::ScaleX(sx))
                    },
                    "scaley" => {
                        let sy = Number::parse(context, input)?;
                        Ok(generic::TransformOperation::ScaleY(sy))
                    },
                    "scalez" => {
                        let sz = Number::parse(context, input)?;
                        Ok(generic::TransformOperation::ScaleZ(sz))
                    },
                    "scale3d" => {
                        let sx = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let sy = Number::parse(context, input)?;
                        input.expect_comma()?;
                        let sz = Number::parse(context, input)?;
                        Ok(generic::TransformOperation::Scale3D(sx, sy, sz))
                    },
                    "rotate" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(generic::TransformOperation::Rotate(theta))
                    },
                    "rotatex" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(generic::TransformOperation::RotateX(theta))
                    },
                    "rotatey" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(generic::TransformOperation::RotateY(theta))
                    },
                    "rotatez" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(generic::TransformOperation::RotateZ(theta))
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
                        Ok(generic::TransformOperation::Rotate3D(ax, ay, az, theta))
                    },
                    "skew" => {
                        let ax = specified::Angle::parse_with_unitless(context, input)?;
                        if input.try(|input| input.expect_comma()).is_ok() {
                            let ay = specified::Angle::parse_with_unitless(context, input)?;
                            Ok(generic::TransformOperation::Skew(ax, Some(ay)))
                        } else {
                            Ok(generic::TransformOperation::Skew(ax, None))
                        }
                    },
                    "skewx" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(generic::TransformOperation::SkewX(theta))
                    },
                    "skewy" => {
                        let theta = specified::Angle::parse_with_unitless(context, input)?;
                        Ok(generic::TransformOperation::SkewY(theta))
                    },
                    "perspective" => {
                        let d = specified::Length::parse_non_negative(context, input)?;
                        Ok(generic::TransformOperation::Perspective(d))
                    },
                    _ => Err(()),
                };
                result.map_err(|()| {
                    location
                        .new_custom_error(StyleParseErrorKind::UnexpectedFunction(function.clone()))
                })
            })
        })?))
    }
}

impl Parse for Transform {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Transform::parse_internal(context, input)
    }
}

/// The specified value of a component of a CSS `<transform-origin>`.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss)]
pub enum OriginComponent<S> {
    /// `center`
    Center,
    /// `<lop>`
    Length(LengthOrPercentage),
    /// `<side>`
    Side(S),
}

/// A specified timing function.
pub type TimingFunction = generic::TimingFunction<Integer, Number>;

impl Parse for TransformOrigin {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let parse_depth = |input: &mut Parser| {
            input
                .try(|i| Length::parse(context, i))
                .unwrap_or(Length::from_px(0.))
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
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
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
            OriginComponent::Center => {
                ComputedLengthOrPercentage::Percentage(ComputedPercentage(0.5))
            },
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
    unsafe { mozilla::StaticPrefs_sVarCache_layout_css_frames_timing_enabled }
}

#[cfg(feature = "servo")]
#[inline]
fn allow_frames_timing() -> bool {
    true
}

impl Parse for TimingFunction {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(keyword) = input.try(TimingKeyword::parse) {
            return Ok(generic::TimingFunction::Keyword(keyword));
        }
        if let Ok(ident) = input.try(|i| i.expect_ident_cloned()) {
            let position = match_ignore_ascii_case! { &ident,
                "step-start" => StepPosition::Start,
                "step-end" => StepPosition::End,
                _ => return Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(ident.clone()))),
            };
            return Ok(generic::TimingFunction::Steps(Integer::new(1), position));
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

                    Ok(generic::TimingFunction::CubicBezier { x1, y1, x2, y2 })
                },
                "steps" => {
                    let steps = Integer::parse_positive(context, i)?;
                    let position = i.try(|i| {
                        i.expect_comma()?;
                        StepPosition::parse(i)
                    }).unwrap_or(StepPosition::End);
                    Ok(generic::TimingFunction::Steps(steps, position))
                },
                "frames" => {
                    if allow_frames_timing() {
                        let frames = Integer::parse_with_minimum(context, i, 2)?;
                        Ok(generic::TimingFunction::Frames(frames))
                    } else {
                        Err(())
                    }
                },
                _ => Err(()),
            }).map_err(|()| {
                location.new_custom_error(StyleParseErrorKind::UnexpectedFunction(function.clone()))
            })
        })
    }
}

impl ToComputedValue for TimingFunction {
    type ComputedValue = ComputedTimingFunction;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            generic::TimingFunction::Keyword(keyword) => generic::TimingFunction::Keyword(keyword),
            generic::TimingFunction::CubicBezier { x1, y1, x2, y2 } => {
                generic::TimingFunction::CubicBezier {
                    x1: x1.to_computed_value(context),
                    y1: y1.to_computed_value(context),
                    x2: x2.to_computed_value(context),
                    y2: y2.to_computed_value(context),
                }
            },
            generic::TimingFunction::Steps(steps, position) => {
                generic::TimingFunction::Steps(steps.to_computed_value(context) as u32, position)
            },
            generic::TimingFunction::Frames(frames) => {
                generic::TimingFunction::Frames(frames.to_computed_value(context) as u32)
            },
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            generic::TimingFunction::Keyword(keyword) => generic::TimingFunction::Keyword(keyword),
            generic::TimingFunction::CubicBezier {
                ref x1,
                ref y1,
                ref x2,
                ref y2,
            } => generic::TimingFunction::CubicBezier {
                x1: Number::from_computed_value(x1),
                y1: Number::from_computed_value(y1),
                x2: Number::from_computed_value(x2),
                y2: Number::from_computed_value(y2),
            },
            generic::TimingFunction::Steps(steps, position) => generic::TimingFunction::Steps(
                Integer::from_computed_value(&(steps as i32)),
                position,
            ),
            generic::TimingFunction::Frames(frames) => {
                generic::TimingFunction::Frames(Integer::from_computed_value(&(frames as i32)))
            },
        }
    }
}

/// A specified CSS `rotate`
pub type Rotate = generic::Rotate<Number, Angle>;

impl Parse for Rotate {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(generic::Rotate::None);
        }

        if let Ok(rx) = input.try(|i| Number::parse(context, i)) {
            // 'rotate: <number>{3} <angle>'
            let ry = Number::parse(context, input)?;
            let rz = Number::parse(context, input)?;
            let angle = specified::Angle::parse(context, input)?;
            return Ok(generic::Rotate::Rotate3D(rx, ry, rz, angle));
        }

        // 'rotate: <angle>'
        let angle = specified::Angle::parse(context, input)?;
        Ok(generic::Rotate::Rotate(angle))
    }
}

/// A specified CSS `translate`
pub type Translate = generic::Translate<LengthOrPercentage, Length>;

impl Parse for Translate {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(generic::Translate::None);
        }

        let tx = specified::LengthOrPercentage::parse(context, input)?;
        if let Ok(ty) = input.try(|i| specified::LengthOrPercentage::parse(context, i)) {
            if let Ok(tz) = input.try(|i| specified::Length::parse(context, i)) {
                // 'translate: <length-percentage> <length-percentage> <length>'
                return Ok(generic::Translate::Translate3D(tx, ty, tz));
            }

            // translate: <length-percentage> <length-percentage>'
            return Ok(generic::Translate::Translate(tx, ty));
        }

        // 'translate: <length-percentage> '
        Ok(generic::Translate::TranslateX(tx))
    }
}

/// A specified CSS `scale`
pub type Scale = generic::Scale<Number>;

impl Parse for Scale {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(generic::Scale::None);
        }

        let sx = Number::parse(context, input)?;
        if let Ok(sy) = input.try(|i| Number::parse(context, i)) {
            if let Ok(sz) = input.try(|i| Number::parse(context, i)) {
                // 'scale: <number> <number> <number>'
                return Ok(generic::Scale::Scale3D(sx, sy, sz));
            }

            // 'scale: <number> <number>'
            return Ok(generic::Scale::Scale(sx, sy));
        }

        // 'scale: <number>'
        Ok(generic::Scale::ScaleX(sx))
    }
}
