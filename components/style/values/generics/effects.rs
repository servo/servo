/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values related to effects.

/// A generic value for a single `box-shadow`.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericBoxShadow<Color, SizeLength, BlurShapeLength, ShapeLength> {
    /// The base shadow.
    pub base: GenericSimpleShadow<Color, SizeLength, BlurShapeLength>,
    /// The spread radius.
    pub spread: ShapeLength,
    /// Whether this is an inset box shadow.
    #[animation(constant)]
    #[css(represents_keyword)]
    pub inset: bool,
}

pub use self::GenericBoxShadow as BoxShadow;

/// A generic value for a single `filter`.
#[cfg_attr(feature = "servo", derive(Deserialize, Serialize))]
#[animation(no_bound(U))]
#[derive(
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericFilter<Angle, NonNegativeFactor, ZeroToOneFactor, Length, Shadow, U> {
    /// `blur(<length>)`
    #[css(function)]
    Blur(Length),
    /// `brightness(<factor>)`
    #[css(function)]
    Brightness(NonNegativeFactor),
    /// `contrast(<factor>)`
    #[css(function)]
    Contrast(NonNegativeFactor),
    /// `grayscale(<factor>)`
    #[css(function)]
    Grayscale(ZeroToOneFactor),
    /// `hue-rotate(<angle>)`
    #[css(function)]
    HueRotate(Angle),
    /// `invert(<factor>)`
    #[css(function)]
    Invert(ZeroToOneFactor),
    /// `opacity(<factor>)`
    #[css(function)]
    Opacity(ZeroToOneFactor),
    /// `saturate(<factor>)`
    #[css(function)]
    Saturate(NonNegativeFactor),
    /// `sepia(<factor>)`
    #[css(function)]
    Sepia(ZeroToOneFactor),
    /// `drop-shadow(...)`
    #[css(function)]
    DropShadow(Shadow),
    /// `<url>`
    #[animation(error)]
    Url(U),
}

pub use self::GenericFilter as Filter;

/// A generic value for the `drop-shadow()` filter and the `text-shadow` property.
///
/// Contrary to the canonical order from the spec, the color is serialised
/// first, like in Gecko and Webkit.
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericSimpleShadow<Color, SizeLength, ShapeLength> {
    /// Color.
    pub color: Color,
    /// Horizontal radius.
    pub horizontal: SizeLength,
    /// Vertical radius.
    pub vertical: SizeLength,
    /// Blur radius.
    pub blur: ShapeLength,
}

pub use self::GenericSimpleShadow as SimpleShadow;
