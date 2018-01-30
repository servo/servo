/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`basic-shape`][basic-shape]s
//!
//! [basic-shape]: https://drafts.csswg.org/css-shapes/#typedef-basic-shape

use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};
use values::computed::{LengthOrPercentage, ComputedUrl, Image};
use values::generics::basic_shape::{BasicShape as GenericBasicShape};
use values::generics::basic_shape::{Circle as GenericCircle, ClippingShape as GenericClippingShape};
use values::generics::basic_shape::{Ellipse as GenericEllipse, FloatAreaShape as GenericFloatAreaShape};
use values::generics::basic_shape::{InsetRect as GenericInsetRect, ShapeRadius as GenericShapeRadius};

/// A computed clipping shape.
pub type ClippingShape = GenericClippingShape<BasicShape, ComputedUrl>;

/// A computed float area shape.
pub type FloatAreaShape = GenericFloatAreaShape<BasicShape, Image>;

/// A computed basic shape.
pub type BasicShape = GenericBasicShape<LengthOrPercentage, LengthOrPercentage, LengthOrPercentage>;

/// The computed value of `inset()`
pub type InsetRect = GenericInsetRect<LengthOrPercentage>;

/// A computed circle.
pub type Circle = GenericCircle<LengthOrPercentage, LengthOrPercentage, LengthOrPercentage>;

/// A computed ellipse.
pub type Ellipse = GenericEllipse<LengthOrPercentage, LengthOrPercentage, LengthOrPercentage>;

/// The computed value of `ShapeRadius`
pub type ShapeRadius = GenericShapeRadius<LengthOrPercentage>;

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
