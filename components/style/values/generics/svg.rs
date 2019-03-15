/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values in SVG

use crate::parser::{Parse, ParserContext};
use crate::values::{Either, None_};
use cssparser::Parser;
use style_traits::{ParseError, StyleParseErrorKind};

/// An SVG paint value
///
/// <https://www.w3.org/TR/SVG2/painting.html#SpecifyingPaint>
#[animation(no_bound(UrlPaintServer))]
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
)]
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
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
)]
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
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
)]
pub enum SvgLengthPercentageOrNumber<LengthPercentage, Number> {
    /// <number>
    ///
    /// Note that this needs to be before, so it gets parsed before the length,
    /// to handle `0` correctly as a number instead of a `<length>`.
    Number(Number),
    /// <length> | <percentage>
    LengthPercentage(LengthPercentage),
}

/// Whether the `context-value` value is enabled.
#[cfg(feature = "gecko")]
pub fn is_context_value_enabled(_: &ParserContext) -> bool {
    use crate::gecko_bindings::structs::mozilla;
    unsafe { mozilla::StaticPrefs_sVarCache_gfx_font_rendering_opentype_svg_enabled }
}

/// Whether the `context-value` value is enabled.
#[cfg(not(feature = "gecko"))]
pub fn is_context_value_enabled(_: &ParserContext) -> bool {
    false
}

/// An SVG length value supports `context-value` in addition to length.
#[derive(
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
)]
pub enum SVGLength<LengthType> {
    /// `<length> | <percentage> | <number>`
    Length(LengthType),
    /// `context-value`
    #[parse(condition = "is_context_value_enabled")]
    ContextValue,
}

/// Generic value for stroke-dasharray.
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToComputedValue,
    ToCss,
)]
pub enum SVGStrokeDashArray<LengthType> {
    /// `[ <length> | <percentage> | <number> ]#`
    #[css(comma)]
    Values(#[css(if_empty = "none", iterable)] Vec<LengthType>),
    /// `context-value`
    ContextValue,
}

/// An SVG opacity value accepts `context-{fill,stroke}-opacity` in
/// addition to opacity value.
#[derive(
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    Parse,
    SpecifiedValueInfo,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
)]
pub enum SVGOpacity<OpacityType> {
    /// `<opacity-value>`
    Opacity(OpacityType),
    /// `context-fill-opacity`
    ContextFillOpacity,
    /// `context-stroke-opacity`
    ContextStrokeOpacity,
}
