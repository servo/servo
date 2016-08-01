/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`basic-shape`][basic-shape]s
//!
//! [basic-shape]: https://drafts.csswg.org/css-shapes/#typedef-basic-shape

use app_units::Au;
use std::fmt;
use cssparser::{Parser, ToCss};
use properties::shorthands::parse_four_sides;
use values::specified::{BorderRadiusSize, Length, LengthOrPercentage};
use values::specified::position::{Position, PositionComponent};
use values::computed::{Context, ToComputedValue, ComputedValueAsSpecified};
use values::computed::basic_shape as computed_basic_shape;

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
        if let Ok(result) = input.try(InsetRect::parse) {
            Ok(BasicShape::Inset(result))
        } else if let Ok(result) = input.try(Circle::parse) {
            Ok(BasicShape::Circle(result))
        } else if let Ok(result) = input.try(Ellipse::parse) {
            Ok(BasicShape::Ellipse(result))
        } else if let Ok(result) = input.try(Polygon::parse) {
            Ok(BasicShape::Polygon(result))
        } else {
            Err(())
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
                Ok(try!(input.parse_nested_block(InsetRect::parse_function)))
            },
            _ => Err(())
        }
    }
    pub fn parse_function(input: &mut Parser) -> Result<InsetRect, ()> {
        let (t,r,b,l) = try!(parse_four_sides(input, LengthOrPercentage::parse));
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
                Ok(try!(input.parse_nested_block(Circle::parse_function)))
            },
            _ => Err(())
        }
    }
    pub fn parse_function(input: &mut Parser) -> Result<Circle, ()> {
        let radius = input.try(ShapeRadius::parse).ok().unwrap_or_else(Default::default);
        let position = if let Ok(_) = input.try(|input| input.expect_ident_matching("at")) {
            try!(Position::parse(input))
        } else {
            // Defaults to origin
            try!(Position::new(PositionComponent::Center, PositionComponent::Center))
        };
        Ok(Circle {
            radius: radius,
            position: position,
        })
    }
}

impl ToCss for Circle {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if ShapeRadius::ClosestSide != self.radius {
            try!(self.radius.to_css(dest));
            try!(dest.write_str(" "));
        }
        try!(dest.write_str("at "));
        self.position.to_css(dest)
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
    pub semiaxis_a: ShapeRadius,
    pub semiaxis_b: ShapeRadius,
    pub position: Position,
}


impl Ellipse {
    pub fn parse(input: &mut Parser) -> Result<Ellipse, ()> {
        match_ignore_ascii_case! { try!(input.expect_function()),
            "ellipse" => {
                Ok(try!(input.parse_nested_block(Ellipse::parse_function)))
            },
            _ => Err(())
        }
    }
    pub fn parse_function(input: &mut Parser) -> Result<Ellipse, ()> {
        let (a, b) = input.try(|input| -> Result<_, ()> {
            Ok((try!(ShapeRadius::parse(input)), try!(ShapeRadius::parse(input))))
        }).unwrap_or((Default::default(), Default::default()));
        let position = if let Ok(_) = input.try(|input| input.expect_ident_matching("at")) {
            try!(Position::parse(input))
        } else {
            // Defaults to origin
            try!(Position::new(PositionComponent::Center, PositionComponent::Center))
        };
        Ok(Ellipse {
            semiaxis_a: a,
            semiaxis_b: b,
            position: position,
        })
    }
}

impl ToCss for Ellipse {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if ShapeRadius::ClosestSide != self.semiaxis_a
            && ShapeRadius::ClosestSide != self.semiaxis_b {
            try!(self.semiaxis_a.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.semiaxis_b.to_css(dest));
            try!(dest.write_str(" "));
        }
        try!(dest.write_str("at "));
        self.position.to_css(dest)
    }
}

impl ToComputedValue for Ellipse {
    type ComputedValue = computed_basic_shape::Ellipse;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        computed_basic_shape::Ellipse {
            semiaxis_a: self.semiaxis_a.to_computed_value(cx),
            semiaxis_b: self.semiaxis_b.to_computed_value(cx),
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
                Ok(try!(input.parse_nested_block(Polygon::parse_function)))
            },
            _ => Err(())
        }
    }
    pub fn parse_function(input: &mut Parser) -> Result<Polygon, ()> {
        let fill = input.try(|input| {
            let fill = FillRule::parse(input);
            // only eat the comma if there is something before it
            try!(input.expect_comma());
            fill
        }).ok().unwrap_or_else(Default::default);
        let first = (try!(LengthOrPercentage::parse(input)),
                     try!(LengthOrPercentage::parse(input)));
        let mut buf = vec![first];
        while !input.is_exhausted() {
            buf.push((try!(LengthOrPercentage::parse(input)),
                      try!(LengthOrPercentage::parse(input))));
        }
        Ok(Polygon {
            fill: fill,
            coordinates: buf,
        })
    }
}

impl ToCss for Polygon {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut need_space = false;
        if self.fill != Default::default() {
            try!(self.fill.to_css(dest));
            try!(dest.write_str(", "));
        }
        for coord in &self.coordinates {
            if need_space {
                try!(dest.write_str(" "));
            }
            try!(coord.0.to_css(dest));
            try!(dest.write_str(" "));
            try!(coord.1.to_css(dest));
            need_space = true;
        }
        Ok(())
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
    pub bottom_left: BorderRadiusSize,
    pub bottom_right: BorderRadiusSize,
}

impl ToCss for BorderRadius {
    // XXXManishearth: We should be producing minimal output:
    // if height=width for all, we should not be printing the part after
    // the slash. For any set of four values,
    // we should try to reduce them to one or two. This probably should be
    // a helper function somewhere, for all the parse_four_sides-like
    // values
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(self.top_left.0.width.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.top_right.0.width.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.bottom_left.0.width.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.bottom_right.0.width.to_css(dest));
        try!(dest.write_str(" / "));
        try!(self.top_left.0.height.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.top_right.0.height.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.bottom_left.0.height.to_css(dest));
        try!(dest.write_str(" "));
        try!(self.bottom_right.0.height.to_css(dest));
        dest.write_str(" ")
    }
}

impl BorderRadius {
    pub fn parse(input: &mut Parser) -> Result<BorderRadius, ()> {
        let widths = try!(parse_one_set_of_border_values(input));
        let mut heights = widths.clone();
        if input.try(|input| input.expect_delim('/')).is_ok() {
            heights = try!(parse_one_set_of_border_values(input));
        }
        Ok(BorderRadius {
            top_left: BorderRadiusSize::new(widths[1], heights[1]),
            top_right: BorderRadiusSize::new(widths[2], heights[2]),
            bottom_left: BorderRadiusSize::new(widths[3], heights[3]),
            bottom_right: BorderRadiusSize::new(widths[4], heights[4]),
        })
    }
}

fn parse_one_set_of_border_values(mut input: &mut Parser)
                                 -> Result<[LengthOrPercentage; 4], ()> {
    let mut count = 0;
    let mut values = [LengthOrPercentage::Length(Length::Absolute(Au(0))); 4];
    while count < 4 {
        if let Ok(value) = input.try(LengthOrPercentage::parse) {
            values[count] = value;
            count += 1;
        } else {
            break
        }
    }

    match count {
        1 => Ok([values[0], values[0], values[0], values[0]]),
        2 => Ok([values[0], values[1], values[0], values[1]]),
        3 => Ok([values[0], values[1], values[2], values[1]]),
        4 => Ok([values[0], values[1], values[2], values[3]]),
        _ => Err(()),
    }
}


impl ToComputedValue for BorderRadius {
    type ComputedValue = computed_basic_shape::BorderRadius;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        computed_basic_shape::BorderRadius {
            top_left: self.top_left.to_computed_value(cx),
            top_right: self.top_right.to_computed_value(cx),
            bottom_left: self.bottom_left.to_computed_value(cx),
            bottom_right: self.bottom_right.to_computed_value(cx),
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
