/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for color properties.

/// Ratios representing the contribution of color and currentcolor to
/// the final color value.
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

/// This enum represents a combined color from a numeric color and
/// the current foreground color (currentcolor keyword).
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToAnimatedValue, ToShmem)]
#[repr(C, u8)]
pub enum GenericColor<RGBA> {
    ///  Numeric RGBA color.
    Numeric(RGBA),

    /// The current foreground color.
    CurrentColor,

    /// A linear combination of numeric color and currentcolor.
    /// The formula is: `color * ratios.bg + currentcolor * ratios.fg`.
    Complex {
        /// The actual numeric color.
        color: RGBA,
        /// The ratios of mixing between numeric and currentcolor.
        ratios: ComplexColorRatios,
    },
}

pub use self::GenericColor as Color;

impl<RGBA> Color<RGBA> {
    /// Create a color based upon the specified ratios.
    pub fn with_ratios(color: RGBA, ratios: ComplexColorRatios) -> Self {
        if ratios == ComplexColorRatios::NUMERIC {
            Color::Numeric(color)
        } else if ratios == ComplexColorRatios::CURRENT_COLOR {
            Color::CurrentColor
        } else {
            Color::Complex { color, ratios }
        }
    }

    /// Returns a numeric color representing the given RGBA value.
    pub fn rgba(color: RGBA) -> Self {
        Color::Numeric(color)
    }

    /// Returns a complex color value representing currentcolor.
    pub fn currentcolor() -> Self {
        Color::CurrentColor
    }

    /// Whether it is a numeric color (no currentcolor component).
    pub fn is_numeric(&self) -> bool {
        matches!(*self, Color::Numeric(..))
    }

    /// Whether it is a currentcolor value (no numeric color component).
    pub fn is_currentcolor(&self) -> bool {
        matches!(*self, Color::CurrentColor)
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
