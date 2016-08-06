/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`basic-shape`][basic-shape]s
//!
//! [basic-shape]: https://drafts.csswg.org/css-shapes/#typedef-basic-shape

use cssparser::{Parser, ToCss};
use properties::shorthands::{parse_four_sides, serialize_four_sides};
use std::fmt;
use values::computed::basic_shape as computed_basic_shape;
use values::computed::{Context, ToComputedValue, ComputedValueAsSpecified};
use values::specified::position::Position;
use values::specified::{BorderRadiusSize, LengthOrPercentage, Percentage};

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum BasicShape {
    Inset(InsetRect),
    Circle(Circle),
    Ellipse(Ellipse),
    Polygon(Polygon),
}

impl BasicShape {
    pub fn parse(input: &mut Parser) -> Result<BasicShape, ()> {
        match_ignore_ascii_case! { try!(input.expect_function()),
            "inset" => {
                Ok(BasicShape::Inset(
                   try!(input.parse_nested_block(InsetRect::parse_function_arguments))))
            },
            "circle" => {
                Ok(BasicShape::Circle(
                   try!(input.parse_nested_block(Circle::parse_function_arguments))))
            },
            "ellipse" => {
                Ok(BasicShape::Ellipse(
                   try!(input.parse_nested_block(Ellipse::parse_function_arguments))))
            },
            "polygon" => {
                Ok(BasicShape::Polygon(
                   try!(input.parse_nested_block(Polygon::parse_function_arguments))))
            },
            _ => Err(())
        }
    }
}

impl ToCss for BasicShape {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            BasicShape::Inset(rect) => rect.to_css(dest),
            BasicShape::Circle(circle) => circle.to_css(dest),
            BasicShape::Ellipse(e) => e.to_css(dest),
            BasicShape::Polygon(ref poly) => poly.to_css(dest),
        }
    }
}

impl ToComputedValue for BasicShape {
    type ComputedValue = computed_basic_shape::BasicShape;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        match *self {
            BasicShape::Inset(rect) => computed_basic_shape::BasicShape::Inset(rect.to_computed_value(cx)),
            BasicShape::Circle(circle) => computed_basic_shape::BasicShape::Circle(circle.to_computed_value(cx)),
            BasicShape::Ellipse(e) => computed_basic_shape::BasicShape::Ellipse(e.to_computed_value(cx)),
            BasicShape::Polygon(ref poly) => computed_basic_shape::BasicShape::Polygon(poly.to_computed_value(cx)),
        }
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-shapes/#funcdef-inset
pub struct InsetRect {
    pub top: LengthOrPercentage,
    pub right: LengthOrPercentage,
    pub bottom: LengthOrPercentage,
    pub left: LengthOrPercentage,
    pub round: Option<BorderRadius>,
}

impl InsetRect {
    pub fn parse(input: &mut Parser) -> Result<InsetRect, ()> {
        match_ignore_ascii_case! { try!(input.expect_function()),
            "inset" => {
                Ok(try!(input.parse_nested_block(InsetRect::parse_function_arguments)))
            },
            _ => Err(())
        }
    }
    pub fn parse_function_arguments(input: &mut Parser) -> Result<InsetRect, ()> {
        let (t, r, b, l) = try!(parse_four_sides(input, LengthOrPercentage::parse));
        let mut rect = InsetRect {
            top: t,
            right: r,
            bottom: b,
            left: l,
            round: None,
        };
        if let Ok(_) = input.try(|input| input.expect_ident_matching("round")) {
            rect.round = Some(try!(BorderRadius::parse(input)));
        }
        Ok(rect)
    }
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

impl ToComputedValue for InsetRect {
    type ComputedValue = computed_basic_shape::InsetRect;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        computed_basic_shape::InsetRect {
            top: self.top.to_computed_value(cx),
            right: self.right.to_computed_value(cx),
            bottom: self.bottom.to_computed_value(cx),
            left: self.left.to_computed_value(cx),
            round: self.round.map(|r| r.to_computed_value(cx)),
        }
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-shapes/#funcdef-circle
pub struct Circle {
    pub radius: ShapeRadius,
    pub position: Position,
}

impl Circle {
    pub fn parse(input: &mut Parser) -> Result<Circle, ()> {
        match_ignore_ascii_case! { try!(input.expect_function()),
            "circle" => {
                Ok(try!(input.parse_nested_block(Circle::parse_function_arguments)))
            },
            _ => Err(())
        }
    }
    pub fn parse_function_arguments(input: &mut Parser) -> Result<Circle, ()> {
        let radius = input.try(ShapeRadius::parse).ok().unwrap_or_else(Default::default);
        let position = if let Ok(_) = input.try(|input| input.expect_ident_matching("at")) {
            try!(Position::parse(input))
        } else {
            // Defaults to origin
            Position {
                horizontal: LengthOrPercentage::Percentage(Percentage(0.5)),
                vertical: LengthOrPercentage::Percentage(Percentage(0.5)),
            }
        };
        Ok(Circle {
            radius: radius,
            position: position,
        })
    }
}

impl ToCss for Circle {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("circle("));
        if ShapeRadius::ClosestSide != self.radius {
            try!(self.radius.to_css(dest));
            try!(dest.write_str(" "));
        }
        try!(dest.write_str("at "));
        try!(self.position.to_css(dest));
        dest.write_str(")")
    }
}

impl ToComputedValue for Circle {
    type ComputedValue = computed_basic_shape::Circle;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        computed_basic_shape::Circle {
            radius: self.radius.to_computed_value(cx),
            position: self.position.to_computed_value(cx),
        }
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-shapes/#funcdef-ellipse
pub struct Ellipse {
    pub semiaxis_x: ShapeRadius,
    pub semiaxis_y: ShapeRadius,
    pub position: Position,
}


impl Ellipse {
    pub fn parse(input: &mut Parser) -> Result<Ellipse, ()> {
        match_ignore_ascii_case! { try!(input.expect_function()),
            "ellipse" => {
                Ok(try!(input.parse_nested_block(Ellipse::parse_function_arguments)))
            },
            _ => Err(())
        }
    }
    pub fn parse_function_arguments(input: &mut Parser) -> Result<Ellipse, ()> {
        let (a, b) = input.try(|input| -> Result<_, ()> {
            Ok((try!(ShapeRadius::parse(input)), try!(ShapeRadius::parse(input))))
        }).ok().unwrap_or_default();
        let position = if let Ok(_) = input.try(|input| input.expect_ident_matching("at")) {
            try!(Position::parse(input))
        } else {
            // Defaults to origin
            Position {
                horizontal: LengthOrPercentage::Percentage(Percentage(0.5)),
                vertical: LengthOrPercentage::Percentage(Percentage(0.5)),
            }
        };
        Ok(Ellipse {
            semiaxis_x: a,
            semiaxis_y: b,
            position: position,
        })
    }
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

impl ToComputedValue for Ellipse {
    type ComputedValue = computed_basic_shape::Ellipse;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        computed_basic_shape::Ellipse {
            semiaxis_x: self.semiaxis_x.to_computed_value(cx),
            semiaxis_y: self.semiaxis_y.to_computed_value(cx),
            position: self.position.to_computed_value(cx),
        }
    }
}


#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-shapes/#funcdef-polygon
pub struct Polygon {
    pub fill: FillRule,
    pub coordinates: Vec<(LengthOrPercentage, LengthOrPercentage)>,
}

impl Polygon {
    pub fn parse(input: &mut Parser) -> Result<Polygon, ()> {
        match_ignore_ascii_case! { try!(input.expect_function()),
            "polygon" => {
                Ok(try!(input.parse_nested_block(Polygon::parse_function_arguments)))
            },
            _ => Err(())
        }
    }
    pub fn parse_function_arguments(input: &mut Parser) -> Result<Polygon, ()> {
        let fill = input.try(|input| {
            let fill = FillRule::parse(input);
            // only eat the comma if there is something before it
            try!(input.expect_comma());
            fill
        }).ok().unwrap_or_else(Default::default);
        let buf = try!(input.parse_comma_separated(|input| {
            Ok((try!(LengthOrPercentage::parse(input)),
                try!(LengthOrPercentage::parse(input))))
        }));
        Ok(Polygon {
            fill: fill,
            coordinates: buf,
        })
    }
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

impl ToComputedValue for Polygon {
    type ComputedValue = computed_basic_shape::Polygon;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        computed_basic_shape::Polygon {
            fill: self.fill.to_computed_value(cx),
            coordinates: self.coordinates.iter()
                                         .map(|c| {
                                            (c.0.to_computed_value(cx),
                                             c.1.to_computed_value(cx))
                                         })
                                         .collect(),
        }
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

impl ShapeRadius {
    pub fn parse(input: &mut Parser) -> Result<ShapeRadius, ()> {
        input.try(LengthOrPercentage::parse).map(ShapeRadius::Length)
                                            .or_else(|_| {
            match_ignore_ascii_case! { try!(input.expect_ident()),
                "closest-side" => Ok(ShapeRadius::ClosestSide),
                "farthest-side" => Ok(ShapeRadius::FarthestSide),
                _ => Err(())
            }
        })
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


impl ToComputedValue for ShapeRadius {
    type ComputedValue = computed_basic_shape::ShapeRadius;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        match *self {
            ShapeRadius::Length(lop) => {
                computed_basic_shape::ShapeRadius::Length(lop.to_computed_value(cx))
            }
            ShapeRadius::ClosestSide => computed_basic_shape::ShapeRadius::ClosestSide,
            ShapeRadius::FarthestSide => computed_basic_shape::ShapeRadius::FarthestSide,
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

impl BorderRadius {
    pub fn parse(input: &mut Parser) -> Result<BorderRadius, ()> {
        let widths = try!(parse_one_set_of_border_values(input));
        let heights = if input.try(|input| input.expect_delim('/')).is_ok() {
            try!(parse_one_set_of_border_values(input))
        } else {
            widths.clone()
        };
        Ok(BorderRadius {
            top_left: BorderRadiusSize::new(widths[0], heights[0]),
            top_right: BorderRadiusSize::new(widths[1], heights[1]),
            bottom_right: BorderRadiusSize::new(widths[2], heights[2]),
            bottom_left: BorderRadiusSize::new(widths[3], heights[3]),
        })
    }
}

fn parse_one_set_of_border_values(mut input: &mut Parser)
                                 -> Result<[LengthOrPercentage; 4], ()> {
    let a = try!(LengthOrPercentage::parse(input));

    let b = if let Ok(b) = input.try(LengthOrPercentage::parse) {
        b
    } else {
        return Ok([a, a, a, a])
    };

    let c = if let Ok(c) = input.try(LengthOrPercentage::parse) {
        c
    } else {
        return Ok([a, b, a, b])
    };

    if let Ok(d) = input.try(LengthOrPercentage::parse) {
        Ok([a, b, c, d])
    } else {
        Ok([a, b, c, b])
    }
}


impl ToComputedValue for BorderRadius {
    type ComputedValue = computed_basic_shape::BorderRadius;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        computed_basic_shape::BorderRadius {
            top_left: self.top_left.to_computed_value(cx),
            top_right: self.top_right.to_computed_value(cx),
            bottom_right: self.bottom_right.to_computed_value(cx),
            bottom_left: self.bottom_left.to_computed_value(cx),
        }
    }
}

/// https://drafts.csswg.org/css-shapes/#typedef-fill-rule
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum FillRule {
    NonZero,
    EvenOdd,
    // basic-shapes spec says that these are the only two values, however
    // https://www.w3.org/TR/SVG/painting.html#FillRuleProperty
    // says that it can also be `inherit`
}

impl ComputedValueAsSpecified for FillRule {}

impl FillRule {
    pub fn parse(input: &mut Parser) -> Result<FillRule, ()> {
        match_ignore_ascii_case! { try!(input.expect_ident()),
            "nonzero" => Ok(FillRule::NonZero),
            "evenodd" => Ok(FillRule::EvenOdd),
            _ => Err(())
        }
    }
}

impl Default for FillRule {
    fn default() -> Self {
        FillRule::NonZero
    }
}

impl ToCss for FillRule {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            FillRule::NonZero => dest.write_str("nonzero"),
            FillRule::EvenOdd => dest.write_str("evenodd"),
        }
    }
}
