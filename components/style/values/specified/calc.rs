/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! [Calc expressions][calc].
//!
//! [calc]: https://drafts.csswg.org/css-values/#calc-notation

use crate::parser::ParserContext;
use crate::values::generics::calc as generic;
use crate::values::generics::calc::{MinMaxOp, SortKey};
use crate::values::specified::length::ViewportPercentageLength;
use crate::values::specified::length::{AbsoluteLength, FontRelativeLength, NoCalcLength};
use crate::values::specified::{self, Angle, Time};
use crate::values::{CSSFloat, CSSInteger};
use cssparser::{AngleOrNumber, CowRcStr, NumberOrPercentage, Parser, Token};
use smallvec::SmallVec;
use std::cmp;
use std::fmt::{self, Write};
use style_traits::values::specified::AllowedNumericType;
use style_traits::{CssWriter, ParseError, SpecifiedValueInfo, StyleParseErrorKind, ToCss};

/// The name of the mathematical function that we're parsing.
#[derive(Clone, Copy, Debug, Parse)]
pub enum MathFunction {
    /// `calc()`: https://drafts.csswg.org/css-values-4/#funcdef-calc
    Calc,
    /// `min()`: https://drafts.csswg.org/css-values-4/#funcdef-min
    Min,
    /// `max()`: https://drafts.csswg.org/css-values-4/#funcdef-max
    Max,
    /// `clamp()`: https://drafts.csswg.org/css-values-4/#funcdef-clamp
    Clamp,
    /// `sin()`: https://drafts.csswg.org/css-values-4/#funcdef-sin
    Sin,
    /// `cos()`: https://drafts.csswg.org/css-values-4/#funcdef-cos
    Cos,
    /// `tan()`: https://drafts.csswg.org/css-values-4/#funcdef-tan
    Tan,
    /// `asin()`: https://drafts.csswg.org/css-values-4/#funcdef-asin
    Asin,
    /// `acos()`: https://drafts.csswg.org/css-values-4/#funcdef-acos
    Acos,
    /// `atan()`: https://drafts.csswg.org/css-values-4/#funcdef-atan
    Atan,
}

/// A leaf node inside a `Calc` expression's AST.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToShmem)]
pub enum Leaf {
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
}

impl ToCss for Leaf {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        match *self {
            Self::Length(ref l) => l.to_css(dest),
            Self::Number(ref n) => n.to_css(dest),
            Self::Percentage(p) => crate::values::serialize_percentage(p, dest),
            Self::Angle(ref a) => a.to_css(dest),
            Self::Time(ref t) => t.to_css(dest),
        }
    }
}

/// An expected unit we intend to parse within a `calc()` expression.
///
/// This is used as a hint for the parser to fast-reject invalid expressions.
#[derive(Clone, Copy, PartialEq)]
enum CalcUnit {
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
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss, ToShmem)]
#[allow(missing_docs)]
pub struct CalcLengthPercentage {
    #[css(skip)]
    pub clamping_mode: AllowedNumericType,
    pub node: CalcNode,
}

impl SpecifiedValueInfo for CalcLengthPercentage {}

impl PartialOrd for Leaf {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        use self::Leaf::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return None;
        }

        match (self, other) {
            // NOTE: Percentages can't be compared reasonably here because the
            // percentage basis might be negative, see bug 1709018.
            // Conveniently, we only use this for <length-percentage> (for raw
            // percentages, we go through resolve()).
            (&Percentage(..), &Percentage(..)) => None,
            (&Length(ref one), &Length(ref other)) => one.partial_cmp(other),
            (&Angle(ref one), &Angle(ref other)) => one.degrees().partial_cmp(&other.degrees()),
            (&Time(ref one), &Time(ref other)) => one.seconds().partial_cmp(&other.seconds()),
            (&Number(ref one), &Number(ref other)) => one.partial_cmp(other),
            _ => {
                match *self {
                    Length(..) | Percentage(..) | Angle(..) | Time(..) | Number(..) => {},
                }
                unsafe {
                    debug_unreachable!("Forgot a branch?");
                }
            },
        }
    }
}

impl generic::CalcNodeLeaf for Leaf {
    fn is_negative(&self) -> bool {
        match *self {
            Self::Length(ref l) => l.is_negative(),
            Self::Percentage(n) | Self::Number(n) => n < 0.,
            Self::Angle(ref a) => a.degrees() < 0.,
            Self::Time(ref t) => t.seconds() < 0.,
        }
    }

    fn mul_by(&mut self, scalar: f32) {
        match *self {
            Self::Length(ref mut l) => {
                // FIXME: For consistency this should probably convert absolute
                // lengths into pixels.
                *l = *l * scalar;
            },
            Self::Number(ref mut n) => {
                *n *= scalar;
            },
            Self::Angle(ref mut a) => {
                *a = Angle::from_calc(a.degrees() * scalar);
            },
            Self::Time(ref mut t) => {
                *t = Time::from_calc(t.seconds() * scalar);
            },
            Self::Percentage(ref mut p) => {
                *p *= scalar;
            },
        }
    }

    fn sort_key(&self) -> SortKey {
        match *self {
            Self::Number(..) => SortKey::Number,
            Self::Percentage(..) => SortKey::Percentage,
            Self::Time(..) => SortKey::Sec,
            Self::Angle(..) => SortKey::Deg,
            Self::Length(ref l) => match *l {
                NoCalcLength::Absolute(..) => SortKey::Px,
                NoCalcLength::FontRelative(ref relative) => match *relative {
                    FontRelativeLength::Ch(..) => SortKey::Ch,
                    FontRelativeLength::Em(..) => SortKey::Em,
                    FontRelativeLength::Ex(..) => SortKey::Ex,
                    FontRelativeLength::Cap(..) => SortKey::Cap,
                    FontRelativeLength::Ic(..) => SortKey::Ic,
                    FontRelativeLength::Rem(..) => SortKey::Rem,
                },
                NoCalcLength::ViewportPercentage(ref vp) => match *vp {
                    ViewportPercentageLength::Vh(..) => SortKey::Vh,
                    ViewportPercentageLength::Svh(..) => SortKey::Svh,
                    ViewportPercentageLength::Lvh(..) => SortKey::Lvh,
                    ViewportPercentageLength::Dvh(..) => SortKey::Dvh,
                    ViewportPercentageLength::Vw(..) => SortKey::Vw,
                    ViewportPercentageLength::Svw(..) => SortKey::Svw,
                    ViewportPercentageLength::Lvw(..) => SortKey::Lvw,
                    ViewportPercentageLength::Dvw(..) => SortKey::Dvw,
                    ViewportPercentageLength::Vmax(..) => SortKey::Vmax,
                    ViewportPercentageLength::Svmax(..) => SortKey::Svmax,
                    ViewportPercentageLength::Lvmax(..) => SortKey::Lvmax,
                    ViewportPercentageLength::Dvmax(..) => SortKey::Dvmax,
                    ViewportPercentageLength::Vmin(..) => SortKey::Vmin,
                    ViewportPercentageLength::Svmin(..) => SortKey::Svmin,
                    ViewportPercentageLength::Lvmin(..) => SortKey::Lvmin,
                    ViewportPercentageLength::Dvmin(..) => SortKey::Dvmin,
                },
                NoCalcLength::ServoCharacterWidth(..) => unreachable!(),
            },
        }
    }

    fn simplify(&mut self) {
        if let Self::Length(NoCalcLength::Absolute(ref mut abs)) = *self {
            *abs = AbsoluteLength::Px(abs.to_px());
        }
    }

    /// Tries to merge one sum to another, that is, perform `x` + `y`.
    ///
    /// Only handles leaf nodes, it's the caller's responsibility to simplify
    /// them before calling this if needed.
    fn try_sum_in_place(&mut self, other: &Self) -> Result<(), ()> {
        use self::Leaf::*;

        if std::mem::discriminant(self) != std::mem::discriminant(other) {
            return Err(());
        }

        match (self, other) {
            (&mut Number(ref mut one), &Number(ref other)) |
            (&mut Percentage(ref mut one), &Percentage(ref other)) => {
                *one += *other;
            },
            (&mut Angle(ref mut one), &Angle(ref other)) => {
                *one = specified::Angle::from_calc(one.degrees() + other.degrees());
            },
            (&mut Time(ref mut one), &Time(ref other)) => {
                *one = specified::Time::from_calc(one.seconds() + other.seconds());
            },
            (&mut Length(ref mut one), &Length(ref other)) => {
                *one = one.try_sum(other)?;
            },
            _ => {
                match *other {
                    Number(..) | Percentage(..) | Angle(..) | Time(..) | Length(..) => {},
                }
                unsafe {
                    debug_unreachable!();
                }
            },
        }

        Ok(())
    }
}

fn trig_enabled() -> bool {
    #[cfg(feature = "gecko")]
    return static_prefs::pref!("layout.css.trig.enabled");
    #[cfg(feature = "servo")]
    return false;
}

/// A calc node representation for specified values.
pub type CalcNode = generic::GenericCalcNode<Leaf>;

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
            (&Token::Number { value, .. }, _) => Ok(CalcNode::Leaf(Leaf::Number(value))),
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
            ) => match NoCalcLength::parse_dimension(context, value, unit) {
                Ok(l) => Ok(CalcNode::Leaf(Leaf::Length(l))),
                Err(()) => Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
            },
            (
                &Token::Dimension {
                    value, ref unit, ..
                },
                CalcUnit::Angle,
            ) => {
                match Angle::parse_dimension(value, unit, /* from_calc = */ true) {
                    Ok(a) => Ok(CalcNode::Leaf(Leaf::Angle(a))),
                    Err(()) => {
                        Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                    },
                }
            },
            (
                &Token::Dimension {
                    value, ref unit, ..
                },
                CalcUnit::Time,
            ) => {
                match Time::parse_dimension(value, unit, /* from_calc = */ true) {
                    Ok(t) => Ok(CalcNode::Leaf(Leaf::Time(t))),
                    Err(()) => {
                        Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
                    },
                }
            },
            (&Token::Percentage { unit_value, .. }, CalcUnit::LengthPercentage) |
            (&Token::Percentage { unit_value, .. }, CalcUnit::Percentage) => {
                Ok(CalcNode::Leaf(Leaf::Percentage(unit_value)))
            },
            (&Token::ParenthesisBlock, _) => input.parse_nested_block(|input| {
                CalcNode::parse_argument(context, input, expected_unit)
            }),
            (&Token::Function(ref name), _) => {
                let function = CalcNode::math_function(name, location)?;
                CalcNode::parse(context, input, function, expected_unit)
            },
            (&Token::Ident(ref ident), _) => {
                if !trig_enabled() {
                    return Err(location.new_unexpected_token_error(Token::Ident(ident.clone())));
                }
                let number = match_ignore_ascii_case! { &**ident,
                    "e" => std::f32::consts::E,
                    "pi" => std::f32::consts::PI,
                    _ => return Err(location.new_unexpected_token_error(Token::Ident(ident.clone()))),
                };
                Ok(CalcNode::Leaf(Leaf::Number(number)))
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
                MathFunction::Min | MathFunction::Max => {
                    // TODO(emilio): The common case for parse_comma_separated
                    // is just one element, but for min / max is two, really...
                    //
                    // Consider adding an API to cssparser to specify the
                    // initial vector capacity?
                    let arguments = input.parse_comma_separated(|input| {
                        Self::parse_argument(context, input, expected_unit)
                    })?;

                    let op = match function {
                        MathFunction::Min => MinMaxOp::Min,
                        MathFunction::Max => MinMaxOp::Max,
                        _ => unreachable!(),
                    };

                    Ok(Self::MinMax(arguments.into(), op))
                },
                MathFunction::Sin | MathFunction::Cos | MathFunction::Tan => {
                    let argument = Self::parse_argument(context, input, CalcUnit::Angle)?;
                    let radians = match argument.to_number() {
                        Ok(v) => v,
                        Err(()) => match argument.to_angle() {
                            Ok(angle) => angle.radians(),
                            Err(()) => {
                                return Err(
                                    input.new_custom_error(StyleParseErrorKind::UnspecifiedError)
                                )
                            },
                        },
                    };
                    let number = match function {
                        MathFunction::Sin => radians.sin(),
                        MathFunction::Cos => radians.cos(),
                        MathFunction::Tan => radians.tan(),
                        _ => unsafe {
                            debug_unreachable!("We just checked!");
                        },
                    };
                    Ok(Self::Leaf(Leaf::Number(number)))
                },
                MathFunction::Asin | MathFunction::Acos | MathFunction::Atan => {
                    let argument = Self::parse_argument(context, input, CalcUnit::Number)?;
                    let number = match argument.to_number() {
                        Ok(v) => v,
                        Err(()) => {
                            return Err(
                                input.new_custom_error(StyleParseErrorKind::UnspecifiedError)
                            )
                        },
                    };

                    let radians = match function {
                        MathFunction::Asin => number.asin(),
                        MathFunction::Acos => number.acos(),
                        MathFunction::Atan => number.atan(),
                        _ => unsafe {
                            debug_unreachable!("We just checked!");
                        },
                    };

                    Ok(Self::Leaf(Leaf::Angle(Angle::from_radians(radians))))
                },
            }
        })
    }

    fn parse_argument<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
        expected_unit: CalcUnit,
    ) -> Result<Self, ParseError<'i>> {
        let mut sum = SmallVec::<[CalcNode; 1]>::new();
        sum.push(Self::parse_product(context, input, expected_unit)?);

        loop {
            let start = input.state();
            match input.next_including_whitespace() {
                Ok(&Token::WhiteSpace(_)) => {
                    if input.is_exhausted() {
                        break; // allow trailing whitespace
                    }
                    match *input.next()? {
                        Token::Delim('+') => {
                            sum.push(Self::parse_product(context, input, expected_unit)?);
                        },
                        Token::Delim('-') => {
                            let mut rhs = Self::parse_product(context, input, expected_unit)?;
                            rhs.negate();
                            sum.push(rhs);
                        },
                        _ => {
                            input.reset(&start);
                            break;
                        },
                    }
                },
                _ => {
                    input.reset(&start);
                    break;
                },
            }
        }

        Ok(if sum.len() == 1 {
            sum.drain(..).next().unwrap()
        } else {
            Self::Sum(sum.into_boxed_slice().into())
        })
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
        let mut node = Self::parse_one(context, input, expected_unit)?;

        loop {
            let start = input.state();
            match input.next() {
                Ok(&Token::Delim('*')) => {
                    let rhs = Self::parse_one(context, input, expected_unit)?;
                    if let Ok(rhs) = rhs.to_number() {
                        node.mul_by(rhs);
                    } else if let Ok(number) = node.to_number() {
                        node = rhs;
                        node.mul_by(number);
                    } else {
                        // One of the two parts of the multiplication has to be
                        // a number, at least until we implement unit math.
                        return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                    }
                },
                Ok(&Token::Delim('/')) => {
                    let rhs = Self::parse_one(context, input, expected_unit)?;
                    // Dividing by units is not ok.
                    //
                    // TODO(emilio): Eventually it should be.
                    let number = match rhs.to_number() {
                        Ok(n) if n != 0. => n,
                        _ => {
                            return Err(
                                input.new_custom_error(StyleParseErrorKind::UnspecifiedError)
                            );
                        },
                    };
                    node.mul_by(1. / number);
                },
                _ => {
                    input.reset(&start);
                    break;
                },
            }
        }

        Ok(node)
    }

    /// Tries to simplify this expression into a `<length>` or `<percentage>`
    /// value.
    fn into_length_or_percentage(
        mut self,
        clamping_mode: AllowedNumericType,
    ) -> Result<CalcLengthPercentage, ()> {
        // Keep track of whether there's any invalid member of the calculation,
        // so as to reject the calculation properly at parse-time.
        let mut any_invalid = false;
        self.visit_depth_first(|node| {
            if let CalcNode::Leaf(ref l) = *node {
                any_invalid |= !matches!(*l, Leaf::Percentage(..) | Leaf::Length(..));
            }
            node.simplify_and_sort_direct_children();
        });

        if any_invalid {
            return Err(());
        }

        Ok(CalcLengthPercentage {
            clamping_mode,
            node: self,
        })
    }

    /// Tries to simplify this expression into a `<time>` value.
    fn to_time(&self) -> Result<Time, ()> {
        let seconds = self.resolve(|leaf| match *leaf {
            Leaf::Time(ref t) => Ok(t.seconds()),
            _ => Err(()),
        })?;
        Ok(Time::from_calc(crate::values::normalize(seconds)))
    }

    /// Tries to simplify this expression into an `Angle` value.
    fn to_angle(&self) -> Result<Angle, ()> {
        let degrees = self.resolve(|leaf| match *leaf {
            Leaf::Angle(ref angle) => Ok(angle.degrees()),
            _ => Err(()),
        })?;
        Ok(Angle::from_calc(crate::values::normalize(degrees)))
    }

    /// Tries to simplify this expression into a `<number>` value.
    fn to_number(&self) -> Result<CSSFloat, ()> {
        self.resolve(|leaf| match *leaf {
            Leaf::Number(n) => Ok(n),
            _ => Err(()),
        })
    }

    /// Tries to simplify this expression into a `<percentage>` value.
    fn to_percentage(&self) -> Result<CSSFloat, ()> {
        self.resolve(|leaf| match *leaf {
            Leaf::Percentage(p) => Ok(p),
            _ => Err(()),
        })
    }

    /// Given a function name, and the location from where the token came from,
    /// return a mathematical function corresponding to that name or an error.
    #[inline]
    pub fn math_function<'i>(
        name: &CowRcStr<'i>,
        location: cssparser::SourceLocation,
    ) -> Result<MathFunction, ParseError<'i>> {
        use self::MathFunction::*;

        let function = match MathFunction::from_ident(&*name) {
            Ok(f) => f,
            Err(()) => {
                return Err(location.new_unexpected_token_error(Token::Function(name.clone())))
            },
        };

        if matches!(function, Sin | Cos | Tan | Asin | Acos | Atan) && !trig_enabled() {
            return Err(location.new_unexpected_token_error(Token::Function(name.clone())));
        }

        Ok(function)
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
            .into_length_or_percentage(clamping_mode)
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
            .map(crate::values::normalize)
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
            .into_length_or_percentage(clamping_mode)
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
            .map(crate::values::normalize)
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
