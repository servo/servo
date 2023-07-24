/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values that are related to transformations.

use crate::parser::{Parse, ParserContext};
use crate::values::computed::{Context, LengthPercentage as ComputedLengthPercentage};
use crate::values::computed::{Percentage as ComputedPercentage, ToComputedValue};
use crate::values::generics::transform as generic;
use crate::values::generics::transform::{Matrix, Matrix3D};
use crate::values::specified::position::{
    HorizontalPositionKeyword, Side, VerticalPositionKeyword,
};
use crate::values::specified::{
    self, Angle, Integer, Length, LengthPercentage, Number, NumberOrPercentage,
};
use crate::Zero;
use cssparser::Parser;
use style_traits::{ParseError, StyleParseErrorKind};

pub use crate::values::generics::transform::TransformStyle;

/// A single operation in a specified CSS `transform`
pub type TransformOperation =
    generic::TransformOperation<Angle, Number, Length, Integer, LengthPercentage>;

/// A specified CSS `transform`
pub type Transform = generic::Transform<TransformOperation>;

/// The specified value of a CSS `<transform-origin>`
pub type TransformOrigin = generic::TransformOrigin<
    OriginComponent<HorizontalPositionKeyword>,
    OriginComponent<VerticalPositionKeyword>,
    Length,
>;

impl TransformOrigin {
    /// Returns the initial specified value for `transform-origin`.
    #[inline]
    pub fn initial_value() -> Self {
        Self::new(
            OriginComponent::Length(LengthPercentage::Percentage(ComputedPercentage(0.5))),
            OriginComponent::Length(LengthPercentage::Percentage(ComputedPercentage(0.5))),
            Length::zero(),
        )
    }

    /// Returns the `0 0` value.
    pub fn zero_zero() -> Self {
        Self::new(
            OriginComponent::Length(LengthPercentage::zero()),
            OriginComponent::Length(LengthPercentage::zero()),
            Length::zero(),
        )
    }
}

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
            .try_parse(|input| input.expect_ident_matching("none"))
            .is_ok()
        {
            return Ok(generic::Transform::none());
        }

        Ok(generic::Transform(
            Space::parse(input, |input| {
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
                            let sx = specified::LengthPercentage::parse(context, input)?;
                            if input.try_parse(|input| input.expect_comma()).is_ok() {
                                let sy = specified::LengthPercentage::parse(context, input)?;
                                Ok(generic::TransformOperation::Translate(sx, sy))
                            } else {
                                Ok(generic::TransformOperation::Translate(sx, Zero::zero()))
                            }
                        },
                        "translatex" => {
                            let tx = specified::LengthPercentage::parse(context, input)?;
                            Ok(generic::TransformOperation::TranslateX(tx))
                        },
                        "translatey" => {
                            let ty = specified::LengthPercentage::parse(context, input)?;
                            Ok(generic::TransformOperation::TranslateY(ty))
                        },
                        "translatez" => {
                            let tz = specified::Length::parse(context, input)?;
                            Ok(generic::TransformOperation::TranslateZ(tz))
                        },
                        "translate3d" => {
                            let tx = specified::LengthPercentage::parse(context, input)?;
                            input.expect_comma()?;
                            let ty = specified::LengthPercentage::parse(context, input)?;
                            input.expect_comma()?;
                            let tz = specified::Length::parse(context, input)?;
                            Ok(generic::TransformOperation::Translate3D(tx, ty, tz))
                        },
                        "scale" => {
                            let sx = NumberOrPercentage::parse(context, input)?.to_number();
                            if input.try_parse(|input| input.expect_comma()).is_ok() {
                                let sy = NumberOrPercentage::parse(context, input)?.to_number();
                                Ok(generic::TransformOperation::Scale(sx, sy))
                            } else {
                                Ok(generic::TransformOperation::Scale(sx, sx))
                            }
                        },
                        "scalex" => {
                            let sx = NumberOrPercentage::parse(context, input)?.to_number();
                            Ok(generic::TransformOperation::ScaleX(sx))
                        },
                        "scaley" => {
                            let sy = NumberOrPercentage::parse(context, input)?.to_number();
                            Ok(generic::TransformOperation::ScaleY(sy))
                        },
                        "scalez" => {
                            let sz = NumberOrPercentage::parse(context, input)?.to_number();
                            Ok(generic::TransformOperation::ScaleZ(sz))
                        },
                        "scale3d" => {
                            let sx = NumberOrPercentage::parse(context, input)?.to_number();
                            input.expect_comma()?;
                            let sy = NumberOrPercentage::parse(context, input)?.to_number();
                            input.expect_comma()?;
                            let sz = NumberOrPercentage::parse(context, input)?.to_number();
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
                            if input.try_parse(|input| input.expect_comma()).is_ok() {
                                let ay = specified::Angle::parse_with_unitless(context, input)?;
                                Ok(generic::TransformOperation::Skew(ax, ay))
                            } else {
                                Ok(generic::TransformOperation::Skew(ax, Zero::zero()))
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
                            let p = match input.try_parse(|input| specified::Length::parse_non_negative(context, input)) {
                                Ok(p) => generic::PerspectiveFunction::Length(p),
                                Err(..) => {
                                    input.expect_ident_matching("none")?;
                                    generic::PerspectiveFunction::None
                                }
                            };
                            Ok(generic::TransformOperation::Perspective(p))
                        },
                        _ => Err(()),
                    };
                    result.map_err(|()| {
                        location.new_custom_error(StyleParseErrorKind::UnexpectedFunction(
                            function.clone(),
                        ))
                    })
                })
            })?
            .into(),
        ))
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
#[derive(Clone, Debug, MallocSizeOf, Parse, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub enum OriginComponent<S> {
    /// `center`
    Center,
    /// `<length-percentage>`
    Length(LengthPercentage),
    /// `<side>`
    Side(S),
}

impl Parse for TransformOrigin {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let parse_depth = |input: &mut Parser| {
            input
                .try_parse(|i| Length::parse(context, i))
                .unwrap_or(Length::zero())
        };
        match input.try_parse(|i| OriginComponent::parse(context, i)) {
            Ok(x_origin @ OriginComponent::Center) => {
                if let Ok(y_origin) = input.try_parse(|i| OriginComponent::parse(context, i)) {
                    let depth = parse_depth(input);
                    return Ok(Self::new(x_origin, y_origin, depth));
                }
                let y_origin = OriginComponent::Center;
                if let Ok(x_keyword) = input.try_parse(HorizontalPositionKeyword::parse) {
                    let x_origin = OriginComponent::Side(x_keyword);
                    let depth = parse_depth(input);
                    return Ok(Self::new(x_origin, y_origin, depth));
                }
                let depth = Length::from_px(0.);
                return Ok(Self::new(x_origin, y_origin, depth));
            },
            Ok(x_origin) => {
                if let Ok(y_origin) = input.try_parse(|i| OriginComponent::parse(context, i)) {
                    let depth = parse_depth(input);
                    return Ok(Self::new(x_origin, y_origin, depth));
                }
                let y_origin = OriginComponent::Center;
                let depth = Length::from_px(0.);
                return Ok(Self::new(x_origin, y_origin, depth));
            },
            Err(_) => {},
        }
        let y_keyword = VerticalPositionKeyword::parse(input)?;
        let y_origin = OriginComponent::Side(y_keyword);
        if let Ok(x_keyword) = input.try_parse(HorizontalPositionKeyword::parse) {
            let x_origin = OriginComponent::Side(x_keyword);
            let depth = parse_depth(input);
            return Ok(Self::new(x_origin, y_origin, depth));
        }
        if input
            .try_parse(|i| i.expect_ident_matching("center"))
            .is_ok()
        {
            let x_origin = OriginComponent::Center;
            let depth = parse_depth(input);
            return Ok(Self::new(x_origin, y_origin, depth));
        }
        let x_origin = OriginComponent::Center;
        let depth = Length::from_px(0.);
        Ok(Self::new(x_origin, y_origin, depth))
    }
}

impl<S> ToComputedValue for OriginComponent<S>
where
    S: Side,
{
    type ComputedValue = ComputedLengthPercentage;

    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            OriginComponent::Center => {
                ComputedLengthPercentage::new_percent(ComputedPercentage(0.5))
            },
            OriginComponent::Length(ref length) => length.to_computed_value(context),
            OriginComponent::Side(ref keyword) => {
                let p = ComputedPercentage(if keyword.is_start() { 0. } else { 1. });
                ComputedLengthPercentage::new_percent(p)
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
        OriginComponent::Length(LengthPercentage::Percentage(ComputedPercentage::zero()))
    }
}

/// A specified CSS `rotate`
pub type Rotate = generic::Rotate<Number, Angle>;

impl Parse for Rotate {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try_parse(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(generic::Rotate::None);
        }

        // Parse <angle> or [ x | y | z | <number>{3} ] && <angle>.
        //
        // The rotate axis and angle could be in any order, so we parse angle twice to cover
        // two cases. i.e. `<number>{3} <angle>` or `<angle> <number>{3}`
        let angle = input
            .try_parse(|i| specified::Angle::parse(context, i))
            .ok();
        let axis = input
            .try_parse(|i| {
                Ok(try_match_ident_ignore_ascii_case! { i,
                    "x" => (Number::new(1.), Number::new(0.), Number::new(0.)),
                    "y" => (Number::new(0.), Number::new(1.), Number::new(0.)),
                    "z" => (Number::new(0.), Number::new(0.), Number::new(1.)),
                })
            })
            .or_else(|_: ParseError| -> Result<_, ParseError> {
                input.try_parse(|i| {
                    Ok((
                        Number::parse(context, i)?,
                        Number::parse(context, i)?,
                        Number::parse(context, i)?,
                    ))
                })
            })
            .ok();
        let angle = match angle {
            Some(a) => a,
            None => specified::Angle::parse(context, input)?,
        };

        Ok(match axis {
            Some((x, y, z)) => generic::Rotate::Rotate3D(x, y, z, angle),
            None => generic::Rotate::Rotate(angle),
        })
    }
}

/// A specified CSS `translate`
pub type Translate = generic::Translate<LengthPercentage, Length>;

impl Parse for Translate {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try_parse(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(generic::Translate::None);
        }

        let tx = specified::LengthPercentage::parse(context, input)?;
        if let Ok(ty) = input.try_parse(|i| specified::LengthPercentage::parse(context, i)) {
            if let Ok(tz) = input.try_parse(|i| specified::Length::parse(context, i)) {
                // 'translate: <length-percentage> <length-percentage> <length>'
                return Ok(generic::Translate::Translate(tx, ty, tz));
            }

            // translate: <length-percentage> <length-percentage>'
            return Ok(generic::Translate::Translate(
                tx,
                ty,
                specified::Length::zero(),
            ));
        }

        // 'translate: <length-percentage> '
        Ok(generic::Translate::Translate(
            tx,
            specified::LengthPercentage::zero(),
            specified::Length::zero(),
        ))
    }
}

/// A specified CSS `scale`
pub type Scale = generic::Scale<Number>;

impl Parse for Scale {
    /// Scale accepts <number> | <percentage>, so we parse it as NumberOrPercentage,
    /// and then convert into an Number if it's a Percentage.
    /// https://github.com/w3c/csswg-drafts/pull/4396
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try_parse(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(generic::Scale::None);
        }

        let sx = NumberOrPercentage::parse(context, input)?.to_number();
        if let Ok(sy) = input.try_parse(|i| NumberOrPercentage::parse(context, i)) {
            let sy = sy.to_number();
            if let Ok(sz) = input.try_parse(|i| NumberOrPercentage::parse(context, i)) {
                // 'scale: <number> <number> <number>'
                return Ok(generic::Scale::Scale(sx, sy, sz.to_number()));
            }

            // 'scale: <number> <number>'
            return Ok(generic::Scale::Scale(sx, sy, Number::new(1.0)));
        }

        // 'scale: <number>'
        Ok(generic::Scale::Scale(sx, sx, Number::new(1.0)))
    }
}
