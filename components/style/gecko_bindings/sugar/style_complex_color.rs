/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Rust helpers to interact with Gecko's StyleComplexColor.

use cssparser::Color;
use gecko::values::{convert_nscolor_to_rgba, convert_rgba_to_nscolor};
use gecko_bindings::structs::{nscolor, StyleComplexColor};

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

impl From<Color> for StyleComplexColor {
    fn from(other: Color) -> Self {
        match other {
            Color::RGBA(rgba) => convert_rgba_to_nscolor(&rgba).into(),
            Color::CurrentColor => StyleComplexColor::current_color(),
        }
    }
}

impl From<StyleComplexColor> for Color {
    fn from(other: StyleComplexColor) -> Self {
        if other.mForegroundRatio == 0 {
            Color::RGBA(convert_nscolor_to_rgba(other.mColor))
        } else if other.mForegroundRatio == 255 {
            Color::CurrentColor
        } else {
            // FIXME #13546 handle interpolation values
            Color::CurrentColor
        }
    }
}
