/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! [Calc expressions][calc].
//!
//! [calc]: https://drafts.csswg.org/css-values/#calc-notation

use crate::parser::ParserContext;
use crate::values::computed;
use crate::values::specified::length::ViewportPercentageLength;
use crate::values::specified::length::{AbsoluteLength, FontRelativeLength, NoCalcLength};
use crate::values::specified::{Angle, Time};
use crate::values::{CSSFloat, CSSInteger};
use cssparser::{AngleOrNumber, CowRcStr, NumberOrPercentage, Parser, Token};
use std::fmt::{self, Write};
use style_traits::values::specified::AllowedNumericType;
use style_traits::{CssWriter, ParseError, SpecifiedValueInfo, StyleParseErrorKind, ToCss};

/// The name of the mathematical function that we're parsing.
#[derive(Debug, Copy, Clone)]
pub enum MathFunction {
    /// `calc()`: https://drafts.csswg.org/css-values-4/#funcdef-calc
    Calc,
    /// `min()`: https://drafts.csswg.org/css-values-4/#funcdef-min
    Min,
    /// `max()`: https://drafts.csswg.org/css-values-4/#funcdef-max
    Max,
    /// `clamp()`: https://drafts.csswg.org/css-values-4/#funcdef-clamp
    Clamp,
}

/// A node inside a `Calc` expression's AST.
#[derive(Clone, Debug)]
pub enum CalcNode {
    /// `<length>`
    Length(NoCalcLength),
    /// `<angle>`
    Angle(Angle),
    /// `<time>`
    Time(Time),
    /// `<percentage>`
    Percentage(CSSFloat),
    /// `<number>`
    Number(CSSFloat),
    /// An expression of the form `x + y`
    Sum(Box<CalcNode>, Box<CalcNode>),
    /// An expression of the form `x - y`
    Sub(Box<CalcNode>, Box<CalcNode>),
    /// An expression of the form `x * y`
    Mul(Box<CalcNode>, Box<CalcNode>),
    /// An expression of the form `x / y`
    Div(Box<CalcNode>, Box<CalcNode>),
    /// A `min()` function.
    Min(Box<[CalcNode]>),
    /// A `max()` function.
    Max(Box<[CalcNode]>),
    /// A `clamp()` function.
    Clamp {
        /// The minimum value.
        min: Box<CalcNode>,
        /// The central value.
        center: Box<CalcNode>,
        /// The maximum value.
        max: Box<CalcNode>,
    },
}

/// An expected unit we intend to parse within a `calc()` expression.
///
/// This is used as a hint for the parser to fast-reject invalid expressions.
#[derive(Clone, Copy, PartialEq)]
pub enum CalcUnit {
    /// `<number>`
    Number,
    /// `<length>`
    Length,
    /// `<percentage>`
    Percentage,
    /// `<length> | <percentage>`
    LengthPercentage,
    /// `<angle>`
    Angle,
    /// `<time>`
    Time,
}

/// A struct to hold a simplified `<length>` or `<percentage>` expression.
///
/// In some cases, e.g. DOMMatrix, we support calc(), but reject all the
/// relative lengths, and to_computed_pixel_length_without_context() handles
/// this case. Therefore, if you want to add a new field, please make sure this
/// function work properly.
#[derive(Clone, Copy, Debug, Default, MallocSizeOf, PartialEq, ToShmem)]
#[allow(missing_docs)]
pub struct CalcLengthPercentage {
    pub clamping_mode: AllowedNumericType,
    pub absolute: Option<AbsoluteLength>,
    pub vw: Option<CSSFloat>,
    pub vh: Option<CSSFloat>,
    pub vmin: Option<CSSFloat>,
    pub vmax: Option<CSSFloat>,
    pub em: Option<CSSFloat>,
    pub ex: Option<CSSFloat>,
    pub ch: Option<CSSFloat>,
    pub rem: Option<CSSFloat>,
    pub percentage: Option<computed::Percentage>,
}

impl ToCss for CalcLengthPercentage {
    /// <https://drafts.csswg.org/css-values/#calc-serialize>
    ///
    /// FIXME(emilio): Should this simplify away zeros?
    #[allow(unused_assignments)]
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        use num_traits::Zero;

        let mut first_value = true;
        macro_rules! first_value_check {
            ($val:expr) => {
                if !first_value {
                    dest.write_str(if $val < Zero::zero() { " - " } else { " + " })?;
                } else if $val < Zero::zero() {
                    dest.write_str("-")?;
                }
                first_value = false;
            };
        }

        macro_rules! serialize {
            ( $( $val:ident ),* ) => {
                $(
                    if let Some(val) = self.$val {
                        first_value_check!(val);
                        val.abs().to_css(dest)?;
                        dest.write_str(stringify!($val))?;
                    }
                )*
            };
        }

        macro_rules! serialize_abs {
            ( $( $val:ident ),+ ) => {
                $(
                    if let Some(AbsoluteLength::$val(v)) = self.absolute {
                        first_value_check!(v);
                        AbsoluteLength::$val(v.abs()).to_css(dest)?;
                    }
                )+
            };
        }

        dest.write_str("calc(")?;

        // NOTE(emilio): Percentages first because of web-compat problems, see:
        // https://github.com/w3c/csswg-drafts/issues/1731
        if let Some(val) = self.percentage {
            first_value_check!(val.0);
            val.abs().to_css(dest)?;
        }

        // NOTE(emilio): The order here it's very intentional, and alphabetic
        // per the spec linked above.
        serialize!(ch);
        serialize_abs!(Cm);
        serialize!(em, ex);
        serialize_abs!(In, Mm, Pc, Pt, Px, Q);
        serialize!(rem, vh, vmax, vmin, vw);

        dest.write_str(")")
    }
}

impl SpecifiedValueInfo for CalcLengthPercentage {}

macro_rules! impl_generic_to_type {
    ($self:ident, $self_variant:ident, $to_self:ident, $to_float:ident, $from_float:path) => {{
        if let Self::$self_variant(ref v) = *$self {
            return Ok(v.clone());
        }

        Ok(match *$self {
            Self::Sub(ref a, ref b) => $from_float(a.$to_self()?.$to_float() - b.$to_self()?.$to_float()),
            Self::Sum(ref a, ref b) => $from_float(a.$to_self()?.$to_float() + b.$to_self()?.$to_float()),
            Self::Mul(ref a, ref b) => match a.$to_self() {
                Ok(lhs) => {
                    let rhs = b.to_number()?;
                    $from_float(lhs.$to_float() * rhs)
                },
                Err(..) => {
                    let lhs = a.to_number()?;
                    let rhs = b.$to_self()?;
                    $from_float(lhs * rhs.$to_float())
                },
            },
            Self::Div(ref a, ref b) => {
                let lhs = a.$to_self()?;
                let rhs = b.to_number()?;
                if rhs == 0. {
                    return Err(());
                }
                $from_float(lhs.$to_float() / rhs)
            },
            Self::Clamp { ref min, ref center, ref max } => {
                let min = min.$to_self()?;
                let center = center.$to_self()?;
                let max = max.$to_self()?;

                // Equivalent to cmp::max(min, cmp::min(center, max))
                //
                // But preserving units when appropriate.
                let mut result = center;
                if result.$to_float() > max.$to_float() {
                    result = max;
                }
                if result.$to_float() < min.$to_float() {
                    result = min;
                }
                result
            },
            Self::Min(ref nodes) => {
                let mut min = nodes[0].$to_self()?;
                for node in nodes.iter().skip(1) {
                    let candidate = node.$to_self()?;
                    if candidate.$to_float() < min.$to_float() {
                        min = candidate;
                    }
                }
                min
            },
            Self::Max(ref nodes) => {
                let mut max = nodes[0].$to_self()?;
                for node in nodes.iter().skip(1) {
                    let candidate = node.$to_self()?;
                    if candidate.$to_float() > max.$to_float() {
                        max = candidate;
                    }
                }
                max
            },
            Self::Length(..) |
            Self::Angle(..) |
            Self::Time(..) |
            Self::Percentage(..) |
            Self::Number(..) => return Err(()),
        })
    }}
}

impl CalcNode {
    /// Tries to parse a single element in the expression, that is, a
    /// `<length>`, `<angle>`, `<time>`, `<percentage>`, according to
    /// `expected_unit`.
    ///
    /// May return a "complex" `CalcNode`, in the presence of a parenthesized
    /// expression, for example.
    fn parse_one<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        expected_unit: CalcUnit,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        match (input.next()?, expected_unit) {
            (&Token::Number { value, .. }, _) => Ok(CalcNode::Number(value)),
            (
                &Token::Dimension {
                    value, ref unit, ..
                },
                CalcUnit::Length,
            ) |
            (
                &Token::Dimension {
                    value, ref unit, ..
                },
                CalcUnit::LengthPercentage,
            ) => NoCalcLength::parse_dimension(context, value, unit)
                .map(CalcNode::Length)
                .map_err(|()| location.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
            (
                &Token::Dimension {
                    value, ref unit, ..
                },
                CalcUnit::Angle,
            ) => {
                Angle::parse_dimension(value, unit, /* from_calc = */ true)
                    .map(CalcNode::Angle)
                    .map_err(|()| location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            },
            (
                &Token::Dimension {
                    value, ref unit, ..
                },
                CalcUnit::Time,
            ) => {
                Time::parse_dimension(value, unit, /* from_calc = */ true)
                    .map(CalcNode::Time)
                    .map_err(|()| location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
            },
            (&Token::Percentage { unit_value, .. }, CalcUnit::LengthPercentage) |
            (&Token::Percentage { unit_value, .. }, CalcUnit::Percentage) => {
                Ok(CalcNode::Percentage(unit_value))
            },
            (&Token::ParenthesisBlock, _) => {
                input.parse_nested_block(|input| {
                    CalcNode::parse_argument(context, input, expected_unit)
                })
            },
            (&Token::Function(ref name), _) => {
                let function = CalcNode::math_function(name, location)?;
                CalcNode::parse(context, input, function, expected_unit)
            },
            (t, _) => Err(location.new_unexpected_token_error(t.clone())),
        }
    }

    /// Parse a top-level `calc` expression, with all nested sub-expressions.
    ///
    /// This is in charge of parsing, for example, `2 + 3 * 100%`.
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        function: MathFunction,
        expected_unit: CalcUnit,
    ) -> Result<Self, ParseError<'i>> {
        // TODO: Do something different based on the function name. In
        // particular, for non-calc function we need to take a list of
        // comma-separated arguments and such.
        input.parse_nested_block(|input| {
            match function {
                MathFunction::Calc => Self::parse_argument(context, input, expected_unit),
                MathFunction::Clamp => {
                    let min = Self::parse_argument(context, input, expected_unit)?;
                    input.expect_comma()?;
                    let center = Self::parse_argument(context, input, expected_unit)?;
                    input.expect_comma()?;
                    let max = Self::parse_argument(context, input, expected_unit)?;
                    Ok(Self::Clamp {
                        min: Box::new(min),
                        center: Box::new(center),
                        max: Box::new(max),
                    })
                },
                MathFunction::Min |
                MathFunction::Max => {
                    // TODO(emilio): The common case for parse_comma_separated
                    // is just one element, but for min / max is two, really...
                    //
                    // Consider adding an API to cssparser to specify the
                    // initial vector capacity?
                    let arguments = input.parse_comma_separated(|input| {
                        Self::parse_argument(context, input, expected_unit)
                    })?.into_boxed_slice();

                    Ok(match function {
                        MathFunction::Min => Self::Min(arguments),
                        MathFunction::Max => Self::Max(arguments),
                        _ => unreachable!(),
                    })
                }
            }
        })
    }

    fn parse_argument<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        expected_unit: CalcUnit,
    ) -> Result<Self, ParseError<'i>> {
        let mut root = Self::parse_product(context, input, expected_unit)?;

        loop {
            let start = input.state();
            match input.next_including_whitespace() {
                Ok(&Token::WhiteSpace(_)) => {
                    if input.is_exhausted() {
                        break; // allow trailing whitespace
                    }
                    match *input.next()? {
                        Token::Delim('+') => {
                            let rhs = Self::parse_product(context, input, expected_unit)?;
                            let new_root = CalcNode::Sum(Box::new(root), Box::new(rhs));
                            root = new_root;
                        },
                        Token::Delim('-') => {
                            let rhs = Self::parse_product(context, input, expected_unit)?;
                            let new_root = CalcNode::Sub(Box::new(root), Box::new(rhs));
                            root = new_root;
                        },
                        ref t => {
                            let t = t.clone();
                            return Err(input.new_unexpected_token_error(t));
                        },
                    }
                },
                _ => {
                    input.reset(&start);
                    break;
                },
            }
        }

        Ok(root)
    }

    /// Parse a top-level `calc` expression, and all the products that may
    /// follow, and stop as soon as a non-product expression is found.
    ///
    /// This should parse correctly:
    ///
    /// * `2`
    /// * `2 * 2`
    /// * `2 * 2 + 2` (but will leave the `+ 2` unparsed).
    ///
    fn parse_product<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        expected_unit: CalcUnit,
    ) -> Result<Self, ParseError<'i>> {
        let mut root = Self::parse_one(context, input, expected_unit)?;

        loop {
            let start = input.state();
            match input.next() {
                Ok(&Token::Delim('*')) => {
                    let rhs = Self::parse_one(context, input, expected_unit)?;
                    let new_root = CalcNode::Mul(Box::new(root), Box::new(rhs));
                    root = new_root;
                },
                Ok(&Token::Delim('/')) => {
                    let rhs = Self::parse_one(context, input, expected_unit)?;
                    let new_root = CalcNode::Div(Box::new(root), Box::new(rhs));
                    root = new_root;
                },
                _ => {
                    input.reset(&start);
                    break;
                },
            }
        }

        Ok(root)
    }

    /// Tries to simplify this expression into a `<length>` or `<percentage`>
    /// value.
    fn to_length_or_percentage(
        &self,
        clamping_mode: AllowedNumericType,
    ) -> Result<CalcLengthPercentage, ()> {
        let mut ret = CalcLengthPercentage {
            clamping_mode,
            ..Default::default()
        };
        self.add_length_or_percentage_to(&mut ret, 1.0)?;
        Ok(ret)
    }

    /// Puts this `<length>` or `<percentage>` into `ret`, or error.
    ///
    /// `factor` is the sign or multiplicative factor to account for the sign
    /// (this allows adding and substracting into the return value).
    fn add_length_or_percentage_to(
        &self,
        ret: &mut CalcLengthPercentage,
        factor: CSSFloat,
    ) -> Result<(), ()> {
        match *self {
            CalcNode::Percentage(pct) => {
                ret.percentage = Some(computed::Percentage(
                    ret.percentage.map_or(0., |p| p.0) + pct * factor,
                ));
            },
            CalcNode::Length(ref l) => match *l {
                NoCalcLength::Absolute(abs) => {
                    ret.absolute = Some(match ret.absolute {
                        Some(value) => value + abs * factor,
                        None => abs * factor,
                    });
                },
                NoCalcLength::FontRelative(rel) => match rel {
                    FontRelativeLength::Em(em) => {
                        ret.em = Some(ret.em.unwrap_or(0.) + em * factor);
                    },
                    FontRelativeLength::Ex(ex) => {
                        ret.ex = Some(ret.ex.unwrap_or(0.) + ex * factor);
                    },
                    FontRelativeLength::Ch(ch) => {
                        ret.ch = Some(ret.ch.unwrap_or(0.) + ch * factor);
                    },
                    FontRelativeLength::Rem(rem) => {
                        ret.rem = Some(ret.rem.unwrap_or(0.) + rem * factor);
                    },
                },
                NoCalcLength::ViewportPercentage(rel) => match rel {
                    ViewportPercentageLength::Vh(vh) => {
                        ret.vh = Some(ret.vh.unwrap_or(0.) + vh * factor)
                    },
                    ViewportPercentageLength::Vw(vw) => {
                        ret.vw = Some(ret.vw.unwrap_or(0.) + vw * factor)
                    },
                    ViewportPercentageLength::Vmax(vmax) => {
                        ret.vmax = Some(ret.vmax.unwrap_or(0.) + vmax * factor)
                    },
                    ViewportPercentageLength::Vmin(vmin) => {
                        ret.vmin = Some(ret.vmin.unwrap_or(0.) + vmin * factor)
                    },
                },
                NoCalcLength::ServoCharacterWidth(..) => unreachable!(),
            },
            CalcNode::Sub(ref a, ref b) => {
                a.add_length_or_percentage_to(ret, factor)?;
                b.add_length_or_percentage_to(ret, factor * -1.0)?;
            },
            CalcNode::Sum(ref a, ref b) => {
                a.add_length_or_percentage_to(ret, factor)?;
                b.add_length_or_percentage_to(ret, factor)?;
            },
            CalcNode::Mul(ref a, ref b) => match b.to_number() {
                Ok(rhs) => {
                    a.add_length_or_percentage_to(ret, factor * rhs)?;
                },
                Err(..) => {
                    let lhs = a.to_number()?;
                    b.add_length_or_percentage_to(ret, factor * lhs)?;
                },
            },
            CalcNode::Div(ref a, ref b) => {
                let new_factor = b.to_number()?;
                if new_factor == 0. {
                    return Err(());
                }
                a.add_length_or_percentage_to(ret, factor / new_factor)?;
            },
            CalcNode::Max(..) |
            CalcNode::Min(..) |
            CalcNode::Clamp { .. } => {
                // FIXME(emilio): Implement min/max/clamp for length-percentage.
                return Err(())
            },
            CalcNode::Angle(..) | CalcNode::Time(..) | CalcNode::Number(..) => return Err(()),
        }

        Ok(())
    }

    /// Tries to simplify this expression into a `<time>` value.
    fn to_time(&self) -> Result<Time, ()> {
        impl_generic_to_type!(self, Time, to_time, seconds, Time::from_calc)
    }

    /// Tries to simplify this expression into an `Angle` value.
    fn to_angle(&self) -> Result<Angle, ()> {
        impl_generic_to_type!(self, Angle, to_angle, degrees, Angle::from_calc)
    }

    /// Tries to simplify this expression into a `<number>` value.
    fn to_number(&self) -> Result<CSSFloat, ()> {
        impl_generic_to_type!(self, Number, to_number, clone, From::from)
    }

    /// Tries to simplify this expression into a `<percentage>` value.
    fn to_percentage(&self) -> Result<CSSFloat, ()> {
        impl_generic_to_type!(self, Percentage, to_percentage, clone, From::from)
    }

    /// Given a function name, and the location from where the token came from,
    /// return a mathematical function corresponding to that name or an error.
    #[inline]
    pub fn math_function<'i>(
        name: &CowRcStr<'i>,
        location: cssparser::SourceLocation,
    ) -> Result<MathFunction, ParseError<'i>> {
        // TODO(emilio): Unify below when the pref for math functions is gone.
        if name.eq_ignore_ascii_case("calc") {
            return Ok(MathFunction::Calc);
        }

        if !static_prefs::pref!("layout.css.comparison-functions.enabled") {
            return Err(location.new_unexpected_token_error(Token::Function(name.clone())));
        }

        Ok(match_ignore_ascii_case! { &*name,
            "min" => MathFunction::Min,
            "max" => MathFunction::Max,
            "clamp" => MathFunction::Clamp,
            _ => return Err(location.new_unexpected_token_error(Token::Function(name.clone()))),
        })
    }

    /// Convenience parsing function for integers.
    pub fn parse_integer<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        function: MathFunction,
    ) -> Result<CSSInteger, ParseError<'i>> {
        Self::parse_number(context, input, function).map(|n| n.round() as CSSInteger)
    }

    /// Convenience parsing function for `<length> | <percentage>`.
    pub fn parse_length_or_percentage<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        clamping_mode: AllowedNumericType,
        function: MathFunction,
    ) -> Result<CalcLengthPercentage, ParseError<'i>> {
        Self::parse(context, input, function, CalcUnit::LengthPercentage)?
            .to_length_or_percentage(clamping_mode)
            .map_err(|()| input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }

    /// Convenience parsing function for percentages.
    pub fn parse_percentage<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        function: MathFunction,
    ) -> Result<CSSFloat, ParseError<'i>> {
        Self::parse(context, input, function, CalcUnit::Percentage)?
            .to_percentage()
            .map_err(|()| input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }

    /// Convenience parsing function for `<length>`.
    pub fn parse_length<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        clamping_mode: AllowedNumericType,
        function: MathFunction,
    ) -> Result<CalcLengthPercentage, ParseError<'i>> {
        Self::parse(context, input, function, CalcUnit::Length)?
            .to_length_or_percentage(clamping_mode)
            .map_err(|()| input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }

    /// Convenience parsing function for `<number>`.
    pub fn parse_number<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        function: MathFunction,
    ) -> Result<CSSFloat, ParseError<'i>> {
        Self::parse(context, input, function, CalcUnit::Number)?
            .to_number()
            .map_err(|()| input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }

    /// Convenience parsing function for `<angle>`.
    pub fn parse_angle<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        function: MathFunction,
    ) -> Result<Angle, ParseError<'i>> {
        Self::parse(context, input, function, CalcUnit::Angle)?
            .to_angle()
            .map_err(|()| input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }

    /// Convenience parsing function for `<time>`.
    pub fn parse_time<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        function: MathFunction,
    ) -> Result<Time, ParseError<'i>> {
        Self::parse(context, input, function, CalcUnit::Time)?
            .to_time()
            .map_err(|()| input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }

    /// Convenience parsing function for `<number>` or `<percentage>`.
    pub fn parse_number_or_percentage<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        function: MathFunction,
    ) -> Result<NumberOrPercentage, ParseError<'i>> {
        let node = Self::parse(context, input, function, CalcUnit::Percentage)?;

        if let Ok(value) = node.to_number() {
            return Ok(NumberOrPercentage::Number { value });
        }

        match node.to_percentage() {
            Ok(unit_value) => Ok(NumberOrPercentage::Percentage { unit_value }),
            Err(()) => Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
        }
    }

    /// Convenience parsing function for `<number>` or `<angle>`.
    pub fn parse_angle_or_number<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        function: MathFunction,
    ) -> Result<AngleOrNumber, ParseError<'i>> {
        let node = Self::parse(context, input, function, CalcUnit::Angle)?;

        if let Ok(angle) = node.to_angle() {
            let degrees = angle.degrees();
            return Ok(AngleOrNumber::Angle { degrees });
        }

        match node.to_number() {
            Ok(value) => Ok(AngleOrNumber::Number { value }),
            Err(()) => Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
        }
    }
}
