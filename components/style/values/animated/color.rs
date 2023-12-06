/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animated types for CSS colors.

use crate::color::mix::ColorInterpolationMethod;
use crate::color::AbsoluteColor;
use crate::values::animated::{Animate, Procedure, ToAnimatedZero};
use crate::values::computed::Percentage;
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::color::{GenericColor, GenericColorMix};

impl Animate for AbsoluteColor {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let (left_weight, right_weight) = procedure.weights();
        Ok(crate::color::mix::mix(
            ColorInterpolationMethod::best_interpolation_between(self, other),
            self,
            left_weight as f32,
            other,
            right_weight as f32,
            /* normalize_weights = */ false,
        ))
    }
}

impl ComputeSquaredDistance for AbsoluteColor {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let start = [
            self.alpha,
            self.components.0 * self.alpha,
            self.components.1 * self.alpha,
            self.components.2 * self.alpha,
        ];
        let end = [
            other.alpha,
            other.components.0 * other.alpha,
            other.components.1 * other.alpha,
            other.components.2 * other.alpha,
        ];
        start
            .iter()
            .zip(&end)
            .map(|(this, other)| this.compute_squared_distance(other))
            .sum()
    }
}

/// An animated value for `<color>`.
pub type Color = GenericColor<Percentage>;

/// An animated value for `<color-mix>`.
pub type ColorMix = GenericColorMix<Color, Percentage>;

impl Animate for Color {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let (left_weight, right_weight) = procedure.weights();
        Ok(Self::from_color_mix(ColorMix {
            interpolation: ColorInterpolationMethod::srgb(),
            left: self.clone(),
            left_percentage: Percentage(left_weight as f32),
            right: other.clone(),
            right_percentage: Percentage(right_weight as f32),
            // See https://github.com/w3c/csswg-drafts/issues/7324
            normalize_weights: false,
        }))
    }
}

impl ComputeSquaredDistance for Color {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let current_color = AbsoluteColor::transparent();
        self.resolve_to_absolute(&current_color)
            .compute_squared_distance(&other.resolve_to_absolute(&current_color))
    }
}

impl ToAnimatedZero for Color {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(Color::Absolute(AbsoluteColor::transparent()))
    }
}
