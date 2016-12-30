/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use app_units::Au;
use cssparser::{Parser, Token};
use font_metrics::FontMetrics;
use parser::{Parse, ParserContext};
use std::ascii::AsciiExt;
use std::cmp;
use std::fmt;
use std::mem;
use std::ops::Mul;
use style_traits::ToCss;
use style_traits::values::specified::AllowedNumericType;
use super::{Angle, Number, SimplifiedValueNode, SimplifiedSumNode, Time};
use values::{Auto, Either, None_, Normal};
use values::{CSSFloat, FONT_MEDIUM_PX, HasViewportPercentage, NoViewportPercentage};
use values::computed::Context;

pub use super::image::{AngleOrCorner, ColorStop, EndingShape as GradientEndingShape, Gradient};
pub use super::image::{GradientKind, HorizontalDirection, Image, LengthOrKeyword, LengthOrPercentageOrKeyword};
pub use super::image::{SizeKeyword, VerticalDirection};

const NUM_FONT_UNITS: usize = 4;
const NUM_VIEWPORT_UNITS: usize = 4;

#[repr(u8)]
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum FontUnit {
    Em,
    Ex,
    Ch,
    Rem,
}

impl FontUnit {
    pub fn from_usize(i: usize) -> FontUnit {
        assert!(i <= NUM_FONT_UNITS, "cannot convert {} to FontUnit", i);
        unsafe { mem::transmute(i as u8) }
    }
}

impl ToCss for FontUnit {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let unit = match *self {
            FontUnit::Em => "em",
            FontUnit::Ex => "ex",
            FontUnit::Ch => "ch",
            FontUnit::Rem => "rem",
        };

        dest.write_str(unit)
    }
}

#[repr(u8)]
#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum ViewportUnit {
    Vw,
    Vh,
    Vmin,
    Vmax,
}

impl ViewportUnit {
    pub fn from_usize(i: usize) -> ViewportUnit {
        assert!(i <= NUM_VIEWPORT_UNITS, "cannot convert {} to ViewportUnit", i);
        unsafe { mem::transmute(i as u8) }
    }
}

impl ToCss for ViewportUnit {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let unit = match *self {
            ViewportUnit::Vw => "vw",
            ViewportUnit::Vh => "vh",
            ViewportUnit::Vmin => "vmin",
            ViewportUnit::Vmax => "vmax",
        };

        dest.write_str(unit)
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct FontRelativeLength {
    value: CSSFloat,
    unit: FontUnit,
}

impl ToCss for FontRelativeLength {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(self.value.to_css(dest));
        self.unit.to_css(dest)
    }
}

impl FontRelativeLength {
    pub fn new(value: CSSFloat, unit: FontUnit) -> FontRelativeLength {
        FontRelativeLength {
            value: value,
            unit: unit,
        }
    }

    pub fn parse_dimension(value: CSSFloat, unit: &str) -> Result<FontRelativeLength, ()> {
        let unit = match_ignore_ascii_case! { unit,
            "em" => FontUnit::Em,
            "ex" => FontUnit::Ex,
            "ch" => FontUnit::Ch,
            "rem" => FontUnit::Rem,
            _ => return Err(())
        };

        Ok(FontRelativeLength { value: value, unit: unit })
    }

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

    pub fn to_computed_value(&self, context: &Context) -> Au {
        self.get_computed_value(context, false)
    }

    pub fn to_computed_value_inherited(&self, context: &Context) -> Au {
        self.get_computed_value(context, true)
    }

    // NB: The use_inherited flag is used to special-case the computation of
    // font-family.
    fn get_computed_value(&self, context: &Context, use_inherited: bool) -> Au {
        let reference_font_size = if use_inherited {
            context.inherited_style().get_font().clone_font_size()
        } else {
            context.style().get_font().clone_font_size()
        };

        let root_font_size = context.style().root_font_size;
        match self.unit {
            FontUnit::Em => reference_font_size.scale_by(self.value),
            FontUnit::Ex => {
                match Self::find_first_available_font_metrics(context) {
                    Some(metrics) => metrics.x_height,
                    // https://drafts.csswg.org/css-values/#ex
                    //
                    //     In the cases where it is impossible or impractical to
                    //     determine the x-height, a value of 0.5em must be
                    //     assumed.
                    //
                    None => reference_font_size.scale_by(0.5 * self.value),
                }
            },
            FontUnit::Ch => {
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
                            reference_font_size.scale_by(self.value)
                        } else {
                            reference_font_size.scale_by(0.5 * self.value)
                        }
                    }
                }
            }
            FontUnit::Rem => root_font_size.scale_by(self.value)
        }
    }
}

impl NoViewportPercentage for FontRelativeLength {}

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct ViewportPercentageLength {
    value: CSSFloat,
    unit: ViewportUnit,
}

impl HasViewportPercentage for ViewportPercentageLength {
    fn has_viewport_percentage(&self) -> bool {
        true
    }
}

impl ToCss for ViewportPercentageLength {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(self.value.to_css(dest));
        self.unit.to_css(dest)
    }
}

impl ViewportPercentageLength {
    pub fn new(value: CSSFloat, unit: ViewportUnit) -> ViewportPercentageLength {
        ViewportPercentageLength {
            value: value,
            unit: unit,
        }
    }

    pub fn parse_dimension(value: CSSFloat, unit: &str) -> Result<ViewportPercentageLength, ()> {
        let unit = match_ignore_ascii_case!{ unit,
            "vw" => ViewportUnit::Vw,
            "vh" => ViewportUnit::Vh,
            "vmin" => ViewportUnit::Vmin,
            "vmax" => ViewportUnit::Vmax,
            _ => return Err(())
        };

        Ok(ViewportPercentageLength { value: value, unit: unit })
    }

    pub fn to_computed_value(&self, context: &Context) -> Au {
        let viewport_size = context.viewport_size();
        let scale_factor = match self.unit {
            ViewportUnit::Vw => viewport_size.width,
            ViewportUnit::Vh => viewport_size.height,
            ViewportUnit::Vmin => cmp::min(viewport_size.width, viewport_size.height),
            ViewportUnit::Vmax => cmp::max(viewport_size.width, viewport_size.height),
        };

        Au::from_f32_px(self.value * (scale_factor.to_f32_px() / 100.0))
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct CharacterWidth(pub i32);

impl CharacterWidth {
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

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Length {
    Absolute(Au),  // application units
    FontRelative(FontRelativeLength),
    ViewportPercentage(ViewportPercentageLength),

    /// HTML5 "character width", as defined in HTML5 § 14.5.4.
    ///
    /// This cannot be specified by the user directly and is only generated by
    /// `Stylist::synthesize_rules_for_legacy_attributes()`.
    ServoCharacterWidth(CharacterWidth),

    Calc(CalcLengthOrPercentage, AllowedNumericType),
}

impl HasViewportPercentage for Length {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            Length::ViewportPercentage(_) => true,
            Length::Calc(ref calc, _) => calc.has_viewport_percentage(),
            _ => false
        }
    }
}

impl ToCss for Length {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            Length::Absolute(length) => write!(dest, "{}px", length.to_f32_px()),
            Length::FontRelative(length) => length.to_css(dest),
            Length::ViewportPercentage(length) => length.to_css(dest),
            Length::Calc(ref calc, _) => calc.to_css(dest),
            /* This should only be reached from style dumping code */
            Length::ServoCharacterWidth(CharacterWidth(i)) => write!(dest, "CharWidth({})", i),
        }
    }
}

impl Mul<CSSFloat> for Length {
    type Output = Length;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> Length {
        match self {
            Length::Absolute(Au(v)) => Length::Absolute(Au(((v as f32) * scalar) as i32)),
            Length::FontRelative(v) => Length::FontRelative(v * scalar),
            Length::ViewportPercentage(v) => Length::ViewportPercentage(v * scalar),
            Length::Calc(..) => panic!("Can't multiply Calc!"),
            Length::ServoCharacterWidth(_) => panic!("Can't multiply ServoCharacterWidth!"),
        }
    }
}

impl Mul<CSSFloat> for FontRelativeLength {
    type Output = FontRelativeLength;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> FontRelativeLength {
        FontRelativeLength {
            value: self.value * scalar,
            unit: self.unit,
        }
    }
}

impl Mul<CSSFloat> for ViewportPercentageLength {
    type Output = ViewportPercentageLength;

    #[inline]
    fn mul(self, scalar: CSSFloat) -> ViewportPercentageLength {
        ViewportPercentageLength {
            value: self.value * scalar,
            unit: self.unit,
        }
    }
}

const AU_PER_PX: CSSFloat = 60.;
const AU_PER_IN: CSSFloat = AU_PER_PX * 96.;
const AU_PER_CM: CSSFloat = AU_PER_IN / 2.54;
const AU_PER_MM: CSSFloat = AU_PER_IN / 25.4;
const AU_PER_Q: CSSFloat = AU_PER_MM / 4.;
const AU_PER_PT: CSSFloat = AU_PER_IN / 72.;
const AU_PER_PC: CSSFloat = AU_PER_PT * 12.;

impl Length {
    // https://drafts.csswg.org/css-fonts-3/#font-size-prop
    pub fn from_str(s: &str) -> Option<Length> {
        Some(match_ignore_ascii_case! { s,
            "xx-small" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX) * 3 / 5),
            "x-small" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX) * 3 / 4),
            "small" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX) * 8 / 9),
            "medium" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX)),
            "large" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX) * 6 / 5),
            "x-large" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX) * 3 / 2),
            "xx-large" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX) * 2),

            // https://github.com/servo/servo/issues/3423#issuecomment-56321664
            "smaller" => Length::FontRelative(FontRelativeLength::new(0.85, FontUnit::Em)),
            "larger" => Length::FontRelative(FontRelativeLength::new(1.2, FontUnit::Em)),
            _ => return None
        })
    }

    #[inline]
    fn parse_internal(input: &mut Parser, context: AllowedNumericType) -> Result<Length, ()> {
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                Length::parse_dimension(value.value, unit),
            Token::Number(ref value) if value.value == 0. =>
                Ok(Length::Absolute(Au(0))),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") =>
                input.parse_nested_block(|input| {
                    CalcLengthOrPercentage::parse_length(input, context)
                }),
            _ => Err(())
        }
    }

    pub fn parse_non_negative(input: &mut Parser) -> Result<Length, ()> {
        Length::parse_internal(input, AllowedNumericType::NonNegative)
    }

    pub fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Length, ()> {
        if let Ok(f) = FontRelativeLength::parse_dimension(value, unit) {
            Ok(Length::FontRelative(f))
        } else if let Ok(v) = ViewportPercentageLength::parse_dimension(value, unit) {
            Ok(Length::ViewportPercentage(v))
        } else {
            let scale_factor = match_ignore_ascii_case! { unit,
                "px" => AU_PER_PX,
                "in" => AU_PER_IN,
                "cm" => AU_PER_CM,
                "mm" => AU_PER_MM,
                "q" => AU_PER_Q,
                "pt" => AU_PER_PT,
                "pc" => AU_PER_PC,
                _ => return Err(())
            };

            Ok(Length::Absolute(Au((value * scale_factor) as i32)))
        }
    }

    #[inline]
    pub fn from_px(px_value: CSSFloat) -> Length {
        Length::Absolute(Au((px_value * AU_PER_PX) as i32))
    }
}

impl Parse for Length {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        Length::parse_internal(input, AllowedNumericType::All)
    }
}

impl<T> Either<Length, T> {
    #[inline]
    pub fn parse_non_negative_length(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        Length::parse_internal(input, AllowedNumericType::NonNegative).map(Either::First)
    }
}

#[derive(Clone, Debug)]
pub struct CalcSumNode {
    pub products: Vec<CalcProductNode>,
}

#[derive(Clone, Debug)]
pub struct CalcProductNode {
    values: Vec<CalcValueNode>
}

#[derive(Clone, Debug)]
pub enum CalcValueNode {
    Length(Length),
    Angle(Angle),
    Time(Time),
    Percentage(CSSFloat),
    Number(CSSFloat),
    Sum(Box<CalcSumNode>),
}

#[derive(Clone, Copy, PartialEq)]
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
pub struct CalcLengthOrPercentage {
    pub absolute: Option<Au>,
    /// Values with font-relative units. They're stored in an array with the units
    /// denoting the index of the value (as per the order of variants in `FontUnit`)
    pub font_values: [Option<CSSFloat>; NUM_FONT_UNITS],
    /// Same as `font_values` (only difference is that this is for `ViewportUnit`)
    pub viewport_values: [Option<CSSFloat>; NUM_VIEWPORT_UNITS],
    pub percentage: Option<Percentage>,
}

impl CalcLengthOrPercentage {
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
                Length::parse_dimension(value.value, unit).map(CalcValueNode::Length)
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
                        CalcValueNode::Length(l) => SimplifiedValueNode::Length(l),
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
            Length::Calc(calc, context)
        })
    }

    fn parse_length_or_percentage(input: &mut Parser) -> Result<CalcLengthOrPercentage, ()> {
        CalcLengthOrPercentage::parse(input, CalcUnit::LengthOrPercentage)
    }

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
        let mut font_values = [None; NUM_FONT_UNITS];
        let mut viewport_values = [None; NUM_VIEWPORT_UNITS];
        let mut percentage = None;

        for value in simplified {
            match value {
                SimplifiedValueNode::Percentage(p) =>
                    percentage = Some(percentage.unwrap_or(0.) + p),
                SimplifiedValueNode::Length(Length::Absolute(Au(au))) =>
                    absolute = Some(absolute.unwrap_or(0) + au),
                SimplifiedValueNode::Length(Length::ViewportPercentage(v)) => {
                    let idx = v.unit as usize;
                    viewport_values[idx] = Some(viewport_values[idx].unwrap_or(0.) + v.value);
                },
                SimplifiedValueNode::Length(Length::FontRelative(f)) => {
                    let idx = f.unit as usize;
                    font_values[idx] = Some(font_values[idx].unwrap_or(0.) + f.value);
                },
                // TODO Add support for top level number in calc(). See servo/servo#14421.
                _ => return Err(()),
            }
        }

        Ok(CalcLengthOrPercentage {
            absolute: absolute.map(Au),
            font_values: font_values,
            viewport_values: viewport_values,
            percentage: percentage.map(Percentage),
        })
    }

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
        self.viewport_values.iter().any(|v| v.is_some())
    }
}

impl ToCss for CalcLengthOrPercentage {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        let mut first_value = true;
        let count = self.font_values.iter()
                                    .filter(|&v| v.is_some())
                                    .zip(self.viewport_values.iter().filter(|&v| v.is_some()))
                                    .count();
        assert!(count > 0);
        if count > 1 {
            try!(dest.write_str("calc("));
        }

        macro_rules! serialize {
            ($array: expr, $f: expr) => {
                for (i, value) in $array.iter().enumerate() {
                    if let Some(val) = *value {
                        if !first_value {
                            try!(dest.write_str(" + "));
                        } else {
                            first_value = false;
                        }

                        try!(val.to_css(dest));
                        try!($f(i).to_css(dest));
                    }
                }
            }
        }

        serialize!(self.font_values, FontUnit::from_usize);
        serialize!(self.viewport_values, ViewportUnit::from_usize);

        if count > 1 {
            try!(dest.write_str(")"));
        }

        Ok(())
     }
}

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Percentage(pub CSSFloat); // [0 .. 100%] maps to [0.0 .. 1.0]

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

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LengthOrPercentage {
    Length(Length),
    Percentage(Percentage),
    Calc(CalcLengthOrPercentage),
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
            LengthOrPercentage::Length(length) => length.to_css(dest),
            LengthOrPercentage::Percentage(percentage) => percentage.to_css(dest),
            LengthOrPercentage::Calc(calc) => calc.to_css(dest),
        }
    }
}
impl LengthOrPercentage {
    pub fn zero() -> LengthOrPercentage {
        LengthOrPercentage::Length(Length::Absolute(Au(0)))
    }

    fn parse_internal(input: &mut Parser, context: AllowedNumericType)
                      -> Result<LengthOrPercentage, ()>
    {
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                Length::parse_dimension(value.value, unit).map(LengthOrPercentage::Length),
            Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                Ok(LengthOrPercentage::Percentage(Percentage(value.unit_value))),
            Token::Number(ref value) if value.value == 0. =>
                Ok(LengthOrPercentage::Length(Length::Absolute(Au(0)))),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                let calc = try!(input.parse_nested_block(CalcLengthOrPercentage::parse_length_or_percentage));
                Ok(LengthOrPercentage::Calc(calc))
            },
            _ => Err(())
        }
    }

    #[inline]
    pub fn parse_non_negative(input: &mut Parser) -> Result<LengthOrPercentage, ()> {
        LengthOrPercentage::parse_internal(input, AllowedNumericType::NonNegative)
    }
}

impl Parse for LengthOrPercentage {
    #[inline]
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        LengthOrPercentage::parse_internal(input, AllowedNumericType::All)
    }
}

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LengthOrPercentageOrAuto {
    Length(Length),
    Percentage(Percentage),
    Auto,
    Calc(CalcLengthOrPercentage),
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
            LengthOrPercentageOrAuto::Length(length) => length.to_css(dest),
            LengthOrPercentageOrAuto::Percentage(percentage) => percentage.to_css(dest),
            LengthOrPercentageOrAuto::Auto => dest.write_str("auto"),
            LengthOrPercentageOrAuto::Calc(calc) => calc.to_css(dest),
        }
    }
}

impl LengthOrPercentageOrAuto {
    fn parse_internal(input: &mut Parser, context: AllowedNumericType)
                      -> Result<LengthOrPercentageOrAuto, ()>
    {
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                Length::parse_dimension(value.value, unit).map(LengthOrPercentageOrAuto::Length),
            Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                Ok(LengthOrPercentageOrAuto::Percentage(Percentage(value.unit_value))),
            Token::Number(ref value) if value.value == 0. =>
                Ok(LengthOrPercentageOrAuto::Length(Length::Absolute(Au(0)))),
            Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") =>
                Ok(LengthOrPercentageOrAuto::Auto),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                let calc = try!(input.parse_nested_block(CalcLengthOrPercentage::parse_length_or_percentage));
                Ok(LengthOrPercentageOrAuto::Calc(calc))
            },
            _ => Err(())
        }
    }
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

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LengthOrPercentageOrNone {
    Length(Length),
    Percentage(Percentage),
    Calc(CalcLengthOrPercentage),
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
                Length::parse_dimension(value.value, unit).map(LengthOrPercentageOrNone::Length),
            Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                Ok(LengthOrPercentageOrNone::Percentage(Percentage(value.unit_value))),
            Token::Number(ref value) if value.value == 0. =>
                Ok(LengthOrPercentageOrNone::Length(Length::Absolute(Au(0)))),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                let calc = try!(input.parse_nested_block(CalcLengthOrPercentage::parse_length_or_percentage));
                Ok(LengthOrPercentageOrNone::Calc(calc))
            },
            Token::Ident(ref value) if value.eq_ignore_ascii_case("none") =>
                Ok(LengthOrPercentageOrNone::None),
            _ => Err(())
        }
    }
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

pub type LengthOrNone = Either<Length, None_>;

pub type LengthOrNormal = Either<Length, Normal>;

pub type LengthOrAuto = Either<Length, Auto>;

#[derive(Clone, PartialEq, Copy, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum LengthOrPercentageOrAutoOrContent {
    Length(Length),
    Percentage(Percentage),
    Calc(CalcLengthOrPercentage),
    Auto,
    Content
}

impl HasViewportPercentage for LengthOrPercentageOrAutoOrContent {
    fn has_viewport_percentage(&self) -> bool {
        match *self {
            LengthOrPercentageOrAutoOrContent::Length(length) => length.has_viewport_percentage(),
            LengthOrPercentageOrAutoOrContent::Calc(ref calc) => calc.has_viewport_percentage(),
            _ => false
        }
    }
}

impl ToCss for LengthOrPercentageOrAutoOrContent {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match *self {
            LengthOrPercentageOrAutoOrContent::Length(len) => len.to_css(dest),
            LengthOrPercentageOrAutoOrContent::Percentage(perc) => perc.to_css(dest),
            LengthOrPercentageOrAutoOrContent::Auto => dest.write_str("auto"),
            LengthOrPercentageOrAutoOrContent::Content => dest.write_str("content"),
            LengthOrPercentageOrAutoOrContent::Calc(calc) => calc.to_css(dest),
        }
    }
}

impl Parse for LengthOrPercentageOrAutoOrContent {
    fn parse(_context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        let context = AllowedNumericType::NonNegative;
        match try!(input.next()) {
            Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                Length::parse_dimension(value.value, unit).map(LengthOrPercentageOrAutoOrContent::Length),
            Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                Ok(LengthOrPercentageOrAutoOrContent::Percentage(Percentage(value.unit_value))),
            Token::Number(ref value) if value.value == 0. =>
                Ok(LengthOrPercentageOrAutoOrContent::Length(Length::Absolute(Au(0)))),
            Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") =>
                Ok(LengthOrPercentageOrAutoOrContent::Auto),
            Token::Ident(ref value) if value.eq_ignore_ascii_case("content") =>
                Ok(LengthOrPercentageOrAutoOrContent::Content),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                let calc = try!(input.parse_nested_block(CalcLengthOrPercentage::parse_length_or_percentage));
                Ok(LengthOrPercentageOrAutoOrContent::Calc(calc))
            },
            _ => Err(())
        }
    }
}

pub type LengthOrNumber = Either<Length, Number>;

impl LengthOrNumber {
    pub fn parse_non_negative(input: &mut Parser) -> Result<Self, ()> {
        if let Ok(v) = input.try(Length::parse_non_negative) {
            Ok(Either::First(v))
        } else {
            Number::parse_non_negative(input).map(Either::Second)
        }
    }
}
