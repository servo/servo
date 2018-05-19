/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust helpers to interact with Gecko's StyleComplexColor.

use gecko::values::{convert_nscolor_to_rgba, convert_rgba_to_nscolor};
use gecko_bindings::structs::{nscolor, StyleComplexColor};
use values::computed::Color as ComputedColor;
use values::generics::ui::CaretColor;

impl From<nscolor> for StyleComplexColor {
    fn from(other: nscolor) -> Self {
        StyleComplexColor {
            mColor: other,
            mForegroundRatio: 0,
            mIsAuto: false,
        }
    }
}

impl StyleComplexColor {
    /// Create a `StyleComplexColor` value that represents `currentColor`.
    pub fn current_color() -> Self {
        StyleComplexColor {
            mColor: 0,
            mForegroundRatio: 255,
            mIsAuto: false,
        }
    }

    /// Create a `StyleComplexColor` value that represents `auto`.
    pub fn auto() -> Self {
        StyleComplexColor {
            mColor: 0,
            mForegroundRatio: 255,
            mIsAuto: true,
        }
    }
}

impl From<ComputedColor> for StyleComplexColor {
    fn from(other: ComputedColor) -> Self {
        StyleComplexColor {
            mColor: convert_rgba_to_nscolor(&other.color).into(),
            mForegroundRatio: other.foreground_ratio,
            mIsAuto: false,
        }
    }
}

impl From<StyleComplexColor> for ComputedColor {
    fn from(other: StyleComplexColor) -> Self {
        debug_assert!(!other.mIsAuto);
        ComputedColor {
            color: convert_nscolor_to_rgba(other.mColor),
            foreground_ratio: other.mForegroundRatio,
        }
    }
}

impl<Color> From<CaretColor<Color>> for StyleComplexColor
where
    Color: Into<StyleComplexColor>,
{
    fn from(other: CaretColor<Color>) -> Self {
        match other {
            CaretColor::Color(color) => color.into(),
            CaretColor::Auto => StyleComplexColor::auto(),
        }
    }
}

impl<Color> From<StyleComplexColor> for CaretColor<Color>
where
    StyleComplexColor: Into<Color>,
{
    fn from(other: StyleComplexColor) -> Self {
        if !other.mIsAuto {
            CaretColor::Color(other.into())
        } else {
            CaretColor::Auto
        }
    }
}
