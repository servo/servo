/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use euclid::size::Size2D;
use properties::ComputedValues;
use std::fmt;
use super::{CSSFloat, specified};

pub use cssparser::Color as CSSColor;
pub use self::image::{EndingShape as GradientShape, Gradient, GradientKind, Image};
pub use self::image::{LengthOrKeyword, LengthOrPercentageOrKeyword};
pub use super::specified::{Angle, BorderStyle, Time, UrlExtraData, UrlOrNone};
pub use self::length::{CalcLengthOrPercentage, LengthOrPercentage, LengthOrPercentageOrAuto};
pub use self::length::{LengthOrPercentageOrAutoOrContent, LengthOrPercentageOrNone, LengthOrNone};

pub mod basic_shape;
pub mod image;
pub mod length;
pub mod position;

pub struct Context<'a> {
    pub is_root_element: bool,
    pub viewport_size: Size2D<Au>,
    pub inherited_style: &'a ComputedValues,

    /// Values access through this need to be in the properties "computed early":
    /// color, text-decoration, font-size, display, position, float, border-*-style, outline-style
    pub style: ComputedValues,
}

impl<'a> Context<'a> {
    pub fn is_root_element(&self) -> bool { self.is_root_element }
    pub fn viewport_size(&self) -> Size2D<Au> { self.viewport_size }
    pub fn inherited_style(&self) -> &ComputedValues { &self.inherited_style }
    pub fn style(&self) -> &ComputedValues { &self.style }
    pub fn mutate_style(&mut self) -> &mut ComputedValues { &mut self.style }
}

pub trait ToComputedValue {
    type ComputedValue;

    #[inline]
    fn to_computed_value(&self, _context: &Context) -> Self::ComputedValue;

    #[inline]
    /// Convert a computed value to specified value form.
    ///
    /// This will be used for recascading during animation.
    /// Such from_computed_valued values should recompute to the same value.
    fn from_computed_value(computed: &Self::ComputedValue) -> Self;
}

pub trait ComputedValueAsSpecified {}

impl<T> ToComputedValue for T where T: ComputedValueAsSpecified + Clone {
    type ComputedValue = T;

    #[inline]
    fn to_computed_value(&self, _context: &Context) -> T {
        self.clone()
    }

    #[inline]
    fn from_computed_value(computed: &T) -> Self {
        computed.clone()
    }
}

impl ToComputedValue for specified::CSSColor {
    type ComputedValue = CSSColor;

    #[inline]
    fn to_computed_value(&self, _context: &Context) -> CSSColor {
        self.parsed
    }

    #[inline]
    fn from_computed_value(computed: &CSSColor) -> Self {
        specified::CSSColor {
            parsed: *computed,
            authored: None,
        }
    }
}

impl ComputedValueAsSpecified for specified::BorderStyle {}

impl ToComputedValue for specified::Length {
    type ComputedValue = Au;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Au {
        match *self {
            specified::Length::Absolute(length) => length,
            specified::Length::Calc(calc, range) => range.clamp(calc.to_computed_value(context).length()),
            specified::Length::FontRelative(length) =>
                length.to_computed_value(context.style().get_font().clone_font_size(),
                                         context.style().root_font_size()),
            specified::Length::ViewportPercentage(length) =>
                length.to_computed_value(context.viewport_size()),
            specified::Length::ServoCharacterWidth(length) =>
                length.to_computed_value(context.style().get_font().clone_font_size())
        }
    }

    #[inline]
    fn from_computed_value(computed: &Au) -> Self {
        specified::Length::Absolute(*computed)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct BorderRadiusSize(pub Size2D<LengthOrPercentage>);

impl BorderRadiusSize {
    pub fn zero() -> BorderRadiusSize {
        BorderRadiusSize(Size2D::new(LengthOrPercentage::Length(Au(0)), LengthOrPercentage::Length(Au(0))))
    }
}

impl ToComputedValue for specified::BorderRadiusSize {
    type ComputedValue = BorderRadiusSize;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> BorderRadiusSize {
        let w = self.0.width.to_computed_value(context);
        let h = self.0.height.to_computed_value(context);
        BorderRadiusSize(Size2D::new(w, h))
    }

    #[inline]
    fn from_computed_value(computed: &BorderRadiusSize) -> Self {
        let w = ToComputedValue::from_computed_value(&computed.0.width);
        let h = ToComputedValue::from_computed_value(&computed.0.height);
        specified::BorderRadiusSize(Size2D::new(w, h))
    }
}

impl ::cssparser::ToCss for BorderRadiusSize {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(self.0.width.to_css(dest));
        try!(dest.write_str("/"));
        self.0.height.to_css(dest)
    }
}


pub type Length = Au;
pub type Number = CSSFloat;
pub type Opacity = CSSFloat;
