/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`basic-shape`][basic-shape]s
//!
//! [basic-shape]: https://drafts.csswg.org/css-shapes/#typedef-basic-shape

use crate::values::computed::url::ComputedUrl;
use crate::values::computed::{Image, LengthPercentage, NonNegativeLengthPercentage};
use crate::values::generics::basic_shape as generic;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// A computed alias for FillRule.
pub use crate::values::generics::basic_shape::FillRule;

/// A computed clipping shape.
pub type ClippingShape = generic::ClippingShape<BasicShape, ComputedUrl>;

/// A computed float area shape.
pub type FloatAreaShape = generic::FloatAreaShape<BasicShape, Image>;

/// A computed basic shape.
pub type BasicShape = generic::GenericBasicShape<
    LengthPercentage,
    LengthPercentage,
    LengthPercentage,
    NonNegativeLengthPercentage,
>;

/// The computed value of `inset()`
pub type InsetRect = generic::InsetRect<LengthPercentage, NonNegativeLengthPercentage>;

/// A computed circle.
pub type Circle = generic::Circle<LengthPercentage, LengthPercentage, NonNegativeLengthPercentage>;

/// A computed ellipse.
pub type Ellipse =
    generic::Ellipse<LengthPercentage, LengthPercentage, NonNegativeLengthPercentage>;

/// The computed value of `ShapeRadius`
pub type ShapeRadius = generic::GenericShapeRadius<NonNegativeLengthPercentage>;

impl ToCss for Circle {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str("circle(")?;
        self.radius.to_css(dest)?;
        dest.write_str(" at ")?;
        self.position.to_css(dest)?;
        dest.write_str(")")
    }
}

impl ToCss for Ellipse {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str("ellipse(")?;
        if (self.semiaxis_x, self.semiaxis_y) != Default::default() {
            self.semiaxis_x.to_css(dest)?;
            dest.write_str(" ")?;
            self.semiaxis_y.to_css(dest)?;
            dest.write_str(" ")?;
        }
        dest.write_str("at ")?;
        self.position.to_css(dest)?;
        dest.write_str(")")
    }
}
