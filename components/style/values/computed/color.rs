/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed color values.

use cssparser::{Color as CSSParserColor, RGBA};
use std::fmt;
use style_traits::{CssWriter, ToCss};
use values::animated::ToAnimatedValue;
use values::animated::color::RGBA as AnimatedRGBA;
use values::generics::color::Color as GenericColor;

/// Computed value type for the specified RGBAColor.
pub type RGBAColor = RGBA;

/// The computed value of the `color` property.
pub type ColorPropertyValue = RGBA;

/// A computed value for `<color>`.
pub type Color = GenericColor<RGBAColor>;

impl Color {
    /// Returns a complex color value representing transparent.
    pub fn transparent() -> Color {
        Color::rgba(RGBA::transparent())
    }

    /// Combine this complex color with the given foreground color into
    /// a numeric RGBA color. It currently uses linear blending.
    pub fn to_rgba(&self, fg_color: RGBA) -> RGBA {
        let (color, ratios) = match *self {
            // Common cases that the complex color is either pure numeric
            // color or pure currentcolor.
            GenericColor::Numeric(color) => return color,
            GenericColor::Foreground => return fg_color,
            GenericColor::Complex(color, ratios) => (color, ratios),
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

impl ToCss for Color {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            GenericColor::Numeric(color) => color.to_css(dest),
            GenericColor::Foreground => CSSParserColor::CurrentColor.to_css(dest),
            _ => Ok(()),
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
