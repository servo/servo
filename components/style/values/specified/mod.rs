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
use self::grid::{TrackBreadth as GenericTrackBreadth, TrackSize as GenericTrackSize};
use self::url::SpecifiedUrl;
use std::ascii::AsciiExt;
use std::f32::consts::PI;
use std::fmt;
use std::ops::Mul;
use style_traits::ToCss;
use super::{Auto, CSSFloat, HasViewportPercentage, Either, None_};
use super::computed::{ComputedValueAsSpecified, Context, ToComputedValue};
use super::computed::Shadow as ComputedShadow;

#[cfg(feature = "gecko")]
pub use self::align::{AlignItems, AlignJustifyContent, AlignJustifySelf, JustifyItems};
pub use self::grid::{GridLine, TrackKeyword};
pub use self::image::{AngleOrCorner, ColorStop, EndingShape as GradientEndingShape, Gradient};
pub use self::image::{GradientKind, HorizontalDirection, Image, LengthOrKeyword, LengthOrPercentageOrKeyword};
pub use self::image::{SizeKeyword, VerticalDirection};
pub use self::length::{FontRelativeLength, ViewportPercentageLength, CharacterWidth, Length, CalcLengthOrPercentage};
pub use self::length::{Percentage, LengthOrNone, LengthOrNumber, LengthOrPercentage, LengthOrPercentageOrAuto};
pub use self::length::{LengthOrPercentageOrNone, LengthOrPercentageOrAutoOrContent, NoCalcLength, CalcUnit};
pub use self::length::{MaxLength, MinLength};
pub use self::position::{HorizontalPosition, Position, VerticalPosition};

#[cfg(feature = "gecko")]
pub mod align;
pub mod basic_shape;
pub mod grid;
pub mod image;
pub mod length;
pub mod position;
pub mod url;

no_viewport_percentage!(i32);  // For PropertyDeclaration::Order

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct CSSColor {
    pub parsed: cssparser::Color,
    pub authored: Option<Box<str>>,
}

impl Parse for CSSColor {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let start_position = input.position();
        let authored = match input.next() {
            Ok(Token::Ident(s)) => Some(s.into_owned().into_boxed_str()),
            _ => None,
        };
        input.reset(start_position);
        Ok(CSSColor {
            parsed: try!(cssparser::Color::parse(input)),
            authored: authored,
        })
    }
}

no_viewport_percentage!(CSSColor);

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
    pub authored: Option<Box<str>>,
}

no_viewport_percentage!(CSSRGBA);

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

no_viewport_percentage!(BorderRadiusSize);

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

no_viewport_percentage!(BorderStyle);

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

no_viewport_percentage!(Number);

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

no_viewport_percentage!(Opacity);

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

/// The specified value of a grid `<track-breadth>`
pub type TrackBreadth = GenericTrackBreadth<LengthOrPercentage>;

/// The specified value of a grid `<track-size>`
pub type TrackSize = GenericTrackSize<LengthOrPercentage>;

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

no_viewport_percentage!(SVGPaint);

/// An SVG paint value
///
/// https://www.w3.org/TR/SVG2/painting.html#SpecifyingPaint
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct SVGPaint {
    /// The paint source
    pub kind: SVGPaintKind,
    /// The fallback color
    pub fallback: Option<CSSColor>,
}

/// An SVG paint value without the fallback
///
/// Whereas the spec only allows PaintServer
/// to have a fallback, Gecko lets the context
/// properties have a fallback as well.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum SVGPaintKind {
    /// `none`
    None,
    /// `<color>`
    Color(CSSColor),
    /// `url(...)`
    PaintServer(SpecifiedUrl),
    /// `context-fill`
    ContextFill,
    /// `context-stroke`
    ContextStroke,
}

impl SVGPaintKind {
    fn parse_ident(input: &mut Parser) -> Result<Self, ()> {
        Ok(match_ignore_ascii_case! { input.expect_ident()?,
            "none" => SVGPaintKind::None,
            "context-fill" => SVGPaintKind::ContextFill,
            "context-stroke" => SVGPaintKind::ContextStroke,
            _ => return Err(())
        })
    }
}

impl Parse for SVGPaint {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(url) = input.try(|i| SpecifiedUrl::parse(context, i)) {
            let fallback = input.try(|i| CSSColor::parse(context, i));
            Ok(SVGPaint {
                kind: SVGPaintKind::PaintServer(url),
                fallback: fallback.ok(),
            })
        } else if let Ok(kind) = input.try(SVGPaintKind::parse_ident) {
            if kind == SVGPaintKind::None {
                Ok(SVGPaint {
                    kind: kind,
                    fallback: None,
                })
            } else {
                let fallback = input.try(|i| CSSColor::parse(context, i));
                Ok(SVGPaint {
                    kind: kind,
                    fallback: fallback.ok(),
                })
            }
        } else if let Ok(color) = input.try(|i| CSSColor::parse(context, i)) {
            Ok(SVGPaint {
                kind: SVGPaintKind::Color(color),
                fallback: None,
            })
        } else {
            Err(())
        }
    }
}

impl ToCss for SVGPaintKind {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            SVGPaintKind::None => dest.write_str("none"),
            SVGPaintKind::ContextStroke => dest.write_str("context-stroke"),
            SVGPaintKind::ContextFill => dest.write_str("context-fill"),
            SVGPaintKind::Color(ref color) => color.to_css(dest),
            SVGPaintKind::PaintServer(ref server) => server.to_css(dest),
        }
    }
}

impl ToCss for SVGPaint {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.kind.to_css(dest)?;
        if let Some(ref fallback) = self.fallback {
            fallback.to_css(dest)?;
        }
        Ok(())
    }
}


impl ToComputedValue for SVGPaint {
    type ComputedValue = super::computed::SVGPaint;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        super::computed::SVGPaint {
            kind: self.kind.to_computed_value(context),
            fallback: self.fallback.as_ref().map(|f| f.to_computed_value(context))
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        SVGPaint {
            kind: ToComputedValue::from_computed_value(&computed.kind),
            fallback: computed.fallback.as_ref().map(ToComputedValue::from_computed_value)
        }
    }
}

impl ToComputedValue for SVGPaintKind {
    type ComputedValue = super::computed::SVGPaintKind;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        match *self {
            SVGPaintKind::None => super::computed::SVGPaintKind::None,
            SVGPaintKind::ContextStroke => super::computed::SVGPaintKind::ContextStroke,
            SVGPaintKind::ContextFill => super::computed::SVGPaintKind::ContextFill,
            SVGPaintKind::Color(ref color) => {
                super::computed::SVGPaintKind::Color(color.to_computed_value(context))
            }
            SVGPaintKind::PaintServer(ref server) => {
                super::computed::SVGPaintKind::PaintServer(server.to_computed_value(context))
            }
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        match *computed {
            super::computed::SVGPaintKind::None => SVGPaintKind::None,
            super::computed::SVGPaintKind::ContextStroke => SVGPaintKind::ContextStroke,
            super::computed::SVGPaintKind::ContextFill => SVGPaintKind::ContextFill,
            super::computed::SVGPaintKind::Color(ref color) => {
                SVGPaintKind::Color(ToComputedValue::from_computed_value(color))
            }
            super::computed::SVGPaintKind::PaintServer(ref server) => {
                SVGPaintKind::PaintServer(ToComputedValue::from_computed_value(server))
            }
        }
    }
}

/// <length> | <percentage> | <number>
pub type LoPOrNumber = Either<LengthOrPercentage, Number>;

impl LoPOrNumber {
    /// parse a <length-percentage> | <number> enforcing that the contents aren't negative
    pub fn parse_non_negative(_: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(lop) = input.try(LengthOrPercentage::parse_non_negative) {
            Ok(Either::First(lop))
        } else if let Ok(num) = input.try(Number::parse_non_negative) {
            Ok(Either::Second(num))
        } else {
            Err(())
        }
    }
}

impl HasViewportPercentage for ClipRect {
    fn has_viewport_percentage(&self) -> bool {
        self.top.as_ref().map_or(false, |x| x.has_viewport_percentage()) ||
        self.right.as_ref().map_or(false, |x| x.has_viewport_percentage()) ||
        self.bottom.as_ref().map_or(false, |x| x.has_viewport_percentage()) ||
        self.left.as_ref().map_or(false, |x| x.has_viewport_percentage())
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// rect(<top>, <left>, <bottom>, <right>) used by clip and image-region
pub struct ClipRect {
    /// <top> (<length> | <auto>)
    pub top: Option<Length>,
    /// <right> (<length> | <auto>)
    pub right: Option<Length>,
    /// <bottom> (<length> | <auto>)
    pub bottom: Option<Length>,
    /// <left> (<length> | <auto>)
    pub left: Option<Length>,
}


impl ToCss for ClipRect {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("rect("));

        if let Some(ref top) = self.top {
            try!(top.to_css(dest));
            try!(dest.write_str(", "));
        } else {
            try!(dest.write_str("auto, "));
        }

        if let Some(ref right) = self.right {
            try!(right.to_css(dest));
            try!(dest.write_str(", "));
        } else {
            try!(dest.write_str("auto, "));
        }

        if let Some(ref bottom) = self.bottom {
            try!(bottom.to_css(dest));
            try!(dest.write_str(", "));
        } else {
            try!(dest.write_str("auto, "));
        }

        if let Some(ref left) = self.left {
            try!(left.to_css(dest));
        } else {
            try!(dest.write_str("auto"));
        }

        try!(dest.write_str(")"));
        Ok(())
    }
}

impl ToComputedValue for ClipRect {
    type ComputedValue = super::computed::ClipRect;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> super::computed::ClipRect {
        super::computed::ClipRect {
            top: self.top.as_ref().map(|top| top.to_computed_value(context)),
            right: self.right.as_ref().map(|right| right.to_computed_value(context)),
            bottom: self.bottom.as_ref().map(|bottom| bottom.to_computed_value(context)),
            left: self.left.as_ref().map(|left| left.to_computed_value(context)),
        }
    }

    #[inline]
    fn from_computed_value(computed: &super::computed::ClipRect) -> Self {
        ClipRect {
            top: computed.top.map(|top| ToComputedValue::from_computed_value(&top)),
            right: computed.right.map(|right| ToComputedValue::from_computed_value(&right)),
            bottom: computed.bottom.map(|bottom| ToComputedValue::from_computed_value(&bottom)),
            left: computed.left.map(|left| ToComputedValue::from_computed_value(&left)),
        }
    }
}

impl Parse for ClipRect {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        use values::specified::Length;

        fn parse_argument(context: &ParserContext, input: &mut Parser) -> Result<Option<Length>, ()> {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                Ok(None)
            } else {
                Length::parse(context, input).map(Some)
            }
        }

        if !try!(input.expect_function()).eq_ignore_ascii_case("rect") {
            return Err(())
        }

        input.parse_nested_block(|input| {
            let top = try!(parse_argument(context, input));
            let right;
            let bottom;
            let left;

            if input.try(|input| input.expect_comma()).is_ok() {
                right = try!(parse_argument(context, input));
                try!(input.expect_comma());
                bottom = try!(parse_argument(context, input));
                try!(input.expect_comma());
                left = try!(parse_argument(context, input));
            } else {
                right = try!(parse_argument(context, input));
                bottom = try!(parse_argument(context, input));
                left = try!(parse_argument(context, input));
            }
            Ok(ClipRect {
                top: top,
                right: right,
                bottom: bottom,
                left: left,
            })
        })
    }
}

/// rect(...) | auto
pub type ClipRectOrAuto = Either<ClipRect, Auto>;
