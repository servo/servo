/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for color properties.

/// Ratios representing the contribution of color and currentcolor to
/// the final color value.
///
/// NOTE(emilio): For animated colors, the sum of these two might be more than
/// one (because the background color would've been scaled down already). So
/// beware that it is not generally safe to assume that if bg is 1 then fg is 0,
/// for example.
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToAnimatedValue, ToShmem)]
#[repr(C)]
pub struct ComplexColorRatios {
    /// Numeric color contribution.
    pub bg: f32,
    /// currentcolor contribution.
    pub fg: f32,
}

impl ComplexColorRatios {
    /// Ratios representing a `Numeric` color.
    pub const NUMERIC: ComplexColorRatios = ComplexColorRatios { bg: 1., fg: 0. };
    /// Ratios representing the `CurrentColor` color.
    pub const CURRENT_COLOR: ComplexColorRatios = ComplexColorRatios { bg: 0., fg: 1. };
}

/// This struct represents a combined color from a numeric color and
/// the current foreground color (currentcolor keyword).
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToAnimatedValue, ToShmem)]
#[repr(C)]
pub struct GenericColor<RGBA> {
    /// The actual numeric color.
    pub color: RGBA,
    /// The ratios of mixing between numeric and currentcolor.
    /// The formula is: `color * ratios.bg + currentcolor * ratios.fg`.
    pub ratios: ComplexColorRatios,
}

pub use self::GenericColor as Color;

impl Color<cssparser::RGBA> {
    /// Returns a color value representing currentcolor.
    pub fn currentcolor() -> Self {
        Color {
            color: cssparser::RGBA::transparent(),
            ratios: ComplexColorRatios::CURRENT_COLOR,
        }
    }
}

impl<RGBA> Color<RGBA> {
    /// Create a color based upon the specified ratios.
    pub fn new(color: RGBA, ratios: ComplexColorRatios) -> Self {
        Self { color, ratios }
    }

    /// Returns a numeric color representing the given RGBA value.
    pub fn rgba(color: RGBA) -> Self {
        Self {
            color,
            ratios: ComplexColorRatios::NUMERIC,
        }
    }

    /// Whether it is a numeric color (no currentcolor component).
    pub fn is_numeric(&self) -> bool {
        self.ratios == ComplexColorRatios::NUMERIC
    }

    /// Whether it is a currentcolor value (no numeric color component).
    pub fn is_currentcolor(&self) -> bool {
        self.ratios == ComplexColorRatios::CURRENT_COLOR
    }
}

impl<RGBA> From<RGBA> for Color<RGBA> {
    fn from(color: RGBA) -> Self {
        Self::rgba(color)
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
