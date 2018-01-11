/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for SVG properties.

use app_units::Au;
use values::RGBA;
use values::computed::{ComputedUrl, LengthOrPercentage, NonNegativeLength};
use values::computed::{NonNegativeNumber, NonNegativeLengthOrPercentage, Number};
use values::computed::Opacity;
use values::generics::svg as generic;

pub use values::specified::SVGPaintOrder;

/// Computed SVG Paint value
pub type SVGPaint = generic::SVGPaint<RGBA, ComputedUrl>;
/// Computed SVG Paint Kind value
pub type SVGPaintKind = generic::SVGPaintKind<RGBA, ComputedUrl>;

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

/// A value of <length> | <percentage> | <number> for stroke-dashoffset.
/// <https://www.w3.org/TR/SVG11/painting.html#StrokeProperties>
pub type SvgLengthOrPercentageOrNumber =
    generic::SvgLengthOrPercentageOrNumber<LengthOrPercentage, Number>;

/// <length> | <percentage> | <number> | context-value
pub type SVGLength = generic::SVGLength<SvgLengthOrPercentageOrNumber>;

impl From<Au> for SVGLength {
    fn from(length: Au) -> Self {
        generic::SVGLength::Length(
            generic::SvgLengthOrPercentageOrNumber::LengthOrPercentage(length.into()))
    }
}

/// A value of <length> | <percentage> | <number> for stroke-width/stroke-dasharray.
/// <https://www.w3.org/TR/SVG11/painting.html#StrokeProperties>
pub type NonNegativeSvgLengthOrPercentageOrNumber =
    generic::SvgLengthOrPercentageOrNumber<NonNegativeLengthOrPercentage, NonNegativeNumber>;

impl Into<NonNegativeSvgLengthOrPercentageOrNumber> for SvgLengthOrPercentageOrNumber {
    fn into(self) -> NonNegativeSvgLengthOrPercentageOrNumber {
        match self {
            generic::SvgLengthOrPercentageOrNumber::LengthOrPercentage(lop) =>{
                generic::SvgLengthOrPercentageOrNumber::LengthOrPercentage(lop.into())
            },
            generic::SvgLengthOrPercentageOrNumber::Number(num) => {
                generic::SvgLengthOrPercentageOrNumber::Number(num.into())
            },
        }
    }
}

/// An non-negative wrapper of SVGLength.
pub type SVGWidth = generic::SVGLength<NonNegativeSvgLengthOrPercentageOrNumber>;

impl From<NonNegativeLength> for SVGWidth {
    fn from(length: NonNegativeLength) -> Self {
        generic::SVGLength::Length(
            generic::SvgLengthOrPercentageOrNumber::LengthOrPercentage(length.into()))
    }
}

/// [ <length> | <percentage> | <number> ]# | context-value
pub type SVGStrokeDashArray = generic::SVGStrokeDashArray<NonNegativeSvgLengthOrPercentageOrNumber>;

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
