/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values in SVG

use app_units::Au;
use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::{ParseError, StyleParseError, ToCss};
use values::animated::ToAnimatedValue;
use values::animated::svg::SvgLengthOrPercentageOrNumber as AnimatedSvgLengthOrPercentageOrNumber;
use values::computed::{NonNegativeNumber, Number};
use values::computed::length::{CalcLengthOrPercentage, LengthOrPercentage};
use values::computed::length::NonNegativeLengthOrPercentage;

/// An SVG paint value
///
/// https://www.w3.org/TR/SVG2/painting.html#SpecifyingPaint
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Animate, Clone, ComputeSquaredDistance, Debug, PartialEq)]
#[derive(ToAnimatedValue, ToComputedValue, ToCss)]
pub struct SVGPaint<ColorType, UrlPaintServer> {
    /// The paint source
    pub kind: SVGPaintKind<ColorType, UrlPaintServer>,
    /// The fallback color
    pub fallback: Option<ColorType>,
}

/// An SVG paint value without the fallback
///
/// Whereas the spec only allows PaintServer
/// to have a fallback, Gecko lets the context
/// properties have a fallback as well.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Animate, Clone, ComputeSquaredDistance, Debug, PartialEq)]
#[derive(ToAnimatedValue, ToAnimatedZero, ToComputedValue, ToCss)]
pub enum SVGPaintKind<ColorType, UrlPaintServer> {
    /// `none`
    #[animation(error)]
    None,
    /// `<color>`
    Color(ColorType),
    /// `url(...)`
    #[animation(error)]
    PaintServer(UrlPaintServer),
    /// `context-fill`
    ContextFill,
    /// `context-stroke`
    ContextStroke,
}

impl<ColorType, UrlPaintServer> SVGPaintKind<ColorType, UrlPaintServer> {
    /// Parse a keyword value only
    fn parse_ident<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        try_match_ident_ignore_ascii_case! { input.expect_ident()?,
            "none" => Ok(SVGPaintKind::None),
            "context-fill" => Ok(SVGPaintKind::ContextFill),
            "context-stroke" => Ok(SVGPaintKind::ContextStroke),
        }
    }
}

/// Parse SVGPaint's fallback.
/// fallback is keyword(none) or Color.
/// https://svgwg.org/svg2-draft/painting.html#SpecifyingPaint
fn parse_fallback<'i, 't, ColorType: Parse>(context: &ParserContext,
                                            input: &mut Parser<'i, 't>)
                                            -> Option<ColorType> {
    if input.try(|i| i.expect_ident_matching("none")).is_ok() {
        None
    } else {
        input.try(|i| ColorType::parse(context, i)).ok()
    }
}

impl<ColorType: Parse, UrlPaintServer: Parse> Parse for SVGPaint<ColorType, UrlPaintServer> {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(url) = input.try(|i| UrlPaintServer::parse(context, i)) {
            Ok(SVGPaint {
                kind: SVGPaintKind::PaintServer(url),
                fallback: parse_fallback(context, input),
            })
        } else if let Ok(kind) = input.try(SVGPaintKind::parse_ident) {
            if let SVGPaintKind::None = kind {
                Ok(SVGPaint {
                    kind: kind,
                    fallback: None,
                })
            } else {
                Ok(SVGPaint {
                    kind: kind,
                    fallback: parse_fallback(context, input),
                })
            }
        } else if let Ok(color) = input.try(|i| ColorType::parse(context, i)) {
            Ok(SVGPaint {
                kind: SVGPaintKind::Color(color),
                fallback: None,
            })
        } else {
            Err(StyleParseError::UnspecifiedError.into())
        }
    }
}

/// A value of <length> | <percentage> | <number> for svg which allow unitless length.
/// https://www.w3.org/TR/SVG11/painting.html#StrokeProperties
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, ToComputedValue, ToCss)]
pub enum SvgLengthOrPercentageOrNumber<L, N> {
    /// <length> | <percentage>
    LengthOrPercentage(L),
    /// <number>
    Number(N),
}

/// Following From implements use for converting animated value.
impl<L, N> ToAnimatedValue for SvgLengthOrPercentageOrNumber<L, N>
    where
        L: Into<AnimatedSvgLengthOrPercentageOrNumber> + From<LengthOrPercentage>,
        N: Into<AnimatedSvgLengthOrPercentageOrNumber> + From<Number>
{
    type AnimatedValue = AnimatedSvgLengthOrPercentageOrNumber;

    #[inline]
    fn to_animated_value(self) -> Self::AnimatedValue {
        match self {
            SvgLengthOrPercentageOrNumber::LengthOrPercentage(lop) => lop.into(),
            SvgLengthOrPercentageOrNumber::Number(num) => num.into(),
        }
    }

    #[inline]
    fn from_animated_value(animated: Self::AnimatedValue) -> Self {
        match animated {
            AnimatedSvgLengthOrPercentageOrNumber::Number(num) =>
            {
                SvgLengthOrPercentageOrNumber::Number(From::from(num))
            },
            AnimatedSvgLengthOrPercentageOrNumber::Percentage(p) =>
            {
                SvgLengthOrPercentageOrNumber::LengthOrPercentage(
                    From::from(LengthOrPercentage::Percentage(p)))
            },
            AnimatedSvgLengthOrPercentageOrNumber::Calc(calc) =>
            {
                SvgLengthOrPercentageOrNumber::LengthOrPercentage(
                    From::from(LengthOrPercentage::Calc(calc)))
            },
        }
    }
}

/// We need to a following conversion bounds for converting to an animated
/// value of SvgLengthOrPercentageOrNumber.
impl From<NonNegativeLengthOrPercentage> for AnimatedSvgLengthOrPercentageOrNumber {
    fn from(lop: NonNegativeLengthOrPercentage) -> AnimatedSvgLengthOrPercentageOrNumber {
        lop.0.into()
    }
}

impl From<NonNegativeNumber> for AnimatedSvgLengthOrPercentageOrNumber {
    fn from(num: NonNegativeNumber) -> AnimatedSvgLengthOrPercentageOrNumber {
        num.0.into()
    }
}

impl From<LengthOrPercentage> for AnimatedSvgLengthOrPercentageOrNumber {
    fn from(lop: LengthOrPercentage) -> AnimatedSvgLengthOrPercentageOrNumber {
        match lop {
            LengthOrPercentage::Length(len) => {
                AnimatedSvgLengthOrPercentageOrNumber::Number(len.to_f32_px())
            },
            LengthOrPercentage::Percentage(p) => {
                AnimatedSvgLengthOrPercentageOrNumber::Percentage(p)
            },
            LengthOrPercentage::Calc(calc) => {
                AnimatedSvgLengthOrPercentageOrNumber::Calc(calc.into())
            },
        }
    }

}

impl From<Number> for AnimatedSvgLengthOrPercentageOrNumber {
    fn from(num: Number) -> AnimatedSvgLengthOrPercentageOrNumber {
        AnimatedSvgLengthOrPercentageOrNumber::Number(num)
    }
}

impl From<AnimatedSvgLengthOrPercentageOrNumber> for CalcLengthOrPercentage {
    fn from(nopoc: AnimatedSvgLengthOrPercentageOrNumber) -> CalcLengthOrPercentage {
        match nopoc {
            AnimatedSvgLengthOrPercentageOrNumber::Number(num) =>
                CalcLengthOrPercentage::new(Au::from_f32_px(num), None),
            AnimatedSvgLengthOrPercentageOrNumber::Percentage(p) =>
                CalcLengthOrPercentage::new(Au(0), Some(p)),
            AnimatedSvgLengthOrPercentageOrNumber::Calc(calc) =>
                calc.into(),
        }
    }
}

/// Parsing the SvgLengthOrPercentageOrNumber. At first, we need to parse number
/// since prevent converting to the length.
impl <L: Parse, N: Parse> Parse for SvgLengthOrPercentageOrNumber<L, N> {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<Self, ParseError<'i>> {
        if let Ok(num) = input.try(|i| N::parse(context, i)) {
            return Ok(SvgLengthOrPercentageOrNumber::Number(num));
        }

        if let Ok(lop) = input.try(|i| L::parse(context, i)) {
            return Ok(SvgLengthOrPercentageOrNumber::LengthOrPercentage(lop));
        }
        Err(StyleParseError::UnspecifiedError.into())
    }
}

/// An SVG length value supports `context-value` in addition to length.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, ComputeSquaredDistance, Copy, Debug, PartialEq)]
#[derive(ToAnimatedValue, ToAnimatedZero)]
#[derive(ToComputedValue, ToCss)]
pub enum SVGLength<LengthType> {
    /// `<length> | <percentage> | <number>`
    Length(LengthType),
    /// `context-value`
    ContextValue,
}

/// Generic value for stroke-dasharray.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, ComputeSquaredDistance, Debug, PartialEq)]
#[derive(ToAnimatedValue, ToComputedValue)]
pub enum SVGStrokeDashArray<LengthType> {
    /// `[ <length> | <percentage> | <number> ]#`
    Values(Vec<LengthType>),
    /// `context-value`
    ContextValue,
}

impl<LengthType> ToCss for SVGStrokeDashArray<LengthType> where LengthType: ToCss {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        match self {
            &SVGStrokeDashArray::Values(ref values) => {
                let mut iter = values.iter();
                if let Some(first) = iter.next() {
                    first.to_css(dest)?;
                    for item in iter {
                        dest.write_str(", ")?;
                        item.to_css(dest)?;
                    }
                    Ok(())
                } else {
                    dest.write_str("none")
                }
            }
            &SVGStrokeDashArray::ContextValue => {
                dest.write_str("context-value")
            }
        }
    }
}

/// An SVG opacity value accepts `context-{fill,stroke}-opacity` in
/// addition to opacity value.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, ComputeSquaredDistance, Copy, Debug)]
#[derive(PartialEq, ToAnimatedZero, ToComputedValue, ToCss)]
pub enum SVGOpacity<OpacityType> {
    /// `<opacity-value>`
    Opacity(OpacityType),
    /// `context-fill-opacity`
    ContextFillOpacity,
    /// `context-stroke-opacity`
    ContextStrokeOpacity,
}
