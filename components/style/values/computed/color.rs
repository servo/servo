/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed color values.

use crate::color::AbsoluteColor;
use crate::values::animated::ToAnimatedZero;
use crate::values::computed::percentage::Percentage;
use crate::values::generics::color::{
    GenericCaretColor, GenericColor, GenericColorMix, GenericColorOrAuto,
};
use cssparser::Color as CSSParserColor;
use std::fmt;
use style_traits::{CssWriter, ToCss};

pub use crate::values::specified::color::{ColorScheme, ForcedColorAdjust, PrintColorAdjust};

/// The computed value of the `color` property.
pub type ColorPropertyValue = AbsoluteColor;

/// The computed value of `-moz-font-smoothing-background-color`.
pub type MozFontSmoothingBackgroundColor = AbsoluteColor;

/// A computed value for `<color>`.
pub type Color = GenericColor<Percentage>;

/// A computed color-mix().
pub type ColorMix = GenericColorMix<Color, Percentage>;

impl ToCss for Color {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {
            Self::Absolute(ref c) => c.to_css(dest),
            Self::CurrentColor => cssparser::ToCss::to_css(&CSSParserColor::CurrentColor, dest),
            Self::ColorMix(ref m) => m.to_css(dest),
        }
    }
}

impl Color {
    /// Create a new computed [`Color`] from a given color-mix, simplifying it to an absolute color
    /// if possible.
    pub fn from_color_mix(color_mix: ColorMix) -> Self {
        if let Some(absolute) = color_mix.mix_to_absolute() {
            Self::Absolute(absolute)
        } else {
            Self::ColorMix(Box::new(color_mix))
        }
    }

    /// Returns a complex color value representing transparent.
    pub fn transparent() -> Color {
        Color::Absolute(AbsoluteColor::transparent())
    }

    /// Returns opaque black.
    pub fn black() -> Color {
        Color::Absolute(AbsoluteColor::black())
    }

    /// Returns opaque white.
    pub fn white() -> Color {
        Color::Absolute(AbsoluteColor::white())
    }

    /// Combine this complex color with the given foreground color into an
    /// absolute color.
    pub fn resolve_to_absolute(&self, current_color: &AbsoluteColor) -> AbsoluteColor {
        use crate::values::specified::percentage::ToPercentage;

        match *self {
            Self::Absolute(c) => c,
            Self::CurrentColor => *current_color,
            Self::ColorMix(ref mix) => {
                let left = mix.left.resolve_to_absolute(current_color);
                let right = mix.right.resolve_to_absolute(current_color);
                crate::color::mix::mix(
                    mix.interpolation,
                    &left,
                    mix.left_percentage.to_percentage(),
                    &right,
                    mix.right_percentage.to_percentage(),
                    mix.normalize_weights,
                )
            },
        }
    }
}

impl ToAnimatedZero for AbsoluteColor {
    fn to_animated_zero(&self) -> Result<Self, ()> {
        Ok(Self::transparent())
    }
}

/// auto | <color>
pub type ColorOrAuto = GenericColorOrAuto<Color>;

/// caret-color
pub type CaretColor = GenericCaretColor<Color>;
