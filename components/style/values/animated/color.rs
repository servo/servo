/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Animated types for CSS colors.

use crate::values::animated::{Animate, Procedure, ToAnimatedZero};
use crate::values::distance::{ComputeSquaredDistance, SquaredDistance};
use crate::values::generics::color::{Color as GenericColor, ComplexColorRatios};

/// An animated RGBA color.
///
/// Unlike in computed values, each component value may exceed the
/// range `[0.0, 1.0]`.
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, ToAnimatedZero)]
pub struct RGBA {
    /// The red component.
    pub red: f32,
    /// The green component.
    pub green: f32,
    /// The blue component.
    pub blue: f32,
    /// The alpha component.
    pub alpha: f32,
}

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
            red: red,
            green: green,
            blue: blue,
            alpha: alpha,
        }
    }
}

/// Unlike Animate for computed colors, we don't clamp any component values.
///
/// FIXME(nox): Why do computed colors even implement Animate?
impl Animate for RGBA {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let mut alpha = self.alpha.animate(&other.alpha, procedure)?;
        if alpha <= 0. {
            // Ideally we should return color value that only alpha component is
            // 0, but this is what current gecko does.
            return Ok(RGBA::transparent());
        }

        alpha = alpha.min(1.);
        let red =
            (self.red * self.alpha).animate(&(other.red * other.alpha), procedure)? * 1. / alpha;
        let green = (self.green * self.alpha).animate(&(other.green * other.alpha), procedure)? *
            1. /
            alpha;
        let blue =
            (self.blue * self.alpha).animate(&(other.blue * other.alpha), procedure)? * 1. / alpha;

        Ok(RGBA::new(red, green, blue, alpha))
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
pub type Color = GenericColor<RGBA>;

impl Color {
    fn effective_intermediate_rgba(&self) -> RGBA {
        match *self {
            GenericColor::Numeric(color) => color,
            GenericColor::CurrentColor => RGBA::transparent(),
            GenericColor::Complex { color, ratios } => RGBA {
                alpha: color.alpha * ratios.bg,
                ..color.clone()
            },
        }
    }

    fn effective_ratios(&self) -> ComplexColorRatios {
        match *self {
            GenericColor::Numeric(..) => ComplexColorRatios::NUMERIC,
            GenericColor::CurrentColor => ComplexColorRatios::CURRENT_COLOR,
            GenericColor::Complex { ratios, .. } => ratios,
        }
    }
}

impl Animate for Color {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        use self::GenericColor::*;

        // Common cases are interpolating between two numeric colors,
        // two currentcolors, and a numeric color and a currentcolor.
        let (this_weight, other_weight) = procedure.weights();

        Ok(match (*self, *other, procedure) {
            // Any interpolation of currentcolor with currentcolor returns currentcolor.
            (CurrentColor, CurrentColor, Procedure::Interpolate { .. }) => CurrentColor,
            // Animating two numeric colors.
            (Numeric(c1), Numeric(c2), _) => Numeric(c1.animate(&c2, procedure)?),
            // Combinations of numeric color and currentcolor
            (CurrentColor, Numeric(color), _) => Self::with_ratios(
                color,
                ComplexColorRatios {
                    bg: other_weight as f32,
                    fg: this_weight as f32,
                },
            ),
            (Numeric(color), CurrentColor, _) => Self::with_ratios(
                color,
                ComplexColorRatios {
                    bg: this_weight as f32,
                    fg: other_weight as f32,
                },
            ),

            // Any other animation of currentcolor with currentcolor.
            (CurrentColor, CurrentColor, _) => Self::with_ratios(
                RGBA::transparent(),
                ComplexColorRatios {
                    bg: 0.,
                    fg: (this_weight + other_weight) as f32,
                },
            ),

            // Defer to complex calculations
            _ => {
                // Compute the "scaled" contribution for `color`.
                fn scaled_rgba(color: &Color) -> RGBA {
                    match *color {
                        GenericColor::Numeric(color) => color,
                        GenericColor::CurrentColor => RGBA::transparent(),
                        GenericColor::Complex { color, ratios } => RGBA {
                            red: color.red * ratios.bg,
                            green: color.green * ratios.bg,
                            blue: color.blue * ratios.bg,
                            alpha: color.alpha * ratios.bg,
                        },
                    }
                }

                // Each `Color`, represents a complex combination of foreground color and
                // background color where fg and bg represent the overall
                // contributions. ie:
                //
                //    color = { bg * mColor, fg * foreground }
                //          =   { bg_color , fg_color }
                //          =     bg_color + fg_color
                //
                // where `foreground` is `currentcolor`, and `bg_color`,
                // `fg_color` are the scaled background and foreground
                // contributions.
                //
                // Each operation, lerp, addition, or accumulate, can be
                // represented as a scaled-addition each complex color. ie:
                //
                //    p * col1 + q * col2
                //
                // where p = (1 - a), q = a for lerp(a), p = 1, q = 1 for
                // addition, etc.
                //
                // Therefore:
                //
                //    col1 op col2
                //    = p * col1 + q * col2
                //    = p * { bg_color1, fg_color1 } + q * { bg_color2, fg_color2 }
                //    = p * (bg_color1 + fg_color1) + q * (bg_color2 + fg_color2)
                //    = p * bg_color1 + p * fg_color1 + q * bg_color2 + p * fg_color2
                //    = (p * bg_color1 + q * bg_color2) + (p * fg_color1 + q * fg_color2)
                //    = (bg_color1 op bg_color2) + (fg_color1 op fg_color2)
                //
                // fg_color1 op fg_color2 is equivalent to (fg1 op fg2) * foreground,
                // so the final color is:
                //
                //    = { bg_color, fg_color }
                //    = { 1 * (bg_color1 op bg_color2), (fg1 op fg2) * foreground }

                // To perform the operation on two complex colors, we need to
                // generate the scaled contributions of each background color
                // component.
                let bg_color1 = scaled_rgba(self);
                let bg_color2 = scaled_rgba(other);
                // Perform bg_color1 op bg_color2
                let bg_color = bg_color1.animate(&bg_color2, procedure)?;

                // Calculate the final foreground color ratios; perform
                // animation on effective fg ratios.
                let ComplexColorRatios { fg: fg1, .. } = self.effective_ratios();
                let ComplexColorRatios { fg: fg2, .. } = other.effective_ratios();
                // Perform fg1 op fg2
                let fg = fg1.animate(&fg2, procedure)?;

                Self::with_ratios(bg_color, ComplexColorRatios { bg: 1., fg })
            },
        })
    }
}

impl ComputeSquaredDistance for Color {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        use self::GenericColor::*;

        // All comments from the Animate impl also applies here.
        Ok(match (*self, *other) {
            (CurrentColor, CurrentColor) => SquaredDistance::from_sqrt(0.),
            (Numeric(c1), Numeric(c2)) => c1.compute_squared_distance(&c2)?,
            (CurrentColor, Numeric(color)) | (Numeric(color), CurrentColor) => {
                // `computed_squared_distance` is symmetric.
                color.compute_squared_distance(&RGBA::transparent())? +
                    SquaredDistance::from_sqrt(1.)
            },
            (_, _) => {
                let self_color = self.effective_intermediate_rgba();
                let other_color = other.effective_intermediate_rgba();
                let self_ratios = self.effective_ratios();
                let other_ratios = other.effective_ratios();

                self_color.compute_squared_distance(&other_color)? +
                    self_ratios.bg.compute_squared_distance(&other_ratios.bg)? +
                    self_ratios.fg.compute_squared_distance(&other_ratios.fg)?
            },
        })
    }
}

impl ToAnimatedZero for Color {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(RGBA::transparent().into())
    }
}
