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
use std::f32;
use std::f32::consts::PI;
use std::fmt;
use std::ops::Mul;
use style_traits::ToCss;
use style_traits::values::specified::AllowedNumericType;
use super::{Auto, CSSFloat, CSSInteger, HasViewportPercentage, Either, None_};
use super::computed::{self, Context};
use super::computed::{Shadow as ComputedShadow, ToComputedValue};
use super::generics::BorderRadiusSize as GenericBorderRadiusSize;

#[cfg(feature = "gecko")]
pub use self::align::{AlignItems, AlignJustifyContent, AlignJustifySelf, JustifyItems};
pub use self::color::Color;
pub use self::grid::{GridLine, TrackKeyword};
pub use self::image::{AngleOrCorner, ColorStop, EndingShape as GradientEndingShape, Gradient};
pub use self::image::{GradientKind, HorizontalDirection, Image, ImageRect, LengthOrKeyword};
pub use self::image::{LengthOrPercentageOrKeyword, SizeKeyword, VerticalDirection};
pub use self::length::AbsoluteLength;
pub use self::length::{FontRelativeLength, ViewportPercentageLength, CharacterWidth, Length, CalcLengthOrPercentage};
pub use self::length::{Percentage, LengthOrNone, LengthOrNumber, LengthOrPercentage, LengthOrPercentageOrAuto};
pub use self::length::{LengthOrPercentageOrNone, LengthOrPercentageOrAutoOrContent, NoCalcLength, CalcUnit};
pub use self::length::{MaxLength, MinLength};
pub use self::position::{HorizontalPosition, Position, VerticalPosition};

#[cfg(feature = "gecko")]
pub mod align;
pub mod basic_shape;
pub mod color;
pub mod grid;
pub mod image;
pub mod length;
pub mod position;

/// Common handling for the specified value CSS url() values.
pub mod url {
use cssparser::Parser;
use parser::{Parse, ParserContext};
use values::HasViewportPercentage;
use values::computed::ComputedValueAsSpecified;

#[cfg(feature = "servo")]
pub use ::servo::url::*;
#[cfg(feature = "gecko")]
pub use ::gecko::url::*;

impl Parse for SpecifiedUrl {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let url = try!(input.expect_url());
        Self::parse_from_string(url, context)
    }
}

impl Eq for SpecifiedUrl {}

// TODO(emilio): Maybe consider ComputedUrl to save a word in style structs?
impl ComputedValueAsSpecified for SpecifiedUrl {}

no_viewport_percentage!(SpecifiedUrl);
}

no_viewport_percentage!(i32);  // For PropertyDeclaration::Order

#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct CSSColor {
    pub parsed: Color,
    pub authored: Option<Box<str>>,
}

impl Parse for CSSColor {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let start_position = input.position();
        let authored = match input.next() {
            Ok(Token::Ident(s)) => Some(s.into_owned().into_boxed_str()),
            _ => None,
        };
        input.reset(start_position);
        Ok(CSSColor {
            parsed: try!(Parse::parse(context, input)),
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

impl From<Color> for CSSColor {
    fn from(color: Color) -> Self {
        CSSColor {
            parsed: color,
            authored: None,
        }
    }
}

impl CSSColor {
    #[inline]
    /// Returns currentcolor value.
    pub fn currentcolor() -> CSSColor {
        Color::CurrentColor.into()
    }

    #[inline]
    /// Returns transparent value.
    pub fn transparent() -> CSSColor {
        // We should probably set authored to "transparent", but maybe it doesn't matter.
        Color::RGBA(cssparser::RGBA::transparent()).into()
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
    Angle(CSSFloat),
    Time(CSSFloat),
    Percentage(CSSFloat),
    Number(CSSFloat),
    Sum(Box<SimplifiedSumNode>),
}

impl<'a> Mul<CSSFloat> for &'a SimplifiedValueNode {
    type Output = SimplifiedValueNode;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> SimplifiedValueNode {
        match *self {
            SimplifiedValueNode::Length(ref l) => {
                SimplifiedValueNode::Length(l.clone() * scalar)
            },
            SimplifiedValueNode::Percentage(p) => {
                SimplifiedValueNode::Percentage(p * scalar)
            },
            SimplifiedValueNode::Angle(a) => {
                SimplifiedValueNode::Angle(a * scalar)
            },
            SimplifiedValueNode::Time(t) => {
                SimplifiedValueNode::Time(t * scalar)
            },
            SimplifiedValueNode::Number(n) => {
                SimplifiedValueNode::Number(n * scalar)
            },
            SimplifiedValueNode::Sum(ref s) => {
                let sum = &**s * scalar;
                SimplifiedValueNode::Sum(Box::new(sum))
            },
        }
    }
}

#[allow(missing_docs)]
pub fn parse_integer(context: &ParserContext, input: &mut Parser) -> Result<Integer, ()> {
    match try!(input.next()) {
        Token::Number(ref value) => value.int_value.ok_or(()).map(Integer::new),
        Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
            let ast = try!(input.parse_nested_block(|i| {
                CalcLengthOrPercentage::parse_sum(context, i, CalcUnit::Integer)
            }));

            let mut result = None;

            for ref node in ast.products {
                match try!(CalcLengthOrPercentage::simplify_product(node)) {
                    SimplifiedValueNode::Number(val) =>
                        result = Some(result.unwrap_or(0) + val as CSSInteger),
                    _ => unreachable!()
                }
            }

            match result {
                Some(result) => Ok(Integer::from_calc(result)),
                _ => Err(())
            }
        }
        _ => Err(())
    }
}

#[allow(missing_docs)]
pub fn parse_number(context: &ParserContext, input: &mut Parser) -> Result<Number, ()> {
    parse_number_with_clamping_mode(context, input, AllowedNumericType::All)
}

#[allow(missing_docs)]
pub fn parse_number_with_clamping_mode(context: &ParserContext,
                                       input: &mut Parser,
                                       clamping_mode: AllowedNumericType)
                                       -> Result<Number, ()> {
    match try!(input.next()) {
        Token::Number(ref value) if clamping_mode.is_ok(value.value) => {
            Ok(Number {
                value: value.value.min(f32::MAX).max(f32::MIN),
                calc_clamping_mode: None,
            })
        },
        Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
            let ast = try!(input.parse_nested_block(|i| {
                CalcLengthOrPercentage::parse_sum(context, i, CalcUnit::Number)
            }));

            let mut result = None;

            for ref node in ast.products {
                match try!(CalcLengthOrPercentage::simplify_product(node)) {
                    SimplifiedValueNode::Number(val) =>
                        result = Some(result.unwrap_or(0.) + val),
                    _ => unreachable!()
                }
            }

            match result {
                Some(result) => {
                    Ok(Number {
                        value: result.min(f32::MAX).max(f32::MIN),
                        calc_clamping_mode: Some(clamping_mode),
                    })
                },
                _ => Err(())
            }
        }
        _ => Err(())
    }
}

/// The specified value of `BorderRadiusSize`
pub type BorderRadiusSize = GenericBorderRadiusSize<LengthOrPercentage>;

impl Parse for BorderRadiusSize {
    #[inline]
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let first = try!(LengthOrPercentage::parse_non_negative(context, input));
        let second = input.try(|i| LengthOrPercentage::parse_non_negative(context, i))
            .unwrap_or_else(|()| first.clone());
        Ok(GenericBorderRadiusSize(Size2D::new(first, second)))
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
/// An angle consisting of a value and a unit.
pub struct Angle {
    value: CSSFloat,
    unit: AngleUnit,
    was_calc: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
/// A unit used together with an angle.
pub enum AngleUnit {
    /// Degrees, short name "deg".
    Degree,
    /// Gradians, short name "grad".
    Gradian,
    /// Radians, short name "rad".
    Radian,
    /// Turns, short name "turn".
    Turn,
}

impl ToCss for AngleUnit {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use self::AngleUnit::*;
        dest.write_str(match *self {
            Degree => "deg",
            Gradian => "grad",
            Radian => "rad",
            Turn => "turn",
        })
    }
}

impl ToCss for Angle {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if self.was_calc {
            dest.write_str("calc(")?;
        }
        self.value.to_css(dest)?;
        self.unit.to_css(dest)?;
        if self.was_calc {
            dest.write_str(")")?;
        }
        Ok(())
    }
}

impl ToComputedValue for Angle {
    type ComputedValue = computed::Angle;

    fn to_computed_value(&self, _context: &Context) -> Self::ComputedValue {
        computed::Angle::from_radians(self.radians())
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Angle::from_radians(computed.radians())
    }
}

impl Angle {
    /// Returns an angle with the given value in degrees.
    pub fn from_degrees(value: CSSFloat) -> Self {
        Angle { value: value, unit: AngleUnit::Degree, was_calc: false }
    }
    /// Returns an angle with the given value in gradians.
    pub fn from_gradians(value: CSSFloat) -> Self {
        Angle { value: value, unit: AngleUnit::Gradian, was_calc: false }
    }
    /// Returns an angle with the given value in turns.
    pub fn from_turns(value: CSSFloat) -> Self {
        Angle { value: value, unit: AngleUnit::Turn, was_calc: false }
    }
    /// Returns an angle with the given value in radians.
    pub fn from_radians(value: CSSFloat) -> Self {
        Angle { value: value, unit: AngleUnit::Radian, was_calc: false }
    }

    #[inline]
    #[allow(missing_docs)]
    pub fn radians(self) -> f32 {
        use self::AngleUnit::*;

        const RAD_PER_DEG: CSSFloat = PI / 180.0;
        const RAD_PER_GRAD: CSSFloat = PI / 200.0;
        const RAD_PER_TURN: CSSFloat = PI * 2.0;

        let radians = match self.unit {
            Degree => self.value * RAD_PER_DEG,
            Gradian => self.value * RAD_PER_GRAD,
            Turn => self.value * RAD_PER_TURN,
            Radian => self.value,
        };
        radians.min(f32::MAX).max(f32::MIN)
    }

    /// Returns an angle value that represents zero.
    pub fn zero() -> Self {
        Self::from_degrees(0.0)
    }

    /// Returns an `Angle` parsed from a `calc()` expression.
    pub fn from_calc(radians: CSSFloat) -> Self {
        Angle {
            value: radians,
            unit: AngleUnit::Radian,
            was_calc: true,
        }
    }
}

impl Parse for Angle {
    /// Parses an angle according to CSS-VALUES § 6.1.
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) => Angle::parse_dimension(value.value, unit),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                input.parse_nested_block(|i| CalcLengthOrPercentage::parse_angle(context, i))
            },
            _ => Err(())
        }
    }
}

impl Angle {
    #[allow(missing_docs)]
    pub fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Angle, ()> {
        let angle = match_ignore_ascii_case! { unit,
            "deg" => Angle::from_degrees(value),
            "grad" => Angle::from_gradians(value),
            "turn" => Angle::from_turns(value),
            "rad" => Angle::from_radians(value),
             _ => return Err(())
        };
        Ok(angle)
    }
    /// Parse an angle, including unitless 0 degree.
    /// Note that numbers without any AngleUnit, including unitless 0
    /// angle, should be invalid. However, some properties still accept
    /// unitless 0 angle and stores it as '0deg'. We can remove this and
    /// get back to the unified version Angle::parse once
    /// https://github.com/w3c/csswg-drafts/issues/1162 is resolved.
    pub fn parse_with_unitless(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) => Angle::parse_dimension(value.value, unit),
            Token::Number(ref value) if value.value == 0. => Ok(Angle::zero()),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                input.parse_nested_block(|i| CalcLengthOrPercentage::parse_angle(context, i))
            },
            _ => Err(())
        }
    }
}

#[allow(missing_docs)]
pub fn parse_border_radius(context: &ParserContext, input: &mut Parser) -> Result<BorderRadiusSize, ()> {
    input.try(|i| BorderRadiusSize::parse(context, i)).or_else(|_| {
        match_ignore_ascii_case! { &try!(input.expect_ident()),
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
pub fn parse_border_width(context: &ParserContext, input: &mut Parser) -> Result<Length, ()> {
    input.try(|i| Length::parse_non_negative(context, i)).or_else(|()| {
        match_ignore_ascii_case! { &try!(input.expect_ident()),
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
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<BorderWidth, ()> {
        match input.try(|i| Length::parse_non_negative(context, i)) {
            Ok(length) => Ok(BorderWidth::Width(length)),
            Err(_) => match_ignore_ascii_case! { &try!(input.expect_ident()),
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
pub struct Time {
    seconds: CSSFloat,
    was_calc: bool,
}

impl Time {
    /// Return a `<time>` value that represents `seconds` seconds.
    pub fn from_seconds(seconds: CSSFloat) -> Self {
        Time {
            seconds: seconds,
            was_calc: false,
        }
    }

    /// Returns a time that represents a duration of zero.
    pub fn zero() -> Self {
        Self::from_seconds(0.0)
    }

    /// Returns the time in fractional seconds.
    pub fn seconds(self) -> CSSFloat {
        self.seconds
    }

    /// Parses a time according to CSS-VALUES § 6.2.
    fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Time, ()> {
        let seconds = match_ignore_ascii_case! { unit,
            "s" => value,
            "ms" => value / 1000.0,
            _ => return Err(()),
        };

        Ok(Time {
            seconds: seconds,
            was_calc: false,
        })
    }

    /// Returns a `Time` value from a CSS `calc()` expression.
    pub fn from_calc(seconds: CSSFloat) -> Self {
        Time {
            seconds: seconds,
            was_calc: true,
        }
    }
}

impl ToComputedValue for Time {
    type ComputedValue = computed::Time;

    fn to_computed_value(&self, _context: &Context) -> Self::ComputedValue {
        computed::Time::from_seconds(self.seconds())
    }

    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Time {
            seconds: computed.seconds(),
            was_calc: false,
        }
    }
}

impl Parse for Time {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        match input.next() {
            Ok(Token::Dimension(ref value, ref unit)) => {
                Time::parse_dimension(value.value, &unit)
            }
            Ok(Token::Function(ref name)) if name.eq_ignore_ascii_case("calc") => {
                input.parse_nested_block(|i| CalcLengthOrPercentage::parse_time(context, i))
            }
            _ => Err(())
        }
    }
}

impl ToCss for Time {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        if self.was_calc {
            dest.write_str("calc(")?;
        }
        write!(dest, "{}s", self.seconds)?;
        if self.was_calc {
            dest.write_str(")")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct Number {
    /// The numeric value itself.
    value: CSSFloat,
    /// If this number came from a calc() expression, this tells how clamping
    /// should be done on the value.
    calc_clamping_mode: Option<AllowedNumericType>,
}

no_viewport_percentage!(Number);

impl Parse for Number {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        parse_number(context, input)
    }
}

impl Number {
    /// Returns a new number with the value `val`.
    pub fn new(val: CSSFloat) -> Self {
        Number {
            value: val,
            calc_clamping_mode: None,
        }
    }

    /// Returns the numeric value, clamped if needed.
    pub fn get(&self) -> f32 {
        self.calc_clamping_mode.map_or(self.value, |mode| mode.clamp(self.value))
    }

    #[allow(missing_docs)]
    pub fn parse_non_negative(context: &ParserContext, input: &mut Parser) -> Result<Number, ()> {
        parse_number_with_clamping_mode(context, input, AllowedNumericType::NonNegative)
    }

    #[allow(missing_docs)]
    pub fn parse_at_least_one(context: &ParserContext, input: &mut Parser) -> Result<Number, ()> {
        parse_number_with_clamping_mode(context, input, AllowedNumericType::AtLeastOne)
    }
}

impl ToComputedValue for Number {
    type ComputedValue = CSSFloat;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> CSSFloat { self.get() }

    #[inline]
    fn from_computed_value(computed: &CSSFloat) -> Self {
        Number {
            value: *computed,
            calc_clamping_mode: None,
        }
    }
}

impl ToCss for Number {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        if self.calc_clamping_mode.is_some() {
            dest.write_str("calc(")?;
        }
        self.value.to_css(dest)?;
        if self.calc_clamping_mode.is_some() {
            dest.write_str(")")?;
        }
        Ok(())
    }
}

/// <number-percentage>
/// Accepts only non-negative numbers.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum NumberOrPercentage {
    Percentage(Percentage),
    Number(Number),
}

no_viewport_percentage!(NumberOrPercentage);

impl Parse for NumberOrPercentage {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(per) = input.try(Percentage::parse_non_negative) {
            return Ok(NumberOrPercentage::Percentage(per));
        }

        Number::parse_non_negative(context, input).map(NumberOrPercentage::Number)
    }
}

impl ToCss for NumberOrPercentage {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            NumberOrPercentage::Percentage(percentage) => percentage.to_css(dest),
            NumberOrPercentage::Number(number) => number.to_css(dest),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct Opacity(Number);

no_viewport_percentage!(Opacity);

impl Parse for Opacity {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        parse_number(context, input).map(Opacity)
    }
}

impl ToComputedValue for Opacity {
    type ComputedValue = CSSFloat;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> CSSFloat {
        self.0.to_computed_value(context).min(1.0).max(0.0)
    }

    #[inline]
    fn from_computed_value(computed: &CSSFloat) -> Self {
        Opacity(Number::from_computed_value(computed))
    }
}

impl ToCss for Opacity {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        self.0.to_css(dest)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct Integer {
    value: CSSInteger,
    was_calc: bool,
}

impl Integer {
    /// Trivially constructs a new `Integer` value.
    pub fn new(val: CSSInteger) -> Self {
        Integer {
            value: val,
            was_calc: false,
        }
    }

    /// Returns the integer value associated with this value.
    pub fn value(&self) -> CSSInteger {
        self.value
    }

    /// Trivially constructs a new integer value from a `calc()` expression.
    pub fn from_calc(val: CSSInteger) -> Self {
        Integer {
            value: val,
            was_calc: true,
        }
    }
}

no_viewport_percentage!(Integer);

impl Parse for Integer {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        parse_integer(context, input)
    }
}

impl Integer {
    fn parse_with_minimum(context: &ParserContext, input: &mut Parser, min: i32) -> Result<Integer, ()> {
        match parse_integer(context, input) {
            Ok(value) if value.value() >= min => Ok(value),
            _ => Err(()),
        }
    }

    #[allow(missing_docs)]
    pub fn parse_non_negative(context: &ParserContext, input: &mut Parser) -> Result<Integer, ()> {
        Integer::parse_with_minimum(context, input, 0)
    }

    #[allow(missing_docs)]
    pub fn parse_positive(context: &ParserContext, input: &mut Parser) -> Result<Integer, ()> {
        Integer::parse_with_minimum(context, input, 1)
    }
}

impl ToComputedValue for Integer {
    type ComputedValue = i32;

    #[inline]
    fn to_computed_value(&self, _: &Context) -> i32 { self.value }

    #[inline]
    fn from_computed_value(computed: &i32) -> Self {
        Integer::new(*computed)
    }
}

impl ToCss for Integer {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        if self.was_calc {
            dest.write_str("calc(")?;
        }
        write!(dest, "{}", self.value)?;
        if self.was_calc {
            dest.write_str(")")?;
        }
        Ok(())
    }
}

/// <integer> | auto
pub type IntegerOrAuto = Either<Integer, Auto>;

impl IntegerOrAuto {
    #[allow(missing_docs)]
    pub fn parse_positive(context: &ParserContext,
                          input: &mut Parser)
                          -> Result<IntegerOrAuto, ()> {
        match IntegerOrAuto::parse(context, input) {
            Ok(Either::First(integer)) if integer.value() <= 0 => Err(()),
            result => result,
        }
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
                        .map(|color| color.to_computed_value(context))
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
    pub fn parse(context: &ParserContext, input: &mut Parser, disable_spread_and_inset: bool) -> Result<Shadow, ()> {
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
                    lengths[1] = try!(Length::parse(context, input));
                    if let Ok(value) = input.try(|i| Length::parse_non_negative(context, i)) {
                        lengths[2] = value;
                        if !disable_spread_and_inset {
                            if let Ok(value) = input.try(|i| Length::parse(context, i)) {
                                lengths[3] = value;
                            }
                        }
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

        debug_assert!(!disable_spread_and_inset || lengths[3] == Length::zero());
        Ok(Shadow {
            offset_x: lengths[0].take(),
            offset_y: lengths[1].take(),
            blur_radius: lengths[2].take(),
            spread_radius: lengths[3].take(),
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
        Ok(match_ignore_ascii_case! { &input.expect_ident()?,
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
pub type LengthOrPercentageOrNumber = Either<LengthOrPercentage, Number>;

impl LengthOrPercentageOrNumber {
    /// parse a <length-percentage> | <number> enforcing that the contents aren't negative
    pub fn parse_non_negative(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        // NB: Parse numbers before Lengths so we are consistent about how to
        // recognize and serialize "0".
        if let Ok(num) = input.try(|i| Number::parse_non_negative(context, i)) {
            return Ok(Either::Second(num))
        }

        LengthOrPercentage::parse_non_negative(context, input).map(Either::First)
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

/// <color> | auto
pub type ColorOrAuto = Either<CSSColor, Auto>;
