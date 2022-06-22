/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for color properties.

use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, ToCss};
use crate::values::{Parse, ParserContext, Parser};
use crate::values::specified::percentage::ToPercentage;
use crate::values::animated::ToAnimatedValue;
use crate::values::animated::color::AnimatedRGBA;

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

/// A color space as defined in [1].
///
/// [1]: https://drafts.csswg.org/css-color-4/#typedef-color-space
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq, ToAnimatedValue, ToComputedValue, ToCss, ToResolvedValue, ToShmem)]
#[repr(u8)]
pub enum ColorSpace {
    /// The sRGB color space.
    Srgb,
    /// The linear-sRGB color space.
    LinearSrgb,
    /// The CIEXYZ color space.
    #[parse(aliases = "xyz-d65")]
    Xyz,
    /// https://drafts.csswg.org/css-color-4/#valdef-color-xyz
    XyzD50,
    /// The CIELAB color space.
    Lab,
    /// https://drafts.csswg.org/css-color-4/#valdef-hsl-hsl
    Hsl,
    /// https://drafts.csswg.org/css-color-4/#valdef-hwb-hwb
    Hwb,
    /// The CIELAB color space, expressed in cylindrical coordinates.
    Lch,
    // TODO: Oklab, Lch
}

impl ColorSpace {
    /// Returns whether this is a `<polar-color-space>`.
    pub fn is_polar(self) -> bool {
        match self {
            Self::Srgb | Self::LinearSrgb | Self::Xyz | Self::XyzD50 | Self::Lab => false,
            Self::Hsl | Self::Hwb | Self::Lch => true,
        }
    }
}

/// A hue-interpolation-method as defined in [1].
///
/// [1]: https://drafts.csswg.org/css-color-4/#typedef-hue-interpolation-method
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Parse, PartialEq, ToAnimatedValue, ToComputedValue, ToCss, ToResolvedValue, ToShmem)]
#[repr(u8)]
pub enum HueInterpolationMethod {
    /// https://drafts.csswg.org/css-color-4/#shorter
    Shorter,
    /// https://drafts.csswg.org/css-color-4/#longer
    Longer,
    /// https://drafts.csswg.org/css-color-4/#increasing
    Increasing,
    /// https://drafts.csswg.org/css-color-4/#decreasing
    Decreasing,
    /// https://drafts.csswg.org/css-color-4/#specified
    Specified,
}

/// https://drafts.csswg.org/css-color-4/#color-interpolation-method
#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, PartialEq, ToShmem, ToAnimatedValue, ToComputedValue, ToResolvedValue)]
#[repr(C)]
pub struct ColorInterpolationMethod {
    /// The color-space the interpolation should be done in.
    pub space: ColorSpace,
    /// The hue interpolation method.
    pub hue: HueInterpolationMethod,
}

impl ColorInterpolationMethod {
    /// Returns the srgb interpolation method.
    pub fn srgb() -> Self {
        Self {
            space: ColorSpace::Srgb,
            hue: HueInterpolationMethod::Shorter,
        }
    }
}

impl Parse for ColorInterpolationMethod {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_ident_matching("in")?;
        let space = ColorSpace::parse(input)?;
        // https://drafts.csswg.org/css-color-4/#hue-interpolation
        //     Unless otherwise specified, if no specific hue interpolation
        //     algorithm is selected by the host syntax, the default is shorter.
        let hue = if space.is_polar() {
            input.try_parse(|input| -> Result<_, ParseError<'i>> {
                let hue = HueInterpolationMethod::parse(input)?;
                input.expect_ident_matching("hue")?;
                Ok(hue)
            }).unwrap_or(HueInterpolationMethod::Shorter)
        } else {
            HueInterpolationMethod::Shorter
        };
        Ok(Self { space, hue })
    }
}

impl ToCss for ColorInterpolationMethod {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str("in ")?;
        self.space.to_css(dest)?;
        if self.hue != HueInterpolationMethod::Shorter {
            dest.write_char(' ')?;
            self.hue.to_css(dest)?;
            dest.write_str(" hue")?;
        }
        Ok(())
    }
}

/// A restricted version of the css `color-mix()` function, which only supports
/// percentages.
///
/// https://drafts.csswg.org/css-color-5/#color-mix
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToAnimatedValue, ToComputedValue, ToResolvedValue, ToShmem)]
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
        fn can_omit<Percentage: ToPercentage>(percent: &Percentage, other: &Percentage, is_left: bool) -> bool {
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
            dest.write_str(" ")?;
            self.left_percentage.to_css(dest)?;
        }
        dest.write_str(", ")?;
        self.right.to_css(dest)?;
        if !can_omit(&self.right_percentage, &self.left_percentage, false) {
            dest.write_str(" ")?;
            self.right_percentage.to_css(dest)?;
        }
        dest.write_str(")")
    }
}

impl<RGBA, Percentage> ColorMix<GenericColor<RGBA, Percentage>, Percentage> {
    fn to_rgba(&self) -> Option<RGBA>
    where
        RGBA: Clone + ToAnimatedValue<AnimatedValue = AnimatedRGBA>,
        Percentage: ToPercentage,
    {
        use crate::values::animated::color::Color as AnimatedColor;
        let left = self.left.as_numeric()?.clone().to_animated_value();
        let right = self.right.as_numeric()?.clone().to_animated_value();
        Some(ToAnimatedValue::from_animated_value(AnimatedColor::mix(
            &self.interpolation,
            &left,
            self.left_percentage.to_percentage(),
            &right,
            self.right_percentage.to_percentage(),
            self.normalize_weights,
        )))
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
