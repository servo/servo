/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for legacy Gecko-only properties.

use crate::parser::{Parse, ParserContext};
use crate::values::computed::length::CSSPixelLength;
use crate::values::computed::{self, LengthPercentage};
use crate::values::generics::gecko::ScrollSnapPoint as GenericScrollSnapPoint;
use crate::values::generics::rect::Rect;
use crate::values::specified::length::LengthPercentage as SpecifiedLengthPercentage;
use cssparser::{Parser, Token};
use std::fmt;
use style_traits::values::SequenceWriter;
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

/// A specified type for scroll snap points.
pub type ScrollSnapPoint = GenericScrollSnapPoint<SpecifiedLengthPercentage>;

impl Parse for ScrollSnapPoint {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(GenericScrollSnapPoint::None);
        }
        input.expect_function_matching("repeat")?;
        // FIXME(emilio): This won't clamp properly when animating.
        let length = input
            .parse_nested_block(|i| SpecifiedLengthPercentage::parse_non_negative(context, i))?;
        Ok(GenericScrollSnapPoint::Repeat(length))
    }
}

fn parse_pixel_or_percent<'i, 't>(
    _context: &ParserContext,
    input: &mut Parser<'i, 't>,
) -> Result<LengthPercentage, ParseError<'i>> {
    let location = input.current_source_location();
    let token = input.next()?;
    let value = match *token {
        Token::Dimension {
            value, ref unit, ..
        } => {
            match_ignore_ascii_case! { unit,
                "px" => Ok(LengthPercentage::new(CSSPixelLength::new(value), None)),
                _ => Err(()),
            }
        },
        Token::Percentage { unit_value, .. } => Ok(LengthPercentage::new_percent(
            computed::Percentage(unit_value),
        )),
        _ => Err(()),
    };
    value.map_err(|()| location.new_custom_error(StyleParseErrorKind::UnspecifiedError))
}

/// The value of an IntersectionObserver's rootMargin property.
///
/// Only bare px or percentage values are allowed. Other length units and
/// calc() values are not allowed.
///
/// <https://w3c.github.io/IntersectionObserver/#parse-a-root-margin>
#[repr(transparent)]
pub struct IntersectionObserverRootMargin(pub Rect<LengthPercentage>);

impl Parse for IntersectionObserverRootMargin {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let rect = Rect::parse_with(context, input, parse_pixel_or_percent)?;
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
