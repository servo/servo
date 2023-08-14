/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed color values.

use crate::values::animated::color::RGBA as AnimatedRGBA;
use crate::values::animated::ToAnimatedValue;
use crate::values::generics::color::{GenericCaretColor, GenericColor, GenericColorOrAuto};
use crate::values::computed::percentage::Percentage;
use cssparser::{Color as CSSParserColor, RGBA};
use std::fmt;
use style_traits::{CssWriter, ToCss};

pub use crate::values::specified::color::{ColorScheme, PrintColorAdjust};

/// The computed value of the `color` property.
pub type ColorPropertyValue = RGBA;

/// The computed value of `-moz-font-smoothing-background-color`.
pub type MozFontSmoothingBackgroundColor = RGBA;

/// A computed value for `<color>`.
pub type Color = GenericColor<RGBA, Percentage>;

impl ToCss for Color {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            Self::Numeric(ref c) => c.to_css(dest),
            Self::CurrentColor => CSSParserColor::CurrentColor.to_css(dest),
            Self::ColorMix(ref m) => m.to_css(dest),
        }
    }
}

impl Color {
    /// Returns a complex color value representing transparent.
    pub fn transparent() -> Color {
        Color::rgba(RGBA::transparent())
    }

    /// Returns opaque black.
    pub fn black() -> Color {
        Color::rgba(RGBA::new(0, 0, 0, 255))
    }

    /// Returns opaque white.
    pub fn white() -> Color {
        Color::rgba(RGBA::new(255, 255, 255, 255))
    }

    /// Combine this complex color with the given foreground color into
    /// a numeric RGBA color.
    pub fn into_rgba(mut self, current_color: RGBA) -> RGBA {
        self.simplify(Some(&current_color));
        *self.as_numeric().unwrap()
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

/// auto | <color>
pub type ColorOrAuto = GenericColorOrAuto<Color>;

/// caret-color
pub type CaretColor = GenericCaretColor<Color>;
