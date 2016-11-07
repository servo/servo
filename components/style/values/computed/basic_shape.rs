/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the computed value of
//! [`basic-shape`][basic-shape]s
//!
//! [basic-shape]: https://drafts.csswg.org/css-shapes/#typedef-basic-shape

use properties::shorthands::serialize_four_sides;
use std::fmt;
use style_traits::ToCss;
use url::Url;
use values::computed::{BorderRadiusSize, LengthOrPercentage};
use values::computed::UrlExtraData;
use values::computed::position::Position;

pub use values::specified::basic_shape::{FillRule, GeometryBox, ShapeBox};

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum ShapeSource<T> {
    Url(Url, UrlExtraData),
    Shape(BasicShape, Option<T>),
    Box(T),
    None,
}

impl<T> Default for ShapeSource<T> {
    fn default() -> Self {
        ShapeSource::None
    }
}

impl<T: ToCss> ToCss for ShapeSource<T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            ShapeSource::Url(ref url, _) => url.to_css(dest),
            ShapeSource::Shape(ref shape, Some(ref reference)) => {
                try!(shape.to_css(dest));
                try!(dest.write_str(" "));
                reference.to_css(dest)
            }
            ShapeSource::Shape(ref shape, None) => shape.to_css(dest),
            ShapeSource::Box(ref reference) => reference.to_css(dest),
            ShapeSource::None => dest.write_str("none"),

        }
    }
}


#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct InsetRect {
    pub top: LengthOrPercentage,
    pub right: LengthOrPercentage,
    pub bottom: LengthOrPercentage,
    pub left: LengthOrPercentage,
    pub round: Option<BorderRadius>,
}

impl ToCss for InsetRect {
    // XXXManishearth again, we should try to reduce the number of values printed here
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("inset("));
        try!(self.top.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.right.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.bottom.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.left.to_css(dest));
        if let Some(ref radius) = self.round {
            try!(dest.write_str(" round "));
            try!(radius.to_css(dest));
        }
        dest.write_str(")")
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
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

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-shapes/#funcdef-polygon
pub struct Polygon {
    pub fill: FillRule,
    pub coordinates: Vec<(LengthOrPercentage, LengthOrPercentage)>,
}

impl ToCss for Polygon {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("polygon("));
        let mut need_space = false;
        if self.fill != Default::default() {
            try!(self.fill.to_css(dest));
            try!(dest.write_str(", "));
        }
        for coord in &self.coordinates {
            if need_space {
                try!(dest.write_str(", "));
            }
            try!(coord.0.to_css(dest));
            try!(dest.write_str(" "));
            try!(coord.1.to_css(dest));
            need_space = true;
        }
        dest.write_str(")")
    }
}

/// https://drafts.csswg.org/css-shapes/#typedef-shape-radius
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum ShapeRadius {
    Length(LengthOrPercentage),
    ClosestSide,
    FarthestSide,
}

impl Default for ShapeRadius {
    fn default() -> Self {
        ShapeRadius::ClosestSide
    }
}

impl ToCss for ShapeRadius {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            ShapeRadius::Length(lop) => lop.to_css(dest),
            ShapeRadius::ClosestSide => dest.write_str("closest-side"),
            ShapeRadius::FarthestSide => dest.write_str("farthest-side"),
        }
    }
}

/// https://drafts.csswg.org/css-backgrounds-3/#border-radius
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct BorderRadius {
    pub top_left: BorderRadiusSize,
    pub top_right: BorderRadiusSize,
    pub bottom_right: BorderRadiusSize,
    pub bottom_left: BorderRadiusSize,
}

impl ToCss for BorderRadius {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if self.top_left.0.width == self.top_left.0.height &&
           self.top_right.0.width == self.top_right.0.height &&
           self.bottom_right.0.width == self.bottom_right.0.height &&
           self.bottom_left.0.width == self.bottom_left.0.height {
            serialize_four_sides(dest,
                                 &self.top_left.0.width,
                                 &self.top_right.0.width,
                                 &self.bottom_right.0.width,
                                 &self.bottom_left.0.width)
        } else {
            try!(serialize_four_sides(dest,
                                      &self.top_left.0.width,
                                      &self.top_right.0.width,
                                      &self.bottom_right.0.width,
                                      &self.bottom_left.0.width));
            try!(dest.write_str(" / "));
            serialize_four_sides(dest,
                                 &self.top_left.0.height,
                                 &self.top_right.0.height,
                                 &self.bottom_right.0.height,
                                 &self.bottom_left.0.height)
        }
    }
}
