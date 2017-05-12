/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`basic-shape`][basic-shape]s
//!
//! [basic-shape]: https://drafts.csswg.org/css-shapes/#typedef-basic-shape

use cssparser::Parser;
use parser::{Parse, ParserContext};
use properties::shorthands::parse_four_sides;
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::fmt;
use style_traits::ToCss;
use values::computed::{ComputedValueAsSpecified, Context, ToComputedValue};
use values::computed::basic_shape as computed_basic_shape;
use values::generics::BorderRadiusSize;
use values::generics::basic_shape::{BorderRadius as GenericBorderRadius, ShapeRadius as GenericShapeRadius};
use values::generics::basic_shape::{InsetRect as GenericInsetRect, Polygon as GenericPolygon, ShapeSource};
use values::specified::{LengthOrPercentage, Percentage, Position, PositionComponent};
use values::specified::position::Side;

/// The specified value used by `clip-path`
pub type ShapeWithGeometryBox = ShapeSource<BasicShape, GeometryBox>;

/// The specified value used by `shape-outside`
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

impl Parse for BasicShape {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<BasicShape, ()> {
        match_ignore_ascii_case! { &input.try(|i| i.expect_function())?,
            "inset" =>
                input.parse_nested_block(|i| InsetRect::parse_function_arguments(context, i))
                     .map(BasicShape::Inset),
            "circle" =>
                input.parse_nested_block(|i| Circle::parse_function_arguments(context, i))
                     .map(BasicShape::Circle),
            "ellipse" =>
                input.parse_nested_block(|i| Ellipse::parse_function_arguments(context, i))
                     .map(BasicShape::Ellipse),
            "polygon" =>
                input.parse_nested_block(|i| Polygon::parse_function_arguments(context, i))
                     .map(BasicShape::Polygon),
            _ => Err(())
        }
    }
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

impl ToComputedValue for BasicShape {
    type ComputedValue = computed_basic_shape::BasicShape;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        match *self {
            BasicShape::Inset(ref rect) => computed_basic_shape::BasicShape::Inset(rect.to_computed_value(cx)),
            BasicShape::Circle(ref circle) => computed_basic_shape::BasicShape::Circle(circle.to_computed_value(cx)),
            BasicShape::Ellipse(ref e) => computed_basic_shape::BasicShape::Ellipse(e.to_computed_value(cx)),
            BasicShape::Polygon(ref poly) => computed_basic_shape::BasicShape::Polygon(poly.to_computed_value(cx)),
        }
    }
    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            computed_basic_shape::BasicShape::Inset(ref rect) =>
                BasicShape::Inset(ToComputedValue::from_computed_value(rect)),
            computed_basic_shape::BasicShape::Circle(ref circle) =>
                BasicShape::Circle(ToComputedValue::from_computed_value(circle)),
            computed_basic_shape::BasicShape::Ellipse(ref e) =>
                BasicShape::Ellipse(ToComputedValue::from_computed_value(e)),
            computed_basic_shape::BasicShape::Polygon(ref poly) =>
                BasicShape::Polygon(ToComputedValue::from_computed_value(poly)),
        }
    }
}

/// The specified value of `inset()`
pub type InsetRect = GenericInsetRect<LengthOrPercentage>;

impl InsetRect {
    /// Parse the inner function arguments of `inset()`
    pub fn parse_function_arguments(context: &ParserContext, input: &mut Parser) -> Result<InsetRect, ()> {
        let (t, r, b, l) = parse_four_sides(input, |i| LengthOrPercentage::parse(context, i))?;
        let mut rect = GenericInsetRect {
            top: t,
            right: r,
            bottom: b,
            left: l,
            round: None,
        };

        if input.try(|i| i.expect_ident_matching("round")).is_ok() {
            rect.round = Some(BorderRadius::parse(context, input)?);
        }

        Ok(rect)
    }
}

impl Parse for InsetRect {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match input.try(|i| i.expect_function()) {
            Ok(ref s) if s.eq_ignore_ascii_case("inset") =>
                input.parse_nested_block(|i| GenericInsetRect::parse_function_arguments(context, i)),
            _ => Err(())
        }
    }
}

/// https://drafts.csswg.org/css-shapes/#basic-shape-serialization
///
/// Positions get serialized differently with basic shapes. Keywords
/// are converted to percentages where possible. Only the two or four
/// value forms are used. In case of two keyword-percentage pairs,
/// the keywords are folded into the percentages
fn serialize_basicshape_position<W>(position: &Position, dest: &mut W) -> fmt::Result
    where W: fmt::Write
{
    fn to_keyword_and_lop<S>(component: &PositionComponent<S>) -> (S, Cow<LengthOrPercentage>)
        where S: Copy + Side
    {
        match *component {
            PositionComponent::Center => {
                (S::start(), Cow::Owned(LengthOrPercentage::Percentage(Percentage(0.5))))
            },
            PositionComponent::Side(keyword, None) => {
                // left | top => 0%
                // right | bottom => 100%
                let p = if keyword.is_start() { 0. } else { 1. };
                (S::start(), Cow::Owned(LengthOrPercentage::Percentage(Percentage(p))))
            },
            PositionComponent::Side(keyword, Some(ref lop)) if !keyword.is_start() => {
                if let LengthOrPercentage::Percentage(p) = *to_non_zero_length(lop) {
                    (S::start(), Cow::Owned(LengthOrPercentage::Percentage(Percentage(1. - p.0))))
                } else {
                    (keyword, Cow::Borrowed(lop))
                }
            },
            PositionComponent::Length(ref lop) |
            PositionComponent::Side(_, Some(ref lop)) => {
                (S::start(), to_non_zero_length(lop))
            },
        }
    }

    fn to_non_zero_length(lop: &LengthOrPercentage) -> Cow<LengthOrPercentage> {
        match *lop {
            LengthOrPercentage::Length(ref l) if l.is_zero() => {
                Cow::Owned(LengthOrPercentage::Percentage(Percentage(0.)))
            },
            _ => {
                Cow::Borrowed(lop)
            }
        }
    }

    fn write_pair<A, B, W>(a: &A, b: &B, dest: &mut W) -> fmt::Result
        where A: ToCss, B: ToCss, W: fmt::Write
    {
        a.to_css(dest)?;
        dest.write_str(" ")?;
        b.to_css(dest)
    }

    let (x_pos, x_lop) = to_keyword_and_lop(&position.horizontal);
    let (y_pos, y_lop) = to_keyword_and_lop(&position.vertical);

    if x_pos.is_start() && y_pos.is_start() {
        return write_pair(&*x_lop, &*y_lop, dest);
    }

    write_pair(&x_pos, &*x_lop, dest)?;
    dest.write_str(" ")?;
    write_pair(&y_pos, &*y_lop, dest)
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-shapes/#funcdef-circle
#[allow(missing_docs)]
pub struct Circle {
    pub radius: ShapeRadius,
    pub position: Position,
}

impl Circle {
    #[allow(missing_docs)]
    pub fn parse_function_arguments(context: &ParserContext, input: &mut Parser) -> Result<Circle, ()> {
        let radius = input.try(|i| ShapeRadius::parse(context, i)).ok().unwrap_or_default();
        let position = if input.try(|i| i.expect_ident_matching("at")).is_ok() {
            Position::parse(context, input)?
        } else {
            Position::center()      // Defaults to origin
        };

        Ok(Circle {
            radius: radius,
            position: position,
        })
    }
}

impl Parse for Circle {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match_ignore_ascii_case! { &try!(input.expect_function()),
           "circle" => {
               input.parse_nested_block(|i| Circle::parse_function_arguments(context, i))
           },
           _ => Err(())
        }
    }
}

impl ToCss for Circle {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("circle("));
        if GenericShapeRadius::ClosestSide != self.radius {
            try!(self.radius.to_css(dest));
            try!(dest.write_str(" "));
        }

        try!(dest.write_str("at "));
        try!(serialize_basicshape_position(&self.position, dest));
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

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Circle {
            radius: ToComputedValue::from_computed_value(&computed.radius),
            position: ToComputedValue::from_computed_value(&computed.position),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-shapes/#funcdef-ellipse
#[allow(missing_docs)]
pub struct Ellipse {
    pub semiaxis_x: ShapeRadius,
    pub semiaxis_y: ShapeRadius,
    pub position: Position,
}

impl Ellipse {
    #[allow(missing_docs)]
    pub fn parse_function_arguments(context: &ParserContext, input: &mut Parser) -> Result<Ellipse, ()> {
        let (a, b) = input.try(|i| -> Result<_, ()> {
            Ok((ShapeRadius::parse(context, i)?, ShapeRadius::parse(context, i)?))
        }).ok().unwrap_or_default();
        let position = if input.try(|i| i.expect_ident_matching("at")).is_ok() {
            Position::parse(context, input)?
        } else {
            Position::center()      // Defaults to origin
        };

        Ok(Ellipse {
            semiaxis_x: a,
            semiaxis_y: b,
            position: position,
        })
    }
}

impl Parse for Ellipse {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match input.try(|i| i.expect_function()) {
            Ok(ref s) if s.eq_ignore_ascii_case("ellipse") =>
                input.parse_nested_block(|i| Ellipse::parse_function_arguments(context, i)),
            _ => Err(())
        }
    }
}

impl ToCss for Ellipse {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("ellipse("));
        if self.semiaxis_x != ShapeRadius::default() || self.semiaxis_y != ShapeRadius::default() {
            try!(self.semiaxis_x.to_css(dest));
            try!(dest.write_str(" "));
            try!(self.semiaxis_y.to_css(dest));
            try!(dest.write_str(" "));
        }

        try!(dest.write_str("at "));
        try!(serialize_basicshape_position(&self.position, dest));
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

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Ellipse {
            semiaxis_x: ToComputedValue::from_computed_value(&computed.semiaxis_x),
            semiaxis_y: ToComputedValue::from_computed_value(&computed.semiaxis_y),
            position: ToComputedValue::from_computed_value(&computed.position),
        }
    }
}

/// The specified value of `Polygon`
pub type Polygon = GenericPolygon<LengthOrPercentage>;

/// The specified value of `ShapeRadius`
pub type ShapeRadius = GenericShapeRadius<LengthOrPercentage>;

impl Parse for ShapeRadius {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
            return Ok(GenericShapeRadius::Length(lop))
        }

        match_ignore_ascii_case! { &input.expect_ident()?,
            "closest-side" => Ok(GenericShapeRadius::ClosestSide),
            "farthest-side" => Ok(GenericShapeRadius::FarthestSide),
            _ => Err(())
        }
    }
}

/// The specified value of `BorderRadius`
pub type BorderRadius = GenericBorderRadius<LengthOrPercentage>;

impl Parse for BorderRadius {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let mut widths = parse_one_set_of_border_values(context, input)?;
        let mut heights = if input.try(|input| input.expect_delim('/')).is_ok() {
            parse_one_set_of_border_values(context, input)?
        } else {
            [widths[0].clone(),
             widths[1].clone(),
             widths[2].clone(),
             widths[3].clone()]
        };

        Ok(BorderRadius {
            top_left: BorderRadiusSize::new(widths[0].take(), heights[0].take()),
            top_right: BorderRadiusSize::new(widths[1].take(), heights[1].take()),
            bottom_right: BorderRadiusSize::new(widths[2].take(), heights[2].take()),
            bottom_left: BorderRadiusSize::new(widths[3].take(), heights[3].take()),
        })
    }
}

fn parse_one_set_of_border_values(context: &ParserContext, mut input: &mut Parser)
                                 -> Result<[LengthOrPercentage; 4], ()> {
    let a = try!(LengthOrPercentage::parse_non_negative(context, input));
    let b = if let Ok(b) = input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
        b
    } else {
        return Ok([a.clone(), a.clone(), a.clone(), a])
    };

    let c = if let Ok(c) = input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
        c
    } else {
        return Ok([a.clone(), b.clone(), a, b])
    };

    if let Ok(d) = input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
        Ok([a, b, c, d])
    } else {
        Ok([a, b.clone(), c, b])
    }
}

/// https://drafts.fxtf.org/css-masking-1/#typedef-geometry-box
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum GeometryBox {
    FillBox,
    StrokeBox,
    ViewBox,
    ShapeBox(ShapeBox),
}

impl Parse for GeometryBox {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(shape_box) = input.try(|i| ShapeBox::parse(i)) {
            return Ok(GeometryBox::ShapeBox(shape_box))
        }

        match_ignore_ascii_case! { &input.expect_ident()?,
            "fill-box" => Ok(GeometryBox::FillBox),
            "stroke-box" => Ok(GeometryBox::StrokeBox),
            "view-box" => Ok(GeometryBox::ViewBox),
            _ => Err(())
        }
    }
}

impl ToCss for GeometryBox {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            GeometryBox::FillBox => dest.write_str("fill-box"),
            GeometryBox::StrokeBox => dest.write_str("stroke-box"),
            GeometryBox::ViewBox => dest.write_str("view-box"),
            GeometryBox::ShapeBox(s) => s.to_css(dest),
        }
    }
}

impl ComputedValueAsSpecified for GeometryBox {}

// https://drafts.csswg.org/css-shapes-1/#typedef-shape-box
define_css_keyword_enum!(ShapeBox:
    "margin-box" => MarginBox,
    "border-box" => BorderBox,
    "padding-box" => PaddingBox,
    "content-box" => ContentBox
);

add_impls_for_keyword_enum!(ShapeBox);
