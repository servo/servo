/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`basic-shape`][basic-shape]s
//!
//! [basic-shape]: https://drafts.csswg.org/css-shapes/#typedef-basic-shape

use cssparser::Parser;
use euclid::size::Size2D;
use parser::{Parse, ParserContext};
use properties::shorthands::{parse_four_sides, serialize_four_sides};
use std::ascii::AsciiExt;
use std::fmt;
use style_traits::ToCss;
use values::HasViewportPercentage;
use values::computed::{ComputedValueAsSpecified, Context, ToComputedValue};
use values::computed::basic_shape as computed_basic_shape;
use values::specified::{BorderRadiusSize, LengthOrPercentage, Percentage};
use values::specified::position::{Keyword, Position};
use values::specified::url::SpecifiedUrl;

/// A shape source, for some reference box
///
/// clip-path uses ShapeSource<GeometryBox>,
/// shape-outside uses ShapeSource<ShapeBox>
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum ShapeSource<T> {
    Url(SpecifiedUrl),
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
            ShapeSource::Url(ref url) => url.to_css(dest),
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

impl<T: Parse> Parse for ShapeSource<T> {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if input.try(|input| input.expect_ident_matching("none")).is_ok() {
            return Ok(ShapeSource::None)
        }

        if let Ok(url) = input.try(|input| SpecifiedUrl::parse(context, input)) {
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
        let mut reference = None;

        while parse_component(context, input, &mut shape) ||
              parse_component(context, input, &mut reference) {
            //
        }

        if let Some(shp) = shape {
            return Ok(ShapeSource::Shape(shp, reference))
        }

        match reference {
            Some(r) => Ok(ShapeSource::Box(r)),
            None => Err(())
        }
    }
}

impl<T: ToComputedValue> ToComputedValue for ShapeSource<T> {
    type ComputedValue = computed_basic_shape::ShapeSource<T::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        match *self {
            ShapeSource::Url(ref url) => computed_basic_shape::ShapeSource::Url(url.to_computed_value(cx)),
            ShapeSource::Shape(ref shape, ref reference) => {
                computed_basic_shape::ShapeSource::Shape(
                    shape.to_computed_value(cx),
                    reference.as_ref().map(|ref r| r.to_computed_value(cx)))
            },
            ShapeSource::Box(ref reference) =>
                computed_basic_shape::ShapeSource::Box(reference.to_computed_value(cx)),
            ShapeSource::None => computed_basic_shape::ShapeSource::None,
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            computed_basic_shape::ShapeSource::Url(ref url) =>
                ShapeSource::Url(SpecifiedUrl::from_computed_value(url)),
            computed_basic_shape::ShapeSource::Shape(ref shape, ref reference) => {
                ShapeSource::Shape(
                    ToComputedValue::from_computed_value(shape),
                    reference.as_ref().map(|r| ToComputedValue::from_computed_value(r)))
            }
            computed_basic_shape::ShapeSource::Box(ref reference) =>
                ShapeSource::Box(ToComputedValue::from_computed_value(reference)),
            computed_basic_shape::ShapeSource::None => ShapeSource::None,
        }
    }
}

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

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-shapes/#funcdef-inset
#[allow(missing_docs)]
pub struct InsetRect {
    pub top: LengthOrPercentage,
    pub right: LengthOrPercentage,
    pub bottom: LengthOrPercentage,
    pub left: LengthOrPercentage,
    pub round: Option<BorderRadius>,
}

impl InsetRect {
    #[allow(missing_docs)]
    pub fn parse_function_arguments(context: &ParserContext, input: &mut Parser) -> Result<InsetRect, ()> {
        let (t, r, b, l) = try!(parse_four_sides(input, |i| LengthOrPercentage::parse(context, i)));
        let mut rect = InsetRect {
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
                input.parse_nested_block(|i| InsetRect::parse_function_arguments(context, i)),
            _ => Err(())
        }
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
            round: computed.round.map(|ref r| ToComputedValue::from_computed_value(r)),
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
    // 0 length should be replaced with 0%
    fn replace_with_percent(input: LengthOrPercentage) -> LengthOrPercentage {
        match input {
            LengthOrPercentage::Length(ref l) if l.is_zero() =>
                LengthOrPercentage::Percentage(Percentage(0.0)),
            _ => input
        }
    }

    // keyword-percentage pairs can be folded into a single percentage
    fn fold_keyword(keyword: Option<Keyword>,
                    length: Option<LengthOrPercentage>) -> Option<LengthOrPercentage> {
        let is_length_none = length.is_none();
        let pc = match length.map(replace_with_percent) {
            Some(LengthOrPercentage::Percentage(pc)) => pc,
            None => Percentage(0.0),        // unspecified length = 0%
            _ => return None
        };

        let percent = match keyword {
            Some(Keyword::Center) => {
                assert!(is_length_none);        // center cannot pair with lengths
                Percentage(0.5)
            },
            Some(Keyword::Left) | Some(Keyword::Top) | None => pc,
            Some(Keyword::Right) | Some(Keyword::Bottom) => Percentage(1.0 - pc.0),
            _ => return None,
        };

        Some(LengthOrPercentage::Percentage(percent))
    }

    fn serialize_position_pair<W>(x: LengthOrPercentage, y: LengthOrPercentage,
                                  dest: &mut W) -> fmt::Result where W: fmt::Write {
        replace_with_percent(x).to_css(dest)?;
        dest.write_str(" ")?;
        replace_with_percent(y).to_css(dest)
    }

    match (position.horizontal.keyword, position.horizontal.position.clone(),
           position.vertical.keyword, position.vertical.position.clone()) {
        (Some(hk), None, Some(vk), None) => {
            // two keywords: serialize as two lengths
            serialize_position_pair(hk.to_length_or_percentage(),
                                    vk.to_length_or_percentage(),
                                    dest)
        }
        (None, Some(hp), None, Some(vp)) => {
            // two lengths: just serialize regularly
            serialize_position_pair(hp, vp, dest)
        }
        (hk, hp, vk, vp) => {
            // only fold if both fold; the three-value form isn't
            // allowed here.
            if let (Some(x), Some(y)) = (fold_keyword(hk, hp.clone()),
                                         fold_keyword(vk, vp.clone())) {
                serialize_position_pair(x, y, dest)
            } else {
                // We failed to reduce it to a two-value form,
                // so we expand it to 4-value
                let zero = LengthOrPercentage::Percentage(Percentage(0.0));
                hk.unwrap_or(Keyword::Left).to_css(dest)?;
                dest.write_str(" ")?;
                replace_with_percent(hp.unwrap_or(zero.clone())).to_css(dest)?;
                dest.write_str(" ")?;
                vk.unwrap_or(Keyword::Top).to_css(dest)?;
                dest.write_str(" ")?;
                replace_with_percent(vp.unwrap_or(zero)).to_css(dest)
            }
        }
    }
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
        if ShapeRadius::ClosestSide != self.radius {
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
        if !self.semiaxis_x.is_default() || !self.semiaxis_y.is_default() {
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


#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-shapes/#funcdef-polygon
#[allow(missing_docs)]
pub struct Polygon {
    pub fill: FillRule,
    pub coordinates: Vec<(LengthOrPercentage, LengthOrPercentage)>,
}

impl Polygon {
    #[allow(missing_docs)]
    pub fn parse_function_arguments(context: &ParserContext, input: &mut Parser) -> Result<Polygon, ()> {
        let fill = input.try(|input| {
            let fill = FillRule::parse(input);
            // only eat the comma if there is something before it
            try!(input.expect_comma());
            fill
        }).ok().unwrap_or_else(Default::default);

        let buf = try!(input.parse_comma_separated(|input| {
            Ok((try!(LengthOrPercentage::parse(context, input)),
                try!(LengthOrPercentage::parse(context, input))))
        }));

        Ok(Polygon {
            fill: fill,
            coordinates: buf,
        })
    }
}

impl Parse for Polygon {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match input.try(|i| i.expect_function()) {
            Ok(ref s) if s.eq_ignore_ascii_case("polygon") =>
                input.parse_nested_block(|i| Polygon::parse_function_arguments(context, i)),
            _ => Err(())
        }
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
                                         }).collect(),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Polygon {
            fill: ToComputedValue::from_computed_value(&computed.fill),
            coordinates: computed.coordinates.iter()
                                             .map(|c| {
                                                (ToComputedValue::from_computed_value(&c.0),
                                                 ToComputedValue::from_computed_value(&c.1))
                                             }).collect(),
        }
    }
}

/// https://drafts.csswg.org/css-shapes/#typedef-shape-radius
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum ShapeRadius {
    Length(LengthOrPercentage),
    ClosestSide,
    FarthestSide,
}

impl ShapeRadius {
    fn is_default(&self) -> bool {
        *self == ShapeRadius::ClosestSide
    }
}

impl Default for ShapeRadius {
    fn default() -> Self {
        ShapeRadius::ClosestSide
    }
}

impl Parse for ShapeRadius {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        input.try(|i| LengthOrPercentage::parse_non_negative(context, i)).map(ShapeRadius::Length).or_else(|_| {
            match_ignore_ascii_case! { &try!(input.expect_ident()),
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
            ShapeRadius::Length(ref lop) => lop.to_css(dest),
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
            ShapeRadius::Length(ref lop) =>
                computed_basic_shape::ShapeRadius::Length(lop.to_computed_value(cx)),
            ShapeRadius::ClosestSide => computed_basic_shape::ShapeRadius::ClosestSide,
            ShapeRadius::FarthestSide => computed_basic_shape::ShapeRadius::FarthestSide,
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            computed_basic_shape::ShapeRadius::Length(ref lop) =>
                ShapeRadius::Length(ToComputedValue::from_computed_value(lop)),
            computed_basic_shape::ShapeRadius::ClosestSide => ShapeRadius::ClosestSide,
            computed_basic_shape::ShapeRadius::FarthestSide => ShapeRadius::FarthestSide,
        }
    }
}

/// https://drafts.csswg.org/css-backgrounds-3/#border-radius
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct BorderRadius {
    pub top_left: BorderRadiusSize,
    pub top_right: BorderRadiusSize,
    pub bottom_right: BorderRadiusSize,
    pub bottom_left: BorderRadiusSize,
}

/// Serialization helper for types of longhands like `border-radius` and `outline-radius`
pub fn serialize_radius_values<L, W>(dest: &mut W, top_left: &Size2D<L>,
                                     top_right: &Size2D<L>, bottom_right: &Size2D<L>,
                                     bottom_left: &Size2D<L>) -> fmt::Result
    where L: ToCss + PartialEq, W: fmt::Write
{
    if top_left.width == top_left.height &&
       top_right.width == top_right.height &&
       bottom_right.width == bottom_right.height &&
       bottom_left.width == bottom_left.height {
        serialize_four_sides(dest,
                             &top_left.width,
                             &top_right.width,
                             &bottom_right.width,
                             &bottom_left.width)
    } else {
        serialize_four_sides(dest,
                             &top_left.width,
                             &top_right.width,
                             &bottom_right.width,
                             &bottom_left.width)?;
        dest.write_str(" / ")?;
        serialize_four_sides(dest,
                             &top_left.height,
                             &top_right.height,
                             &bottom_right.height,
                             &bottom_left.height)
    }
}

impl ToCss for BorderRadius {
    #[inline]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        serialize_radius_values(dest, &self.top_left.0, &self.top_right.0,
                                &self.bottom_right.0, &self.bottom_left.0)
    }
}

impl Parse for BorderRadius {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let mut widths = try!(parse_one_set_of_border_values(context, input));
        let mut heights = if input.try(|input| input.expect_delim('/')).is_ok() {
            try!(parse_one_set_of_border_values(context, input))
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
    fn default() -> Self {
        FillRule::NonZero
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
