/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for SVG properties.

use crate::parser::{Parse, ParserContext};
use crate::values::generics::svg as generic;
use crate::values::specified::color::Color;
use crate::values::specified::url::SpecifiedUrl;
use crate::values::specified::AllowQuirks;
use crate::values::specified::LengthPercentage;
use crate::values::specified::{NonNegativeLengthPercentage, Opacity};
use crate::values::CustomIdent;
use cssparser::{Parser, Token};
use std::fmt::{self, Write};
use style_traits::{CommaWithSpace, CssWriter, ParseError, Separator};
use style_traits::{StyleParseErrorKind, ToCss};

/// Specified SVG Paint value
pub type SVGPaint = generic::GenericSVGPaint<Color, SpecifiedUrl>;

/// <length> | <percentage> | <number> | context-value
pub type SVGLength = generic::SVGLength<LengthPercentage>;

/// A non-negative version of SVGLength.
pub type SVGWidth = generic::SVGLength<NonNegativeLengthPercentage>;

/// [ <length> | <percentage> | <number> ]# | context-value
pub type SVGStrokeDashArray = generic::SVGStrokeDashArray<NonNegativeLengthPercentage>;

/// Whether the `context-value` value is enabled.
#[cfg(feature = "gecko")]
pub fn is_context_value_enabled() -> bool {
    static_prefs::pref!("gfx.font_rendering.opentype_svg.enabled")
}

/// Whether the `context-value` value is enabled.
#[cfg(not(feature = "gecko"))]
pub fn is_context_value_enabled() -> bool {
    false
}

macro_rules! parse_svg_length {
    ($ty:ty, $lp:ty) => {
        impl Parse for $ty {
            fn parse<'i, 't>(
                context: &ParserContext,
                input: &mut Parser<'i, 't>,
            ) -> Result<Self, ParseError<'i>> {
                if let Ok(lp) = input.try(|i| <$lp>::parse_quirky(context, i, AllowQuirks::Always))
                {
                    return Ok(generic::SVGLength::LengthPercentage(lp));
                }

                try_match_ident_ignore_ascii_case! { input,
                    "context-value" if is_context_value_enabled() => {
                        Ok(generic::SVGLength::ContextValue)
                    },
                }
            }
        }
    };
}

parse_svg_length!(SVGLength, LengthPercentage);
parse_svg_length!(SVGWidth, NonNegativeLengthPercentage);

impl Parse for SVGStrokeDashArray {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(values) = input.try(|i| {
            CommaWithSpace::parse(i, |i| {
                NonNegativeLengthPercentage::parse_quirky(context, i, AllowQuirks::Always)
            })
        }) {
            return Ok(generic::SVGStrokeDashArray::Values(values.into()));
        }

        try_match_ident_ignore_ascii_case! { input,
            "context-value" if is_context_value_enabled() => {
                Ok(generic::SVGStrokeDashArray::ContextValue)
            },
            "none" => Ok(generic::SVGStrokeDashArray::Values(Default::default())),
        }
    }
}

/// <opacity-value> | context-fill-opacity | context-stroke-opacity
pub type SVGOpacity = generic::SVGOpacity<Opacity>;

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
pub const PAINT_ORDER_COUNT: u8 = 3;

/// Number of bits for each component
pub const PAINT_ORDER_SHIFT: u8 = 2;

/// Mask with above bits set
pub const PAINT_ORDER_MASK: u8 = 0b11;

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
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(transparent)]
pub struct SVGPaintOrder(pub u8);

impl SVGPaintOrder {
    /// Get default `paint-order` with `0`
    pub fn normal() -> Self {
        SVGPaintOrder(0)
    }

    /// Get variant of `paint-order`
    pub fn order_at(&self, pos: u8) -> PaintOrder {
        // Safe because PaintOrder covers all possible patterns.
        unsafe { std::mem::transmute((self.0 >> pos * PAINT_ORDER_SHIFT) & PAINT_ORDER_MASK) }
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
            for paint in 1..(PAINT_ORDER_COUNT + 1) {
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

bitflags! {
    /// The context properties we understand.
    #[derive(Default, MallocSizeOf, SpecifiedValueInfo, ToComputedValue, ToResolvedValue, ToShmem)]
    #[repr(C)]
    pub struct ContextPropertyBits: u8 {
        /// `fill`
        const FILL = 1 << 0;
        /// `stroke`
        const STROKE = 1 << 1;
        /// `fill-opacity`
        const FILL_OPACITY = 1 << 2;
        /// `stroke-opacity`
        const STROKE_OPACITY = 1 << 3;
    }
}

/// Specified MozContextProperties value.
/// Nonstandard (https://developer.mozilla.org/en-US/docs/Web/CSS/-moz-context-properties)
#[derive(
    Clone,
    Debug,
    Default,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct MozContextProperties {
    #[css(iterable, if_empty = "none")]
    #[ignore_malloc_size_of = "Arc"]
    idents: crate::ArcSlice<CustomIdent>,
    #[css(skip)]
    bits: ContextPropertyBits,
}

impl Parse for MozContextProperties {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<MozContextProperties, ParseError<'i>> {
        let mut values = vec![];
        let mut bits = ContextPropertyBits::empty();
        loop {
            {
                let location = input.current_source_location();
                let ident = input.expect_ident()?;

                if ident.eq_ignore_ascii_case("none") && values.is_empty() {
                    return Ok(Self::default());
                }

                let ident = CustomIdent::from_ident(location, ident, &["all", "none", "auto"])?;

                if ident.0 == atom!("fill") {
                    bits.insert(ContextPropertyBits::FILL);
                } else if ident.0 == atom!("stroke") {
                    bits.insert(ContextPropertyBits::STROKE);
                } else if ident.0 == atom!("fill-opacity") {
                    bits.insert(ContextPropertyBits::FILL_OPACITY);
                } else if ident.0 == atom!("stroke-opacity") {
                    bits.insert(ContextPropertyBits::STROKE_OPACITY);
                }

                values.push(ident);
            }

            let location = input.current_source_location();
            match input.next() {
                Ok(&Token::Comma) => continue,
                Err(..) => break,
                Ok(other) => return Err(location.new_unexpected_token_error(other.clone())),
            }
        }

        if values.is_empty() {
            return Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        Ok(MozContextProperties {
            idents: crate::ArcSlice::from_iter(values.into_iter()),
            bits,
        })
    }
}
