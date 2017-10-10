/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for SVG properties.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use style_traits::{CommaWithSpace, ParseError, Separator, StyleParseErrorKind};
use values::generics::svg as generic;
use values::specified::{LengthOrPercentage, NonNegativeLengthOrPercentage, NonNegativeNumber};
use values::specified::{Number, Opacity, SpecifiedUrl};
use values::specified::color::RGBAColor;

/// Specified SVG Paint value
pub type SVGPaint = generic::SVGPaint<RGBAColor, SpecifiedUrl>;


/// Specified SVG Paint Kind value
pub type SVGPaintKind = generic::SVGPaintKind<RGBAColor, SpecifiedUrl>;

#[cfg(feature = "gecko")]
fn is_context_value_enabled() -> bool {
    // The prefs can only be mutated on the main thread, so it is safe
    // to read whenever we are on the main thread or the main thread is
    // blocked.
    use gecko_bindings::structs::mozilla;
    unsafe { mozilla::StylePrefs_sOpentypeSVGEnabled }
}
#[cfg(not(feature = "gecko"))]
fn is_context_value_enabled() -> bool {
    false
}

fn parse_context_value<'i, 't, T>(input: &mut Parser<'i, 't>, value: T)
                                  -> Result<T, ParseError<'i>> {
    if is_context_value_enabled() {
        if input.expect_ident_matching("context-value").is_ok() {
            return Ok(value);
        }
    }
    Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
}

/// A value of <length> | <percentage> | <number> for stroke-dashoffset.
/// https://www.w3.org/TR/SVG11/painting.html#StrokeProperties
pub type SvgLengthOrPercentageOrNumber =
    generic::SvgLengthOrPercentageOrNumber<LengthOrPercentage, Number>;

/// <length> | <percentage> | <number> | context-value
pub type SVGLength = generic::SVGLength<SvgLengthOrPercentageOrNumber>;

impl Parse for SVGLength {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<Self, ParseError<'i>> {
        input.try(|i| SvgLengthOrPercentageOrNumber::parse(context, i))
             .map(Into::into)
             .or_else(|_| parse_context_value(input, generic::SVGLength::ContextValue))
    }
}

impl From<SvgLengthOrPercentageOrNumber> for SVGLength {
    fn from(length: SvgLengthOrPercentageOrNumber) -> Self {
        generic::SVGLength::Length(length)
    }
}

/// A value of <length> | <percentage> | <number> for stroke-width/stroke-dasharray.
/// https://www.w3.org/TR/SVG11/painting.html#StrokeProperties
pub type NonNegativeSvgLengthOrPercentageOrNumber =
    generic::SvgLengthOrPercentageOrNumber<NonNegativeLengthOrPercentage, NonNegativeNumber>;

/// A non-negative version of SVGLength.
pub type SVGWidth = generic::SVGLength<NonNegativeSvgLengthOrPercentageOrNumber>;

impl Parse for SVGWidth {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<Self, ParseError<'i>> {
        input.try(|i| NonNegativeSvgLengthOrPercentageOrNumber::parse(context, i))
             .map(Into::into)
             .or_else(|_| parse_context_value(input, generic::SVGLength::ContextValue))
    }
}

impl From<NonNegativeSvgLengthOrPercentageOrNumber> for SVGWidth {
    fn from(length: NonNegativeSvgLengthOrPercentageOrNumber) -> Self {
        generic::SVGLength::Length(length)
    }
}

/// [ <length> | <percentage> | <number> ]# | context-value
pub type SVGStrokeDashArray = generic::SVGStrokeDashArray<NonNegativeSvgLengthOrPercentageOrNumber>;

impl Parse for SVGStrokeDashArray {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<Self, ParseError<'i>> {
        if let Ok(values) = input.try(|i| CommaWithSpace::parse(i, |i| {
            NonNegativeSvgLengthOrPercentageOrNumber::parse(context, i)
        })) {
            Ok(generic::SVGStrokeDashArray::Values(values))
        } else if let Ok(_) = input.try(|i| i.expect_ident_matching("none")) {
            Ok(generic::SVGStrokeDashArray::Values(vec![]))
        } else {
            parse_context_value(input, generic::SVGStrokeDashArray::ContextValue)
        }
    }
}

/// <opacity-value> | context-fill-opacity | context-stroke-opacity
pub type SVGOpacity = generic::SVGOpacity<Opacity>;

impl Parse for SVGOpacity {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                     -> Result<Self, ParseError<'i>> {
        if let Ok(opacity) = input.try(|i| Opacity::parse(context, i)) {
            Ok(generic::SVGOpacity::Opacity(opacity))
        } else if is_context_value_enabled() {
            try_match_ident_ignore_ascii_case! { input,
                "context-fill-opacity" => Ok(generic::SVGOpacity::ContextFillOpacity),
                "context-stroke-opacity" => Ok(generic::SVGOpacity::ContextStrokeOpacity),
            }
        } else {
            Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
        }
    }
}
