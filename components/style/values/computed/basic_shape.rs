/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`basic-shape`][basic-shape]s
//!
//! [basic-shape]: https://drafts.csswg.org/css-shapes/#typedef-basic-shape

use std::fmt;
use style_traits::ToCss;
use values::computed::LengthOrPercentage;
use values::computed::position::Position;
use values::generics::basic_shape::{BorderRadius as GenericBorderRadius, ShapeRadius as GenericShapeRadius};
use values::generics::basic_shape::{InsetRect as GenericInsetRect, Polygon as GenericPolygon, ShapeSource};

pub use values::generics::basic_shape::FillRule;
pub use values::specified::basic_shape::{self, GeometryBox, ShapeBox};

/// The computed value used by `clip-path`
pub type ShapeWithGeometryBox = ShapeSource<BasicShape, GeometryBox>;

/// The computed value used by `shape-outside`
pub type ShapeWithShapeBox = ShapeSource<BasicShape, ShapeBox>;

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum BasicShape {
    Inset(InsetRect),
    Circle(Circle),
    Ellipse(Ellipse),
    Polygon(Polygon),
}

impl ToCss for BasicShape {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            BasicShape::Inset(ref rect) => rect.to_css(dest),
            BasicShape::Circle(ref circle) => circle.to_css(dest),
            BasicShape::Ellipse(ref e) => e.to_css(dest),
            BasicShape::Polygon(ref poly) => poly.to_css(dest),
        }
    }
}

/// The computed value of `inset()`
pub type InsetRect = GenericInsetRect<LengthOrPercentage>;

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct Circle {
    pub radius: ShapeRadius,
    pub position: Position,
}

impl ToCss for Circle {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(self.radius.to_css(dest));
        try!(dest.write_str(" at "));
        self.position.to_css(dest)
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct Ellipse {
    pub semiaxis_x: ShapeRadius,
    pub semiaxis_y: ShapeRadius,
    pub position: Position,
}

impl ToCss for Ellipse {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("ellipse("));
        if (self.semiaxis_x, self.semiaxis_y) != Default::default() {
            try!(self.semiaxis_x.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.semiaxis_y.to_css(dest));
            try!(dest.write_str(" "));
        }
        try!(dest.write_str("at "));
        try!(self.position.to_css(dest));
        dest.write_str(")")
    }
}

/// The computed value of `Polygon`
pub type Polygon = GenericPolygon<LengthOrPercentage>;

/// The computed value of `BorderRadius`
pub type BorderRadius = GenericBorderRadius<LengthOrPercentage>;

/// The computed value of `ShapeRadius`
pub type ShapeRadius = GenericShapeRadius<LengthOrPercentage>;

impl Copy for ShapeRadius {}
