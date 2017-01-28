/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified values.
//!
//! TODO(emilio): Enhance docs.

use app_units::Au;
use cssparser::{self, Parser, Token};
use euclid::size::Size2D;
use parser::{ParserContext, Parse};
use self::url::SpecifiedUrl;
use std::ascii::AsciiExt;
use std::f32::consts::PI;
use std::fmt;
use std::ops::Mul;
use style_traits::ToCss;
use super::{CSSFloat, HasViewportPercentage, NoViewportPercentage, Either, None_};
use super::computed::{ComputedValueAsSpecified, Context, ToComputedValue};
use super::computed::Shadow as ComputedShadow;

pub use self::grid::GridLine;
pub use self::image::{AngleOrCorner, ColorStop, EndingShape as GradientEndingShape, Gradient};
pub use self::image::{GradientKind, HorizontalDirection, Image, LengthOrKeyword, LengthOrPercentageOrKeyword};
pub use self::image::{SizeKeyword, VerticalDirection};
pub use self::length::{FontRelativeLength, ViewportPercentageLength, CharacterWidth, Length, CalcLengthOrPercentage};
pub use self::length::{Percentage, LengthOrNone, LengthOrNumber, LengthOrPercentage, LengthOrPercentageOrAuto};
pub use self::length::{LengthOrPercentageOrNone, LengthOrPercentageOrAutoOrContent, NoCalcLength, CalcUnit};

pub mod basic_shape;
pub mod grid;
pub mod image;
pub mod length;
pub mod position;
pub mod url;

impl NoViewportPercentage for i32 {}  // For PropertyDeclaration::Order

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct CSSColor {
    pub parsed: cssparser::Color,
    pub authored: Option<String>,
}

impl Parse for CSSColor {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let start_position = input.position();
        let authored = match input.next() {
            Ok(Token::Ident(s)) => Some(s.into_owned()),
            _ => None,
        };
        input.reset(start_position);
        Ok(CSSColor {
            parsed: try!(cssparser::Color::parse(input)),
            authored: authored,
        })
    }
}

impl NoViewportPercentage for CSSColor {}

impl ToCss for CSSColor {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match self.authored {
            Some(ref s) => dest.write_str(s),
            None => self.parsed.to_css(dest),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct CSSRGBA {
    pub parsed: cssparser::RGBA,
    pub authored: Option<String>,
}

impl NoViewportPercentage for CSSRGBA {}

impl ToCss for CSSRGBA {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match self.authored {
            Some(ref s) => dest.write_str(s),
            None => self.parsed.to_css(dest),
        }
    }
}

#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct SimplifiedSumNode {
    values: Vec<SimplifiedValueNode>,
}
impl<'a> Mul<CSSFloat> for &'a SimplifiedSumNode {
    type Output = SimplifiedSumNode;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> SimplifiedSumNode {
        SimplifiedSumNode {
            values: self.values.iter().map(|p| p * scalar).collect()
        }
    }
}

#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub enum SimplifiedValueNode {
    Length(NoCalcLength),
    Angle(Angle),
    Time(Time),
    Percentage(CSSFloat),
    Number(CSSFloat),
    Sum(Box<SimplifiedSumNode>),
}

impl<'a> Mul<CSSFloat> for &'a SimplifiedValueNode {
    type Output = SimplifiedValueNode;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> SimplifiedValueNode {
        match *self {
            SimplifiedValueNode::Length(ref l) => SimplifiedValueNode::Length(l.clone() * scalar),
            SimplifiedValueNode::Percentage(p) => SimplifiedValueNode::Percentage(p * scalar),
            SimplifiedValueNode::Angle(Angle(a)) => SimplifiedValueNode::Angle(Angle(a * scalar)),
            SimplifiedValueNode::Time(Time(t)) => SimplifiedValueNode::Time(Time(t * scalar)),
            SimplifiedValueNode::Number(n) => SimplifiedValueNode::Number(n * scalar),
            SimplifiedValueNode::Sum(ref s) => {
                let sum = &**s * scalar;
                SimplifiedValueNode::Sum(Box::new(sum))
            }
        }
    }
}

#[allow(missing_docs)]
pub fn parse_integer(input: &mut Parser) -> Result<i32, ()> {
    match try!(input.next()) {
        Token::Number(ref value) => value.int_value.ok_or(()),
        Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
            let ast = try!(input.parse_nested_block(|i| CalcLengthOrPercentage::parse_sum(i, CalcUnit::Integer)));

            let mut result = None;

            for ref node in ast.products {
                match try!(CalcLengthOrPercentage::simplify_product(node)) {
                    SimplifiedValueNode::Number(val) =>
                        result = Some(result.unwrap_or(0) + val as i32),
                    _ => unreachable!()
                }
            }

            match result {
                Some(result) => Ok(result),
                _ => Err(())
            }
        }
        _ => Err(())
    }
}

#[allow(missing_docs)]
pub fn parse_number(input: &mut Parser) -> Result<f32, ()> {
    match try!(input.next()) {
        Token::Number(ref value) => Ok(value.value),
        Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
            let ast = try!(input.parse_nested_block(|i| CalcLengthOrPercentage::parse_sum(i, CalcUnit::Number)));

            let mut result = None;

            for ref node in ast.products {
                match try!(CalcLengthOrPercentage::simplify_product(node)) {
                    SimplifiedValueNode::Number(val) =>
                        result = Some(result.unwrap_or(0.) + val),
                    _ => unreachable!()
                }
            }

            match result {
                Some(result) => Ok(result),
                _ => Err(())
            }
        }
        _ => Err(())
    }
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct BorderRadiusSize(pub Size2D<LengthOrPercentage>);

impl NoViewportPercentage for BorderRadiusSize {}

impl BorderRadiusSize {
    #[allow(missing_docs)]
    pub fn zero() -> BorderRadiusSize {
        let zero = LengthOrPercentage::Length(NoCalcLength::zero());
        BorderRadiusSize(Size2D::new(zero.clone(), zero))
    }

    #[allow(missing_docs)]
    pub fn new(width: LengthOrPercentage, height: LengthOrPercentage) -> BorderRadiusSize {
        BorderRadiusSize(Size2D::new(width, height))
    }

    #[allow(missing_docs)]
    pub fn circle(radius: LengthOrPercentage) -> BorderRadiusSize {
        BorderRadiusSize(Size2D::new(radius.clone(), radius))
    }
}

impl Parse for BorderRadiusSize {
    #[inline]
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let first = try!(LengthOrPercentage::parse_non_negative(input));
        let second = input.try(LengthOrPercentage::parse_non_negative)
            .unwrap_or_else(|()| first.clone());
        Ok(BorderRadiusSize(Size2D::new(first, second)))
    }
}

impl ToCss for BorderRadiusSize {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(self.0.width.to_css(dest));
        try!(dest.write_str(" "));
        self.0.height.to_css(dest)
    }
}

#[derive(Clone, PartialEq, PartialOrd, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
/// An angle, normalized to radians.
pub struct Angle(pub CSSFloat);

impl ToCss for Angle {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        write!(dest, "{}rad", self.0)
    }
}

impl Angle {
    #[inline]
    #[allow(missing_docs)]
    pub fn radians(self) -> f32 {
        self.0
    }

    #[inline]
    #[allow(missing_docs)]
    pub fn from_radians(r: f32) -> Self {
        Angle(r)
    }
}

const RAD_PER_DEG: CSSFloat = PI / 180.0;
const RAD_PER_GRAD: CSSFloat = PI / 200.0;
const RAD_PER_TURN: CSSFloat = PI * 2.0;

impl Parse for Angle {
    /// Parses an angle according to CSS-VALUES § 6.1.
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) => Angle::parse_dimension(value.value, unit),
            Token::Number(ref value) if value.value == 0. => Ok(Angle(0.)),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                input.parse_nested_block(CalcLengthOrPercentage::parse_angle)
            },
            _ => Err(())
        }
    }
}

impl Angle {
    #[allow(missing_docs)]
    pub fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Angle, ()> {
        match_ignore_ascii_case! { unit,
            "deg" => Ok(Angle(value * RAD_PER_DEG)),
            "grad" => Ok(Angle(value * RAD_PER_GRAD)),
            "turn" => Ok(Angle(value * RAD_PER_TURN)),
            "rad" => Ok(Angle(value)),
             _ => Err(())
        }
    }
}

#[allow(missing_docs)]
pub fn parse_border_radius(context: &ParserContext, input: &mut Parser) -> Result<BorderRadiusSize, ()> {
    input.try(|i| BorderRadiusSize::parse(context, i)).or_else(|_| {
        match_ignore_ascii_case! { try!(input.expect_ident()),
            "thin" => Ok(BorderRadiusSize::circle(
                             LengthOrPercentage::Length(NoCalcLength::from_px(1.)))),
            "medium" => Ok(BorderRadiusSize::circle(
                               LengthOrPercentage::Length(NoCalcLength::from_px(3.)))),
            "thick" => Ok(BorderRadiusSize::circle(
                              LengthOrPercentage::Length(NoCalcLength::from_px(5.)))),
            _ => Err(())
        }
    })
}

#[allow(missing_docs)]
pub fn parse_border_width(input: &mut Parser) -> Result<Length, ()> {
    input.try(Length::parse_non_negative).or_else(|()| {
        match_ignore_ascii_case! { try!(input.expect_ident()),
            "thin" => Ok(Length::from_px(1.)),
            "medium" => Ok(Length::from_px(3.)),
            "thick" => Ok(Length::from_px(5.)),
            _ => Err(())
        }
    })
}

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum BorderWidth {
    Thin,
    Medium,
    Thick,
    Width(Length),
}

impl Parse for BorderWidth {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<BorderWidth, ()> {
        match input.try(Length::parse_non_negative) {
            Ok(length) => Ok(BorderWidth::Width(length)),
            Err(_) => match_ignore_ascii_case! { try!(input.expect_ident()),
               "thin" => Ok(BorderWidth::Thin),
               "medium" => Ok(BorderWidth::Medium),
               "thick" => Ok(BorderWidth::Thick),
               _ => Err(())
            }
        }
    }
}

impl BorderWidth {
    #[allow(missing_docs)]
    pub fn from_length(length: Length) -> Self {
        BorderWidth::Width(length)
    }
}

impl ToCss for BorderWidth {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            BorderWidth::Thin => dest.write_str("thin"),
            BorderWidth::Medium => dest.write_str("medium"),
            BorderWidth::Thick => dest.write_str("thick"),
            BorderWidth::Width(ref length) => length.to_css(dest)
        }
    }
}

impl HasViewportPercentage for BorderWidth {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            BorderWidth::Thin | BorderWidth::Medium | BorderWidth::Thick => false,
            BorderWidth::Width(ref length) => length.has_viewport_percentage()
         }
    }
}

impl ToComputedValue for BorderWidth {
    type ComputedValue = Au;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        // We choose the pixel length of the keyword values the same as both spec and gecko.
        // Spec: https://drafts.csswg.org/css-backgrounds-3/#line-width
        // Gecko: https://bugzilla.mozilla.org/show_bug.cgi?id=1312155#c0
        match *self {
            BorderWidth::Thin => Length::from_px(1.).to_computed_value(context),
            BorderWidth::Medium => Length::from_px(3.).to_computed_value(context),
            BorderWidth::Thick => Length::from_px(5.).to_computed_value(context),
            BorderWidth::Width(ref length) => length.to_computed_value(context)
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        BorderWidth::Width(ToComputedValue::from_computed_value(computed))
    }
}

// The integer values here correspond to the border conflict resolution rules in CSS 2.1 §
// 17.6.2.1. Higher values override lower values.
define_numbered_css_keyword_enum! { BorderStyle:
    "none" => none = -1,
    "solid" => solid = 6,
    "double" => double = 7,
    "dotted" => dotted = 4,
    "dashed" => dashed = 5,
    "hidden" => hidden = -2,
    "groove" => groove = 1,
    "ridge" => ridge = 3,
    "inset" => inset = 0,
    "outset" => outset = 2,
}

impl NoViewportPercentage for BorderStyle {}

impl BorderStyle {
    /// Whether this border style is either none or hidden.
    pub fn none_or_hidden(&self) -> bool {
        matches!(*self, BorderStyle::none | BorderStyle::hidden)
    }
}

/// A time in seconds according to CSS-VALUES § 6.2.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Time(pub CSSFloat);

impl Time {
    /// Returns the time in fractional seconds.
    pub fn seconds(self) -> f32 {
        let Time(seconds) = self;
        seconds
    }

    /// Parses a time according to CSS-VALUES § 6.2.
    fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Time, ()> {
        if unit.eq_ignore_ascii_case("s") {
            Ok(Time(value))
        } else if unit.eq_ignore_ascii_case("ms") {
            Ok(Time(value / 1000.0))
        } else {
            Err(())
        }
    }
}

impl ComputedValueAsSpecified for Time {}

impl Parse for Time {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match input.next() {
            Ok(Token::Dimension(ref value, ref unit)) => {
                Time::parse_dimension(value.value, &unit)
            }
            Ok(Token::Function(ref name)) if name.eq_ignore_ascii_case("calc") => {
                input.parse_nested_block(CalcLengthOrPercentage::parse_time)
            }
            _ => Err(())
        }
    }
}

impl ToCss for Time {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        write!(dest, "{}s", self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct Number(pub CSSFloat);

impl NoViewportPercentage for Number {}

impl Parse for Number {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        parse_number(input).map(Number)
    }
}

impl Number {
    fn parse_with_minimum(input: &mut Parser, min: CSSFloat) -> Result<Number, ()> {
        match parse_number(input) {
            Ok(value) if value < min => Err(()),
            value => value.map(Number),
        }
    }

    #[allow(missing_docs)]
    pub fn parse_non_negative(input: &mut Parser) -> Result<Number, ()> {
        Number::parse_with_minimum(input, 0.0)
    }

    #[allow(missing_docs)]
    pub fn parse_at_least_one(input: &mut Parser) -> Result<Number, ()> {
        Number::parse_with_minimum(input, 1.0)
    }
}

impl ToComputedValue for Number {
    type ComputedValue = CSSFloat;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> CSSFloat { self.0 }

    #[inline]
    fn from_computed_value(computed: &CSSFloat) -> Self {
        Number(*computed)
    }
}

impl ToCss for Number {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct Opacity(pub CSSFloat);

impl NoViewportPercentage for Opacity {}

impl Parse for Opacity {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        parse_number(input).map(Opacity)
    }
}

impl ToComputedValue for Opacity {
    type ComputedValue = CSSFloat;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> CSSFloat {
        if self.0 < 0.0 {
            0.0
        } else if self.0 > 1.0 {
            1.0
        } else {
            self.0
        }
    }

    #[inline]
    fn from_computed_value(computed: &CSSFloat) -> Self {
        Opacity(*computed)
    }
}

impl ToCss for Opacity {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)
    }
}

#[allow(missing_docs)]
pub type UrlOrNone = Either<SpecifiedUrl, None_>;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct Shadow {
    pub offset_x: Length,
    pub offset_y: Length,
    pub blur_radius: Length,
    pub spread_radius: Length,
    pub color: Option<CSSColor>,
    pub inset: bool,
}

impl HasViewportPercentage for Shadow {
    fn has_viewport_percentage(&self) -> bool {
        self.offset_x.has_viewport_percentage() ||
        self.offset_y.has_viewport_percentage() ||
        self.blur_radius.has_viewport_percentage() ||
        self.spread_radius.has_viewport_percentage()
    }
}

impl ToComputedValue for Shadow {
    type ComputedValue = ComputedShadow;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        ComputedShadow {
            offset_x: self.offset_x.to_computed_value(context),
            offset_y: self.offset_y.to_computed_value(context),
            blur_radius: self.blur_radius.to_computed_value(context),
            spread_radius: self.spread_radius.to_computed_value(context),
            color: self.color
                        .as_ref()
                        .map(|color| color.parsed)
                        .unwrap_or(cssparser::Color::CurrentColor),
            inset: self.inset,
        }
    }

    #[inline]
    fn from_computed_value(computed: &ComputedShadow) -> Self {
        Shadow {
            offset_x: ToComputedValue::from_computed_value(&computed.offset_x),
            offset_y: ToComputedValue::from_computed_value(&computed.offset_y),
            blur_radius: ToComputedValue::from_computed_value(&computed.blur_radius),
            spread_radius: ToComputedValue::from_computed_value(&computed.spread_radius),
            color: Some(ToComputedValue::from_computed_value(&computed.color)),
            inset: computed.inset,
        }
    }
}

impl Shadow {
    // disable_spread_and_inset is for filter: drop-shadow(...)
    #[allow(missing_docs)]
    pub fn parse(context:  &ParserContext, input: &mut Parser, disable_spread_and_inset: bool) -> Result<Shadow, ()> {
        let length_count = if disable_spread_and_inset { 3 } else { 4 };
        let mut lengths = [Length::zero(), Length::zero(), Length::zero(), Length::zero()];
        let mut lengths_parsed = false;
        let mut color = None;
        let mut inset = false;

        loop {
            if !inset && !disable_spread_and_inset {
                if input.try(|input| input.expect_ident_matching("inset")).is_ok() {
                    inset = true;
                    continue
                }
            }
            if !lengths_parsed {
                if let Ok(value) = input.try(|i| Length::parse(context, i)) {
                    lengths[0] = value;
                    let mut length_parsed_count = 1;
                    while length_parsed_count < length_count {
                        if let Ok(value) = input.try(|i| Length::parse(context, i)) {
                            lengths[length_parsed_count] = value
                        } else {
                            break
                        }
                        length_parsed_count += 1;
                    }

                    // The first two lengths must be specified.
                    if length_parsed_count < 2 {
                        return Err(())
                    }

                    lengths_parsed = true;
                    continue
                }
            }
            if color.is_none() {
                if let Ok(value) = input.try(|i| CSSColor::parse(context, i)) {
                    color = Some(value);
                    continue
                }
            }
            break
        }

        // Lengths must be specified.
        if !lengths_parsed {
            return Err(())
        }

        Ok(Shadow {
            offset_x: lengths[0].take(),
            offset_y: lengths[1].take(),
            blur_radius: lengths[2].take(),
            spread_radius: if disable_spread_and_inset { Length::zero() } else { lengths[3].take() },
            color: color,
            inset: inset,
        })
    }
}
