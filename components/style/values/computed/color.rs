/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed color values.

use cssparser::{Color as CSSParserColor, RGBA};
use std::fmt;
use style_traits::ToCss;

/// This struct represents a combined color from a numeric color and
/// the current foreground color (currentcolor keyword).
/// Conceptually, the formula is "color * (1 - p) + currentcolor * p"
/// where p is foreground_ratio.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Color {
    /// RGBA color.
    pub color: RGBA,
    /// The ratio of currentcolor in complex color.
    pub foreground_ratio: u8,
}

fn blend_color_component(bg: u8, fg: u8, fg_alpha: u8) -> u8 {
    let bg_ratio = (u8::max_value() - fg_alpha) as u32;
    let fg_ratio = fg_alpha as u32;
    let color = bg as u32 * bg_ratio + fg as u32 * fg_ratio;
    // Rounding divide the number by 255
    ((color + 127) / 255) as u8
}

impl Color {
    /// Returns a numeric color representing the given RGBA value.
    pub fn rgba(rgba: RGBA) -> Color {
        Color {
            color: rgba,
            foreground_ratio: 0,
        }
    }

    /// Returns a complex color value representing transparent.
    pub fn transparent() -> Color {
        Color::rgba(RGBA::transparent())
    }

    /// Returns a complex color value representing currentcolor.
    pub fn currentcolor() -> Color {
        Color {
            color: RGBA::transparent(),
            foreground_ratio: u8::max_value(),
        }
    }

    /// Whether it is a numeric color (no currentcolor component).
    pub fn is_numeric(&self) -> bool {
        self.foreground_ratio == 0
    }

    /// Whether it is a currentcolor value (no numeric color component).
    pub fn is_currentcolor(&self) -> bool {
        self.foreground_ratio == u8::max_value()
    }

    /// Combine this complex color with the given foreground color into
    /// a numeric RGBA color. It currently uses linear blending.
    pub fn to_rgba(&self, fg_color: RGBA) -> RGBA {
        // Common cases that the complex color is either pure numeric
        // color or pure currentcolor.
        if self.is_numeric() {
            return self.color;
        }
        if self.is_currentcolor() {
            return fg_color.clone();
        }
        // Common case that alpha channel is equal (usually both are opaque).
        let fg_ratio = self.foreground_ratio;
        if self.color.alpha == fg_color.alpha {
            let r = blend_color_component(self.color.red, fg_color.red, fg_ratio);
            let g = blend_color_component(self.color.green, fg_color.green, fg_ratio);
            let b = blend_color_component(self.color.blue, fg_color.blue, fg_ratio);
            return RGBA::new(r, g, b, fg_color.alpha);
        }

        // For the more complicated case that the alpha value differs,
        // we use the following formula to compute the components:
        // alpha = self_alpha * (1 - fg_ratio) + fg_alpha * fg_ratio
        // color = (self_color * self_alpha * (1 - fg_ratio) +
        //          fg_color * fg_alpha * fg_ratio) / alpha

        let p1 = (1. / 255.) * (255 - fg_ratio) as f32;
        let a1 = self.color.alpha_f32();
        let r1 = a1 * self.color.red_f32();
        let g1 = a1 * self.color.green_f32();
        let b1 = a1 * self.color.blue_f32();

        let p2 = 1. - p1;
        let a2 = fg_color.alpha_f32();
        let r2 = a2 * fg_color.red_f32();
        let g2 = a2 * fg_color.green_f32();
        let b2 = a2 * fg_color.blue_f32();

        let a = p1 * a1 + p2 * a2;
        if a == 0.0 {
            return RGBA::transparent();
        }

        let inverse_a = 1. / a;
        let r = (p1 * r1 + p2 * r2) * inverse_a;
        let g = (p1 * g1 + p2 * g2) * inverse_a;
        let b = (p1 * b1 + p2 * b2) * inverse_a;
        return RGBA::from_floats(r, g, b, a);
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Color) -> bool {
        self.foreground_ratio == other.foreground_ratio &&
            (self.is_currentcolor() || self.color == other.color)
    }
}

impl From<RGBA> for Color {
    fn from(color: RGBA) -> Color {
        Color {
            color: color,
            foreground_ratio: 0,
        }
    }
}

impl ToCss for Color {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if self.is_numeric() {
            self.color.to_css(dest)
        } else if self.is_currentcolor() {
            CSSParserColor::CurrentColor.to_css(dest)
        } else {
            Ok(())
        }
    }
}

/// Computed value type for the specified RGBAColor.
pub type RGBAColor = RGBA;
