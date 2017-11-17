/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`basic-shape`][basic-shape]s
//!
//! [basic-shape]: https://drafts.csswg.org/css-shapes/#typedef-basic-shape

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::borrow::Cow;
use std::fmt;
use style_traits::{ToCss, ParseError, StyleParseErrorKind};
use values::computed::Percentage;
use values::generics::basic_shape::{Circle as GenericCircle};
use values::generics::basic_shape::{ClippingShape as GenericClippingShape, Ellipse as GenericEllipse};
use values::generics::basic_shape::{FillRule, BasicShape as GenericBasicShape};
use values::generics::basic_shape::{FloatAreaShape as GenericFloatAreaShape, InsetRect as GenericInsetRect};
use values::generics::basic_shape::{GeometryBox, ShapeBox, ShapeSource};
use values::generics::basic_shape::{Polygon as GenericPolygon, ShapeRadius as GenericShapeRadius};
use values::generics::rect::Rect;
use values::specified::LengthOrPercentage;
use values::specified::border::BorderRadius;
use values::specified::image::Image;
use values::specified::position::{HorizontalPosition, Position, PositionComponent, Side, VerticalPosition};
use values::specified::url::SpecifiedUrl;

/// A specified clipping shape.
pub type ClippingShape = GenericClippingShape<BasicShape, SpecifiedUrl>;

/// A specified float area shape.
pub type FloatAreaShape = GenericFloatAreaShape<BasicShape, Image>;

/// A specified basic shape.
pub type BasicShape = GenericBasicShape<HorizontalPosition, VerticalPosition, LengthOrPercentage>;

/// The specified value of `inset()`
pub type InsetRect = GenericInsetRect<LengthOrPercentage>;

/// A specified circle.
pub type Circle = GenericCircle<HorizontalPosition, VerticalPosition, LengthOrPercentage>;

/// A specified ellipse.
pub type Ellipse = GenericEllipse<HorizontalPosition, VerticalPosition, LengthOrPercentage>;

/// The specified value of `ShapeRadius`
pub type ShapeRadius = GenericShapeRadius<LengthOrPercentage>;

/// The specified value of `Polygon`
pub type Polygon = GenericPolygon<LengthOrPercentage>;

impl<ReferenceBox, ImageOrUrl> Parse for ShapeSource<BasicShape, ReferenceBox, ImageOrUrl>
where
    ReferenceBox: Parse,
    ImageOrUrl: Parse,
{
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(ShapeSource::None)
        }

        if let Ok(image_or_url) = input.try(|i| ImageOrUrl::parse(context, i)) {
            return Ok(ShapeSource::ImageOrUrl(image_or_url))
        }

        fn parse_component<U: Parse>(context: &ParserContext, input: &mut Parser,
                                     component: &mut Option<U>) -> bool {
            if component.is_some() {
                return false            // already parsed this component
            }

            *component = input.try(|i| U::parse(context, i)).ok();
            component.is_some()
        }

        let mut shape = None;
        let mut ref_box = None;

        while parse_component(context, input, &mut shape) ||
              parse_component(context, input, &mut ref_box) {
            //
        }

        if let Some(shp) = shape {
            return Ok(ShapeSource::Shape(shp, ref_box))
        }

        ref_box.map(|v| ShapeSource::Box(v)).ok_or(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

impl Parse for GeometryBox {
    fn parse<'i, 't>(_context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(shape_box) = input.try(|i| ShapeBox::parse(i)) {
            return Ok(GeometryBox::ShapeBox(shape_box))
        }

        try_match_ident_ignore_ascii_case! { input,
            "fill-box" => Ok(GeometryBox::FillBox),
            "stroke-box" => Ok(GeometryBox::StrokeBox),
            "view-box" => Ok(GeometryBox::ViewBox),
        }
    }
}

impl Parse for BasicShape {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let function = input.expect_function()?.clone();
        input.parse_nested_block(move |i| {
            (match_ignore_ascii_case! { &function,
                "inset" => return InsetRect::parse_function_arguments(context, i).map(GenericBasicShape::Inset),
                "circle" => return Circle::parse_function_arguments(context, i).map(GenericBasicShape::Circle),
                "ellipse" => return Ellipse::parse_function_arguments(context, i).map(GenericBasicShape::Ellipse),
                "polygon" => return Polygon::parse_function_arguments(context, i).map(GenericBasicShape::Polygon),
                _ => Err(())
            }).map_err(|()| location.new_custom_error(StyleParseErrorKind::UnexpectedFunction(function.clone())))
        })
    }
}

impl Parse for InsetRect {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        input.expect_function_matching("inset")?;
        input.parse_nested_block(|i| Self::parse_function_arguments(context, i))
    }
}

impl InsetRect {
    /// Parse the inner function arguments of `inset()`
    pub fn parse_function_arguments<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                            -> Result<Self, ParseError<'i>> {
        let rect = Rect::parse_with(context, input, LengthOrPercentage::parse)?;
        let round = if input.try(|i| i.expect_ident_matching("round")).is_ok() {
            Some(BorderRadius::parse(context, input)?)
        } else {
            None
        };
        Ok(GenericInsetRect {
            rect: rect,
            round: round,
        })
    }
}

impl Parse for Circle {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<Self, ParseError<'i>> {
        input.expect_function_matching("circle")?;
        input.parse_nested_block(|i| Self::parse_function_arguments(context, i))
    }
}

impl Circle {
    #[allow(missing_docs)]
    pub fn parse_function_arguments<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                            -> Result<Self, ParseError<'i>> {
        let radius = input.try(|i| ShapeRadius::parse(context, i)).unwrap_or_default();
        let position = if input.try(|i| i.expect_ident_matching("at")).is_ok() {
            Position::parse(context, input)?
        } else {
            Position::center()
        };

        Ok(GenericCircle {
            radius: radius,
            position: position,
        })
    }
}

impl ToCss for Circle {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str("circle(")?;
        if GenericShapeRadius::ClosestSide != self.radius {
            self.radius.to_css(dest)?;
            dest.write_str(" ")?;
        }

        dest.write_str("at ")?;
        serialize_basicshape_position(&self.position, dest)?;
        dest.write_str(")")
    }
}

impl Parse for Ellipse {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        input.expect_function_matching("ellipse")?;
        input.parse_nested_block(|i| Self::parse_function_arguments(context, i))
    }
}

impl Ellipse {
    #[allow(missing_docs)]
    pub fn parse_function_arguments<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                            -> Result<Self, ParseError<'i>> {
        let (a, b) = input.try(|i| -> Result<_, ParseError> {
            Ok((ShapeRadius::parse(context, i)?, ShapeRadius::parse(context, i)?))
        }).unwrap_or_default();
        let position = if input.try(|i| i.expect_ident_matching("at")).is_ok() {
            Position::parse(context, input)?
        } else {
            Position::center()
        };

        Ok(GenericEllipse {
            semiaxis_x: a,
            semiaxis_y: b,
            position: position,
        })
    }
}

impl ToCss for Ellipse {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str("ellipse(")?;
        if self.semiaxis_x != ShapeRadius::default() || self.semiaxis_y != ShapeRadius::default() {
            self.semiaxis_x.to_css(dest)?;
            dest.write_str(" ")?;
            self.semiaxis_y.to_css(dest)?;
            dest.write_str(" ")?;
        }

        dest.write_str("at ")?;
        serialize_basicshape_position(&self.position, dest)?;
        dest.write_str(")")
    }
}

impl Parse for ShapeRadius {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<Self, ParseError<'i>> {
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
            return Ok(GenericShapeRadius::Length(lop))
        }

        try_match_ident_ignore_ascii_case! { input,
            "closest-side" => Ok(GenericShapeRadius::ClosestSide),
            "farthest-side" => Ok(GenericShapeRadius::FarthestSide),
        }
    }
}

/// <https://drafts.csswg.org/css-shapes/#basic-shape-serialization>
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

impl Parse for Polygon {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        input.expect_function_matching("polygon")?;
        input.parse_nested_block(|i| Self::parse_function_arguments(context, i))
    }
}

impl Polygon {
    /// Parse the inner arguments of a `polygon` function.
    pub fn parse_function_arguments<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                                            -> Result<Self, ParseError<'i>> {
        let fill = input.try(|i| -> Result<_, ParseError> {
            let fill = FillRule::parse(i)?;
            i.expect_comma()?;      // only eat the comma if there is something before it
            Ok(fill)
        }).unwrap_or_default();

        let buf = input.parse_comma_separated(|i| {
            Ok((LengthOrPercentage::parse(context, i)?, LengthOrPercentage::parse(context, i)?))
        })?;

        Ok(Polygon {
            fill: fill,
            coordinates: buf,
        })
    }
}
