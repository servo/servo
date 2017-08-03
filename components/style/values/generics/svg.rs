/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic types for CSS values in SVG

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt;
use style_traits::{ParseError, StyleParseError, ToCss};
use values::specified::url::SpecifiedUrl;

/// An SVG paint value
///
/// https://www.w3.org/TR/SVG2/painting.html#SpecifyingPaint
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq, ToAnimatedValue, ToComputedValue, ToCss)]
pub struct SVGPaint<ColorType> {
    /// The paint source
    pub kind: SVGPaintKind<ColorType>,
    /// The fallback color
    pub fallback: Option<ColorType>,
}

/// An SVG paint value without the fallback
///
/// Whereas the spec only allows PaintServer
/// to have a fallback, Gecko lets the context
/// properties have a fallback as well.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq, ToAnimatedValue, ToComputedValue, ToCss)]
pub enum SVGPaintKind<ColorType> {
    /// `none`
    None,
    /// `<color>`
    Color(ColorType),
    /// `url(...)`
    PaintServer(SpecifiedUrl),
    /// `context-fill`
    ContextFill,
    /// `context-stroke`
    ContextStroke,
}

impl<ColorType> SVGPaintKind<ColorType> {
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

impl<ColorType: Parse> Parse for SVGPaint<ColorType> {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(url) = input.try(|i| SpecifiedUrl::parse(context, i)) {
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

/// An SVG length value supports `context-value` in addition to length.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Copy, Debug, PartialEq, HasViewportPercentage, ToComputedValue, ToCss)]
pub enum SVGLength<LengthType> {
    /// `<length> | <percentage> | <number>`
    Length(LengthType),
    /// `context-value`
    ContextValue,
}

/// Generic value for stroke-dasharray.
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
#[derive(Clone, Debug, PartialEq, HasViewportPercentage, ToComputedValue)]
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
#[derive(Clone, Copy, Debug, PartialEq, HasViewportPercentage, ToComputedValue, ToCss)]
pub enum SVGOpacity<OpacityType> {
    /// `<opacity-value>`
    Opacity(OpacityType),
    /// `context-fill-opacity`
    ContextFillOpacity,
    /// `context-stroke-opacity`
    ContextStrokeOpacity,
}
