/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Computed types for SVG properties.

use crate::values::computed::color::Color;
use crate::values::computed::url::ComputedUrl;
use crate::values::computed::{LengthPercentage, NonNegativeLengthPercentage};
use crate::values::computed::{NonNegativeNumber, Number, Opacity};
use crate::values::generics::svg as generic;
use crate::values::RGBA;

pub use crate::values::specified::SVGPaintOrder;

pub use crate::values::specified::MozContextProperties;

/// Computed SVG Paint value
pub type SVGPaint = generic::SVGPaint<Color, ComputedUrl>;
/// Computed SVG Paint Kind value
pub type SVGPaintKind = generic::SVGPaintKind<Color, ComputedUrl>;

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
        let rgba = RGBA::from_floats(0., 0., 0., 1.).into();
        SVGPaint {
            kind: generic::SVGPaintKind::Color(rgba),
            fallback: None,
        }
    }
}

/// A value of <length> | <percentage> | <number> for stroke-dashoffset.
/// <https://www.w3.org/TR/SVG11/painting.html#StrokeProperties>
pub type SvgLengthPercentageOrNumber =
    generic::SvgLengthPercentageOrNumber<LengthPercentage, Number>;

/// <length> | <percentage> | <number> | context-value
pub type SVGLength = generic::SVGLength<SvgLengthPercentageOrNumber>;

impl SVGLength {
    /// `0px`
    pub fn zero() -> Self {
        generic::SVGLength::Length(generic::SvgLengthPercentageOrNumber::LengthPercentage(
            LengthPercentage::zero(),
        ))
    }
}

/// A value of <length> | <percentage> | <number> for stroke-width/stroke-dasharray.
/// <https://www.w3.org/TR/SVG11/painting.html#StrokeProperties>
pub type NonNegativeSvgLengthPercentageOrNumber =
    generic::SvgLengthPercentageOrNumber<NonNegativeLengthPercentage, NonNegativeNumber>;

// FIXME(emilio): This is really hacky, and can go away with a bit of work on
// the clone_stroke_width code in gecko.mako.rs.
impl Into<NonNegativeSvgLengthPercentageOrNumber> for SvgLengthPercentageOrNumber {
    fn into(self) -> NonNegativeSvgLengthPercentageOrNumber {
        match self {
            generic::SvgLengthPercentageOrNumber::LengthPercentage(lop) => {
                generic::SvgLengthPercentageOrNumber::LengthPercentage(lop.into())
            },
            generic::SvgLengthPercentageOrNumber::Number(num) => {
                generic::SvgLengthPercentageOrNumber::Number(num.into())
            },
        }
    }
}

/// An non-negative wrapper of SVGLength.
pub type SVGWidth = generic::SVGLength<NonNegativeSvgLengthPercentageOrNumber>;

impl SVGWidth {
    /// `1px`.
    pub fn one() -> Self {
        use crate::values::generics::NonNegative;
        generic::SVGLength::Length(generic::SvgLengthPercentageOrNumber::LengthPercentage(
            NonNegative(LengthPercentage::one()),
        ))
    }
}

/// [ <length> | <percentage> | <number> ]# | context-value
pub type SVGStrokeDashArray = generic::SVGStrokeDashArray<NonNegativeSvgLengthPercentageOrNumber>;

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
