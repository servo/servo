/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`basic-shape`][basic-shape]s
//!
//! [basic-shape]: https://drafts.csswg.org/css-shapes/#typedef-basic-shape

use std::fmt;
use app_units::Au;
use euclid::size::Size2D;
use cssparser::{self, Parser, ToCss, Token};
use parser::{ParserContext, ParserContextExtraData};
use url::Url;
use properties::shorthands::parse_four_sides;
use values::specified::{Length, LengthOrPercentage};
use values::specified::BorderRadiusSize;
use values::computed::{Context, ToComputedValue};
use values::computed::basic_shape as computed_basic_shape;

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum BasicShape {
    Inset(InsetRect),
    // Circle(Circle),
    // Ellipse(Ellipse),
    // Polygon(Polygon),
}

impl BasicShape {
    pub fn parse(input: &mut Parser) -> Result<BasicShape, ()> {
        if let Ok(result) = input.try(InsetRect::parse) {
            Ok(BasicShape::Inset(result))
        } else {
            Err(())
        }
    }
}

impl ToCss for BasicShape {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            BasicShape::Inset(rect) => rect.to_css(dest),
        }
    }
}

impl ToComputedValue for BasicShape {
    type ComputedValue = computed_basic_shape::BasicShape;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        match *self {
            BasicShape::Inset(rect) => computed_basic_shape::BasicShape::Inset(rect.to_computed_value(cx)),
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
        if let Ok(_) = input.expect_ident_matching("round") {
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
