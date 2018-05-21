/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to effects.

/// A generic value for a single `box-shadow`.
#[derive(Animate, Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToAnimatedValue, ToAnimatedZero, ToCss)]
pub struct BoxShadow<Color, SizeLength, BlurShapeLength, ShapeLength> {
    /// The base shadow.
    pub base: SimpleShadow<Color, SizeLength, BlurShapeLength>,
    /// The spread radius.
    pub spread: ShapeLength,
    /// Whether this is an inset box shadow.
    #[animation(constant)]
    #[css(represents_keyword)]
    pub inset: bool,
}

/// A generic value for a single `filter`.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[animation(no_bound(Url))]
#[derive(Clone, ComputeSquaredDistance, Debug, MallocSizeOf, PartialEq,
         SpecifiedValueInfo, ToAnimatedValue, ToComputedValue, ToCss)]
pub enum Filter<Angle, Factor, Length, DropShadow, Url> {
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
    #[animation(error)]
    Url(Url),
}

/// A generic value for the `drop-shadow()` filter and the `text-shadow` property.
///
/// Contrary to the canonical order from the spec, the color is serialised
/// first, like in Gecko and Webkit.
#[derive(Animate, Clone, ComputeSquaredDistance, Debug, MallocSizeOf, PartialEq,
         SpecifiedValueInfo, ToAnimatedValue, ToAnimatedZero, ToCss)]
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
