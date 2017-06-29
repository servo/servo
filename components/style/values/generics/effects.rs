/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to effects.

use std::fmt;
use style_traits::values::{SequenceWriter, ToCss};
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
#[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToAnimatedValue, ToComputedValue, ToCss)]
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

/// A generic value for the `drop-shadow()` filter and the `text-shadow` property.
///
/// Contrary to the canonical order from the spec, the color is serialised
/// first, like in Gecko and Webkit.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, HasViewportPercentage, PartialEq, ToAnimatedValue, ToCss)]
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
