/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! [Calc expressions][calc].
//!
//! [calc]: https://drafts.csswg.org/css-values/#calc-notation

use app_units::Au;
use cssparser::{Parser, Token};
use parser::ParserContext;
use std::ascii::AsciiExt;
use std::fmt;
use style_traits::ToCss;
use style_traits::values::specified::AllowedLengthType;
use values::{CSSInteger, CSSFloat, HasViewportPercentage};
use values::specified::{Angle, Time};
use values::specified::length::{FontRelativeLength, NoCalcLength, ViewportPercentageLength};

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
    /// `<length> | <percentage>`
    LengthOrPercentage,
    /// `<angle>`
    Angle,
    /// `<time>`
    Time,
}

/// A struct to hold a simplified `<length>` or `<percentage>` expression.
#[derive(Clone, PartialEq, Copy, Debug, Default)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[allow(missing_docs)]
pub struct CalcLengthOrPercentage {
    pub clamping_mode: AllowedLengthType,
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
    #[cfg(feature = "gecko")]
    pub mozmm: Option<CSSFloat>,
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

        try!(dest.write_str("calc("));

        serialize!(ch, em, ex, rem, vh, vmax, vmin, vw);

        #[cfg(feature = "gecko")]
        {
            serialize!(mozmm);
        }

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

impl CalcNode {
    /// Tries to parse a single element in the expression, that is, a
    /// `<length>`, `<angle>`, `<time>`, `<percentage>`, according to
    /// `expected_unit`.
    ///
    /// May return a "complex" `CalcNode`, in the presence of a parenthesized
    /// expression, for example.
    fn parse_one(
        context: &ParserContext,
        input: &mut Parser,
        expected_unit: CalcUnit)
        -> Result<Self, ()>
    {
        match (try!(input.next()), expected_unit) {
            (Token::Number(ref value), _) => Ok(CalcNode::Number(value.value)),
            (Token::Dimension(ref value, ref unit), CalcUnit::Length) |
            (Token::Dimension(ref value, ref unit), CalcUnit::LengthOrPercentage) => {
                NoCalcLength::parse_dimension(context, value.value, unit)
                    .map(CalcNode::Length)
            }
            (Token::Dimension(ref value, ref unit), CalcUnit::Angle) => {
                Angle::parse_dimension(value.value,
                                       unit,
                                       /* from_calc = */ true)
                    .map(CalcNode::Angle)
            }
            (Token::Dimension(ref value, ref unit), CalcUnit::Time) => {
                Time::parse_dimension(value.value,
                                      unit,
                                      /* from_calc = */ true)
                    .map(CalcNode::Time)
            }
            (Token::Percentage(ref value), CalcUnit::LengthOrPercentage) => {
                Ok(CalcNode::Percentage(value.unit_value))
            }
            (Token::ParenthesisBlock, _) => {
                input.parse_nested_block(|i| {
                    CalcNode::parse(context, i, expected_unit)
                })
            }
            (Token::Function(ref name), _) if name.eq_ignore_ascii_case("calc") => {
                input.parse_nested_block(|i| {
                    CalcNode::parse(context, i, expected_unit)
                })
            }
            _ => Err(())
        }
    }

    /// Parse a top-level `calc` expression, with all nested sub-expressions.
    ///
    /// This is in charge of parsing, for example, `2 + 3 * 100%`.
    fn parse(
        context: &ParserContext,
        input: &mut Parser,
        expected_unit: CalcUnit)
        -> Result<Self, ()>
    {
        let mut root = Self::parse_product(context, input, expected_unit)?;

        loop {
            let position = input.position();
            match input.next_including_whitespace() {
                Ok(Token::WhiteSpace(_)) => {
                    if input.is_exhausted() {
                        break; // allow trailing whitespace
                    }
                    match input.next()? {
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
                        _ => return Err(()),
                    }
                }
                _ => {
                    input.reset(position);
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
    ///     * `2`
    ///     * `2 * 2`
    ///     * `2 * 2 + 2` (but will leave the `+ 2` unparsed).
    ///
    fn parse_product(
        context: &ParserContext,
        input: &mut Parser,
        expected_unit: CalcUnit)
        -> Result<Self, ()>
    {
        let mut root = Self::parse_one(context, input, expected_unit)?;

        loop {
            let position = input.position();
            match input.next() {
                Ok(Token::Delim('*')) => {
                    let rhs = Self::parse_one(context, input, expected_unit)?;
                    let new_root = CalcNode::Mul(Box::new(root), Box::new(rhs));
                    root = new_root;
                }
                // TODO(emilio): Figure out why the `Integer` check.
                Ok(Token::Delim('/')) if expected_unit != CalcUnit::Integer => {
                    let rhs = Self::parse_one(context, input, expected_unit)?;
                    let new_root = CalcNode::Div(Box::new(root), Box::new(rhs));
                    root = new_root;
                }
                _ => {
                    input.reset(position);
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
                ret.percentage = Some(ret.percentage.unwrap_or(0.) + pct * factor)
            }
            CalcNode::Length(ref l) => {
                match *l {
                    NoCalcLength::Absolute(abs) => {
                        ret.absolute = Some(
                            ret.absolute.unwrap_or(Au(0)) +
                            Au::from(abs).scale_by(factor)
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
    pub fn parse_integer(
        context: &ParserContext,
        input: &mut Parser)
        -> Result<CSSInteger, ()>
    {
        Self::parse(context, input, CalcUnit::Integer)?
            .to_number()
            .map(|n| n as CSSInteger)
    }

    /// Convenience parsing function for `<length> | <percentage>`.
    pub fn parse_length_or_percentage(
        context: &ParserContext,
        input: &mut Parser,
        clamping_mode: AllowedLengthType)
        -> Result<CalcLengthOrPercentage, ()>
    {
        Self::parse(context, input, CalcUnit::LengthOrPercentage)?
            .to_length_or_percentage(clamping_mode)
    }

    /// Convenience parsing function for `<length>`.
    pub fn parse_length(
        context: &ParserContext,
        input: &mut Parser,
        clamping_mode: AllowedLengthType)
        -> Result<CalcLengthOrPercentage, ()>
    {
        Self::parse(context, input, CalcUnit::Length)?
            .to_length_or_percentage(clamping_mode)
    }

    /// Convenience parsing function for `<number>`.
    pub fn parse_number(
        context: &ParserContext,
        input: &mut Parser)
        -> Result<CSSFloat, ()>
    {
        Self::parse(context, input, CalcUnit::Number)?
            .to_number()
    }

    /// Convenience parsing function for `<angle>`.
    pub fn parse_angle(
        context: &ParserContext,
        input: &mut Parser)
        -> Result<Angle, ()>
    {
        Self::parse(context, input, CalcUnit::Angle)?
            .to_angle()
    }

    /// Convenience parsing function for `<time>`.
    pub fn parse_time(
        context: &ParserContext,
        input: &mut Parser)
        -> Result<Time, ()>
    {
        Self::parse(context, input, CalcUnit::Time)?
            .to_time()
    }
}
