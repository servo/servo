/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`basic-shape`][basic-shape]s
//!
//! [basic-shape]: https://drafts.csswg.org/css-shapes/#typedef-basic-shape

use crate::parser::{Parse, ParserContext};
use crate::values::generics::basic_shape as generic;
use crate::values::generics::basic_shape::{Path, PolygonCoord};
use crate::values::generics::rect::Rect;
use crate::values::specified::border::BorderRadius;
use crate::values::specified::image::Image;
use crate::values::specified::position::{HorizontalPosition, Position, VerticalPosition};
use crate::values::specified::url::SpecifiedUrl;
use crate::values::specified::SVGPathData;
use crate::values::specified::{LengthPercentage, NonNegativeLengthPercentage};
use crate::Zero;
use cssparser::Parser;
use style_traits::{ParseError, StyleParseErrorKind};

/// A specified alias for FillRule.
pub use crate::values::generics::basic_shape::FillRule;

/// A specified `clip-path` value.
pub type ClipPath = generic::GenericClipPath<BasicShape, SpecifiedUrl>;

/// A specified `shape-outside` value.
pub type ShapeOutside = generic::GenericShapeOutside<BasicShape, Image>;

/// A specified basic shape.
pub type BasicShape = generic::GenericBasicShape<
    HorizontalPosition,
    VerticalPosition,
    LengthPercentage,
    NonNegativeLengthPercentage,
>;

/// The specified value of `inset()`
pub type InsetRect = generic::InsetRect<LengthPercentage, NonNegativeLengthPercentage>;

/// A specified circle.
pub type Circle =
    generic::Circle<HorizontalPosition, VerticalPosition, NonNegativeLengthPercentage>;

/// A specified ellipse.
pub type Ellipse =
    generic::Ellipse<HorizontalPosition, VerticalPosition, NonNegativeLengthPercentage>;

/// The specified value of `ShapeRadius`
pub type ShapeRadius = generic::ShapeRadius<NonNegativeLengthPercentage>;

/// The specified value of `Polygon`
pub type Polygon = generic::GenericPolygon<LengthPercentage>;

#[cfg(feature = "gecko")]
fn is_clip_path_path_enabled(context: &ParserContext) -> bool {
    context.chrome_rules_enabled() || static_prefs::pref!("layout.css.clip-path-path.enabled")
}
#[cfg(feature = "servo")]
fn is_clip_path_path_enabled(_: &ParserContext) -> bool {
    false
}

/// A helper for both clip-path and shape-outside parsing of shapes.
fn parse_shape_or_box<'i, 't, R, ReferenceBox>(
    context: &ParserContext,
    input: &mut Parser<'i, 't>,
    to_shape: impl FnOnce(Box<BasicShape>, ReferenceBox) -> R,
    to_reference_box: impl FnOnce(ReferenceBox) -> R,
) -> Result<R, ParseError<'i>>
where
    ReferenceBox: Default + Parse,
{
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
        return Ok(to_shape(Box::new(shp), ref_box.unwrap_or_default()));
    }

    match ref_box {
        Some(r) => Ok(to_reference_box(r)),
        None => Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
    }
}

impl Parse for ClipPath {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(ClipPath::None);
        }

        if is_clip_path_path_enabled(context) {
            if let Ok(p) = input.try(|i| Path::parse(context, i)) {
                return Ok(ClipPath::Path(p));
            }
        }

        if let Ok(url) = input.try(|i| SpecifiedUrl::parse(context, i)) {
            return Ok(ClipPath::Url(url));
        }

        parse_shape_or_box(context, input, ClipPath::Shape, ClipPath::Box)
    }
}

impl Parse for ShapeOutside {
    #[inline]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        // Need to parse this here so that `Image::parse_with_cors_anonymous`
        // doesn't parse it.
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(ShapeOutside::None);
        }

        if let Ok(image) = input.try(|i| Image::parse_with_cors_anonymous(context, i)) {
            debug_assert_ne!(image, Image::None);
            return Ok(ShapeOutside::Image(image));
        }

        parse_shape_or_box(context, input, ShapeOutside::Shape, ShapeOutside::Box)
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
        let rect = Rect::parse_with(context, input, LengthPercentage::parse)?;
        let round = if input.try(|i| i.expect_ident_matching("round")).is_ok() {
            BorderRadius::parse(context, input)?
        } else {
            BorderRadius::zero()
        };
        Ok(generic::InsetRect { rect, round })
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
            })
            .unwrap_or_default();
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

impl Parse for ShapeRadius {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(lp) = input.try(|i| NonNegativeLengthPercentage::parse(context, i)) {
            return Ok(generic::ShapeRadius::Length(lp));
        }

        try_match_ident_ignore_ascii_case! { input,
            "closest-side" => Ok(generic::ShapeRadius::ClosestSide),
            "farthest-side" => Ok(generic::ShapeRadius::FarthestSide),
        }
    }
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
            })
            .unwrap_or_default();

        let coordinates = input
            .parse_comma_separated(|i| {
                Ok(PolygonCoord(
                    LengthPercentage::parse(context, i)?,
                    LengthPercentage::parse(context, i)?,
                ))
            })?
            .into();

        Ok(Polygon { fill, coordinates })
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
            })
            .unwrap_or_default();
        let path = SVGPathData::parse(context, input)?;
        Ok(Path { fill, path })
    }
}
