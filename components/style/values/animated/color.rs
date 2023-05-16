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
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToAnimatedZero)]
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
        let mut alpha = self.alpha.animate(&other.alpha, procedure)?;
        if alpha <= 0. {
            // Ideally we should return color value that only alpha component is
            // 0, but this is what current gecko does.
            return Ok(RGBA::transparent());
        }

        alpha = alpha.min(1.);
        let red = (self.red * self.alpha).animate(&(other.red * other.alpha), procedure)?;
        let green = (self.green * self.alpha).animate(&(other.green * other.alpha), procedure)?;
        let blue = (self.blue * self.alpha).animate(&(other.blue * other.alpha), procedure)?;
        let inv = 1. / alpha;
        Ok(RGBA::new(red * inv, green * inv, blue * inv, alpha))
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
        if self.ratios.bg == 0. {
            return RGBA::transparent();
        }

        if self.ratios.bg == 1. {
            return self.color;
        }

        RGBA {
            alpha: self.color.alpha * self.ratios.bg,
            ..self.color
        }
    }

    /// Mix two colors into one.
    pub fn mix(
        left_color: &Color,
        left_weight: f32,
        right_color: &Color,
        right_weight: f32,
    ) -> Self {
        let left_bg = left_color.scaled_rgba();
        let right_bg = right_color.scaled_rgba();
        let alpha = (left_bg.alpha * left_weight +
            right_bg.alpha * right_weight)
            .min(1.);

        let mut fg = 0.;
        let mut red = 0.;
        let mut green = 0.;
        let mut blue = 0.;

        let colors = [
            (left_color, &left_bg, left_weight),
            (right_color, &right_bg, right_weight),
        ];

        for &(color, bg, weight) in &colors {
            fg += color.ratios.fg * weight;

            red += bg.red * bg.alpha * weight;
            green += bg.green * bg.alpha * weight;
            blue += bg.blue * bg.alpha * weight;
        }

        let color = if alpha <= 0. {
            RGBA::transparent()
        } else {
            let inv = 1. / alpha;
            RGBA::new(red * inv, green * inv, blue * inv, alpha)
        };

        Self::new(color, ComplexColorRatios { bg: 1., fg })
    }

    fn scaled_rgba(&self) -> RGBA {
        if self.ratios.bg == 0. {
            return RGBA::transparent();
        }

        if self.ratios.bg == 1. {
            return self.color;
        }

        RGBA {
            red: self.color.red * self.ratios.bg,
            green: self.color.green * self.ratios.bg,
            blue: self.color.blue * self.ratios.bg,
            alpha: self.color.alpha * self.ratios.bg,
        }
    }
}

impl Animate for Color {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let self_numeric = self.is_numeric();
        let other_numeric = other.is_numeric();

        if self_numeric && other_numeric {
            return Ok(Self::rgba(self.color.animate(&other.color, procedure)?));
        }

        let self_currentcolor = self.is_currentcolor();
        let other_currentcolor = other.is_currentcolor();

        if self_currentcolor && other_currentcolor {
            let (self_weight, other_weight) = procedure.weights();
            return Ok(Self::new(
                RGBA::transparent(),
                ComplexColorRatios {
                    bg: 0.,
                    fg: (self_weight + other_weight) as f32,
                },
            ));
        }

        // FIXME(emilio): Without these special cases tests fail, looks fairly
        // sketchy!
        if (self_currentcolor && other_numeric) || (self_numeric && other_currentcolor) {
            let (self_weight, other_weight) = procedure.weights();
            return Ok(if self_numeric {
                Self::new(
                    self.color,
                    ComplexColorRatios {
                        bg: self_weight as f32,
                        fg: other_weight as f32,
                    },
                )
            } else {
                Self::new(
                    other.color,
                    ComplexColorRatios {
                        bg: other_weight as f32,
                        fg: self_weight as f32,
                    },
                )
            });
        }

        // Compute the "scaled" contribution for `color`.
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
        //
        // To perform the operation on two complex colors, we need to
        // generate the scaled contributions of each background color
        // component.
        let bg_color1 = self.scaled_rgba();
        let bg_color2 = other.scaled_rgba();

        // Perform bg_color1 op bg_color2
        let bg_color = bg_color1.animate(&bg_color2, procedure)?;

        // Calculate the final foreground color ratios; perform
        // animation on effective fg ratios.
        let fg = self.ratios.fg.animate(&other.ratios.fg, procedure)?;

        Ok(Self::new(bg_color, ComplexColorRatios { bg: 1., fg }))
    }
}

impl ComputeSquaredDistance for Color {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // All comments from the Animate impl also apply here.
        let self_numeric = self.is_numeric();
        let other_numeric = other.is_numeric();

        if self_numeric && other_numeric {
            return self.color.compute_squared_distance(&other.color);
        }

        let self_currentcolor = self.is_currentcolor();
        let other_currentcolor = other.is_currentcolor();
        if self_currentcolor && other_currentcolor {
            return Ok(SquaredDistance::from_sqrt(0.));
        }

        if (self_currentcolor && other_numeric) || (self_numeric && other_currentcolor) {
            let color = if self_numeric {
                &self.color
            } else {
                &other.color
            };
            // `computed_squared_distance` is symmetric.
            return Ok(color.compute_squared_distance(&RGBA::transparent())? +
                SquaredDistance::from_sqrt(1.));
        }

        let self_color = self.effective_intermediate_rgba();
        let other_color = other.effective_intermediate_rgba();
        let self_ratios = self.ratios;
        let other_ratios = other.ratios;

        Ok(self_color.compute_squared_distance(&other_color)? +
            self_ratios.bg.compute_squared_distance(&other_ratios.bg)? +
            self_ratios.fg.compute_squared_distance(&other_ratios.fg)?)
    }
}

impl ToAnimatedZero for Color {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(RGBA::transparent().into())
    }
}
