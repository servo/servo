/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed color values.

use cssparser::{Color as CSSParserColor, RGBA};
use std::fmt;
use style_traits::{CssWriter, ToCss};
use values::animated::ToAnimatedValue;
use values::animated::color::{Color as AnimatedColor, RGBA as AnimatedRGBA};

/// Ratios representing the contribution of color and currentcolor to
/// the final color value.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq)]
pub struct ComplexColorRatios {
    /// Numeric color contribution.
    pub bg: f32,
    /// Foreground color, aka currentcolor, contribution.
    pub fg: f32,
}

impl ComplexColorRatios {
    /// Ratios representing pure numeric color.
    pub const NUMERIC: ComplexColorRatios = ComplexColorRatios { bg: 1., fg: 0. };
    /// Ratios representing pure foreground color.
    pub const FOREGROUND: ComplexColorRatios = ComplexColorRatios { bg: 0., fg: 1. };
}

/// This enum represents a combined color from a numeric color and
/// the current foreground color (currentColor keyword).
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq)]
pub enum Color {
    ///  Numeric RGBA color.
    Numeric(RGBA),

    /// The current foreground color.
    Foreground,

    /// A linear combination of numeric color and currentColor.
    /// The formula is: `color * bg_ratio + currentColor * fg_ratio`.
    Complex(RGBA, ComplexColorRatios),
}

/// Computed value type for the specified RGBAColor.
pub type RGBAColor = RGBA;

/// The computed value of the `color` property.
pub type ColorPropertyValue = RGBA;

impl Color {
    /// Returns a numeric color representing the given RGBA value.
    pub fn rgba(color: RGBA) -> Color {
        Color::Numeric(color)
    }

    /// Returns a complex color value representing transparent.
    pub fn transparent() -> Color {
        Color::rgba(RGBA::transparent())
    }

    /// Returns a complex color value representing currentcolor.
    pub fn currentcolor() -> Color {
        Color::Foreground
    }

    /// Whether it is a numeric color (no currentcolor component).
    pub fn is_numeric(&self) -> bool {
        matches!(*self, Color::Numeric { .. })
    }

    /// Whether it is a currentcolor value (no numeric color component).
    pub fn is_currentcolor(&self) -> bool {
        matches!(*self, Color::Foreground)
    }

    /// Combine this complex color with the given foreground color into
    /// a numeric RGBA color. It currently uses linear blending.
    pub fn to_rgba(&self, fg_color: RGBA) -> RGBA {
        let (color, ratios) = match *self {
            // Common cases that the complex color is either pure numeric
            // color or pure currentcolor.
            Color::Numeric(color) => return color,
            Color::Foreground => return fg_color,
            Color::Complex(color, ratios) => (color, ratios),
        };

        // For the more complicated case that the alpha value differs,
        // we use the following formula to compute the components:
        // alpha = self_alpha * bg_ratio + fg_alpha * fg_ratio
        // color = (self_color * self_alpha * bg_ratio +
        //          fg_color * fg_alpha * fg_ratio) / alpha

        let p1 = ratios.bg;
        let a1 = color.alpha_f32();
        let r1 = a1 * color.red_f32();
        let g1 = a1 * color.green_f32();
        let b1 = a1 * color.blue_f32();

        let p2 = ratios.fg;
        let a2 = fg_color.alpha_f32();
        let r2 = a2 * fg_color.red_f32();
        let g2 = a2 * fg_color.green_f32();
        let b2 = a2 * fg_color.blue_f32();

        let a = p1 * a1 + p2 * a2;
        if a <= 0. {
            return RGBA::transparent();
        }
        let a = f32::min(a, 1.);

        let inverse_a = 1. / a;
        let r = (p1 * r1 + p2 * r2) * inverse_a;
        let g = (p1 * g1 + p2 * g2) * inverse_a;
        let b = (p1 * b1 + p2 * b2) * inverse_a;
        return RGBA::from_floats(r, g, b, a);
    }
}

impl From<RGBA> for Color {
    fn from(color: RGBA) -> Color {
        Color::Numeric(color)
    }
}

impl ToCss for Color {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            Color::Numeric(color) => color.to_css(dest),
            Color::Foreground => CSSParserColor::CurrentColor.to_css(dest),
            _ => Ok(()),
        }
    }
}

impl ToAnimatedValue for Color {
    type AnimatedValue = AnimatedColor;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        match self {
            Color::Numeric(color) => AnimatedColor::Numeric(color.to_animated_value()),
            Color::Foreground => AnimatedColor::Foreground,
            Color::Complex(color, ratios) => {
                AnimatedColor::Complex(color.to_animated_value(), ratios)
            }
        }
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        match animated {
            AnimatedColor::Numeric(color) => Color::Numeric(RGBA::from_animated_value(color)),
            AnimatedColor::Foreground => Color::Foreground,
            AnimatedColor::Complex(color, ratios) => {
                Color::Complex(RGBA::from_animated_value(color), ratios)
            }
        }
    }
}

impl ToAnimatedValue for RGBA {
    type AnimatedValue = AnimatedRGBA;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        AnimatedRGBA::new(
            self.red_f32(),
            self.green_f32(),
            self.blue_f32(),
            self.alpha_f32(),
        )
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        // RGBA::from_floats clamps each component values.
        RGBA::from_floats(animated.red, animated.green, animated.blue, animated.alpha)
    }
}
