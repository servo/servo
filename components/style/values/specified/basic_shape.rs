/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS handling for the specified value of
//! [`basic-shape`][basic-shape]s
//!
//! [basic-shape]: https://drafts.csswg.org/css-shapes/#typedef-basic-shape

use cssparser::Parser;
use parser::{Parse, ParserContext};
use properties::shorthands::{parse_four_sides, serialize_four_sides};
use std::fmt;
use style_traits::ToCss;
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

impl<T: Parse + PartialEq + Copy> ShapeSource<T> {
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(_) = input.try(|input| input.expect_ident_matching("none")) {
            Ok(ShapeSource::None)
        } else if let Ok(url) = input.try(|input| SpecifiedUrl::parse(context, input)) {
            Ok(ShapeSource::Url(url))
        } else {
            fn parse_component<U: Parse>(input: &mut Parser, component: &mut Option<U>) -> bool {
                if component.is_some() {
                    return false; // already parsed this component
                }
                *component = input.try(U::parse).ok();
                component.is_some()
            }

            let mut shape = None;
            let mut reference = None;
            loop {
                if !parse_component(input, &mut shape) &&
                   !parse_component(input, &mut reference) {
                    break;
                }
            }
            match (shape, reference) {
                (Some(shape), _) => Ok(ShapeSource::Shape(shape, reference)),
                (None, Some(reference)) => Ok(ShapeSource::Box(reference)),
                (None, None) => Err(()),
            }
        }
    }
}

impl<T: ToComputedValue> ToComputedValue for ShapeSource<T> {
    type ComputedValue = computed_basic_shape::ShapeSource<T::ComputedValue>;

    #[inline]
    fn to_computed_value(&self, cx: &Context) -> Self::ComputedValue {
        match *self {
            ShapeSource::Url(ref url) => {
                computed_basic_shape::ShapeSource::Url(url.to_computed_value(cx))
            }
            ShapeSource::Shape(ref shape, ref reference) => {
                computed_basic_shape::ShapeSource::Shape(
                    shape.to_computed_value(cx),
                    reference.as_ref().map(|ref r| r.to_computed_value(cx)))
            }
            ShapeSource::Box(ref reference) => {
                computed_basic_shape::ShapeSource::Box(reference.to_computed_value(cx))
            }
            ShapeSource::None => computed_basic_shape::ShapeSource::None,
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            computed_basic_shape::ShapeSource::Url(ref url) => {
                ShapeSource::Url(SpecifiedUrl::from_computed_value(url))
            }
            computed_basic_shape::ShapeSource::Shape(ref shape, ref reference) => {
                ShapeSource::Shape(
                    ToComputedValue::from_computed_value(shape),
                    reference.as_ref().map(|r| ToComputedValue::from_computed_value(r)))
            }
            computed_basic_shape::ShapeSource::Box(ref reference) => {
                ShapeSource::Box(ToComputedValue::from_computed_value(reference))
            }
            computed_basic_shape::ShapeSource::None => ShapeSource::None,
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

impl Parse for BasicShape {
    fn parse(input: &mut Parser) -> Result<BasicShape, ()> {
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
    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            computed_basic_shape::BasicShape::Inset(ref rect) => {
                BasicShape::Inset(ToComputedValue::from_computed_value(rect))
            }
            computed_basic_shape::BasicShape::Circle(ref circle) => {
                BasicShape::Circle(ToComputedValue::from_computed_value(circle))
            }
            computed_basic_shape::BasicShape::Ellipse(ref e) => {
                BasicShape::Ellipse(ToComputedValue::from_computed_value(e))
            }
            computed_basic_shape::BasicShape::Polygon(ref poly) => {
                BasicShape::Polygon(ToComputedValue::from_computed_value(poly))
            }
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

impl Parse for InsetRect {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
        match_ignore_ascii_case! { try!(input.expect_function()),
                                   "inset" => {
                                       Ok(try!(input.parse_nested_block(InsetRect::parse_function_arguments)))
                                   },
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
            round: self.round.map(|r| r.to_computed_value(cx)),
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
fn serialize_basicshape_position<W>(position: &Position, dest: &mut W)
    -> fmt::Result where W: fmt::Write {
        use values::specified::Length;
        use values::specified::position::Keyword;

        // keyword-percentage pairs can be folded into a single percentage
        fn fold_keyword(keyword: Option<Keyword>, length: Option<LengthOrPercentage>)
            -> Option<LengthOrPercentage> {
            let pc = match length.map(replace_with_percent) {
                None => Percentage(0.0), // unspecified length = 0%
                Some(LengthOrPercentage::Percentage(pc)) => pc,
                _ => return None
            };
            let percent = match keyword {
                Some(Keyword::Center) => {
                    // center cannot pair with lengths
                    assert!(length.is_none());
                    Percentage(0.5)
                },
                Some(Keyword::Left) | Some(Keyword::Top) | None => pc,
                Some(Keyword::Right) | Some(Keyword::Bottom) => Percentage(1.0 - pc.0),
            };
            Some(LengthOrPercentage::Percentage(percent))
        }

        // 0 length should be replaced with 0%
        fn replace_with_percent(input: LengthOrPercentage) -> LengthOrPercentage {
            match input {
                LengthOrPercentage::Length(Length::Absolute(au)) if au.0 == 0 => {
                    LengthOrPercentage::Percentage(Percentage(0.0))
                }
                _ => {
                    input
                }
            }
        }

        fn serialize_position_pair<W>(x: LengthOrPercentage, y: LengthOrPercentage,
                                      dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(replace_with_percent(x).to_css(dest));
            try!(dest.write_str(" "));
            replace_with_percent(y).to_css(dest)
        }

        match (position.horiz_keyword, position.horiz_position,
               position.vert_keyword, position.vert_position) {
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
                if let (Some(x), Some(y)) = (fold_keyword(hk, hp), fold_keyword(vk, vp)) {
                    serialize_position_pair(x, y, dest)
                } else {
                    // We failed to reduce it to a two-value form,
                    // so we expand it to 4-value
                    let zero = LengthOrPercentage::Percentage(Percentage(0.0));
                    try!(hk.unwrap_or(Keyword::Left).to_css(dest));
                    try!(dest.write_str(" "));
                    try!(replace_with_percent(hp.unwrap_or(zero)).to_css(dest));
                    try!(dest.write_str(" "));
                    try!(vk.unwrap_or(Keyword::Top).to_css(dest));
                    try!(dest.write_str(" "));
                    replace_with_percent(vp.unwrap_or(zero)).to_css(dest)
                }
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
    pub fn parse_function_arguments(input: &mut Parser) -> Result<Circle, ()> {
        let radius = input.try(ShapeRadius::parse).ok().unwrap_or_else(Default::default);
        let position = if let Ok(_) = input.try(|input| input.expect_ident_matching("at")) {
            try!(Position::parse(input))
        } else {
            // Defaults to origin
            Position {
                horiz_keyword: Some(Keyword::Center),
                horiz_position: None,
                vert_keyword: Some(Keyword::Center),
                vert_position: None,
            }
        };
        Ok(Circle {
            radius: radius,
            position: position,
        })
    }
}

impl Parse for Circle {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
        match_ignore_ascii_case! { try!(input.expect_function()),
                                   "circle" => {
                                       Ok(try!(input.parse_nested_block(Circle::parse_function_arguments)))
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

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// https://drafts.csswg.org/css-shapes/#funcdef-ellipse
pub struct Ellipse {
    pub semiaxis_x: ShapeRadius,
    pub semiaxis_y: ShapeRadius,
    pub position: Position,
}


impl Ellipse {
    pub fn parse_function_arguments(input: &mut Parser) -> Result<Ellipse, ()> {
        let (a, b) = input.try(|input| -> Result<_, ()> {
            Ok((try!(ShapeRadius::parse(input)), try!(ShapeRadius::parse(input))))
        }).ok().unwrap_or_default();
        let position = if let Ok(_) = input.try(|input| input.expect_ident_matching("at")) {
            try!(Position::parse(input))
        } else {
            // Defaults to origin
            Position {
                horiz_keyword: Some(Keyword::Center),
                horiz_position: None,
                vert_keyword: Some(Keyword::Center),
                vert_position: None,
            }
        };
        Ok(Ellipse {
            semiaxis_x: a,
            semiaxis_y: b,
            position: position,
        })
    }
}

impl Parse for Ellipse {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
        match_ignore_ascii_case! { try!(input.expect_function()),
                                   "ellipse" => {
                                       Ok(try!(input.parse_nested_block(Ellipse::parse_function_arguments)))
                                   },
                                   _ => Err(())
        }
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
pub struct Polygon {
    pub fill: FillRule,
    pub coordinates: Vec<(LengthOrPercentage, LengthOrPercentage)>,
}

impl Polygon {
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

impl Parse for Polygon {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
        match_ignore_ascii_case! { try!(input.expect_function()),
                                   "polygon" => {
                                       Ok(try!(input.parse_nested_block(Polygon::parse_function_arguments)))
                                   },
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
                                         })
                                         .collect(),
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

impl Parse for ShapeRadius {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
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

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            computed_basic_shape::ShapeRadius::Length(ref lop) => {
                ShapeRadius::Length(ToComputedValue::from_computed_value(lop))
            }
            computed_basic_shape::ShapeRadius::ClosestSide => ShapeRadius::ClosestSide,
            computed_basic_shape::ShapeRadius::FarthestSide => ShapeRadius::FarthestSide,
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

impl Parse for BorderRadius {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
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

impl Parse for FillRule {
    fn parse(input: &mut Parser) -> Result<FillRule, ()> {
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

/// https://drafts.fxtf.org/css-masking-1/#typedef-geometry-box
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum GeometryBox {
    Fill,
    Stroke,
    View,
    ShapeBox(ShapeBox),
}

impl Parse for GeometryBox {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
        if let Ok(shape_box) = input.try(ShapeBox::parse) {
            Ok(GeometryBox::ShapeBox(shape_box))
        } else {
            match_ignore_ascii_case! { try!(input.expect_ident()),
                "fill-box" => Ok(GeometryBox::Fill),
                "stroke-box" => Ok(GeometryBox::Stroke),
                "view-box" => Ok(GeometryBox::View),
                _ => Err(())
            }
        }
    }
}

impl ToCss for GeometryBox {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            GeometryBox::Fill => dest.write_str("fill-box"),
            GeometryBox::Stroke => dest.write_str("stroke-box"),
            GeometryBox::View => dest.write_str("view-box"),
            GeometryBox::ShapeBox(s) => s.to_css(dest),
        }
    }
}

impl ComputedValueAsSpecified for GeometryBox {}

// https://drafts.csswg.org/css-shapes-1/#typedef-shape-box
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum ShapeBox {
    Margin,
    // https://drafts.csswg.org/css-backgrounds-3/#box
    Border,
    Padding,
    Content,
}

impl Parse for ShapeBox {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
        match_ignore_ascii_case! { try!(input.expect_ident()),
            "margin-box" => Ok(ShapeBox::Margin),
            "border-box" => Ok(ShapeBox::Border),
            "padding-box" => Ok(ShapeBox::Padding),
            "content-box" => Ok(ShapeBox::Content),
            _ => Err(())
        }
    }
}

impl ToCss for ShapeBox {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            ShapeBox::Margin => dest.write_str("margin-box"),
            ShapeBox::Border => dest.write_str("border-box"),
            ShapeBox::Padding => dest.write_str("padding-box"),
            ShapeBox::Content => dest.write_str("content-box"),
        }
    }
}

impl ComputedValueAsSpecified for ShapeBox {}
