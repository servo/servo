/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! [Length values][length].
//!
//! [length]: https://drafts.csswg.org/css-values/#lengths

use app_units::Au;
use cssparser::{Parser, Token};
use euclid::size::Size2D;
use font_metrics::FontMetrics;
use parser::{Parse, ParserContext};
use std::{cmp, fmt, mem};
use std::ascii::AsciiExt;
use std::ops::Mul;
use style_traits::ToCss;
use style_traits::values::specified::AllowedNumericType;
use super::{Angle, Number, SimplifiedValueNode, SimplifiedSumNode, Time};
use values::{Auto, CSSFloat, Either, FONT_MEDIUM_PX, HasViewportPercentage, None_, Normal};
use values::ExtremumLength;
use values::computed::Context;

pub use super::image::{AngleOrCorner, ColorStop, EndingShape as GradientEndingShape, Gradient};
pub use super::image::{GradientKind, HorizontalDirection, Image, LengthOrKeyword, LengthOrPercentageOrKeyword};
pub use super::image::{SizeKeyword, VerticalDirection};

const AU_PER_PX: CSSFloat = 60.;
const AU_PER_IN: CSSFloat = AU_PER_PX * 96.;
const AU_PER_CM: CSSFloat = AU_PER_IN / 2.54;
const AU_PER_MM: CSSFloat = AU_PER_IN / 25.4;
const AU_PER_Q: CSSFloat = AU_PER_MM / 4.;
const AU_PER_PT: CSSFloat = AU_PER_IN / 72.;
const AU_PER_PC: CSSFloat = AU_PER_PT * 12.;

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A font relative length.
pub enum FontRelativeLength {
    /// A "em" value: https://drafts.csswg.org/css-values/#em
    Em(CSSFloat),
    /// A "ex" value: https://drafts.csswg.org/css-values/#ex
    Ex(CSSFloat),
    /// A "ch" value: https://drafts.csswg.org/css-values/#ch
    Ch(CSSFloat),
    /// A "rem" value: https://drafts.csswg.org/css-values/#rem
    Rem(CSSFloat)
}

impl ToCss for FontRelativeLength {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        match *self {
            FontRelativeLength::Em(length) => write!(dest, "{}em", length),
            FontRelativeLength::Ex(length) => write!(dest, "{}ex", length),
            FontRelativeLength::Ch(length) => write!(dest, "{}ch", length),
            FontRelativeLength::Rem(length) => write!(dest, "{}rem", length)
        }
    }
}

impl FontRelativeLength {
    /// Gets the first available font metrics from the current context's
    /// font-family list.
    pub fn find_first_available_font_metrics(context: &Context) -> Option<FontMetrics> {
        use font_metrics::FontMetricsQueryResult::*;
        if let Some(ref metrics_provider) = context.font_metrics_provider {
            for family in context.style().get_font().font_family_iter() {
                if let Available(metrics) = metrics_provider.query(family.atom()) {
                    return metrics;
                }
            }
        }

        None
    }

    /// Computes the font-relative length. We use the use_inherited flag to
    /// special-case the computation of font-size.
    pub fn to_computed_value(&self, context: &Context, use_inherited: bool) -> Au {
        let reference_font_size = if use_inherited {
            context.inherited_style().get_font().clone_font_size()
        } else {
            context.style().get_font().clone_font_size()
        };

        let root_font_size = context.style().root_font_size;
        match *self {
            FontRelativeLength::Em(length) => reference_font_size.scale_by(length),
            FontRelativeLength::Ex(length) => {
                match Self::find_first_available_font_metrics(context) {
                    Some(metrics) => metrics.x_height,
                    // https://drafts.csswg.org/css-values/#ex
                    //
                    //     In the cases where it is impossible or impractical to
                    //     determine the x-height, a value of 0.5em must be
                    //     assumed.
                    //
                    None => reference_font_size.scale_by(0.5 * length),
                }
            },
            FontRelativeLength::Ch(length) => {
                let wm = context.style().writing_mode;

                // TODO(emilio, #14144): Compute this properly once we support
                // all the relevant writing-mode related properties, this should
                // be equivalent to "is the text in the block direction?".
                let vertical = wm.is_vertical();

                match Self::find_first_available_font_metrics(context) {
                    Some(metrics) => {
                        if vertical {
                            metrics.zero_advance_measure.height
                        } else {
                            metrics.zero_advance_measure.width
                        }
                    }
                    // https://drafts.csswg.org/css-values/#ch
                    //
                    //     In the cases where it is impossible or impractical to
                    //     determine the measure of the “0” glyph, it must be
                    //     assumed to be 0.5em wide by 1em tall. Thus, the ch
                    //     unit falls back to 0.5em in the general case, and to
                    //     1em when it would be typeset upright (i.e.
                    //     writing-mode is vertical-rl or vertical-lr and
                    //     text-orientation is upright).
                    //
                    None => {
                        if vertical {
                            reference_font_size.scale_by(length)
                        } else {
                            reference_font_size.scale_by(0.5 * length)
                        }
                    }
                }
            }
            FontRelativeLength::Rem(length) => root_font_size.scale_by(length)
        }
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
/// A viewport-relative length.
///
/// https://drafts.csswg.org/css-values/#viewport-relative-lengths
pub enum ViewportPercentageLength {
    /// A vw unit: https://drafts.csswg.org/css-values/#vw
    Vw(CSSFloat),
    /// A vh unit: https://drafts.csswg.org/css-values/#vh
    Vh(CSSFloat),
    /// https://drafts.csswg.org/css-values/#vmin
    Vmin(CSSFloat),
    /// https://drafts.csswg.org/css-values/#vmax
    Vmax(CSSFloat)
}

impl HasViewportPercentage for ViewportPercentageLength {
    fn has_viewport_percentage(&self) -> bool {
        true
    }
}

impl ToCss for ViewportPercentageLength {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            ViewportPercentageLength::Vw(length) => write!(dest, "{}vw", length),
            ViewportPercentageLength::Vh(length) => write!(dest, "{}vh", length),
            ViewportPercentageLength::Vmin(length) => write!(dest, "{}vmin", length),
            ViewportPercentageLength::Vmax(length) => write!(dest, "{}vmax", length)
        }
    }
}

impl ViewportPercentageLength {
    /// Computes the given viewport-relative length for the given viewport size.
    pub fn to_computed_value(&self, viewport_size: Size2D<Au>) -> Au {
        macro_rules! to_unit {
            ($viewport_dimension:expr) => {
                $viewport_dimension.to_f32_px() / 100.0
            }
        }

        let value = match *self {
            ViewportPercentageLength::Vw(length) =>
                length * to_unit!(viewport_size.width),
            ViewportPercentageLength::Vh(length) =>
                length * to_unit!(viewport_size.height),
            ViewportPercentageLength::Vmin(length) =>
                length * to_unit!(cmp::min(viewport_size.width, viewport_size.height)),
            ViewportPercentageLength::Vmax(length) =>
                length * to_unit!(cmp::max(viewport_size.width, viewport_size.height)),
        };
        Au::from_f32_px(value)
    }
}

/// HTML5 "character width", as defined in HTML5 § 14.5.4.
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct CharacterWidth(pub i32);

impl CharacterWidth {
    /// Computes the given character width.
    pub fn to_computed_value(&self, reference_font_size: Au) -> Au {
        // This applies the *converting a character width to pixels* algorithm as specified
        // in HTML5 § 14.5.4.
        //
        // TODO(pcwalton): Find these from the font.
        let average_advance = reference_font_size.scale_by(0.5);
        let max_advance = reference_font_size;
        average_advance.scale_by(self.0 as CSSFloat - 1.0) + max_advance
    }
}

/// A `<length>` without taking `calc` expressions into account
///
/// https://drafts.csswg.org/css-values/#lengths
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum NoCalcLength {
    /// An absolute length: https://drafts.csswg.org/css-values/#absolute-length
    Absolute(Au),  // application units

    /// A font-relative length:
    ///
    /// https://drafts.csswg.org/css-values/#font-relative-lengths
    FontRelative(FontRelativeLength),

    /// A viewport-relative length.
    ///
    /// https://drafts.csswg.org/css-values/#viewport-relative-lengths
    ViewportPercentage(ViewportPercentageLength),

    /// HTML5 "character width", as defined in HTML5 § 14.5.4.
    ///
    /// This cannot be specified by the user directly and is only generated by
    /// `Stylist::synthesize_rules_for_legacy_attributes()`.
    ServoCharacterWidth(CharacterWidth),
}

impl HasViewportPercentage for NoCalcLength {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            NoCalcLength::ViewportPercentage(_) => true,
            _ => false,
        }
    }
}

impl ToCss for NoCalcLength {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            NoCalcLength::Absolute(length) => write!(dest, "{}px", length.to_f32_px()),
            NoCalcLength::FontRelative(length) => length.to_css(dest),
            NoCalcLength::ViewportPercentage(length) => length.to_css(dest),
            /* This should only be reached from style dumping code */
            NoCalcLength::ServoCharacterWidth(CharacterWidth(i)) => write!(dest, "CharWidth({})", i),
        }
    }
}

impl Mul<CSSFloat> for NoCalcLength {
    type Output = NoCalcLength;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> NoCalcLength {
        match self {
            NoCalcLength::Absolute(Au(v)) => NoCalcLength::Absolute(Au(((v as f32) * scalar) as i32)),
            NoCalcLength::FontRelative(v) => NoCalcLength::FontRelative(v * scalar),
            NoCalcLength::ViewportPercentage(v) => NoCalcLength::ViewportPercentage(v * scalar),
            NoCalcLength::ServoCharacterWidth(_) => panic!("Can't multiply ServoCharacterWidth!"),
        }
    }
}

impl NoCalcLength {
    /// https://drafts.csswg.org/css-fonts-3/#font-size-prop
    pub fn from_str(s: &str) -> Option<NoCalcLength> {
        Some(match_ignore_ascii_case! { s,
            "xx-small" => NoCalcLength::Absolute(Au::from_px(FONT_MEDIUM_PX) * 3 / 5),
            "x-small" => NoCalcLength::Absolute(Au::from_px(FONT_MEDIUM_PX) * 3 / 4),
            "small" => NoCalcLength::Absolute(Au::from_px(FONT_MEDIUM_PX) * 8 / 9),
            "medium" => NoCalcLength::Absolute(Au::from_px(FONT_MEDIUM_PX)),
            "large" => NoCalcLength::Absolute(Au::from_px(FONT_MEDIUM_PX) * 6 / 5),
            "x-large" => NoCalcLength::Absolute(Au::from_px(FONT_MEDIUM_PX) * 3 / 2),
            "xx-large" => NoCalcLength::Absolute(Au::from_px(FONT_MEDIUM_PX) * 2),

            // https://github.com/servo/servo/issues/3423#issuecomment-56321664
            "smaller" => NoCalcLength::FontRelative(FontRelativeLength::Em(0.85)),
            "larger" => NoCalcLength::FontRelative(FontRelativeLength::Em(1.2)),
            _ => return None
        })
    }

    /// https://drafts.csswg.org/css-fonts-3/#font-size-prop
    pub fn from_font_size_int(i: u8) -> Self {
        let au = match i {
            0 | 1 => Au::from_px(FONT_MEDIUM_PX) * 3 / 4,
            2 => Au::from_px(FONT_MEDIUM_PX) * 8 / 9,
            3 => Au::from_px(FONT_MEDIUM_PX),
            4 => Au::from_px(FONT_MEDIUM_PX) * 6 / 5,
            5 => Au::from_px(FONT_MEDIUM_PX) * 3 / 2,
            6 => Au::from_px(FONT_MEDIUM_PX) * 2,
            _ => Au::from_px(FONT_MEDIUM_PX) * 3,
        };
        NoCalcLength::Absolute(au)
    }

    /// Parse a given absolute or relative dimension.
    pub fn parse_dimension(value: CSSFloat, unit: &str) -> Result<NoCalcLength, ()> {
        match_ignore_ascii_case! { unit,
            "px" => Ok(NoCalcLength::Absolute(Au((value * AU_PER_PX) as i32))),
            "in" => Ok(NoCalcLength::Absolute(Au((value * AU_PER_IN) as i32))),
            "cm" => Ok(NoCalcLength::Absolute(Au((value * AU_PER_CM) as i32))),
            "mm" => Ok(NoCalcLength::Absolute(Au((value * AU_PER_MM) as i32))),
            "q" => Ok(NoCalcLength::Absolute(Au((value * AU_PER_Q) as i32))),
            "pt" => Ok(NoCalcLength::Absolute(Au((value * AU_PER_PT) as i32))),
            "pc" => Ok(NoCalcLength::Absolute(Au((value * AU_PER_PC) as i32))),
            // font-relative
            "em" => Ok(NoCalcLength::FontRelative(FontRelativeLength::Em(value))),
            "ex" => Ok(NoCalcLength::FontRelative(FontRelativeLength::Ex(value))),
            "ch" => Ok(NoCalcLength::FontRelative(FontRelativeLength::Ch(value))),
            "rem" => Ok(NoCalcLength::FontRelative(FontRelativeLength::Rem(value))),
            // viewport percentages
            "vw" => Ok(NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vw(value))),
            "vh" => Ok(NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vh(value))),
            "vmin" => Ok(NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vmin(value))),
            "vmax" => Ok(NoCalcLength::ViewportPercentage(ViewportPercentageLength::Vmax(value))),
            _ => Err(())
        }
    }

    #[inline]
    /// Returns a `zero` length.
    pub fn zero() -> NoCalcLength {
        NoCalcLength::Absolute(Au(0))
    }

    #[inline]
    /// Checks whether the length value is zero.
    pub fn is_zero(&self) -> bool {
        *self == NoCalcLength::Absolute(Au(0))
    }

    /// Get an absolute length from a px value.
    #[inline]
    pub fn from_px(px_value: CSSFloat) -> NoCalcLength {
        NoCalcLength::Absolute(Au((px_value * AU_PER_PX) as i32))
    }
}

/// An extension to `NoCalcLength` to parse `calc` expressions.
/// This is commonly used for the `<length>` values.
///
/// https://drafts.csswg.org/css-values/#lengths
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Length {
    /// The internal length type that cannot parse `calc`
    NoCalc(NoCalcLength),
    /// A calc expression.
    ///
    /// https://drafts.csswg.org/css-values/#calc-notation
    ///
    /// TODO(emilio): We have more `Calc` variants around, we should only use
    /// one.
    Calc(Box<CalcLengthOrPercentage>, AllowedNumericType),
}

impl From<NoCalcLength> for Length {
    #[inline]
    fn from(len: NoCalcLength) -> Self {
        Length::NoCalc(len)
    }
}

impl HasViewportPercentage for Length {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            Length::NoCalc(ref inner) => inner.has_viewport_percentage(),
            Length::Calc(ref calc, _) => calc.has_viewport_percentage(),
        }
    }
}

impl ToCss for Length {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Length::NoCalc(ref inner) => inner.to_css(dest),
            Length::Calc(ref calc, _) => calc.to_css(dest),
        }
    }
}

impl Mul<CSSFloat> for Length {
    type Output = Length;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> Length {
        match self {
            Length::NoCalc(inner) => Length::NoCalc(inner * scalar),
            Length::Calc(..) => panic!("Can't multiply Calc!"),
        }
    }
}

impl Mul<CSSFloat> for FontRelativeLength {
    type Output = FontRelativeLength;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> FontRelativeLength {
        match self {
            FontRelativeLength::Em(v) => FontRelativeLength::Em(v * scalar),
            FontRelativeLength::Ex(v) => FontRelativeLength::Ex(v * scalar),
            FontRelativeLength::Ch(v) => FontRelativeLength::Ch(v * scalar),
            FontRelativeLength::Rem(v) => FontRelativeLength::Rem(v * scalar),
        }
    }
}

impl Mul<CSSFloat> for ViewportPercentageLength {
    type Output = ViewportPercentageLength;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> ViewportPercentageLength {
        match self {
            ViewportPercentageLength::Vw(v) => ViewportPercentageLength::Vw(v * scalar),
            ViewportPercentageLength::Vh(v) => ViewportPercentageLength::Vh(v * scalar),
            ViewportPercentageLength::Vmin(v) => ViewportPercentageLength::Vmin(v * scalar),
            ViewportPercentageLength::Vmax(v) => ViewportPercentageLength::Vmax(v * scalar),
        }
    }
}

impl Length {
    #[inline]
    /// Returns a `zero` length.
    pub fn zero() -> Length {
        Length::NoCalc(NoCalcLength::zero())
    }

    /// https://drafts.csswg.org/css-fonts-3/#font-size-prop
    pub fn from_str(s: &str) -> Option<Length> {
        NoCalcLength::from_str(s).map(Length::NoCalc)
    }

    /// Parse a given absolute or relative dimension.
    pub fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Length, ()> {
        NoCalcLength::parse_dimension(value, unit).map(Length::NoCalc)
    }

    /// https://drafts.csswg.org/css-fonts-3/#font-size-prop
    pub fn from_font_size_int(i: u8) -> Self {
        Length::NoCalc(NoCalcLength::from_font_size_int(i))
    }

    #[inline]
    fn parse_internal(input: &mut Parser, context: AllowedNumericType) -> Result<Length, ()> {
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                Length::parse_dimension(value.value, unit),
            Token::Number(ref value) if value.value == 0. => Ok(Length::zero()),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") =>
                input.parse_nested_block(|input| {
                    CalcLengthOrPercentage::parse_length(input, context)
                }),
            _ => Err(())
        }
    }

    /// Parse a non-negative length
    #[inline]
    pub fn parse_non_negative(input: &mut Parser) -> Result<Length, ()> {
        Length::parse_internal(input, AllowedNumericType::NonNegative)
    }

    /// Get an absolute length from a px value.
    #[inline]
    pub fn from_px(px_value: CSSFloat) -> Length {
        Length::NoCalc(NoCalcLength::from_px(px_value))
    }

    /// Extract inner length without a clone, replacing it with a 0 Au
    ///
    /// Use when you need to move out of a length array without cloning
    #[inline]
    pub fn take(&mut self) -> Self {
        mem::replace(self, Length::zero())
    }
}

impl Parse for Length {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        Length::parse_internal(input, AllowedNumericType::All)
    }
}

impl Either<Length, Normal> {
    #[inline]
    #[allow(missing_docs)]
    pub fn parse_non_negative_length(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if input.try(|input| Normal::parse(context, input)).is_ok() {
            return Ok(Either::Second(Normal));
        }
        Length::parse_internal(input, AllowedNumericType::NonNegative).map(Either::First)
    }
}

impl Either<Length, Auto> {
    #[inline]
    #[allow(missing_docs)]
    pub fn parse_non_negative_length(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if input.try(|input| Auto::parse(context, input)).is_ok() {
            return Ok(Either::Second(Auto));
        }
        Length::parse_internal(input, AllowedNumericType::NonNegative).map(Either::First)
    }
}

/// A calc sum expression node.
#[derive(Clone, Debug)]
pub struct CalcSumNode {
    /// The products of this node.
    pub products: Vec<CalcProductNode>,
}

/// A calc product expression node.
#[derive(Clone, Debug)]
pub struct CalcProductNode {
    /// The values inside this product node.
    values: Vec<CalcValueNode>
}

/// A value inside a `Calc` expression.
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub enum CalcValueNode {
    Length(NoCalcLength),
    Angle(Angle),
    Time(Time),
    Percentage(CSSFloat),
    Number(CSSFloat),
    Sum(Box<CalcSumNode>),
}

#[derive(Clone, Copy, PartialEq)]
#[allow(missing_docs)]
pub enum CalcUnit {
    Number,
    Integer,
    Length,
    LengthOrPercentage,
    Angle,
    Time,
}

#[derive(Clone, PartialEq, Copy, Debug, Default)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct CalcLengthOrPercentage {
    pub absolute: Option<Au>,
    pub vw: Option<CSSFloat>,
    pub vh: Option<CSSFloat>,
    pub vmin: Option<CSSFloat>,
    pub vmax: Option<CSSFloat>,
    pub em: Option<CSSFloat>,
    pub ex: Option<CSSFloat>,
    pub ch: Option<CSSFloat>,
    pub rem: Option<CSSFloat>,
    pub percentage: Option<CSSFloat>,
}

impl CalcLengthOrPercentage {
    /// Parse a calc sum node.
    pub fn parse_sum(input: &mut Parser, expected_unit: CalcUnit) -> Result<CalcSumNode, ()> {
        let mut products = Vec::new();
        products.push(try!(CalcLengthOrPercentage::parse_product(input, expected_unit)));

        while let Ok(token) = input.next() {
            match token {
                Token::Delim('+') => {
                    products.push(try!(CalcLengthOrPercentage::parse_product(input, expected_unit)));
                }
                Token::Delim('-') => {
                    let mut right = try!(CalcLengthOrPercentage::parse_product(input, expected_unit));
                    right.values.push(CalcValueNode::Number(-1.));
                    products.push(right);
                }
                _ => return Err(())
            }
        }

        Ok(CalcSumNode { products: products })
    }

    fn parse_product(input: &mut Parser, expected_unit: CalcUnit) -> Result<CalcProductNode, ()> {
        let mut values = Vec::new();
        values.push(try!(CalcLengthOrPercentage::parse_value(input, expected_unit)));

        loop {
            let position = input.position();
            match input.next() {
                Ok(Token::Delim('*')) => {
                    values.push(try!(CalcLengthOrPercentage::parse_value(input, expected_unit)));
                }
                Ok(Token::Delim('/')) if expected_unit != CalcUnit::Integer => {
                    if let Ok(Token::Number(ref value)) = input.next() {
                        if value.value == 0. {
                            return Err(());
                        }
                        values.push(CalcValueNode::Number(1. / value.value));
                    } else {
                        return Err(());
                    }
                }
                _ => {
                    input.reset(position);
                    break
                }
            }
        }

        Ok(CalcProductNode { values: values })
    }

    fn parse_value(input: &mut Parser, expected_unit: CalcUnit) -> Result<CalcValueNode, ()> {
        match (try!(input.next()), expected_unit) {
            (Token::Number(ref value), _) => Ok(CalcValueNode::Number(value.value)),
            (Token::Dimension(ref value, ref unit), CalcUnit::Length) |
            (Token::Dimension(ref value, ref unit), CalcUnit::LengthOrPercentage) => {
                NoCalcLength::parse_dimension(value.value, unit).map(CalcValueNode::Length)
            }
            (Token::Dimension(ref value, ref unit), CalcUnit::Angle) => {
                Angle::parse_dimension(value.value, unit).map(CalcValueNode::Angle)
            }
            (Token::Dimension(ref value, ref unit), CalcUnit::Time) => {
                Time::parse_dimension(value.value, unit).map(CalcValueNode::Time)
            }
            (Token::Percentage(ref value), CalcUnit::LengthOrPercentage) =>
                Ok(CalcValueNode::Percentage(value.unit_value)),
            (Token::ParenthesisBlock, _) => {
                input.parse_nested_block(|i| CalcLengthOrPercentage::parse_sum(i, expected_unit))
                     .map(|result| CalcValueNode::Sum(Box::new(result)))
            },
            _ => Err(())
        }
    }

    fn simplify_value_to_number(node: &CalcValueNode) -> Option<CSSFloat> {
        match *node {
            CalcValueNode::Number(number) => Some(number),
            CalcValueNode::Sum(ref sum) => CalcLengthOrPercentage::simplify_sum_to_number(sum),
            _ => None
        }
    }

    fn simplify_sum_to_number(node: &CalcSumNode) -> Option<CSSFloat> {
        let mut sum = 0.;
        for ref product in &node.products {
            match CalcLengthOrPercentage::simplify_product_to_number(product) {
                Some(number) => sum += number,
                _ => return None
            }
        }
        Some(sum)
    }

    fn simplify_product_to_number(node: &CalcProductNode) -> Option<CSSFloat> {
        let mut product = 1.;
        for ref value in &node.values {
            match CalcLengthOrPercentage::simplify_value_to_number(value) {
                Some(number) => product *= number,
                _ => return None
            }
        }
        Some(product)
    }

    fn simplify_products_in_sum(node: &CalcSumNode) -> Result<SimplifiedValueNode, ()> {
        let mut simplified = Vec::new();
        for product in &node.products {
            match try!(CalcLengthOrPercentage::simplify_product(product)) {
                SimplifiedValueNode::Sum(ref sum) => simplified.extend_from_slice(&sum.values),
                val => simplified.push(val),
            }
        }

        if simplified.len() == 1 {
            Ok(simplified[0].clone())
        } else {
            Ok(SimplifiedValueNode::Sum(Box::new(SimplifiedSumNode { values: simplified })))
        }
    }

    #[allow(missing_docs)]
    pub fn simplify_product(node: &CalcProductNode) -> Result<SimplifiedValueNode, ()> {
        let mut multiplier = 1.;
        let mut node_with_unit = None;
        for node in &node.values {
            match CalcLengthOrPercentage::simplify_value_to_number(&node) {
                Some(number) => multiplier *= number,
                _ if node_with_unit.is_none() => {
                    node_with_unit = Some(match *node {
                        CalcValueNode::Sum(ref sum) =>
                            try!(CalcLengthOrPercentage::simplify_products_in_sum(sum)),
                        CalcValueNode::Length(ref l) => SimplifiedValueNode::Length(l.clone()),
                        CalcValueNode::Angle(a) => SimplifiedValueNode::Angle(a),
                        CalcValueNode::Time(t) => SimplifiedValueNode::Time(t),
                        CalcValueNode::Percentage(p) => SimplifiedValueNode::Percentage(p),
                        _ => unreachable!("Numbers should have been handled by simplify_value_to_nubmer")
                    })
                },
                _ => return Err(()),
            }
        }

        match node_with_unit {
            None => Ok(SimplifiedValueNode::Number(multiplier)),
            Some(ref value) => Ok(value * multiplier)
        }
    }

    fn parse_length(input: &mut Parser,
                    context: AllowedNumericType) -> Result<Length, ()> {
        CalcLengthOrPercentage::parse(input, CalcUnit::Length).map(|calc| {
            Length::Calc(Box::new(calc), context)
        })
    }

    fn parse_length_or_percentage(input: &mut Parser) -> Result<CalcLengthOrPercentage, ()> {
        CalcLengthOrPercentage::parse(input, CalcUnit::LengthOrPercentage)
    }

    #[allow(missing_docs)]
    pub fn parse(input: &mut Parser,
                 expected_unit: CalcUnit) -> Result<CalcLengthOrPercentage, ()> {
        let ast = try!(CalcLengthOrPercentage::parse_sum(input, expected_unit));

        let mut simplified = Vec::new();
        for ref node in ast.products {
            match try!(CalcLengthOrPercentage::simplify_product(node)) {
                SimplifiedValueNode::Sum(sum) => simplified.extend_from_slice(&sum.values),
                value => simplified.push(value),
            }
        }

        let mut absolute = None;
        let mut vw = None;
        let mut vh = None;
        let mut vmax = None;
        let mut vmin = None;
        let mut em = None;
        let mut ex = None;
        let mut ch = None;
        let mut rem = None;
        let mut percentage = None;

        for value in simplified {
            match value {
                SimplifiedValueNode::Percentage(p) =>
                    percentage = Some(percentage.unwrap_or(0.) + p),
                SimplifiedValueNode::Length(NoCalcLength::Absolute(Au(au))) =>
                    absolute = Some(absolute.unwrap_or(0) + au),
                SimplifiedValueNode::Length(NoCalcLength::ViewportPercentage(v)) =>
                    match v {
                        ViewportPercentageLength::Vw(val) =>
                            vw = Some(vw.unwrap_or(0.) + val),
                        ViewportPercentageLength::Vh(val) =>
                            vh = Some(vh.unwrap_or(0.) + val),
                        ViewportPercentageLength::Vmin(val) =>
                            vmin = Some(vmin.unwrap_or(0.) + val),
                        ViewportPercentageLength::Vmax(val) =>
                            vmax = Some(vmax.unwrap_or(0.) + val),
                    },
                SimplifiedValueNode::Length(NoCalcLength::FontRelative(f)) =>
                    match f {
                        FontRelativeLength::Em(val) =>
                            em = Some(em.unwrap_or(0.) + val),
                        FontRelativeLength::Ex(val) =>
                            ex = Some(ex.unwrap_or(0.) + val),
                        FontRelativeLength::Ch(val) =>
                            ch = Some(ch.unwrap_or(0.) + val),
                        FontRelativeLength::Rem(val) =>
                            rem = Some(rem.unwrap_or(0.) + val),
                    },
                // TODO Add support for top level number in calc(). See servo/servo#14421.
                _ => return Err(()),
            }
        }

        Ok(CalcLengthOrPercentage {
            absolute: absolute.map(Au),
            vw: vw,
            vh: vh,
            vmax: vmax,
            vmin: vmin,
            em: em,
            ex: ex,
            ch: ch,
            rem: rem,
            percentage: percentage,
        })
    }

    #[allow(missing_docs)]
    pub fn parse_time(input: &mut Parser) -> Result<Time, ()> {
        let ast = try!(CalcLengthOrPercentage::parse_sum(input, CalcUnit::Time));

        let mut simplified = Vec::new();
        for ref node in ast.products {
            match try!(CalcLengthOrPercentage::simplify_product(node)) {
                SimplifiedValueNode::Sum(sum) => simplified.extend_from_slice(&sum.values),
                value => simplified.push(value),
            }
        }

        let mut time = None;

        for value in simplified {
            match value {
                SimplifiedValueNode::Time(Time(val)) =>
                    time = Some(time.unwrap_or(0.) + val),
                _ => return Err(()),
            }
        }

        match time {
            Some(time) => Ok(Time(time)),
            _ => Err(())
        }
    }

    #[allow(missing_docs)]
    pub fn parse_angle(input: &mut Parser) -> Result<Angle, ()> {
        let ast = try!(CalcLengthOrPercentage::parse_sum(input, CalcUnit::Angle));

        let mut simplified = Vec::new();
        for ref node in ast.products {
            match try!(CalcLengthOrPercentage::simplify_product(node)) {
                SimplifiedValueNode::Sum(sum) => simplified.extend_from_slice(&sum.values),
                value => simplified.push(value),
            }
        }

        let mut angle = None;
        let mut number = None;

        for value in simplified {
            match value {
                SimplifiedValueNode::Angle(Angle(val)) =>
                    angle = Some(angle.unwrap_or(0.) + val),
                SimplifiedValueNode::Number(val) => number = Some(number.unwrap_or(0.) + val),
                _ => unreachable!()
            }
        }

        match (angle, number) {
            (Some(angle), None) => Ok(Angle(angle)),
            (None, Some(value)) if value == 0. => Ok(Angle(0.)),
            _ => Err(())
        }
    }
}

impl HasViewportPercentage for CalcLengthOrPercentage {
    fn has_viewport_percentage(&self) -> bool {
        self.vw.is_some() || self.vh.is_some() ||
            self.vmin.is_some() || self.vmax.is_some()
    }
}

impl ToCss for CalcLengthOrPercentage {
    #[allow(unused_assignments)]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut first_value = true;
        macro_rules! first_value_check {
            () => {
                if !first_value {
                    try!(dest.write_str(" + "));
                } else {
                    first_value = false;
                }
            };
        }

        macro_rules! serialize {
            ( $( $val:ident ),* ) => {
                $(
                    if let Some(val) = self.$val {
                        first_value_check!();
                        try!(val.to_css(dest));
                        try!(dest.write_str(stringify!($val)));
                    }
                )*
            };
        }

        try!(write!(dest, "calc("));

        serialize!(ch, em, ex, rem, vh, vmax, vmin, vw);
        if let Some(val) = self.absolute {
            first_value_check!();
            try!(val.to_css(dest));
        }

        if let Some(val) = self.percentage {
            first_value_check!();
            try!(write!(dest, "{}%", val * 100.));
        }

        write!(dest, ")")
     }
}

/// A percentage value.
///
/// [0 .. 100%] maps to [0.0 .. 1.0]
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Percentage(pub CSSFloat);

impl ToCss for Percentage {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        write!(dest, "{}%", self.0 * 100.)
    }
}

impl Parse for Percentage {
    #[inline]
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let context = AllowedNumericType::All;
        match try!(input.next()) {
            Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                Ok(Percentage(value.unit_value)),
            _ => Err(())
        }
    }
}

/// A length or a percentage value.
///
/// TODO(emilio): Does this make any sense vs. CalcLengthOrPercentage?
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum LengthOrPercentage {
    Length(NoCalcLength),
    Percentage(Percentage),
    Calc(Box<CalcLengthOrPercentage>),
}

impl From<Length> for LengthOrPercentage {
    fn from(len: Length) -> LengthOrPercentage {
        match len {
            Length::NoCalc(l) => LengthOrPercentage::Length(l),
            Length::Calc(l, _) => LengthOrPercentage::Calc(l),
        }
    }
}

impl From<NoCalcLength> for LengthOrPercentage {
    #[inline]
    fn from(len: NoCalcLength) -> Self {
        LengthOrPercentage::Length(len)
    }
}

impl From<Percentage> for LengthOrPercentage {
    #[inline]
    fn from(pc: Percentage) -> Self {
        LengthOrPercentage::Percentage(pc)
    }
}

impl HasViewportPercentage for LengthOrPercentage {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            LengthOrPercentage::Length(ref length) => length.has_viewport_percentage(),
            LengthOrPercentage::Calc(ref calc) => calc.has_viewport_percentage(),
            _ => false
        }
    }
}

impl ToCss for LengthOrPercentage {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrPercentage::Length(ref length) => length.to_css(dest),
            LengthOrPercentage::Percentage(percentage) => percentage.to_css(dest),
            LengthOrPercentage::Calc(ref calc) => calc.to_css(dest),
        }
    }
}
impl LengthOrPercentage {
    /// Returns a `zero` length.
    pub fn zero() -> LengthOrPercentage {
        LengthOrPercentage::Length(NoCalcLength::zero())
    }

    fn parse_internal(input: &mut Parser, context: AllowedNumericType)
                      -> Result<LengthOrPercentage, ()>
    {
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                NoCalcLength::parse_dimension(value.value, unit).map(LengthOrPercentage::Length),
            Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                Ok(LengthOrPercentage::Percentage(Percentage(value.unit_value))),
            Token::Number(ref value) if value.value == 0. =>
                Ok(LengthOrPercentage::zero()),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                let calc = try!(input.parse_nested_block(CalcLengthOrPercentage::parse_length_or_percentage));
                Ok(LengthOrPercentage::Calc(Box::new(calc)))
            },
            _ => Err(())
        }
    }

    /// Parse a non-negative length.
    #[inline]
    pub fn parse_non_negative(input: &mut Parser) -> Result<LengthOrPercentage, ()> {
        LengthOrPercentage::parse_internal(input, AllowedNumericType::NonNegative)
    }

    /// Parse a length, treating dimensionless numbers as pixels
    ///
    /// https://www.w3.org/TR/SVG2/types.html#presentation-attribute-css-value
    pub fn parse_numbers_are_pixels(input: &mut Parser) -> Result<LengthOrPercentage, ()> {
        if let Ok(lop) = input.try(|i| Self::parse_internal(i, AllowedNumericType::All)) {
            Ok(lop)
        } else {
            let num = input.expect_number()?;
            Ok(LengthOrPercentage::Length(NoCalcLength::Absolute(Au((AU_PER_PX * num) as i32))))
        }
    }

    /// Parse a non-negative length, treating dimensionless numbers as pixels
    ///
    /// This is nonstandard behavior used by Firefox for SVG
    pub fn parse_numbers_are_pixels_non_negative(input: &mut Parser) -> Result<LengthOrPercentage, ()> {
        if let Ok(lop) = input.try(|i| Self::parse_internal(i, AllowedNumericType::NonNegative)) {
            Ok(lop)
        } else {
            let num = input.expect_number()?;
            if num >= 0. {
                Ok(LengthOrPercentage::Length(NoCalcLength::Absolute(Au((AU_PER_PX * num) as i32))))
            } else {
                Err(())
            }
        }
    }

    /// Extract value from ref without a clone, replacing it with a 0 Au
    ///
    /// Use when you need to move out of a length array without cloning
    #[inline]
    pub fn take(&mut self) -> Self {
        mem::replace(self, LengthOrPercentage::zero())
    }
}

impl Parse for LengthOrPercentage {
    #[inline]
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        LengthOrPercentage::parse_internal(input, AllowedNumericType::All)
    }
}

/// TODO(emilio): Do the Length and Percentage variants make any sense with
/// CalcLengthOrPercentage?
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum LengthOrPercentageOrAuto {
    Length(NoCalcLength),
    Percentage(Percentage),
    Auto,
    Calc(Box<CalcLengthOrPercentage>),
}


impl From<NoCalcLength> for LengthOrPercentageOrAuto {
    #[inline]
    fn from(len: NoCalcLength) -> Self {
        LengthOrPercentageOrAuto::Length(len)
    }
}

impl From<Percentage> for LengthOrPercentageOrAuto {
    #[inline]
    fn from(pc: Percentage) -> Self {
        LengthOrPercentageOrAuto::Percentage(pc)
    }
}

impl HasViewportPercentage for LengthOrPercentageOrAuto {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            LengthOrPercentageOrAuto::Length(ref length) => length.has_viewport_percentage(),
            LengthOrPercentageOrAuto::Calc(ref calc) => calc.has_viewport_percentage(),
            _ => false
        }
    }
}

impl ToCss for LengthOrPercentageOrAuto {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrPercentageOrAuto::Length(ref length) => length.to_css(dest),
            LengthOrPercentageOrAuto::Percentage(percentage) => percentage.to_css(dest),
            LengthOrPercentageOrAuto::Auto => dest.write_str("auto"),
            LengthOrPercentageOrAuto::Calc(ref calc) => calc.to_css(dest),
        }
    }
}

impl LengthOrPercentageOrAuto {
    fn parse_internal(input: &mut Parser, context: AllowedNumericType)
                      -> Result<LengthOrPercentageOrAuto, ()>
    {
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                NoCalcLength::parse_dimension(value.value, unit).map(LengthOrPercentageOrAuto::Length),
            Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                Ok(LengthOrPercentageOrAuto::Percentage(Percentage(value.unit_value))),
            Token::Number(ref value) if value.value == 0. =>
                Ok(LengthOrPercentageOrAuto::Length(NoCalcLength::zero())),
            Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") =>
                Ok(LengthOrPercentageOrAuto::Auto),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                let calc = try!(input.parse_nested_block(CalcLengthOrPercentage::parse_length_or_percentage));
                Ok(LengthOrPercentageOrAuto::Calc(Box::new(calc)))
            },
            _ => Err(())
        }
    }

    /// Parse a non-negative length, percentage, or auto.
    #[inline]
    pub fn parse_non_negative(input: &mut Parser) -> Result<LengthOrPercentageOrAuto, ()> {
        LengthOrPercentageOrAuto::parse_internal(input, AllowedNumericType::NonNegative)
    }
}

impl Parse for LengthOrPercentageOrAuto {
    #[inline]
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        LengthOrPercentageOrAuto::parse_internal(input, AllowedNumericType::All)
    }
}

/// TODO(emilio): Do the Length and Percentage variants make any sense with
/// CalcLengthOrPercentage?
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum LengthOrPercentageOrNone {
    Length(NoCalcLength),
    Percentage(Percentage),
    Calc(Box<CalcLengthOrPercentage>),
    None,
}

impl HasViewportPercentage for LengthOrPercentageOrNone {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            LengthOrPercentageOrNone::Length(ref length) => length.has_viewport_percentage(),
            LengthOrPercentageOrNone::Calc(ref calc) => calc.has_viewport_percentage(),
            _ => false
        }
    }
}

impl ToCss for LengthOrPercentageOrNone {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrPercentageOrNone::Length(ref length) => length.to_css(dest),
            LengthOrPercentageOrNone::Percentage(ref percentage) => percentage.to_css(dest),
            LengthOrPercentageOrNone::Calc(ref calc) => calc.to_css(dest),
            LengthOrPercentageOrNone::None => dest.write_str("none"),
        }
    }
}
impl LengthOrPercentageOrNone {
    fn parse_internal(input: &mut Parser, context: AllowedNumericType)
                      -> Result<LengthOrPercentageOrNone, ()>
    {
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                NoCalcLength::parse_dimension(value.value, unit).map(LengthOrPercentageOrNone::Length),
            Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                Ok(LengthOrPercentageOrNone::Percentage(Percentage(value.unit_value))),
            Token::Number(ref value) if value.value == 0. =>
                Ok(LengthOrPercentageOrNone::Length(NoCalcLength::zero())),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                let calc = try!(input.parse_nested_block(CalcLengthOrPercentage::parse_length_or_percentage));
                Ok(LengthOrPercentageOrNone::Calc(Box::new(calc)))
            },
            Token::Ident(ref value) if value.eq_ignore_ascii_case("none") =>
                Ok(LengthOrPercentageOrNone::None),
            _ => Err(())
        }
    }
    /// Parse a non-negative LengthOrPercentageOrNone.
    #[inline]
    pub fn parse_non_negative(input: &mut Parser) -> Result<LengthOrPercentageOrNone, ()> {
        LengthOrPercentageOrNone::parse_internal(input, AllowedNumericType::NonNegative)
    }
}

impl Parse for LengthOrPercentageOrNone {
    #[inline]
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        LengthOrPercentageOrNone::parse_internal(input, AllowedNumericType::All)
    }
}

/// Either a `<length>` or the `none` keyword.
pub type LengthOrNone = Either<Length, None_>;

/// Either a `<length>` or the `normal` keyword.
pub type LengthOrNormal = Either<Length, Normal>;

/// Either a `<length>` or the `auto` keyword.
pub type LengthOrAuto = Either<Length, Auto>;

/// Either a `<length>` or a `<percentage>` or the `auto` keyword or the
/// `content` keyword.
///
/// TODO(emilio): Do the Length and Percentage variants make any sense with
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LengthOrPercentageOrAutoOrContent {
    /// A `<length>`.
    Length(NoCalcLength),
    /// A percentage.
    Percentage(Percentage),
    /// A `calc` node.
    Calc(Box<CalcLengthOrPercentage>),
    /// The `auto` keyword.
    Auto,
    /// The `content` keyword.
    Content
}

impl HasViewportPercentage for LengthOrPercentageOrAutoOrContent {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            LengthOrPercentageOrAutoOrContent::Length(ref length) => length.has_viewport_percentage(),
            LengthOrPercentageOrAutoOrContent::Calc(ref calc) => calc.has_viewport_percentage(),
            _ => false
        }
    }
}

impl ToCss for LengthOrPercentageOrAutoOrContent {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrPercentageOrAutoOrContent::Length(ref len) => len.to_css(dest),
            LengthOrPercentageOrAutoOrContent::Percentage(perc) => perc.to_css(dest),
            LengthOrPercentageOrAutoOrContent::Auto => dest.write_str("auto"),
            LengthOrPercentageOrAutoOrContent::Content => dest.write_str("content"),
            LengthOrPercentageOrAutoOrContent::Calc(ref calc) => calc.to_css(dest),
        }
    }
}

impl Parse for LengthOrPercentageOrAutoOrContent {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let context = AllowedNumericType::NonNegative;
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                NoCalcLength::parse_dimension(value.value, unit).map(LengthOrPercentageOrAutoOrContent::Length),
            Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                Ok(LengthOrPercentageOrAutoOrContent::Percentage(Percentage(value.unit_value))),
            Token::Number(ref value) if value.value == 0. =>
                Ok(LengthOrPercentageOrAutoOrContent::Length(NoCalcLength::zero())),
            Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") =>
                Ok(LengthOrPercentageOrAutoOrContent::Auto),
            Token::Ident(ref value) if value.eq_ignore_ascii_case("content") =>
                Ok(LengthOrPercentageOrAutoOrContent::Content),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                let calc = try!(input.parse_nested_block(CalcLengthOrPercentage::parse_length_or_percentage));
                Ok(LengthOrPercentageOrAutoOrContent::Calc(Box::new(calc)))
            },
            _ => Err(())
        }
    }
}

/// Either a `<length>` or a `<number>`.
pub type LengthOrNumber = Either<Length, Number>;

impl LengthOrNumber {
    /// Parse a non-negative LengthOrNumber.
    pub fn parse_non_negative(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        // We try to parse as a Number first because, for cases like LengthOrNumber,
        // we want "0" to be parsed as a plain Number rather than a Length (0px); this
        // matches the behaviour of all major browsers
        if let Ok(v) = input.try(Number::parse_non_negative) {
            Ok(Either::Second(v))
        } else {
            Length::parse_non_negative(input).map(Either::First)
        }
    }
}

/// A value suitable for a `min-width` or `min-height` property.
/// Unlike `max-width` or `max-height` properties, a MinLength can be
/// `auto`, and cannot be `none`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum MinLength {
    LengthOrPercentage(LengthOrPercentage),
    Auto,
    ExtremumLength(ExtremumLength),
}

impl HasViewportPercentage for MinLength {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            MinLength::LengthOrPercentage(ref lop) => lop.has_viewport_percentage(),
            _ => false
        }
    }
}

impl ToCss for MinLength {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            MinLength::LengthOrPercentage(ref lop) =>
                lop.to_css(dest),
            MinLength::Auto =>
                dest.write_str("auto"),
            MinLength::ExtremumLength(ref ext) =>
                ext.to_css(dest),
        }
    }
}

impl Parse for MinLength {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        input.try(ExtremumLength::parse).map(MinLength::ExtremumLength)
            .or_else(|()| input.try(LengthOrPercentage::parse_non_negative).map(MinLength::LengthOrPercentage))
            .or_else(|()| {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    "auto" =>
                        Ok(MinLength::Auto),
                    _ => Err(())
                }
            })
    }
}

/// A value suitable for a `max-width` or `max-height` property.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub enum MaxLength {
    LengthOrPercentage(LengthOrPercentage),
    None,
    ExtremumLength(ExtremumLength),
}

impl HasViewportPercentage for MaxLength {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            MaxLength::LengthOrPercentage(ref lop) => lop.has_viewport_percentage(),
            _ => false
        }
    }
}

impl ToCss for MaxLength {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            MaxLength::LengthOrPercentage(ref lop) =>
                lop.to_css(dest),
            MaxLength::None =>
                dest.write_str("none"),
            MaxLength::ExtremumLength(ref ext) =>
                ext.to_css(dest),
        }
    }
}

impl Parse for MaxLength {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        input.try(ExtremumLength::parse).map(MaxLength::ExtremumLength)
            .or_else(|()| input.try(LengthOrPercentage::parse_non_negative).map(MaxLength::LengthOrPercentage))
            .or_else(|()| {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    "none" =>
                        Ok(MaxLength::None),
                    _ => Err(())
                }
            })
    }
}
