/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Rust helpers to interact with Gecko's StyleComplexColor.

use crate::gecko::values::{convert_nscolor_to_rgba, convert_rgba_to_nscolor};
use crate::gecko_bindings::structs::StyleComplexColor;
use crate::gecko_bindings::structs::StyleComplexColor_Tag as Tag;
use crate::values::computed::{Color as ComputedColor, ColorOrAuto, RGBAColor as ComputedRGBA};
use crate::values::generics::color::{
    Color as GenericColor, ColorOrAuto as GenericColorOrAuto, ComplexColorRatios,
};

impl StyleComplexColor {
    /// Create a `StyleComplexColor` value that represents `currentColor`.
    pub fn current_color() -> Self {
        StyleComplexColor {
            mColor: 0,
            mBgRatio: 0.,
            mFgRatio: 1.,
            mTag: Tag::eForeground,
        }
    }

    /// Create a `StyleComplexColor` value that represents `auto`.
    pub fn auto() -> Self {
        StyleComplexColor {
            mColor: 0,
            mBgRatio: 0.,
            mFgRatio: 1.,
            mTag: Tag::eAuto,
        }
    }
}

impl From<ComputedRGBA> for StyleComplexColor {
    fn from(other: ComputedRGBA) -> Self {
        StyleComplexColor {
            mColor: convert_rgba_to_nscolor(&other),
            mBgRatio: 1.,
            mFgRatio: 0.,
            mTag: Tag::eNumeric,
        }
    }
}

impl From<ComputedColor> for StyleComplexColor {
    fn from(other: ComputedColor) -> Self {
        match other {
            GenericColor::Numeric(color) => color.into(),
            GenericColor::Foreground => Self::current_color(),
            GenericColor::Complex(color, ratios) => {
                debug_assert!(ratios != ComplexColorRatios::NUMERIC);
                debug_assert!(ratios != ComplexColorRatios::FOREGROUND);
                StyleComplexColor {
                    mColor: convert_rgba_to_nscolor(&color).into(),
                    mBgRatio: ratios.bg,
                    mFgRatio: ratios.fg,
                    mTag: Tag::eComplex,
                }
            },
        }
    }
}

impl From<StyleComplexColor> for ComputedColor {
    fn from(other: StyleComplexColor) -> Self {
        match other.mTag {
            Tag::eNumeric => {
                debug_assert!(other.mBgRatio == 1. && other.mFgRatio == 0.);
                GenericColor::Numeric(convert_nscolor_to_rgba(other.mColor))
            },
            Tag::eForeground => {
                debug_assert!(other.mBgRatio == 0. && other.mFgRatio == 1.);
                GenericColor::Foreground
            },
            Tag::eComplex => {
                debug_assert!(other.mBgRatio != 1. || other.mFgRatio != 0.);
                debug_assert!(other.mBgRatio != 0. || other.mFgRatio != 1.);
                GenericColor::Complex(
                    convert_nscolor_to_rgba(other.mColor),
                    ComplexColorRatios {
                        bg: other.mBgRatio,
                        fg: other.mFgRatio,
                    },
                )
            },
            Tag::eAuto => unreachable!("Unsupport StyleComplexColor with tag eAuto"),
        }
    }
}

impl From<ColorOrAuto> for StyleComplexColor {
    fn from(other: ColorOrAuto) -> Self {
        match other {
            GenericColorOrAuto::Color(color) => color.into(),
            GenericColorOrAuto::Auto => StyleComplexColor::auto(),
        }
    }
}

impl From<StyleComplexColor> for ColorOrAuto {
    fn from(other: StyleComplexColor) -> Self {
        if other.mTag != Tag::eAuto {
            GenericColorOrAuto::Color(other.into())
        } else {
            GenericColorOrAuto::Auto
        }
    }
}
