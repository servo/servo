/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for legacy Gecko-only properties.

use crate::gecko::values::GeckoStyleCoordConvertible;
use crate::gecko_bindings::sugar::ns_style_coord::{CoordData, CoordDataMut};
use crate::parser::{Parse, ParserContext};
use crate::values::computed;
use crate::values::computed::length::CSSPixelLength;
use crate::values::generics::gecko::ScrollSnapPoint as GenericScrollSnapPoint;
use crate::values::generics::rect::Rect;
use crate::values::specified::length::LengthOrPercentage;
use cssparser::{Parser, Token};
use std::fmt;
use style_traits::values::SequenceWriter;
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

/// A specified type for scroll snap points.
pub type ScrollSnapPoint = GenericScrollSnapPoint<LengthOrPercentage>;

impl Parse for ScrollSnapPoint {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(GenericScrollSnapPoint::None);
        }
        input.expect_function_matching("repeat")?;
        let length =
            input.parse_nested_block(|i| LengthOrPercentage::parse_non_negative(context, i))?;
        Ok(GenericScrollSnapPoint::Repeat(length))
    }
}

/// A component of an IntersectionObserverRootMargin.
#[derive(Clone, Copy, Debug, PartialEq, ToCss)]
pub enum PixelOrPercentage {
    /// An absolute length in pixels (px)
    Pixel(CSSPixelLength),
    /// A percentage (%)
    Percentage(computed::Percentage),
}

impl Parse for PixelOrPercentage {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let token = input.next()?;
        let value = match *token {
            Token::Dimension {
                value, ref unit, ..
            } => {
                match_ignore_ascii_case! { unit,
                    "px" => Ok(PixelOrPercentage::Pixel(CSSPixelLength::new(value))),
                    _ => Err(()),
                }
            },
            Token::Percentage { unit_value, .. } => Ok(PixelOrPercentage::Percentage(
                computed::Percentage(unit_value),
            )),
            _ => Err(()),
        };
        value.map_err(|()| location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}

impl GeckoStyleCoordConvertible for PixelOrPercentage {
    fn to_gecko_style_coord<T: CoordDataMut>(&self, coord: &mut T) {
        match *self {
            PixelOrPercentage::Pixel(ref l) => l.to_gecko_style_coord(coord),
            PixelOrPercentage::Percentage(ref pc) => pc.to_gecko_style_coord(coord),
        }
    }

    fn from_gecko_style_coord<T: CoordData>(coord: &T) -> Option<Self> {
        CSSPixelLength::from_gecko_style_coord(coord)
            .map(PixelOrPercentage::Pixel)
            .or_else(|| {
                computed::Percentage::from_gecko_style_coord(coord)
                    .map(PixelOrPercentage::Percentage)
            })
    }
}

/// The value of an IntersectionObserver's rootMargin property.
///
/// Only bare px or percentage values are allowed. Other length units and
/// calc() values are not allowed.
///
/// <https://w3c.github.io/IntersectionObserver/#parse-a-root-margin>
pub struct IntersectionObserverRootMargin(pub Rect<PixelOrPercentage>);

impl Parse for IntersectionObserverRootMargin {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let rect = Rect::parse_with(context, input, PixelOrPercentage::parse)?;
        Ok(IntersectionObserverRootMargin(rect))
    }
}

// Strictly speaking this is not ToCss. It's serializing for DOM. But
// we can just reuse the infrastructure of this.
//
// <https://w3c.github.io/IntersectionObserver/#dom-intersectionobserver-rootmargin>
impl ToCss for IntersectionObserverRootMargin {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: fmt::Write,
    {
        // We cannot use the ToCss impl of Rect, because that would
        // merge items when they are equal. We want to list them all.
        let mut writer = SequenceWriter::new(dest, " ");
        let rect = &self.0;
        writer.item(&rect.0)?;
        writer.item(&rect.1)?;
        writer.item(&rect.2)?;
        writer.item(&rect.3)
    }
}
