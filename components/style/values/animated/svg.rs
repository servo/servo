/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animation implementations for various SVG-related types.

use properties::animated_properties::ListAnimation;
use values::animated::color::Color as AnimatedColor;
use values::computed::{NonNegativeNumber, Number, NumberOrPercentage, LengthOrPercentage};
use values::computed::url::ComputedUrl;
use values::computed::length::NonNegativeLengthOrPercentage;
use values::distance::{ComputeSquaredDistance, SquaredDistance};
use values::generics::svg::{SVGLength,  SvgLengthOrPercentageOrNumber, SVGPaint};
use values::generics::svg::{SVGPaintKind, SVGStrokeDashArray, SVGOpacity};
use super::{Animate, Procedure, ToAnimatedZero};

/// Animated SVGPaint.
pub type IntermediateSVGPaint = SVGPaint<AnimatedColor, ComputedUrl>;

impl ToAnimatedZero for IntermediateSVGPaint {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(IntermediateSVGPaint {
            kind: self.kind.to_animated_zero()?,
            fallback: self.fallback.and_then(|v| v.to_animated_zero().ok()),
        })
    }
}

impl From<NonNegativeLengthOrPercentage> for NumberOrPercentage {
    fn from(lop: NonNegativeLengthOrPercentage) -> NumberOrPercentage {
        lop.0.into()
    }
}

impl From<NonNegativeNumber> for NumberOrPercentage {
    fn from(num: NonNegativeNumber) -> NumberOrPercentage {
        num.0.into()
    }
}

impl From<LengthOrPercentage> for NumberOrPercentage {
    fn from(lop: LengthOrPercentage) -> NumberOrPercentage {
        match lop {
            LengthOrPercentage::Length(len) => NumberOrPercentage::Number(len.px()),
            LengthOrPercentage::Percentage(p) => NumberOrPercentage::Percentage(p),
            LengthOrPercentage::Calc(_) => {
                panic!("We dont't expected calc interpolation for SvgLengthOrPercentageOrNumber");
            },
        }
    }
}

impl From<Number> for NumberOrPercentage {
    fn from(num: Number) -> NumberOrPercentage {
        NumberOrPercentage::Number(num)
    }
}

fn convert_to_number_or_percentage<LengthOrPercentageType, NumberType>(
    from: SvgLengthOrPercentageOrNumber<LengthOrPercentageType, NumberType>,
) -> NumberOrPercentage
where
    LengthOrPercentageType: Into<NumberOrPercentage>,
    NumberType: Into<NumberOrPercentage>,
{
    match from {
        SvgLengthOrPercentageOrNumber::LengthOrPercentage(lop) => {
            lop.into()
        }
        SvgLengthOrPercentageOrNumber::Number(num) => {
            num.into()
        }
    }
}

fn convert_from_number_or_percentage<LengthOrPercentageType, NumberType>(
    from: NumberOrPercentage,
) -> SvgLengthOrPercentageOrNumber<LengthOrPercentageType, NumberType>
where
    LengthOrPercentageType: From<LengthOrPercentage>,
    NumberType: From<Number>,
{
    match from {
        NumberOrPercentage::Number(num) =>
            SvgLengthOrPercentageOrNumber::Number(num.into()),
        NumberOrPercentage::Percentage(p) =>
            SvgLengthOrPercentageOrNumber::LengthOrPercentage(
                (LengthOrPercentage::Percentage(p)).into())
    }
}

impl <L, N> Animate for SvgLengthOrPercentageOrNumber<L, N>
where
    L: Animate + From<LengthOrPercentage> + Into<NumberOrPercentage> + Copy,
    N: Animate + From<Number> + Into<NumberOrPercentage>,
    LengthOrPercentage: From<L>,
    Self: Copy,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if self.has_calc() || other.has_calc() {
            // TODO: We need to treat calc value.
            // https://bugzilla.mozilla.org/show_bug.cgi?id=1386967
            return Err(());
        }

        let this = convert_to_number_or_percentage(*self);
        let other = convert_to_number_or_percentage(*other);

        match (this, other) {
            (
                NumberOrPercentage::Number(ref this),
                NumberOrPercentage::Number(ref other),
            ) => {
                Ok(convert_from_number_or_percentage(
                    NumberOrPercentage::Number(this.animate(other, procedure)?)
                ))
            },
            (
                NumberOrPercentage::Percentage(ref this),
                NumberOrPercentage::Percentage(ref other),
            ) => {
                Ok(convert_from_number_or_percentage(
                    NumberOrPercentage::Percentage(this.animate(other, procedure)?)
                ))
            },
            _ => Err(()),
        }
    }
}

impl<L> Animate for SVGLength<L>
where
    L: Animate + Clone,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            (&SVGLength::Length(ref this), &SVGLength::Length(ref other)) => {
                Ok(SVGLength::Length(this.animate(other, procedure)?))
            },
            _ => Err(()),
        }
    }
}

/// <https://www.w3.org/TR/SVG11/painting.html#StrokeDasharrayProperty>
impl<L> Animate for SVGStrokeDashArray<L>
where
    L: Clone + Animate,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        if matches!(procedure, Procedure::Add | Procedure::Accumulate { .. }) {
            // Non-additive.
            return Err(());
        }
        match (self, other) {
            (&SVGStrokeDashArray::Values(ref this), &SVGStrokeDashArray::Values(ref other)) => {
                Ok(SVGStrokeDashArray::Values(this.animate_repeatable_list(other, procedure)?))
            },
            _ => Err(()),
        }
    }
}

impl<L> ComputeSquaredDistance for SVGStrokeDashArray<L>
where
    L: ComputeSquaredDistance,
{
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            (&SVGStrokeDashArray::Values(ref this), &SVGStrokeDashArray::Values(ref other)) => {
                this.squared_distance_repeatable_list(other)
            },
            _ => Err(()),
        }
    }
}

impl<L> ToAnimatedZero for SVGStrokeDashArray<L>
where
    L: ToAnimatedZero,
{
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        match *self {
            SVGStrokeDashArray::Values(ref values) => {
                Ok(SVGStrokeDashArray::Values(
                    values.iter().map(ToAnimatedZero::to_animated_zero).collect::<Result<Vec<_>, _>>()?,
                ))
            }
            SVGStrokeDashArray::ContextValue => Ok(SVGStrokeDashArray::ContextValue),
        }
    }
}

impl<O> Animate for SVGOpacity<O>
where
    O: Animate + Clone,
{
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        match (self, other) {
            (&SVGOpacity::Opacity(ref this), &SVGOpacity::Opacity(ref other)) => {
                Ok(SVGOpacity::Opacity(this.animate(other, procedure)?))
            },
            _ => Err(()),
        }
    }
}
