/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to effects.

use std::fmt;
use style_traits::values::{SequenceWriter, ToCss};
use values::animated::{Restriction, ToAnimatedValue};
#[cfg(feature = "gecko")]
use values::specified::url::SpecifiedUrl;

/// A generic value for a single `box-shadow`.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToAnimatedValue)]
pub struct BoxShadow<Color, SizeLength, ShapeLength> {
    /// The base shadow.
    pub base: SimpleShadow<Color, SizeLength, ShapeLength>,
    /// The spread radius.
    pub spread: ShapeLength,
    /// Whether this is an inset box shadow.
    pub inset: bool,
}

/// A generic value for a single `filter`.
#[cfg_attr(feature = "servo", derive(Deserialize, HeapSizeOf, Serialize))]
#[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToComputedValue, ToCss)]
pub enum Filter<Angle, Factor, Length, DropShadow> {
    /// `blur(<length>)`
    #[css(function)]
    Blur(Length),
    /// `brightness(<factor>)`
    #[css(function)]
    Brightness(Factor),
    /// `contrast(<factor>)`
    #[css(function)]
    Contrast(Factor),
    /// `grayscale(<factor>)`
    #[css(function)]
    Grayscale(Factor),
    /// `hue-rotate(<angle>)`
    #[css(function)]
    HueRotate(Angle),
    /// `invert(<factor>)`
    #[css(function)]
    Invert(Factor),
    /// `opacity(<factor>)`
    #[css(function)]
    Opacity(Factor),
    /// `saturate(<factor>)`
    #[css(function)]
    Saturate(Factor),
    /// `sepia(<factor>)`
    #[css(function)]
    Sepia(Factor),
    /// `drop-shadow(...)`
    #[css(function)]
    DropShadow(DropShadow),
    /// `<url>`
    #[cfg(feature = "gecko")]
    Url(SpecifiedUrl),
}

macro_rules! convert_and_clamp_nonnegative_animated_value {
    ($value: expr) => {{
        ToAnimatedValue::from_animated_value_with_restriction($value, Restriction::NonNegative)
    }}
}

impl<Angle, Factor, Length, DropShadow> ToAnimatedValue for Filter<Angle, Factor, Length, DropShadow>
where
    Angle: ToAnimatedValue,
    Factor: ToAnimatedValue,
    Length: ToAnimatedValue,
    DropShadow: ToAnimatedValue,
{
    type AnimatedValue = Filter<Angle::AnimatedValue,
                                Factor::AnimatedValue,
                                Length::AnimatedValue,
                                DropShadow::AnimatedValue>;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        match self {
            Filter::Blur(length) => Filter::Blur(length.to_animated_value()),
            Filter::Brightness(factor) => Filter::Brightness(factor.to_animated_value()),
            Filter::Contrast(factor) => Filter::Contrast(factor.to_animated_value()),
            Filter::Grayscale(factor) => Filter::Grayscale(factor.to_animated_value()),
            Filter::HueRotate(angle) => Filter::HueRotate(angle.to_animated_value()),
            Filter::Invert(factor) => Filter::Invert(factor.to_animated_value()),
            Filter::Opacity(factor) => Filter::Opacity(factor.to_animated_value()),
            Filter::Saturate(factor) => Filter::Saturate(factor.to_animated_value()),
            Filter::Sepia(factor) => Filter::Sepia(factor.to_animated_value()),
            Filter::DropShadow(shadow) => Filter::DropShadow(shadow.to_animated_value()),
            #[cfg(feature = "gecko")]
            Filter::Url(url) => Filter::Url(url.to_animated_value())
        }
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        match animated {
            Filter::Blur(length) => {
                Filter::Blur(convert_and_clamp_nonnegative_animated_value!(length))
            },
            Filter::Brightness(factor) => {
                Filter::Brightness(convert_and_clamp_nonnegative_animated_value!(factor))
            },
            Filter::Contrast(factor) => {
                Filter::Contrast(convert_and_clamp_nonnegative_animated_value!(factor))
            },
            Filter::Grayscale(factor) => {
                Filter::Grayscale(convert_and_clamp_nonnegative_animated_value!(factor))
            },
            Filter::HueRotate(angle) => {
                Filter::HueRotate(ToAnimatedValue::from_animated_value(angle))
            },
            Filter::Invert(factor) => {
                Filter::Invert(convert_and_clamp_nonnegative_animated_value!(factor))
            },
            Filter::Opacity(factor) => {
                Filter::Opacity(convert_and_clamp_nonnegative_animated_value!(factor))
            },
            Filter::Saturate(factor) => {
                Filter::Saturate(convert_and_clamp_nonnegative_animated_value!(factor))
            },
            Filter::Sepia(factor) => {
                Filter::Sepia(convert_and_clamp_nonnegative_animated_value!(factor))
            },
            Filter::DropShadow(shadow) => {
                Filter::DropShadow(convert_and_clamp_nonnegative_animated_value!(shadow))
            },
            #[cfg(feature = "gecko")]
            Filter::Url(url) => Filter::Url(ToAnimatedValue::from_animated_value(url))
        }
    }
}

/// A generic value for the `drop-shadow()` filter and the `text-shadow` property.
///
/// Contrary to the canonical order from the spec, the color is serialised
/// first, like in Gecko and Webkit.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToCss)]
pub struct SimpleShadow<Color, SizeLength, ShapeLength> {
    /// Color.
    pub color: Color,
    /// Horizontal radius.
    pub horizontal: SizeLength,
    /// Vertical radius.
    pub vertical: SizeLength,
    /// Blur radius.
    pub blur: ShapeLength,
}

impl<Color, SizeLength, ShapeLength> ToAnimatedValue for SimpleShadow<Color, SizeLength, ShapeLength>
where
    Color: ToAnimatedValue,
    SizeLength: ToAnimatedValue,
    ShapeLength: ToAnimatedValue,
{
    type AnimatedValue = SimpleShadow<Color::AnimatedValue,
                                      SizeLength::AnimatedValue,
                                      ShapeLength::AnimatedValue>;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        Self::AnimatedValue {
            color: self.color.to_animated_value(),
            horizontal: self.horizontal.to_animated_value(),
            vertical: self.vertical.to_animated_value(),
            blur: self.blur.to_animated_value(),
        }
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        Self {
            color: ToAnimatedValue::from_animated_value(animated.color),
            horizontal: ToAnimatedValue::from_animated_value(animated.horizontal),
            vertical: ToAnimatedValue::from_animated_value(animated.vertical),
            blur: convert_and_clamp_nonnegative_animated_value!(animated.blur)
        }
    }
}

impl<Color, SizeLength, ShapeLength> ToCss for BoxShadow<Color, SizeLength, ShapeLength>
where
    Color: ToCss,
    SizeLength: ToCss,
    ShapeLength: ToCss,
{
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        {
            let mut writer = SequenceWriter::new(&mut *dest, " ");
            writer.item(&self.base)?;
            writer.item(&self.spread)?;
        }
        if self.inset {
            dest.write_str(" inset")?;
        }
        Ok(())
    }
}
