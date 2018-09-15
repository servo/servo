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
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};
use values::computed::Percentage;
use values::generics::basic_shape as generic;
use values::generics::basic_shape::{FillRule, GeometryBox, Path, PolygonCoord};
use values::generics::basic_shape::{ShapeBox, ShapeSource};
use values::generics::rect::Rect;
use values::specified::LengthOrPercentage;
use values::specified::SVGPathData;
use values::specified::border::BorderRadius;
use values::specified::image::Image;
use values::specified::position::{HorizontalPosition, Position, PositionComponent};
use values::specified::position::{Side, VerticalPosition};
use values::specified::url::SpecifiedUrl;

/// A specified clipping shape.
pub type ClippingShape = generic::ClippingShape<BasicShape, SpecifiedUrl>;

/// A specified float area shape.
pub type FloatAreaShape = generic::FloatAreaShape<BasicShape, Image>;

/// A specified basic shape.
pub type BasicShape = generic::BasicShape<HorizontalPosition, VerticalPosition, LengthOrPercentage>;

/// The specified value of `inset()`
pub type InsetRect = generic::InsetRect<LengthOrPercentage>;

/// A specified circle.
pub type Circle = generic::Circle<HorizontalPosition, VerticalPosition, LengthOrPercentage>;

/// A specified ellipse.
pub type Ellipse = generic::Ellipse<HorizontalPosition, VerticalPosition, LengthOrPercentage>;

/// The specified value of `ShapeRadius`
pub type ShapeRadius = generic::ShapeRadius<LengthOrPercentage>;

/// The specified value of `Polygon`
pub type Polygon = generic::Polygon<LengthOrPercentage>;

#[cfg(feature = "gecko")]
fn is_clip_path_path_enabled(context: &ParserContext) -> bool {
    use gecko_bindings::structs::mozilla;
    context.chrome_rules_enabled() ||
        unsafe { mozilla::StaticPrefs_sVarCache_layout_css_clip_path_path_enabled }
}
#[cfg(feature = "servo")]
fn is_clip_path_path_enabled(_: &ParserContext) -> bool {
    false
}

impl Parse for ClippingShape {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if is_clip_path_path_enabled(context) {
            if let Ok(p) = input.try(|i| Path::parse(context, i)) {
                return Ok(ShapeSource::Path(p));
            }
        }

        if let Ok(url) = input.try(|i| SpecifiedUrl::parse(context, i)) {
            return Ok(ShapeSource::ImageOrUrl(url));
        }

        Self::parse_common(context, input)
    }
}

impl Parse for FloatAreaShape {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(image) = input.try(|i| Image::parse_with_cors_anonymous(context, i)) {
            return Ok(ShapeSource::ImageOrUrl(image));
        }

        Self::parse_common(context, input)
    }
}

impl<ReferenceBox, ImageOrUrl> ShapeSource<BasicShape, ReferenceBox, ImageOrUrl>
where
    ReferenceBox: Parse,
{
    /// The internal parser for ShapeSource.
    fn parse_common<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(ShapeSource::None);
        }

        fn parse_component<U: Parse>(
            context: &ParserContext,
            input: &mut Parser,
            component: &mut Option<U>,
        ) -> bool {
            if component.is_some() {
                return false; // already parsed this component
            }

            *component = input.try(|i| U::parse(context, i)).ok();
            component.is_some()
        }

        let mut shape = None;
        let mut ref_box = None;

        while parse_component(context, input, &mut shape) ||
            parse_component(context, input, &mut ref_box)
        {
            //
        }

        if let Some(shp) = shape {
            return Ok(ShapeSource::Shape(shp, ref_box));
        }

        ref_box
            .map(|v| ShapeSource::Box(v))
            .ok_or(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

impl Parse for GeometryBox {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(shape_box) = input.try(|i| ShapeBox::parse(i)) {
            return Ok(GeometryBox::ShapeBox(shape_box));
        }

        try_match_ident_ignore_ascii_case! { input,
            "fill-box" => Ok(GeometryBox::FillBox),
            "stroke-box" => Ok(GeometryBox::StrokeBox),
            "view-box" => Ok(GeometryBox::ViewBox),
        }
    }
}

impl Parse for BasicShape {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let function = input.expect_function()?.clone();
        input.parse_nested_block(move |i| {
            (match_ignore_ascii_case! { &function,
                "inset" => return InsetRect::parse_function_arguments(context, i).map(generic::BasicShape::Inset),
                "circle" => return Circle::parse_function_arguments(context, i).map(generic::BasicShape::Circle),
                "ellipse" => return Ellipse::parse_function_arguments(context, i).map(generic::BasicShape::Ellipse),
                "polygon" => return Polygon::parse_function_arguments(context, i).map(generic::BasicShape::Polygon),
                _ => Err(())
            }).map_err(|()| {
                location.new_custom_error(StyleParseErrorKind::UnexpectedFunction(function.clone()))
            })
        })
    }
}

impl Parse for InsetRect {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_function_matching("inset")?;
        input.parse_nested_block(|i| Self::parse_function_arguments(context, i))
    }
}

impl InsetRect {
    /// Parse the inner function arguments of `inset()`
    fn parse_function_arguments<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let rect = Rect::parse_with(context, input, LengthOrPercentage::parse)?;
        let round = if input.try(|i| i.expect_ident_matching("round")).is_ok() {
            Some(BorderRadius::parse(context, input)?)
        } else {
            None
        };
        Ok(generic::InsetRect {
            rect: rect,
            round: round,
        })
    }
}

impl Parse for Circle {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_function_matching("circle")?;
        input.parse_nested_block(|i| Self::parse_function_arguments(context, i))
    }
}

impl Circle {
    fn parse_function_arguments<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let radius = input
            .try(|i| ShapeRadius::parse(context, i))
            .unwrap_or_default();
        let position = if input.try(|i| i.expect_ident_matching("at")).is_ok() {
            Position::parse(context, input)?
        } else {
            Position::center()
        };

        Ok(generic::Circle { radius, position })
    }
}

impl ToCss for Circle {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str("circle(")?;
        if generic::ShapeRadius::ClosestSide != self.radius {
            self.radius.to_css(dest)?;
            dest.write_str(" ")?;
        }

        dest.write_str("at ")?;
        serialize_basicshape_position(&self.position, dest)?;
        dest.write_str(")")
    }
}

impl Parse for Ellipse {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_function_matching("ellipse")?;
        input.parse_nested_block(|i| Self::parse_function_arguments(context, i))
    }
}

impl Ellipse {
    fn parse_function_arguments<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let (a, b) = input
            .try(|i| -> Result<_, ParseError> {
                Ok((
                    ShapeRadius::parse(context, i)?,
                    ShapeRadius::parse(context, i)?,
                ))
            }).unwrap_or_default();
        let position = if input.try(|i| i.expect_ident_matching("at")).is_ok() {
            Position::parse(context, input)?
        } else {
            Position::center()
        };

        Ok(generic::Ellipse {
            semiaxis_x: a,
            semiaxis_y: b,
            position: position,
        })
    }
}

impl ToCss for Ellipse {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
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
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(lop) = input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
            return Ok(generic::ShapeRadius::Length(lop));
        }

        try_match_ident_ignore_ascii_case! { input,
            "closest-side" => Ok(generic::ShapeRadius::ClosestSide),
            "farthest-side" => Ok(generic::ShapeRadius::FarthestSide),
        }
    }
}

/// <https://drafts.csswg.org/css-shapes/#basic-shape-serialization>
///
/// Positions get serialized differently with basic shapes. Keywords
/// are converted to percentages where possible. Only the two or four
/// value forms are used. In case of two keyword-percentage pairs,
/// the keywords are folded into the percentages
fn serialize_basicshape_position<W>(position: &Position, dest: &mut CssWriter<W>) -> fmt::Result
where
    W: Write,
{
    fn to_keyword_and_lop<S>(component: &PositionComponent<S>) -> (S, Cow<LengthOrPercentage>)
    where
        S: Copy + Side,
    {
        match *component {
            PositionComponent::Center => (
                S::start(),
                Cow::Owned(LengthOrPercentage::Percentage(Percentage(0.5))),
            ),
            PositionComponent::Side(keyword, None) => {
                // left | top => 0%
                // right | bottom => 100%
                let p = if keyword.is_start() { 0. } else { 1. };
                (
                    S::start(),
                    Cow::Owned(LengthOrPercentage::Percentage(Percentage(p))),
                )
            },
            PositionComponent::Side(keyword, Some(ref lop)) if !keyword.is_start() => {
                if let LengthOrPercentage::Percentage(p) = *to_non_zero_length(lop) {
                    (
                        S::start(),
                        Cow::Owned(LengthOrPercentage::Percentage(Percentage(1. - p.0))),
                    )
                } else {
                    (keyword, Cow::Borrowed(lop))
                }
            },
            PositionComponent::Length(ref lop) | PositionComponent::Side(_, Some(ref lop)) => {
                (S::start(), to_non_zero_length(lop))
            },
        }
    }

    fn to_non_zero_length(lop: &LengthOrPercentage) -> Cow<LengthOrPercentage> {
        match *lop {
            LengthOrPercentage::Length(ref l) if l.is_zero() => {
                Cow::Owned(LengthOrPercentage::Percentage(Percentage(0.)))
            },
            _ => Cow::Borrowed(lop),
        }
    }

    fn write_pair<A, B, W>(a: &A, b: &B, dest: &mut CssWriter<W>) -> fmt::Result
    where
        A: ToCss,
        B: ToCss,
        W: Write,
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
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_function_matching("polygon")?;
        input.parse_nested_block(|i| Self::parse_function_arguments(context, i))
    }
}

impl Polygon {
    /// Parse the inner arguments of a `polygon` function.
    fn parse_function_arguments<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let fill = input
            .try(|i| -> Result<_, ParseError> {
                let fill = FillRule::parse(i)?;
                i.expect_comma()?; // only eat the comma if there is something before it
                Ok(fill)
            }).unwrap_or_default();

        let buf = input.parse_comma_separated(|i| {
            Ok(PolygonCoord(
                LengthOrPercentage::parse(context, i)?,
                LengthOrPercentage::parse(context, i)?,
            ))
        })?;

        Ok(Polygon {
            fill: fill,
            coordinates: buf,
        })
    }
}

impl Parse for Path {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_function_matching("path")?;
        input.parse_nested_block(|i| Self::parse_function_arguments(context, i))
    }
}

impl Path {
    /// Parse the inner arguments of a `path` function.
    fn parse_function_arguments<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let fill = input
            .try(|i| -> Result<_, ParseError> {
                let fill = FillRule::parse(i)?;
                i.expect_comma()?;
                Ok(fill)
            }).unwrap_or_default();
        let path = SVGPathData::parse(context, input)?;
        Ok(Path { fill, path })
    }
}
