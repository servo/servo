/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for color properties.

use crate::color::mix::ColorInterpolationMethod;
use crate::values::animated::color::AnimatedRGBA;
use crate::values::animated::ToAnimatedValue;
use crate::values::specified::percentage::ToPercentage;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// This struct represents a combined color from a numeric color and
/// the current foreground color (currentcolor keyword).
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToAnimatedValue, ToShmem)]
#[repr(C)]
pub enum GenericColor<RGBA, Percentage> {
    /// The actual numeric color.
    Numeric(RGBA),
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

impl<RGBA, Percentage> ColorMix<GenericColor<RGBA, Percentage>, Percentage> {
    fn to_rgba(&self) -> Option<RGBA>
    where
        RGBA: Clone + ToAnimatedValue<AnimatedValue = AnimatedRGBA>,
        Percentage: ToPercentage,
    {
        let left = self.left.as_numeric()?.clone().to_animated_value();
        let right = self.right.as_numeric()?.clone().to_animated_value();
        Some(ToAnimatedValue::from_animated_value(
            crate::color::mix::mix(
                &self.interpolation,
                &left,
                self.left_percentage.to_percentage(),
                &right,
                self.right_percentage.to_percentage(),
                self.normalize_weights,
            ),
        ))
    }
}

pub use self::GenericColor as Color;

impl<RGBA, Percentage> Color<RGBA, Percentage> {
    /// Returns the numeric rgba value if this color is numeric, or None
    /// otherwise.
    pub fn as_numeric(&self) -> Option<&RGBA> {
        match *self {
            Self::Numeric(ref rgba) => Some(rgba),
            _ => None,
        }
    }

    /// Simplifies the color-mix()es to the extent possible given a current
    /// color (or not).
    pub fn simplify(&mut self, current_color: Option<&RGBA>)
    where
        RGBA: Clone + ToAnimatedValue<AnimatedValue = AnimatedRGBA>,
        Percentage: ToPercentage,
    {
        match *self {
            Self::Numeric(..) => {},
            Self::CurrentColor => {
                if let Some(c) = current_color {
                    *self = Self::Numeric(c.clone());
                }
            },
            Self::ColorMix(ref mut mix) => {
                mix.left.simplify(current_color);
                mix.right.simplify(current_color);

                if let Some(mix) = mix.to_rgba() {
                    *self = Self::Numeric(mix);
                }
            },
        }
    }

    /// Returns a color value representing currentcolor.
    pub fn currentcolor() -> Self {
        Self::CurrentColor
    }

    /// Returns a numeric color representing the given RGBA value.
    pub fn rgba(color: RGBA) -> Self {
        Self::Numeric(color)
    }

    /// Whether it is a currentcolor value (no numeric color component).
    pub fn is_currentcolor(&self) -> bool {
        matches!(*self, Self::CurrentColor)
    }

    /// Whether it is a numeric color (no currentcolor component).
    pub fn is_numeric(&self) -> bool {
        matches!(*self, Self::Numeric(..))
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
