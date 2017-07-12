/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for SVG properties.

use app_units::Au;
use super::specified;
use values::RGBA;
use values::computed::{Context, NumberOrPercentage, Opacity};
use values::computed::{Percentage, ToComputedValue};
use values::computed::length::{Length, LengthOrPercentage};
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

/// <length> | <number> | <percentage>
/// This valule is for SVG values which allow the unitless length.
#[allow(missing_docs)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq, ToCss)]
pub enum SVGLengthOrPercentageOrNumber {
    Length(Length),
    NumberOrPercentage(NumberOrPercentage),
}

impl ToComputedValue for specified::SVGLengthOrPercentageOrNumber {
    type ComputedValue = SVGLengthOrPercentageOrNumber;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> SVGLengthOrPercentageOrNumber {
        match *self {
            specified::SVGLengthOrPercentageOrNumber::LengthOrPercentage(ref lop) =>
            {
                match lop.to_computed_value(context) {
                    LengthOrPercentage::Length(length) => {
                        SVGLengthOrPercentageOrNumber::Length(length)
                    },
                    LengthOrPercentage::Percentage(percentage) => {
                        SVGLengthOrPercentageOrNumber::NumberOrPercentage(
                            NumberOrPercentage::Percentage(percentage))
                    },
                    LengthOrPercentage::Calc(_) => {
                        // TODO: We need to support calc() (see bug 1386967)
                        panic!("Unexpected value.");
                    },
                }
            },
            specified::SVGLengthOrPercentageOrNumber::Number(number) =>
                SVGLengthOrPercentageOrNumber::NumberOrPercentage(NumberOrPercentage::Number(number.get())),
        }
    }

    #[inline]
    fn from_computed_value(computed: &SVGLengthOrPercentageOrNumber) -> Self {
        match computed {
            &SVGLengthOrPercentageOrNumber::Length(ref length) =>
                specified::SVGLengthOrPercentageOrNumber::LengthOrPercentage(
                    specified::LengthOrPercentage::Length(
                        specified::length::NoCalcLength::from_px(length.to_f32_px()))),
            &SVGLengthOrPercentageOrNumber::NumberOrPercentage(ref nop) => {
                match nop {
                    &NumberOrPercentage::Percentage(ref percentage) =>
                        specified::SVGLengthOrPercentageOrNumber::LengthOrPercentage(
                            specified::LengthOrPercentage::Percentage(Percentage(percentage.0))),
                    &NumberOrPercentage::Number(number) =>
                        specified::SVGLengthOrPercentageOrNumber::Number(
                            specified::Number::new(number)),
                }
            },
        }
    }
}

/// <length> | <percentage> | <number> | context-value
pub type SVGLength = generic::SVGLength<SVGLengthOrPercentageOrNumber>;

impl From<Au> for SVGLength {
    fn from(length: Au) -> Self {
        generic::SVGLength::Length(SVGLengthOrPercentageOrNumber::Length(length))
    }
}

/// [ <length> | <percentage> | <number> ]# | context-value
pub type SVGStrokeDashArray = generic::SVGStrokeDashArray<SVGLengthOrPercentageOrNumber>;

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
