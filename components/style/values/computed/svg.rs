/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for SVG properties.

use app_units::Au;
use values::{Either, RGBA};
use values::computed::{LengthOrPercentageOrNumber, Opacity};
use values::generics::svg as generic;

/// Computed SVG Paint value
pub type SVGPaint = generic::SVGPaint<RGBA>;
/// Computed SVG Paint Kind value
pub type SVGPaintKind = generic::SVGPaintKind<RGBA>;

impl Default for SVGPaint {
    fn default() -> Self {
        SVGPaint {
            kind: generic::SVGPaintKind::None,
            fallback: None,
        }
    }
}

impl SVGPaint {
    /// Opaque black color
    pub fn black() -> Self {
        let rgba = RGBA::from_floats(0., 0., 0., 1.);
        SVGPaint {
            kind: generic::SVGPaintKind::Color(rgba),
            fallback: None,
        }
    }
}

/// <length> | <percentage> | <number> | context-value
pub type SVGLength = generic::SVGLength<LengthOrPercentageOrNumber>;

impl From<Au> for SVGLength {
    fn from(length: Au) -> Self {
        generic::SVGLength::Length(Either::Second(length.into()))
    }
}

/// [ <length> | <percentage> | <number> ]# | context-value
pub type SVGStrokeDashArray = generic::SVGStrokeDashArray<LengthOrPercentageOrNumber>;

impl Default for SVGStrokeDashArray {
    fn default() -> Self {
        generic::SVGStrokeDashArray::Values(vec![])
    }
}

/// <opacity-value> | context-fill-opacity | context-stroke-opacity
pub type SVGOpacity = generic::SVGOpacity<Opacity>;

impl Default for SVGOpacity {
    fn default() -> Self {
        generic::SVGOpacity::Opacity(1.)
    }
}
