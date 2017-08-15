/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Animated types for CSS colors.

use properties::animated_properties::Animatable;
use values::animated::ToAnimatedZero;
use values::distance::{ComputeSquaredDistance, SquaredDistance};

/// An animated RGBA color.
///
/// Unlike in computed values, each component value may exceed the
/// range `[0.0, 1.0]`.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq)]
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
        RGBA { red: red, green: green, blue: blue, alpha: alpha }
    }
}

/// Unlike Animatable for computed colors, we don't clamp any component values.
///
/// FIXME(nox): Why do computed colors even implement Animatable?
impl Animatable for RGBA {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        let mut alpha = self.alpha.add_weighted(&other.alpha, self_portion, other_portion)?;
        if alpha <= 0. {
            // Ideally we should return color value that only alpha component is
            // 0, but this is what current gecko does.
            return Ok(RGBA::transparent());
        }

        alpha = alpha.min(1.);
        let red = (self.red * self.alpha).add_weighted(
            &(other.red * other.alpha), self_portion, other_portion
        )? * 1. / alpha;
        let green = (self.green * self.alpha).add_weighted(
            &(other.green * other.alpha), self_portion, other_portion
        )? * 1. / alpha;
        let blue = (self.blue * self.alpha).add_weighted(
            &(other.blue * other.alpha), self_portion, other_portion
        )? * 1. / alpha;

        Ok(RGBA::new(red, green, blue, alpha))
    }
}

impl ComputeSquaredDistance for RGBA {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        let start = [ self.alpha, self.red * self.alpha, self.green * self.alpha, self.blue * self.alpha ];
        let end = [ other.alpha, other.red * other.alpha, other.green * other.alpha, other.blue * other.alpha ];
        start.iter().zip(&end).map(|(this, other)| this.compute_squared_distance(other)).sum()
    }
}

impl ToAnimatedZero for RGBA {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(RGBA::transparent())
    }
}

#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub color: RGBA,
    pub foreground_ratio: f32,
}

impl Color {
    fn currentcolor() -> Self {
        Color {
            color: RGBA::transparent(),
            foreground_ratio: 1.,
        }
    }

    /// Returns a transparent intermediate color.
    pub fn transparent() -> Self {
        Color {
            color: RGBA::transparent(),
            foreground_ratio: 0.,
        }
    }

    fn is_currentcolor(&self) -> bool {
        self.foreground_ratio >= 1.
    }

    fn is_numeric(&self) -> bool {
        self.foreground_ratio <= 0.
    }

    fn effective_intermediate_rgba(&self) -> RGBA {
        RGBA {
            alpha: self.color.alpha * (1. - self.foreground_ratio),
            .. self.color
        }
    }
}

impl Animatable for Color {
    #[inline]
    fn add_weighted(&self, other: &Self, self_portion: f64, other_portion: f64) -> Result<Self, ()> {
        // Common cases are interpolating between two numeric colors,
        // two currentcolors, and a numeric color and a currentcolor.
        //
        // Note: this algorithm assumes self_portion + other_portion
        // equals to one, so it may be broken for additive operation.
        // To properly support additive color interpolation, we would
        // need two ratio fields in computed color types.
        if self.foreground_ratio == other.foreground_ratio {
            if self.is_currentcolor() {
                Ok(Color::currentcolor())
            } else {
                Ok(Color {
                    color: self.color.add_weighted(&other.color, self_portion, other_portion)?,
                    foreground_ratio: self.foreground_ratio,
                })
            }
        } else if self.is_currentcolor() && other.is_numeric() {
            Ok(Color {
                color: other.color,
                foreground_ratio: self_portion as f32,
            })
        } else if self.is_numeric() && other.is_currentcolor() {
            Ok(Color {
                color: self.color,
                foreground_ratio: other_portion as f32,
            })
        } else {
            // For interpolating between two complex colors, we need to
            // generate colors with effective alpha value.
            let self_color = self.effective_intermediate_rgba();
            let other_color = other.effective_intermediate_rgba();
            let color = self_color.add_weighted(&other_color, self_portion, other_portion)?;
            // Then we compute the final foreground ratio, and derive
            // the final alpha value from the effective alpha value.
            let foreground_ratio = self.foreground_ratio
                .add_weighted(&other.foreground_ratio, self_portion, other_portion)?;
            let alpha = color.alpha / (1. - foreground_ratio);
            Ok(Color {
                color: RGBA {
                    alpha: alpha,
                    .. color
                },
                foreground_ratio: foreground_ratio,
            })
        }
    }
}

impl ComputeSquaredDistance for Color {
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        // All comments in add_weighted also applies here.
        if self.foreground_ratio == other.foreground_ratio {
            if self.is_currentcolor() {
                Ok(SquaredDistance::Value(0.))
            } else {
                self.color.compute_squared_distance(&other.color)
            }
        } else if self.is_currentcolor() && other.is_numeric() {
            Ok(
                RGBA::transparent().compute_squared_distance(&other.color)? +
                SquaredDistance::Value(1.),
            )
        } else if self.is_numeric() && other.is_currentcolor() {
            Ok(
                self.color.compute_squared_distance(&RGBA::transparent())? +
                SquaredDistance::Value(1.),
            )
        } else {
            let self_color = self.effective_intermediate_rgba();
            let other_color = other.effective_intermediate_rgba();
            Ok(
                self_color.compute_squared_distance(&other_color)? +
                self.foreground_ratio.compute_squared_distance(&other.foreground_ratio)?,
            )
        }
    }
}

impl ToAnimatedZero for Color {
    #[inline]
    fn to_animated_zero(&self) -> Result<Self, ()> {
        // FIXME(nox): This does not look correct to me.
        Err(())
    }
}
