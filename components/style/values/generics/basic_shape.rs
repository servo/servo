/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the [`basic-shape`](https://drafts.csswg.org/css-shapes/#typedef-basic-shape)
//! types that are generic over their `ToCss` implementations.

use cssparser::Parser;
use euclid::size::Size2D;
use parser::{Parse, ParserContext};
use properties::shorthands::serialize_four_sides;
use std::ascii::AsciiExt;
use std::fmt;
use style_traits::ToCss;
use values::HasViewportPercentage;
use values::computed::{ComputedValueAsSpecified, Context, ToComputedValue};
use values::generics::BorderRadiusSize;
use values::specified::url::SpecifiedUrl;

/// A generic type used for `border-radius`, `outline-radius` and `inset()` values.
///
/// https://drafts.csswg.org/css-backgrounds-3/#border-radius
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct BorderRadius<L> {
    /// The top left radius.
    pub top_left: BorderRadiusSize<L>,
    /// The top right radius.
    pub top_right: BorderRadiusSize<L>,
    /// The bottom right radius.
    pub bottom_right: BorderRadiusSize<L>,
    /// The bottom left radius.
    pub bottom_left: BorderRadiusSize<L>,
}

/// Serialization helper for types of longhands like `border-radius` and `outline-radius`
pub fn serialize_radius_values<L, W>(dest: &mut W, top_left: &Size2D<L>,
                                     top_right: &Size2D<L>, bottom_right: &Size2D<L>,
                                     bottom_left: &Size2D<L>) -> fmt::Result
    where L: ToCss + PartialEq, W: fmt::Write
{
    if top_left.width == top_left.height && top_right.width == top_right.height &&
       bottom_right.width == bottom_right.height && bottom_left.width == bottom_left.height {
        serialize_four_sides(dest, &top_left.width, &top_right.width,
                             &bottom_right.width, &bottom_left.width)
    } else {
        serialize_four_sides(dest, &top_left.width, &top_right.width,
                             &bottom_right.width, &bottom_left.width)?;
        dest.write_str(" / ")?;
        serialize_four_sides(dest, &top_left.height, &top_right.height,
                             &bottom_right.height, &bottom_left.height)
    }
}

impl<L: ToCss + PartialEq> ToCss for BorderRadius<L> {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        serialize_radius_values(dest, &self.top_left.0, &self.top_right.0,
                                &self.bottom_right.0, &self.bottom_left.0)
    }
}

impl<L: ToComputedValue> ToComputedValue for BorderRadius<L> {
    type ComputedValue = BorderRadius<L::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        BorderRadius {
            top_left: self.top_left.to_computed_value(cx),
            top_right: self.top_right.to_computed_value(cx),
            bottom_right: self.bottom_right.to_computed_value(cx),
            bottom_left: self.bottom_left.to_computed_value(cx),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        BorderRadius {
            top_left: ToComputedValue::from_computed_value(&computed.top_left),
            top_right: ToComputedValue::from_computed_value(&computed.top_right),
            bottom_right: ToComputedValue::from_computed_value(&computed.bottom_right),
            bottom_left: ToComputedValue::from_computed_value(&computed.bottom_left),
        }
    }
}

/// https://drafts.csswg.org/css-shapes/#typedef-shape-radius
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum ShapeRadius<L> {
    Length(L),
    ClosestSide,
    FarthestSide,
}

impl<L> Default for ShapeRadius<L> {
    #[inline]
    fn default() -> Self { ShapeRadius::ClosestSide }
}

impl<L: ToCss> ToCss for ShapeRadius<L> {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            ShapeRadius::Length(ref lop) => lop.to_css(dest),
            ShapeRadius::ClosestSide => dest.write_str("closest-side"),
            ShapeRadius::FarthestSide => dest.write_str("farthest-side"),
        }
    }
}

impl<L: ToComputedValue> ToComputedValue for ShapeRadius<L> {
    type ComputedValue = ShapeRadius<L::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        match *self {
            ShapeRadius::Length(ref lop) => ShapeRadius::Length(lop.to_computed_value(cx)),
            ShapeRadius::ClosestSide => ShapeRadius::ClosestSide,
            ShapeRadius::FarthestSide => ShapeRadius::FarthestSide,
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            ShapeRadius::Length(ref lop) => ShapeRadius::Length(ToComputedValue::from_computed_value(lop)),
            ShapeRadius::ClosestSide => ShapeRadius::ClosestSide,
            ShapeRadius::FarthestSide => ShapeRadius::FarthestSide,
        }
    }
}

// https://drafts.csswg.org/css-shapes/#typedef-fill-rule
// NOTE: Basic shapes spec says that these are the only two values, however
// https://www.w3.org/TR/SVG/painting.html#FillRuleProperty
// says that it can also be `inherit`
define_css_keyword_enum!(FillRule:
    "nonzero" => NonZero,
    "evenodd" => EvenOdd
);

impl ComputedValueAsSpecified for FillRule {}

impl Default for FillRule {
    #[inline]
    fn default() -> Self { FillRule::NonZero }
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A generic type for representing the `polygon()` function
///
/// https://drafts.csswg.org/css-shapes/#funcdef-polygon
pub struct Polygon<L> {
    /// The filling rule for a polygon.
    pub fill: FillRule,
    /// A collection of (x, y) coordinates to draw the polygon.
    pub coordinates: Vec<(L, L)>,
}

impl<L: Parse> Polygon<L> {
    /// Parse the inner arguments of a `polygon` function.
    pub fn parse_function_arguments(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let fill = input.try(|i| -> Result<_, ()> {
            let fill = FillRule::parse(i)?;
            i.expect_comma()?;      // only eat the comma if there is something before it
            Ok(fill)
        }).ok().unwrap_or_default();

        let buf = input.parse_comma_separated(|i| {
            Ok((L::parse(context, i)?, L::parse(context, i)?))
        })?;

        Ok(Polygon {
            fill: fill,
            coordinates: buf,
        })
    }
}

impl<L: Parse> Parse for Polygon<L> {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match input.expect_function() {
            Ok(ref s) if s.eq_ignore_ascii_case("polygon") =>
                input.parse_nested_block(|i| Polygon::parse_function_arguments(context, i)),
            _ => Err(())
        }
    }
}

impl<L: ToCss> ToCss for Polygon<L> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str("polygon(")?;
        if self.fill != FillRule::default() {
            self.fill.to_css(dest)?;
            dest.write_str(", ")?;
        }

        for (i, coord) in self.coordinates.iter().enumerate() {
            if i > 0 {
                dest.write_str(", ")?;
            }

            coord.0.to_css(dest)?;
            dest.write_str(" ")?;
            coord.1.to_css(dest)?;
        }

        dest.write_str(")")
    }
}

impl<L: ToComputedValue> ToComputedValue for Polygon<L> {
    type ComputedValue = Polygon<L::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        Polygon {
            fill: self.fill.to_computed_value(cx),
            coordinates: self.coordinates.iter().map(|c| {
                (c.0.to_computed_value(cx), c.1.to_computed_value(cx))
            }).collect(),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Polygon {
            fill: ToComputedValue::from_computed_value(&computed.fill),
            coordinates: computed.coordinates.iter().map(|c| {
                (ToComputedValue::from_computed_value(&c.0),
                 ToComputedValue::from_computed_value(&c.1))
            }).collect(),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-shapes/#funcdef-inset
#[allow(missing_docs)]
pub struct InsetRect<L> {
    pub top: L,
    pub right: L,
    pub bottom: L,
    pub left: L,
    pub round: Option<BorderRadius<L>>,
}

impl<L: ToCss + PartialEq> ToCss for InsetRect<L> {
    // XXXManishearth We should try to reduce the number of values printed here
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        dest.write_str("inset(")?;
        self.top.to_css(dest)?;
        dest.write_str(" ")?;
        self.right.to_css(dest)?;
        dest.write_str(" ")?;
        self.bottom.to_css(dest)?;
        dest.write_str(" ")?;
        self.left.to_css(dest)?;
        if let Some(ref radius) = self.round {
            dest.write_str(" round ")?;
            radius.to_css(dest)?;
        }

        dest.write_str(")")
    }
}

impl<L: ToComputedValue> ToComputedValue for InsetRect<L> {
    type ComputedValue = InsetRect<L::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        InsetRect {
            top: self.top.to_computed_value(cx),
            right: self.right.to_computed_value(cx),
            bottom: self.bottom.to_computed_value(cx),
            left: self.left.to_computed_value(cx),
            round: self.round.as_ref().map(|r| r.to_computed_value(cx)),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        InsetRect {
            top: ToComputedValue::from_computed_value(&computed.top),
            right: ToComputedValue::from_computed_value(&computed.right),
            bottom: ToComputedValue::from_computed_value(&computed.bottom),
            left: ToComputedValue::from_computed_value(&computed.left),
            round: computed.round.as_ref().map(|r| ToComputedValue::from_computed_value(r)),
        }
    }
}

/// A shape source, for some reference box
///
/// `clip-path` uses ShapeSource<BasicShape, GeometryBox>,
/// `shape-outside` uses ShapeSource<BasicShape, ShapeBox>
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum ShapeSource<B, T> {
    Url(SpecifiedUrl),
    Shape(B, Option<T>),
    Box(T),
    None,
}

impl<B, T> HasViewportPercentage for ShapeSource<B, T> {
    #[inline]
    fn has_viewport_percentage(&self) -> bool { false }
}

impl<B: ToCss, T: ToCss> ToCss for ShapeSource<B, T> {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            ShapeSource::Url(ref url) => url.to_css(dest),
            ShapeSource::Shape(ref shape, Some(ref ref_box)) => {
                shape.to_css(dest)?;
                dest.write_str(" ")?;
                ref_box.to_css(dest)
            },
            ShapeSource::Shape(ref shape, None) => shape.to_css(dest),
            ShapeSource::Box(ref val) => val.to_css(dest),
            ShapeSource::None => dest.write_str("none"),
        }
    }
}

impl<B: Parse, T: Parse> Parse for ShapeSource<B, T> {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(ShapeSource::None)
        }

        if let Ok(url) = input.try(|i| SpecifiedUrl::parse(context, i)) {
            return Ok(ShapeSource::Url(url))
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

        ref_box.map(|v| ShapeSource::Box(v)).ok_or(())
    }
}

impl<B: ToComputedValue, T: ToComputedValue> ToComputedValue for ShapeSource<B, T> {
    type ComputedValue = ShapeSource<B::ComputedValue, T::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        match *self {
            ShapeSource::Url(ref url) => ShapeSource::Url(url.to_computed_value(cx)),
            ShapeSource::Shape(ref shape, ref ref_box) => {
                ShapeSource::Shape(shape.to_computed_value(cx),
                                   ref_box.as_ref().map(|ref val| val.to_computed_value(cx)))
            },
            ShapeSource::Box(ref ref_box) => ShapeSource::Box(ref_box.to_computed_value(cx)),
            ShapeSource::None => ShapeSource::None,
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            ShapeSource::Url(ref url) => ShapeSource::Url(SpecifiedUrl::from_computed_value(url)),
            ShapeSource::Shape(ref shape, ref ref_box) => {
                ShapeSource::Shape(ToComputedValue::from_computed_value(shape),
                                    ref_box.as_ref().map(|val| ToComputedValue::from_computed_value(val)))
            },
            ShapeSource::Box(ref ref_box) => ShapeSource::Box(ToComputedValue::from_computed_value(ref_box)),
            ShapeSource::None => ShapeSource::None,
        }
    }
}
