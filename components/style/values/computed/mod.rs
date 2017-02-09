/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values.

use app_units::Au;
use euclid::size::Size2D;
use font_metrics::FontMetricsProvider;
use properties::ComputedValues;
use std::fmt;
use style_traits::ToCss;
use super::{CSSFloat, specified};

pub use cssparser::Color as CSSColor;
pub use self::image::{AngleOrCorner, EndingShape as GradientShape, Gradient, GradientKind, Image};
pub use self::image::{LengthOrKeyword, LengthOrPercentageOrKeyword};
pub use super::{Auto, Either, None_};
pub use super::specified::{Angle, BorderStyle, GridLine, Time, UrlOrNone};
pub use super::specified::url::UrlExtraData;
pub use self::length::{CalcLengthOrPercentage, Length, LengthOrNumber, LengthOrPercentage, LengthOrPercentageOrAuto};
pub use self::length::{LengthOrPercentageOrAutoOrContent, LengthOrPercentageOrNone, LengthOrNone};
pub use self::position::Position;

pub mod basic_shape;
pub mod image;
pub mod length;
pub mod position;

/// A `Context` is all the data a specified value could ever need to compute
/// itself and be transformed to a computed value.
pub struct Context<'a> {
    /// Whether the current element is the root element.
    pub is_root_element: bool,

    /// The current viewport size.
    pub viewport_size: Size2D<Au>,

    /// The style we're inheriting from.
    pub inherited_style: &'a ComputedValues,

    /// Values access through this need to be in the properties "computed
    /// early": color, text-decoration, font-size, display, position, float,
    /// border-*-style, outline-style, font-family, writing-mode...
    pub style: ComputedValues,

    /// A font metrics provider, used to access font metrics to implement
    /// font-relative units.
    ///
    /// TODO(emilio): This should be required, see #14079.
    pub font_metrics_provider: Option<&'a FontMetricsProvider>,
}

impl<'a> Context<'a> {
    /// Whether the current element is the root element.
    pub fn is_root_element(&self) -> bool { self.is_root_element }
    /// The current viewport size.
    pub fn viewport_size(&self) -> Size2D<Au> { self.viewport_size }
    /// The style we're inheriting from.
    pub fn inherited_style(&self) -> &ComputedValues { &self.inherited_style }
    /// The current style. Note that only "eager" properties should be accessed
    /// from here, see the comment in the member.
    pub fn style(&self) -> &ComputedValues { &self.style }
    /// A mutable reference to the current style.
    pub fn mutate_style(&mut self) -> &mut ComputedValues { &mut self.style }
}

/// A trait to represent the conversion between computed and specified values.
pub trait ToComputedValue {
    /// The computed value type we're going to be converted to.
    type ComputedValue;

    /// Convert a specified value to a computed value, using itself and the data
    /// inside the `Context`.
    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue;

    #[inline]
    /// Convert a computed value to specified value form.
    ///
    /// This will be used for recascading during animation.
    /// Such from_computed_valued values should recompute to the same value.
    fn from_computed_value(computed: &Self::ComputedValue) -> Self;
}

/// A marker trait to represent that the specified value is also the computed
/// value.
pub trait ComputedValueAsSpecified {}

impl<T> ToComputedValue for T
    where T: ComputedValueAsSpecified + Clone,
{
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

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct BorderRadiusSize(pub Size2D<LengthOrPercentage>);

impl BorderRadiusSize {
    #[allow(missing_docs)]
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

impl ToCss for BorderRadiusSize {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(self.0.width.to_css(dest));
        try!(dest.write_str("/"));
        self.0.height.to_css(dest)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct Shadow {
    pub offset_x: Au,
    pub offset_y: Au,
    pub blur_radius: Au,
    pub spread_radius: Au,
    pub color: CSSColor,
    pub inset: bool,
}

/// A `<number>` value.
pub type Number = CSSFloat;

/// A type used for opacity.
pub type Opacity = CSSFloat;


#[derive(Clone, PartialEq, Eq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
/// A computed cliprect for clip and image-region
pub struct ClipRect {
    pub top: Au,
    pub right: Option<Au>,
    pub bottom: Option<Au>,
    pub left: Au,
}

impl ToCss for ClipRect {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("rect("));
        try!(self.top.to_css(dest));
        try!(dest.write_str(", "));
        if let Some(right) = self.right {
            try!(right.to_css(dest));
            try!(dest.write_str(", "));
        } else {
            try!(dest.write_str("auto, "));
        }

        if let Some(bottom) = self.bottom {
            try!(bottom.to_css(dest));
            try!(dest.write_str(", "));
        } else {
            try!(dest.write_str("auto, "));
        }

        try!(self.left.to_css(dest));
        dest.write_str(")")
    }
}

/// rect(...) | auto
pub type ClipRectOrAuto = Either<ClipRect, Auto>;

impl ClipRectOrAuto {
    /// Return an auto (default for clip-rect and image-region) value
    pub fn auto() -> Self {
        Either::Second(Auto)
    }

    /// Check if it is auto
    pub fn is_auto(&self) -> bool {
        match *self {
            Either::Second(_) => true,
            _ => false
        }
    }
}
