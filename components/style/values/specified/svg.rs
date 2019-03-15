/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for SVG properties.

use crate::parser::{Parse, ParserContext};
use crate::values::generics::svg as generic;
use crate::values::specified::color::Color;
use crate::values::specified::url::SpecifiedUrl;
use crate::values::specified::LengthPercentage;
use crate::values::specified::{NonNegativeLengthPercentage, NonNegativeNumber};
use crate::values::specified::{Number, Opacity};
use crate::values::CustomIdent;
use cssparser::Parser;
use std::fmt::{self, Write};
use style_traits::{CommaWithSpace, CssWriter, ParseError, Separator};
use style_traits::{StyleParseErrorKind, ToCss};

/// Specified SVG Paint value
pub type SVGPaint = generic::SVGPaint<Color, SpecifiedUrl>;

/// Specified SVG Paint Kind value
pub type SVGPaintKind = generic::SVGPaintKind<Color, SpecifiedUrl>;

#[cfg(feature = "gecko")]
fn is_context_value_enabled() -> bool {
    // The prefs can only be mutated on the main thread, so it is safe
    // to read whenever we are on the main thread or the main thread is
    // blocked.
    use crate::gecko_bindings::structs::mozilla;
    unsafe { mozilla::StaticPrefs_sVarCache_gfx_font_rendering_opentype_svg_enabled }
}
#[cfg(not(feature = "gecko"))]
fn is_context_value_enabled() -> bool {
    false
}

fn parse_context_value<'i, 't, T>(
    input: &mut Parser<'i, 't>,
    value: T,
) -> Result<T, ParseError<'i>> {
    if !is_context_value_enabled() {
        return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
    }

    input.expect_ident_matching("context-value")?;
    Ok(value)
}

/// A value of <length> | <percentage> | <number> for stroke-dashoffset.
/// <https://www.w3.org/TR/SVG11/painting.html#StrokeProperties>
pub type SvgLengthPercentageOrNumber =
    generic::SvgLengthPercentageOrNumber<LengthPercentage, Number>;

/// <length> | <percentage> | <number> | context-value
pub type SVGLength = generic::SVGLength<SvgLengthPercentageOrNumber>;

impl Parse for SVGLength {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input
            .try(|i| SvgLengthPercentageOrNumber::parse(context, i))
            .map(Into::into)
            .or_else(|_| parse_context_value(input, generic::SVGLength::ContextValue))
    }
}

impl From<SvgLengthPercentageOrNumber> for SVGLength {
    fn from(length: SvgLengthPercentageOrNumber) -> Self {
        generic::SVGLength::Length(length)
    }
}

/// A value of <length> | <percentage> | <number> for stroke-width/stroke-dasharray.
/// <https://www.w3.org/TR/SVG11/painting.html#StrokeProperties>
pub type NonNegativeSvgLengthPercentageOrNumber =
    generic::SvgLengthPercentageOrNumber<NonNegativeLengthPercentage, NonNegativeNumber>;

/// A non-negative version of SVGLength.
pub type SVGWidth = generic::SVGLength<NonNegativeSvgLengthPercentageOrNumber>;

impl Parse for SVGWidth {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input
            .try(|i| NonNegativeSvgLengthPercentageOrNumber::parse(context, i))
            .map(Into::into)
            .or_else(|_| parse_context_value(input, generic::SVGLength::ContextValue))
    }
}

impl From<NonNegativeSvgLengthPercentageOrNumber> for SVGWidth {
    fn from(length: NonNegativeSvgLengthPercentageOrNumber) -> Self {
        generic::SVGLength::Length(length)
    }
}

/// [ <length> | <percentage> | <number> ]# | context-value
pub type SVGStrokeDashArray = generic::SVGStrokeDashArray<NonNegativeSvgLengthPercentageOrNumber>;

impl Parse for SVGStrokeDashArray {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(values) = input.try(|i| {
            CommaWithSpace::parse(i, |i| {
                NonNegativeSvgLengthPercentageOrNumber::parse(context, i)
            })
        }) {
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
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(opacity) = input.try(|i| Opacity::parse(context, i)) {
            return Ok(generic::SVGOpacity::Opacity(opacity));
        }

        try_match_ident_ignore_ascii_case! { input,
            "context-fill-opacity" => Ok(generic::SVGOpacity::ContextFillOpacity),
            "context-stroke-opacity" => Ok(generic::SVGOpacity::ContextStrokeOpacity),
        }
    }
}

/// The specified value for a single CSS paint-order property.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, ToCss)]
pub enum PaintOrder {
    /// `normal` variant
    Normal = 0,
    /// `fill` variant
    Fill = 1,
    /// `stroke` variant
    Stroke = 2,
    /// `markers` variant
    Markers = 3,
}

/// Number of non-normal components
const PAINT_ORDER_COUNT: u8 = 3;

/// Number of bits for each component
const PAINT_ORDER_SHIFT: u8 = 2;

/// Mask with above bits set
const PAINT_ORDER_MASK: u8 = 0b11;

/// The specified value is tree `PaintOrder` values packed into the
/// bitfields below, as a six-bit field, of 3 two-bit pairs
///
/// Each pair can be set to FILL, STROKE, or MARKERS
/// Lowest significant bit pairs are highest priority.
///  `normal` is the empty bitfield. The three pairs are
/// never zero in any case other than `normal`.
///
/// Higher priority values, i.e. the values specified first,
/// will be painted first (and may be covered by paintings of lower priority)
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToComputedValue)]
pub struct SVGPaintOrder(pub u8);

impl SVGPaintOrder {
    /// Get default `paint-order` with `0`
    pub fn normal() -> Self {
        SVGPaintOrder(0)
    }

    /// Get variant of `paint-order`
    pub fn order_at(&self, pos: u8) -> PaintOrder {
        // Safe because PaintOrder covers all possible patterns.
        unsafe { ::std::mem::transmute((self.0 >> pos * PAINT_ORDER_SHIFT) & PAINT_ORDER_MASK) }
    }
}

impl Parse for SVGPaintOrder {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<SVGPaintOrder, ParseError<'i>> {
        if let Ok(()) = input.try(|i| i.expect_ident_matching("normal")) {
            return Ok(SVGPaintOrder::normal());
        }

        let mut value = 0;
        // bitfield representing what we've seen so far
        // bit 1 is fill, bit 2 is stroke, bit 3 is markers
        let mut seen = 0;
        let mut pos = 0;

        loop {
            let result: Result<_, ParseError> = input.try(|input| {
                try_match_ident_ignore_ascii_case! { input,
                    "fill" => Ok(PaintOrder::Fill),
                    "stroke" => Ok(PaintOrder::Stroke),
                    "markers" => Ok(PaintOrder::Markers),
                }
            });

            match result {
                Ok(val) => {
                    if (seen & (1 << val as u8)) != 0 {
                        // don't parse the same ident twice
                        return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
                    }

                    value |= (val as u8) << (pos * PAINT_ORDER_SHIFT);
                    seen |= 1 << (val as u8);
                    pos += 1;
                },
                Err(_) => break,
            }
        }

        if value == 0 {
            // Couldn't find any keyword
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        // fill in rest
        for i in pos..PAINT_ORDER_COUNT {
            for paint in 0..PAINT_ORDER_COUNT {
                // if not seen, set bit at position, mark as seen
                if (seen & (1 << paint)) == 0 {
                    seen |= 1 << paint;
                    value |= paint << (i * PAINT_ORDER_SHIFT);
                    break;
                }
            }
        }

        Ok(SVGPaintOrder(value))
    }
}

impl ToCss for SVGPaintOrder {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        if self.0 == 0 {
            return dest.write_str("normal");
        }

        let mut last_pos_to_serialize = 0;
        for i in (1..PAINT_ORDER_COUNT).rev() {
            let component = self.order_at(i);
            let earlier_component = self.order_at(i - 1);
            if component < earlier_component {
                last_pos_to_serialize = i - 1;
                break;
            }
        }

        for pos in 0..last_pos_to_serialize + 1 {
            if pos != 0 {
                dest.write_str(" ")?
            }
            self.order_at(pos).to_css(dest)?;
        }
        Ok(())
    }
}

/// Specified MozContextProperties value.
/// Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-context-properties)
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToComputedValue, ToCss)]
pub struct MozContextProperties(pub CustomIdent);

impl Parse for MozContextProperties {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<MozContextProperties, ParseError<'i>> {
        let location = input.current_source_location();
        let i = input.expect_ident()?;
        Ok(MozContextProperties(CustomIdent::from_ident(
            location,
            i,
            &["all", "none", "auto"],
        )?))
    }
}
