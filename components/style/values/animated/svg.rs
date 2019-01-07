/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animation implementations for various SVG-related types.

use super::{Animate, Procedure, ToAnimatedZero};
use crate::properties::animated_properties::ListAnimation;
use crate::values::animated::color::Color as AnimatedColor;
use crate::values::computed::url::ComputedUrl;
use crate::values::computed::{LengthPercentage, Number, NumberOrPercentage};
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::svg::{SVGLength, SVGPaint, SvgLengthPercentageOrNumber};
use crate::values::generics::svg::{SVGOpacity, SVGStrokeDashArray};

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

// FIXME: We need to handle calc here properly, see
// https://bugzilla.mozilla.org/show_bug.cgi?id=1386967
fn to_number_or_percentage(
    value: &SvgLengthPercentageOrNumber<LengthPercentage, Number>,
) -> Result<NumberOrPercentage, ()> {
    Ok(match *value {
        SvgLengthPercentageOrNumber::LengthPercentage(ref l) => match l.percentage {
            Some(p) => {
                if l.unclamped_length().px() != 0. {
                    return Err(());
                }
                NumberOrPercentage::Percentage(p)
            },
            None => NumberOrPercentage::Number(l.length().px()),
        },
        SvgLengthPercentageOrNumber::Number(ref n) => NumberOrPercentage::Number(*n),
    })
}

impl Animate for SvgLengthPercentageOrNumber<LengthPercentage, Number> {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let this = to_number_or_percentage(self)?;
        let other = to_number_or_percentage(other)?;

        match (this, other) {
            (NumberOrPercentage::Number(ref this), NumberOrPercentage::Number(ref other)) => Ok(
                SvgLengthPercentageOrNumber::Number(this.animate(other, procedure)?),
            ),
            (
                NumberOrPercentage::Percentage(ref this),
                NumberOrPercentage::Percentage(ref other),
            ) => Ok(SvgLengthPercentageOrNumber::LengthPercentage(
                LengthPercentage::new_percent(this.animate(other, procedure)?),
            )),
            _ => Err(()),
        }
    }
}

impl ComputeSquaredDistance for SvgLengthPercentageOrNumber<LengthPercentage, Number> {
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        to_number_or_percentage(self)?.compute_squared_distance(&to_number_or_percentage(other)?)
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
            (&SVGStrokeDashArray::Values(ref this), &SVGStrokeDashArray::Values(ref other)) => Ok(
                SVGStrokeDashArray::Values(this.animate_repeatable_list(other, procedure)?),
            ),
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
            SVGStrokeDashArray::Values(ref values) => Ok(SVGStrokeDashArray::Values(
                values
                    .iter()
                    .map(ToAnimatedZero::to_animated_zero)
                    .collect::<Result<Vec<_>, _>>()?,
            )),
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
