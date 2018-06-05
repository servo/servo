/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated types for CSS colors.

use values::animated::{Animate, Procedure, ToAnimatedZero};
use values::distance::{ComputeSquaredDistance, SquaredDistance};
use values::generics::color::{Color as GenericColor, ComplexColorRatios};

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
            1. / alpha;
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

impl Animate for ComplexColorRatios {
    #[inline]
    fn animate(&self, other: &Self, procedure: Procedure) -> Result<Self, ()> {
        let bg = self.bg.animate(&other.bg, procedure)?;
        let fg = self.fg.animate(&other.fg, procedure)?;

        Ok(ComplexColorRatios { bg, fg })
    }
}

/// An animated value for `<color>`.
pub type Color = GenericColor<RGBA>;

impl Color {
    fn effective_intermediate_rgba(&self) -> RGBA {
        match *self {
            GenericColor::Numeric(color) => color,
            GenericColor::Foreground => RGBA::transparent(),
            GenericColor::Complex(color, ratios) => RGBA {
                alpha: color.alpha * ratios.bg,
                ..color.clone()
            },
        }
    }

    fn effective_ratios(&self) -> ComplexColorRatios {
        match *self {
            GenericColor::Numeric(..) => ComplexColorRatios::NUMERIC,
            GenericColor::Foreground => ComplexColorRatios::FOREGROUND,
            GenericColor::Complex(.., ratios) => ratios,
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
            // Any interpolation of currentColor with currentColor returns currentColor.
            (Foreground, Foreground, Procedure::Interpolate { .. }) => Color::currentcolor(),
            // Animating two numeric colors.
            (Numeric(c1), Numeric(c2), _) => Numeric(c1.animate(&c2, procedure)?),
            // Combinations of numeric color and currentColor
            (Foreground, Numeric(color), _) => Self::with_ratios(
                color,
                ComplexColorRatios {
                    bg: other_weight as f32,
                    fg: this_weight as f32,
                },
            ),
            (Numeric(color), Foreground, _) => Self::with_ratios(
                color,
                ComplexColorRatios {
                    bg: this_weight as f32,
                    fg: other_weight as f32,
                },
            ),

            // Any other animation of currentColor with currentColor.
            (Foreground, Foreground, _) => Self::with_ratios(
                RGBA::transparent(),
                ComplexColorRatios {
                    bg: 0.,
                    fg: (this_weight + other_weight) as f32,
                },
            ),

            // Defer to complex calculations
            _ => {
                // For interpolating between two complex colors, we need to
                // generate colors with effective alpha value.
                let self_color = self.effective_intermediate_rgba();
                let other_color = other.effective_intermediate_rgba();
                let color = self_color.animate(&other_color, procedure)?;
                // Then we compute the final background ratio, and derive
                // the final alpha value from the effective alpha value.
                let self_ratios = self.effective_ratios();
                let other_ratios = other.effective_ratios();
                let ratios = self_ratios.animate(&other_ratios, procedure)?;
                let alpha = color.alpha / ratios.bg;
                let color = RGBA { alpha, ..color };

                Self::with_ratios(color, ratios)
            }
        })
    }
}

impl ComputeSquaredDistance for Color {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        use self::GenericColor::*;

        // All comments from the Animate impl also applies here.
        Ok(match (*self, *other) {
            (Foreground, Foreground) => SquaredDistance::from_sqrt(0.),
            (Numeric(c1), Numeric(c2)) => c1.compute_squared_distance(&c2)?,
            (Foreground, Numeric(color)) | (Numeric(color), Foreground) => {
                // `computed_squared_distance` is symmetic.
                color.compute_squared_distance(&RGBA::transparent())?
                    + SquaredDistance::from_sqrt(1.)
            }
            (_, _) => {
                let self_color = self.effective_intermediate_rgba();
                let other_color = other.effective_intermediate_rgba();
                let self_ratios = self.effective_ratios();
                let other_ratios = other.effective_ratios();

                self_color.compute_squared_distance(&other_color)?
                    + self_ratios.bg.compute_squared_distance(&other_ratios.bg)?
                    + self_ratios.fg.compute_squared_distance(&other_ratios.fg)?
            }
        })
    }
}

impl ToAnimatedZero for Color {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        // FIXME(nox): This does not look correct to me.
        Err(())
    }
}
