/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values in SVG

use cssparser::Parser;
use parser::{Parse, ParserContext};
use style_traits::{ParseError, StyleParseErrorKind};
use values::{Either, None_};
use values::computed::NumberOrPercentage;
use values::computed::length::LengthOrPercentage;
use values::distance::{ComputeSquaredDistance, SquaredDistance};

/// An SVG paint value
///
/// <https://www.w3.org/TR/SVG2/painting.html#SpecifyingPaint>
#[animation(no_bound(UrlPaintServer))]
#[derive(Animate, Clone, ComputeSquaredDistance, Debug, MallocSizeOf, PartialEq,
         SpecifiedValueInfo, ToAnimatedValue, ToComputedValue, ToCss)]
pub struct SVGPaint<ColorType, UrlPaintServer> {
    /// The paint source
    pub kind: SVGPaintKind<ColorType, UrlPaintServer>,
    /// The fallback color. It would be empty, the `none` keyword or <color>.
    pub fallback: Option<Either<ColorType, None_>>,
}

/// An SVG paint value without the fallback
///
/// Whereas the spec only allows PaintServer
/// to have a fallback, Gecko lets the context
/// properties have a fallback as well.
#[animation(no_bound(UrlPaintServer))]
#[derive(Animate, Clone, ComputeSquaredDistance, Debug, MallocSizeOf, PartialEq,
         SpecifiedValueInfo, ToAnimatedValue, ToAnimatedZero, ToComputedValue,
         ToCss)]
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
        try_match_ident_ignore_ascii_case! { input,
            "none" => Ok(SVGPaintKind::None),
            "context-fill" => Ok(SVGPaintKind::ContextFill),
            "context-stroke" => Ok(SVGPaintKind::ContextStroke),
        }
    }
}

/// Parse SVGPaint's fallback.
/// fallback is keyword(none), Color or empty.
/// <https://svgwg.org/svg2-draft/painting.html#SpecifyingPaint>
fn parse_fallback<'i, 't, ColorType: Parse>(
    context: &ParserContext,
    input: &mut Parser<'i, 't>,
) -> Option<Either<ColorType, None_>> {
    if input.try(|i| i.expect_ident_matching("none")).is_ok() {
        Some(Either::Second(None_))
    } else {
        if let Ok(color) = input.try(|i| ColorType::parse(context, i)) {
            Some(Either::First(color))
        } else {
            None
        }
    }
}

impl<ColorType: Parse, UrlPaintServer: Parse> Parse for SVGPaint<ColorType, UrlPaintServer> {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
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
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}

/// A value of <length> | <percentage> | <number> for svg which allow unitless length.
/// <https://www.w3.org/TR/SVG11/painting.html#StrokeProperties>
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToAnimatedValue, ToAnimatedZero, ToComputedValue, ToCss)]
pub enum SvgLengthOrPercentageOrNumber<LengthOrPercentage, Number> {
    /// <length> | <percentage>
    LengthOrPercentage(LengthOrPercentage),
    /// <number>
    Number(Number),
}

impl<L, N> ComputeSquaredDistance for SvgLengthOrPercentageOrNumber<L, N>
where
    L: ComputeSquaredDistance + Copy + Into<NumberOrPercentage>,
    N: ComputeSquaredDistance + Copy + Into<NumberOrPercentage>,
{
    #[inline]
    fn compute_squared_distance(&self, other: &Self) -> Result<SquaredDistance, ()> {
        match (self, other) {
            (
                &SvgLengthOrPercentageOrNumber::LengthOrPercentage(ref from),
                &SvgLengthOrPercentageOrNumber::LengthOrPercentage(ref to),
            ) => from.compute_squared_distance(to),
            (
                &SvgLengthOrPercentageOrNumber::Number(ref from),
                &SvgLengthOrPercentageOrNumber::Number(ref to),
            ) => from.compute_squared_distance(to),
            (
                &SvgLengthOrPercentageOrNumber::LengthOrPercentage(from),
                &SvgLengthOrPercentageOrNumber::Number(to),
            ) => from.into().compute_squared_distance(&to.into()),
            (
                &SvgLengthOrPercentageOrNumber::Number(from),
                &SvgLengthOrPercentageOrNumber::LengthOrPercentage(to),
            ) => from.into().compute_squared_distance(&to.into()),
        }
    }
}

impl<LengthOrPercentageType, NumberType>
    SvgLengthOrPercentageOrNumber<LengthOrPercentageType, NumberType>
where
    LengthOrPercentage: From<LengthOrPercentageType>,
    LengthOrPercentageType: Copy,
{
    /// return true if this struct has calc value.
    pub fn has_calc(&self) -> bool {
        match self {
            &SvgLengthOrPercentageOrNumber::LengthOrPercentage(lop) => {
                match LengthOrPercentage::from(lop) {
                    LengthOrPercentage::Calc(_) => true,
                    _ => false,
                }
            },
            _ => false,
        }
    }
}

/// Parsing the SvgLengthOrPercentageOrNumber. At first, we need to parse number
/// since prevent converting to the length.
impl<LengthOrPercentageType: Parse, NumberType: Parse> Parse
    for SvgLengthOrPercentageOrNumber<LengthOrPercentageType, NumberType>
{
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(num) = input.try(|i| NumberType::parse(context, i)) {
            return Ok(SvgLengthOrPercentageOrNumber::Number(num));
        }

        if let Ok(lop) = input.try(|i| LengthOrPercentageType::parse(context, i)) {
            return Ok(SvgLengthOrPercentageOrNumber::LengthOrPercentage(lop));
        }
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

/// An SVG length value supports `context-value` in addition to length.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq,
         SpecifiedValueInfo, ToAnimatedValue, ToAnimatedZero, ToComputedValue,
         ToCss)]
pub enum SVGLength<LengthType> {
    /// `<length> | <percentage> | <number>`
    Length(LengthType),
    /// `context-value`
    ContextValue,
}

/// Generic value for stroke-dasharray.
#[derive(Clone, ComputeSquaredDistance, Debug, MallocSizeOf, PartialEq,
         SpecifiedValueInfo, ToAnimatedValue, ToComputedValue, ToCss)]
pub enum SVGStrokeDashArray<LengthType> {
    /// `[ <length> | <percentage> | <number> ]#`
    #[css(comma)]
    Values(
        #[css(if_empty = "none", iterable)]
        #[distance(field_bound)]
        Vec<LengthType>,
    ),
    /// `context-value`
    ContextValue,
}

/// An SVG opacity value accepts `context-{fill,stroke}-opacity` in
/// addition to opacity value.
#[derive(Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf, PartialEq,
         SpecifiedValueInfo, ToAnimatedZero, ToComputedValue, ToCss)]
pub enum SVGOpacity<OpacityType> {
    /// `<opacity-value>`
    Opacity(OpacityType),
    /// `context-fill-opacity`
    ContextFillOpacity,
    /// `context-stroke-opacity`
    ContextStrokeOpacity,
}
