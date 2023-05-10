/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for color properties.

use crate::color::mix::ColorInterpolationMethod;
use crate::color::AbsoluteColor;
use crate::values::specified::percentage::ToPercentage;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// This struct represents a combined color from a numeric color and
/// the current foreground color (currentcolor keyword).
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToAnimatedValue, ToShmem)]
#[repr(C)]
pub enum GenericColor<Percentage> {
    /// The actual numeric color.
    Absolute(AbsoluteColor),
    /// The `CurrentColor` keyword.
    CurrentColor,
    /// The color-mix() function.
    ColorMix(Box<GenericColorMix<Self, Percentage>>),
}

/// A restricted version of the css `color-mix()` function, which only supports
/// percentages.
///
/// https://drafts.csswg.org/css-color-5/#color-mix
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    ToAnimatedValue,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[allow(missing_docs)]
#[repr(C)]
pub struct GenericColorMix<Color, Percentage> {
    pub interpolation: ColorInterpolationMethod,
    pub left: Color,
    pub left_percentage: Percentage,
    pub right: Color,
    pub right_percentage: Percentage,
    pub normalize_weights: bool,
}

pub use self::GenericColorMix as ColorMix;

impl<Color: ToCss, Percentage: ToCss + ToPercentage> ToCss for ColorMix<Color, Percentage> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        fn can_omit<Percentage: ToPercentage>(
            percent: &Percentage,
            other: &Percentage,
            is_left: bool,
        ) -> bool {
            if percent.is_calc() {
                return false;
            }
            if percent.to_percentage() == 0.5 {
                return other.to_percentage() == 0.5;
            }
            if is_left {
                return false;
            }
            (1.0 - percent.to_percentage() - other.to_percentage()).abs() <= f32::EPSILON
        }

        dest.write_str("color-mix(")?;
        self.interpolation.to_css(dest)?;
        dest.write_str(", ")?;
        self.left.to_css(dest)?;
        if !can_omit(&self.left_percentage, &self.right_percentage, true) {
            dest.write_char(' ')?;
            self.left_percentage.to_css(dest)?;
        }
        dest.write_str(", ")?;
        self.right.to_css(dest)?;
        if !can_omit(&self.right_percentage, &self.left_percentage, false) {
            dest.write_char(' ')?;
            self.right_percentage.to_css(dest)?;
        }
        dest.write_char(')')
    }
}

impl<Percentage> ColorMix<GenericColor<Percentage>, Percentage> {
    /// Mix the colors so that we get a single color. If any of the 2 colors are
    /// not mixable (perhaps not absolute?), then return None.
    pub fn mix_to_absolute(&self) -> Option<AbsoluteColor>
    where
        Percentage: ToPercentage,
    {
        let left = self.left.as_absolute()?;
        let right = self.right.as_absolute()?;

        Some(crate::color::mix::mix(
            self.interpolation,
            &left,
            self.left_percentage.to_percentage(),
            &right,
            self.right_percentage.to_percentage(),
            self.normalize_weights,
        ))
    }
}

pub use self::GenericColor as Color;

impl<Percentage> Color<Percentage> {
    /// If this color is absolute return it's value, otherwise return None.
    pub fn as_absolute(&self) -> Option<&AbsoluteColor> {
        match *self {
            Self::Absolute(ref absolute) => Some(absolute),
            _ => None,
        }
    }

    /// Returns a color value representing currentcolor.
    pub fn currentcolor() -> Self {
        Self::CurrentColor
    }

    /// Whether it is a currentcolor value (no numeric color component).
    pub fn is_currentcolor(&self) -> bool {
        matches!(*self, Self::CurrentColor)
    }

    /// Whether this color is an absolute color.
    pub fn is_absolute(&self) -> bool {
        matches!(*self, Self::Absolute(..))
    }
}

/// Either `<color>` or `auto`.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToResolvedValue,
    ToCss,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericColorOrAuto<C> {
    /// A `<color>`.
    Color(C),
    /// `auto`
    Auto,
}

pub use self::GenericColorOrAuto as ColorOrAuto;

/// Caret color is effectively a ColorOrAuto, but resolves `auto` to
/// currentColor.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToShmem,
)]
#[repr(transparent)]
pub struct GenericCaretColor<C>(pub GenericColorOrAuto<C>);

impl<C> GenericCaretColor<C> {
    /// Returns the `auto` value.
    pub fn auto() -> Self {
        GenericCaretColor(GenericColorOrAuto::Auto)
    }
}

pub use self::GenericCaretColor as CaretColor;
