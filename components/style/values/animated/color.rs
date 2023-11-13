/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animated types for CSS colors.

use crate::color::mix::ColorInterpolationMethod;
use crate::color::{AbsoluteColor, ColorComponents, ColorSpace};
use crate::values::animated::{Animate, Procedure, ToAnimatedZero};
use crate::values::computed::Percentage;
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::color::{GenericColor, GenericColorMix};

/// An animated RGBA color.
///
/// Unlike in computed values, each component value may exceed the
/// range `[0.0, 1.0]`.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToAnimatedZero, ToAnimatedValue)]
#[repr(C)]
pub struct AnimatedRGBA {
    /// The red component.
    pub red: f32,
    /// The green component.
    pub green: f32,
    /// The blue component.
    pub blue: f32,
    /// The alpha component.
    pub alpha: f32,
}

impl From<AbsoluteColor> for AnimatedRGBA {
    fn from(value: AbsoluteColor) -> Self {
        let srgb = value.to_color_space(ColorSpace::Srgb);

        Self::new(
            srgb.components.0,
            srgb.components.1,
            srgb.components.2,
            srgb.alpha,
        )
    }
}

impl From<AnimatedRGBA> for AbsoluteColor {
    fn from(value: AnimatedRGBA) -> Self {
        Self::new(
            ColorSpace::Srgb,
            ColorComponents(value.red, value.green, value.blue),
            value.alpha,
        )
    }
}

use self::AnimatedRGBA as RGBA;

impl RGBA {
    /// Returns a transparent color.
    #[inline]
    pub fn transparent() -> Self {
        Self::new(0., 0., 0., 0.)
    }

    /// Returns a new color.
    #[inline]
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        RGBA {
            red,
            green,
            blue,
            alpha,
        }
    }
}

impl Animate for RGBA {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let (left_weight, right_weight) = procedure.weights();
        Ok(crate::color::mix::mix(
            &ColorInterpolationMethod::srgb(),
            &AbsoluteColor::from(self.clone()),
            left_weight as f32,
            &AbsoluteColor::from(other.clone()),
            right_weight as f32,
            /* normalize_weights = */ false,
        )
        .into())
    }
}

impl ComputeSquaredDistance for RGBA {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let start = [
            self.alpha,
            self.red * self.alpha,
            self.green * self.alpha,
            self.blue * self.alpha,
        ];
        let end = [
            other.alpha,
            other.red * other.alpha,
            other.green * other.alpha,
            other.blue * other.alpha,
        ];
        start
            .iter()
            .zip(&end)
            .map(|(this, other)| this.compute_squared_distance(other))
            .sum()
    }
}

/// An animated value for `<color>`.
pub type Color = GenericColor<RGBA, Percentage>;

/// An animated value for `<color-mix>`.
pub type ColorMix = GenericColorMix<Color, Percentage>;

impl Color {
    fn to_rgba(&self, current_color: RGBA) -> RGBA {
        let mut clone = self.clone();
        clone.simplify(Some(&current_color));
        *clone.as_numeric().unwrap()
    }
}

impl Animate for Color {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let (left_weight, right_weight) = procedure.weights();
        let mut color = Color::ColorMix(Box::new(ColorMix {
            interpolation: ColorInterpolationMethod::srgb(),
            left: self.clone(),
            left_percentage: Percentage(left_weight as f32),
            right: other.clone(),
            right_percentage: Percentage(right_weight as f32),
            // See https://github.com/w3c/csswg-drafts/issues/7324
            normalize_weights: false,
        }));
        color.simplify(None);
        Ok(color)
    }
}

impl ComputeSquaredDistance for Color {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let current_color = RGBA::transparent();
        self.to_rgba(current_color)
            .compute_squared_distance(&other.to_rgba(current_color))
    }
}

impl ToAnimatedZero for Color {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(Color::rgba(RGBA::transparent()))
    }
}
