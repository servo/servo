/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! [Calc expressions][calc].
//!
//! [calc]: https://drafts.csswg.org/css-values/#calc-notation

use cssparser::{Parser, Token, BasicParseError};
use parser::ParserContext;
use std::ascii::AsciiExt;
use std::fmt;
use style_traits::{ToCss, ParseError, StyleParseError};
use style_traits::values::specified::AllowedLengthType;
use values::{CSSInteger, CSSFloat};
use values::computed;
use values::specified::{Angle, Time};
use values::specified::length::{AbsoluteLength, FontRelativeLength, NoCalcLength};
use values::specified::length::ViewportPercentageLength;

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
}

/// An expected unit we intend to parse within a `calc()` expression.
///
/// This is used as a hint for the parser to fast-reject invalid expressions.
#[derive(Clone, Copy, PartialEq)]
pub enum CalcUnit {
    /// `<number>`
    Number,
    /// `<integer>`
    Integer,
    /// `<length>`
    Length,
    /// `<percentage>`
    Percentage,
    /// `<length> | <percentage>`
    LengthOrPercentage,
    /// `<angle>`
    Angle,
    /// `<time>`
    Time,
}

/// A struct to hold a simplified `<length>` or `<percentage>` expression.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct CalcLengthOrPercentage {
    pub clamping_mode: AllowedLengthType,
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
    #[cfg(feature = "gecko")]
    pub mozmm: Option<CSSFloat>,
}

impl ToCss for CalcLengthOrPercentage {
    /// https://drafts.csswg.org/css-values/#calc-serialize
    ///
    /// FIXME(emilio): Should this simplify away zeros?
    #[allow(unused_assignments)]
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        use num_traits::Zero;

        let mut first_value = true;
        macro_rules! first_value_check {
            ($val:expr) => {
                if !first_value {
                    dest.write_str(if $val < Zero::zero() {
                        " - "
                    } else {
                        " + "
                    })?;
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
        serialize_abs!(In);

        #[cfg(feature = "gecko")]
        {
            serialize!(mozmm);
        }

        serialize_abs!(Mm, Pc, Pt, Px, Q);
        serialize!(rem, vh, vmax, vmin, vw);

        dest.write_str(")")
    }
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
        expected_unit: CalcUnit
    ) -> Result<Self, ParseError<'i>> {
        // FIXME: remove early returns when lifetimes are non-lexical
        match (input.next()?, expected_unit) {
            (&Token::Number { value, .. }, _) => return Ok(CalcNode::Number(value)),
            (&Token::Dimension { value, ref unit, .. }, CalcUnit::Length) |
            (&Token::Dimension { value, ref unit, .. }, CalcUnit::LengthOrPercentage) => {
                return NoCalcLength::parse_dimension(context, value, unit)
                    .map(CalcNode::Length)
                    .map_err(|()| StyleParseError::UnspecifiedError.into())
            }
            (&Token::Dimension { value, ref unit, .. }, CalcUnit::Angle) => {
                return Angle::parse_dimension(value, unit, /* from_calc = */ true)
                    .map(CalcNode::Angle)
                    .map_err(|()| StyleParseError::UnspecifiedError.into())
            }
            (&Token::Dimension { value, ref unit, .. }, CalcUnit::Time) => {
                return Time::parse_dimension(value, unit, /* from_calc = */ true)
                    .map(CalcNode::Time)
                    .map_err(|()| StyleParseError::UnspecifiedError.into())
            }
            (&Token::Percentage { unit_value, .. }, CalcUnit::LengthOrPercentage) |
            (&Token::Percentage { unit_value, .. }, CalcUnit::Percentage) => {
                return Ok(CalcNode::Percentage(unit_value))
            }
            (&Token::ParenthesisBlock, _) => {}
            (&Token::Function(ref name), _) if name.eq_ignore_ascii_case("calc") => {}
            (t, _) => return Err(BasicParseError::UnexpectedToken(t.clone()).into())
        }
        input.parse_nested_block(|i| {
            CalcNode::parse(context, i, expected_unit)
        })
    }

    /// Parse a top-level `calc` expression, with all nested sub-expressions.
    ///
    /// This is in charge of parsing, for example, `2 + 3 * 100%`.
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        expected_unit: CalcUnit)
        -> Result<Self, ParseError<'i>>
    {
        let mut root = Self::parse_product(context, input, expected_unit)?;

        loop {
            let start = input.state();
            match input.next_including_whitespace() {
                Ok(&Token::WhiteSpace(_)) => {
                    if input.is_exhausted() {
                        break; // allow trailing whitespace
                    }
                    // FIXME: remove clone() when lifetimes are non-lexical
                    match input.next()?.clone() {
                        Token::Delim('+') => {
                            let rhs =
                                Self::parse_product(context, input, expected_unit)?;
                            let new_root =
                                CalcNode::Sum(Box::new(root), Box::new(rhs));
                            root = new_root;
                        }
                        Token::Delim('-') => {
                            let rhs =
                                Self::parse_product(context, input, expected_unit)?;
                            let new_root =
                                CalcNode::Sub(Box::new(root), Box::new(rhs));
                            root = new_root;
                        }
                        t => return Err(BasicParseError::UnexpectedToken(t).into()),
                    }
                }
                _ => {
                    input.reset(&start);
                    break
                }
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
        expected_unit: CalcUnit)
        -> Result<Self, ParseError<'i>>
    {
        let mut root = Self::parse_one(context, input, expected_unit)?;

        loop {
            let start = input.state();
            match input.next() {
                Ok(&Token::Delim('*')) => {
                    let rhs = Self::parse_one(context, input, expected_unit)?;
                    let new_root = CalcNode::Mul(Box::new(root), Box::new(rhs));
                    root = new_root;
                }
                // TODO(emilio): Figure out why the `Integer` check.
                Ok(&Token::Delim('/')) if expected_unit != CalcUnit::Integer => {
                    let rhs = Self::parse_one(context, input, expected_unit)?;
                    let new_root = CalcNode::Div(Box::new(root), Box::new(rhs));
                    root = new_root;
                }
                _ => {
                    input.reset(&start);
                    break
                }
            }
        }

        Ok(root)
    }

    /// Tries to simplify this expression into a `<length>` or `<percentage`>
    /// value.
    fn to_length_or_percentage(&self, clamping_mode: AllowedLengthType)
                               -> Result<CalcLengthOrPercentage, ()> {
        let mut ret = CalcLengthOrPercentage {
            clamping_mode: clamping_mode,
            .. Default::default()
        };
        self.add_length_or_percentage_to(&mut ret, 1.0)?;
        Ok(ret)
    }

    /// Tries to simplify this expression into a `<percentage>` value.
    fn to_percentage(&self) -> Result<CSSFloat, ()> {
        Ok(match *self {
            CalcNode::Percentage(percentage) => percentage,
            CalcNode::Sub(ref a, ref b) => {
                a.to_percentage()? - b.to_percentage()?
            }
            CalcNode::Sum(ref a, ref b) => {
                a.to_percentage()? + b.to_percentage()?
            }
            CalcNode::Mul(ref a, ref b) => {
                match a.to_percentage() {
                    Ok(lhs) => {
                        let rhs = b.to_number()?;
                        lhs * rhs
                    }
                    Err(..) => {
                        let lhs = a.to_number()?;
                        let rhs = b.to_percentage()?;
                        lhs * rhs
                    }
                }
            }
            CalcNode::Div(ref a, ref b) => {
                let lhs = a.to_percentage()?;
                let rhs = b.to_number()?;
                if rhs == 0. {
                    return Err(())
                }
                lhs / rhs
            }
            CalcNode::Number(..) |
            CalcNode::Length(..) |
            CalcNode::Angle(..) |
            CalcNode::Time(..) => return Err(()),
        })
    }

    /// Puts this `<length>` or `<percentage>` into `ret`, or error.
    ///
    /// `factor` is the sign or multiplicative factor to account for the sign
    /// (this allows adding and substracting into the return value).
    fn add_length_or_percentage_to(
        &self,
        ret: &mut CalcLengthOrPercentage,
        factor: CSSFloat)
        -> Result<(), ()>
    {
        match *self {
            CalcNode::Percentage(pct) => {
                ret.percentage = Some(computed::Percentage(
                    ret.percentage.map_or(0., |p| p.0) + pct * factor,
                ));
            }
            CalcNode::Length(ref l) => {
                match *l {
                    NoCalcLength::Absolute(abs) => {
                        ret.absolute = Some(
                            match ret.absolute {
                                Some(value) => value + abs * factor,
                                None => abs * factor,
                            }
                        );
                    }
                    NoCalcLength::FontRelative(rel) => {
                        match rel {
                            FontRelativeLength::Em(em) => {
                                ret.em = Some(ret.em.unwrap_or(0.) + em * factor);
                            }
                            FontRelativeLength::Ex(ex) => {
                                ret.ex = Some(ret.em.unwrap_or(0.) + ex * factor);
                            }
                            FontRelativeLength::Ch(ch) => {
                                ret.ch = Some(ret.ch.unwrap_or(0.) + ch * factor);
                            }
                            FontRelativeLength::Rem(rem) => {
                                ret.rem = Some(ret.rem.unwrap_or(0.) + rem * factor);
                            }
                        }
                    }
                    NoCalcLength::ViewportPercentage(rel) => {
                        match rel {
                            ViewportPercentageLength::Vh(vh) => {
                                ret.vh = Some(ret.vh.unwrap_or(0.) + vh * factor)
                            }
                            ViewportPercentageLength::Vw(vw) => {
                                ret.vw = Some(ret.vw.unwrap_or(0.) + vw * factor)
                            }
                            ViewportPercentageLength::Vmax(vmax) => {
                                ret.vmax = Some(ret.vmax.unwrap_or(0.) + vmax * factor)
                            }
                            ViewportPercentageLength::Vmin(vmin) => {
                                ret.vmin = Some(ret.vmin.unwrap_or(0.) + vmin * factor)
                            }
                        }
                    }
                    NoCalcLength::ServoCharacterWidth(..) => unreachable!(),
                    #[cfg(feature = "gecko")]
                    NoCalcLength::Physical(physical) => {
                        ret.mozmm = Some(ret.mozmm.unwrap_or(0.) + physical.0 * factor);
                    }
                }
            }
            CalcNode::Sub(ref a, ref b) => {
                a.add_length_or_percentage_to(ret, factor)?;
                b.add_length_or_percentage_to(ret, factor * -1.0)?;
            }
            CalcNode::Sum(ref a, ref b) => {
                a.add_length_or_percentage_to(ret, factor)?;
                b.add_length_or_percentage_to(ret, factor)?;
            }
            CalcNode::Mul(ref a, ref b) => {
                match b.to_number() {
                    Ok(rhs) => {
                        a.add_length_or_percentage_to(ret, factor * rhs)?;
                    }
                    Err(..) => {
                        let lhs = a.to_number()?;
                        b.add_length_or_percentage_to(ret, factor * lhs)?;
                    }
                }
            }
            CalcNode::Div(ref a, ref b) => {
                let new_factor = b.to_number()?;
                if new_factor == 0. {
                    return Err(());
                }
                a.add_length_or_percentage_to(ret, factor / new_factor)?;
            }
            CalcNode::Angle(..) |
            CalcNode::Time(..) |
            CalcNode::Number(..) => return Err(()),
        }

        Ok(())
    }

    /// Tries to simplify this expression into a `<time>` value.
    fn to_time(&self) -> Result<Time, ()> {
        Ok(match *self {
            CalcNode::Time(ref time) => time.clone(),
            CalcNode::Sub(ref a, ref b) => {
                let lhs = a.to_time()?;
                let rhs = b.to_time()?;
                Time::from_calc(lhs.seconds() - rhs.seconds())
            }
            CalcNode::Sum(ref a, ref b) => {
                let lhs = a.to_time()?;
                let rhs = b.to_time()?;
                Time::from_calc(lhs.seconds() + rhs.seconds())
            }
            CalcNode::Mul(ref a, ref b) => {
                match b.to_number() {
                    Ok(rhs) => {
                        let lhs = a.to_time()?;
                        Time::from_calc(lhs.seconds() * rhs)
                    }
                    Err(()) => {
                        let lhs = a.to_number()?;
                        let rhs = b.to_time()?;
                        Time::from_calc(lhs * rhs.seconds())
                    }
                }
            }
            CalcNode::Div(ref a, ref b) => {
                let lhs = a.to_time()?;
                let rhs = b.to_number()?;
                if rhs == 0. {
                    return Err(())
                }
                Time::from_calc(lhs.seconds() / rhs)
            }
            CalcNode::Number(..) |
            CalcNode::Length(..) |
            CalcNode::Percentage(..) |
            CalcNode::Angle(..) => return Err(()),
        })
    }

    /// Tries to simplify this expression into an `Angle` value.
    fn to_angle(&self) -> Result<Angle, ()> {
        Ok(match *self {
            CalcNode::Angle(ref angle) => angle.clone(),
            CalcNode::Sub(ref a, ref b) => {
                let lhs = a.to_angle()?;
                let rhs = b.to_angle()?;
                Angle::from_calc(lhs.radians() - rhs.radians())
            }
            CalcNode::Sum(ref a, ref b) => {
                let lhs = a.to_angle()?;
                let rhs = b.to_angle()?;
                Angle::from_calc(lhs.radians() + rhs.radians())
            }
            CalcNode::Mul(ref a, ref b) => {
                match a.to_angle() {
                    Ok(lhs) => {
                        let rhs = b.to_number()?;
                        Angle::from_calc(lhs.radians() * rhs)
                    }
                    Err(..) => {
                        let lhs = a.to_number()?;
                        let rhs = b.to_angle()?;
                        Angle::from_calc(lhs * rhs.radians())
                    }
                }
            }
            CalcNode::Div(ref a, ref b) => {
                let lhs = a.to_angle()?;
                let rhs = b.to_number()?;
                if rhs == 0. {
                    return Err(())
                }
                Angle::from_calc(lhs.radians() / rhs)
            }
            CalcNode::Number(..) |
            CalcNode::Length(..) |
            CalcNode::Percentage(..) |
            CalcNode::Time(..) => return Err(()),
        })
    }

    /// Tries to simplify this expression into a `<number>` value.
    fn to_number(&self) -> Result<CSSFloat, ()> {
        Ok(match *self {
            CalcNode::Number(n) => n,
            CalcNode::Sum(ref a, ref b) => {
                a.to_number()? + b.to_number()?
            }
            CalcNode::Sub(ref a, ref b) => {
                a.to_number()? - b.to_number()?
            }
            CalcNode::Mul(ref a, ref b) => {
                a.to_number()? * b.to_number()?
            }
            CalcNode::Div(ref a, ref b) => {
                let lhs = a.to_number()?;
                let rhs = b.to_number()?;
                if rhs == 0. {
                    return Err(())
                }
                lhs / rhs
            }
            CalcNode::Length(..) |
            CalcNode::Percentage(..) |
            CalcNode::Angle(..) |
            CalcNode::Time(..) => return Err(()),
        })
    }

    /// Convenience parsing function for integers.
    pub fn parse_integer<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<CSSInteger, ParseError<'i>> {
        Self::parse(context, input, CalcUnit::Integer)?
            .to_number()
            .map(|n| n as CSSInteger)
            .map_err(|()| StyleParseError::UnspecifiedError.into())
    }

    /// Convenience parsing function for `<length> | <percentage>`.
    pub fn parse_length_or_percentage<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        clamping_mode: AllowedLengthType
    ) -> Result<CalcLengthOrPercentage, ParseError<'i>> {
        Self::parse(context, input, CalcUnit::LengthOrPercentage)?
            .to_length_or_percentage(clamping_mode)
            .map_err(|()| StyleParseError::UnspecifiedError.into())
    }

    /// Convenience parsing function for percentages.
    pub fn parse_percentage<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<CSSFloat, ParseError<'i>> {
        Self::parse(context, input, CalcUnit::Percentage)?
            .to_percentage()
            .map_err(|()| StyleParseError::UnspecifiedError.into())
    }

    /// Convenience parsing function for `<length>`.
    pub fn parse_length<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        clamping_mode: AllowedLengthType
    ) -> Result<CalcLengthOrPercentage, ParseError<'i>> {
        Self::parse(context, input, CalcUnit::Length)?
            .to_length_or_percentage(clamping_mode)
            .map_err(|()| StyleParseError::UnspecifiedError.into())
    }

    /// Convenience parsing function for `<number>`.
    pub fn parse_number<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<CSSFloat, ParseError<'i>> {
        Self::parse(context, input, CalcUnit::Number)?
            .to_number()
            .map_err(|()| StyleParseError::UnspecifiedError.into())
    }

    /// Convenience parsing function for `<angle>`.
    pub fn parse_angle<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Angle, ParseError<'i>> {
        Self::parse(context, input, CalcUnit::Angle)?
            .to_angle()
            .map_err(|()| StyleParseError::UnspecifiedError.into())
    }

    /// Convenience parsing function for `<time>`.
    pub fn parse_time<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Time, ParseError<'i>> {
        Self::parse(context, input, CalcUnit::Time)?
            .to_time()
            .map_err(|()| StyleParseError::UnspecifiedError.into())
    }
}
