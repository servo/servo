/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Copy of Stylo Gecko's [`style::values::specified::gecko::IntersectionObserverRootMargin`] implementation.
//! TODO(#35907): make a thin wrapper and remove copied codes

use std::fmt;

use app_units::Au;
use cssparser::{Parser, Token, match_ignore_ascii_case};
use euclid::default::{Rect, SideOffsets2D};
use style::parser::{Parse, ParserContext};
use style::values::computed::{self, Length, LengthPercentage};
use style::values::generics::rect::Rect as StyleRect;
use style_traits::values::SequenceWriter;
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

fn parse_pixel_or_percent<'i>(
    _context: &ParserContext,
    input: &mut Parser<'i, '_>,
) -> Result<LengthPercentage, ParseError<'i>> {
    let location = input.current_source_location();
    let token = input.next()?;
    let value = match *token {
        Token::Dimension {
            value, ref unit, ..
        } => {
            match_ignore_ascii_case! { unit,
                "px" => Ok(LengthPercentage::new_length(Length::new(value))),
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
pub struct IntersectionObserverRootMargin(pub StyleRect<LengthPercentage>);

impl Parse for IntersectionObserverRootMargin {
    fn parse<'i>(
        context: &ParserContext,
        input: &mut Parser<'i, '_>,
    ) -> Result<Self, ParseError<'i>> {
        use style::Zero;
        if input.is_exhausted() {
            // If there are zero elements in tokens, set tokens to ["0px"].
            return Ok(IntersectionObserverRootMargin(StyleRect::all(
                LengthPercentage::zero(),
            )));
        }
        let rect = StyleRect::parse_with(context, input, parse_pixel_or_percent)?;
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

// TODO(stevennovaryo): move this to the wrapper later
impl IntersectionObserverRootMargin {
    // Resolve to used values.
    pub(crate) fn resolve_percentages_with_basis(
        &self,
        containing_block: Rect<Au>,
    ) -> SideOffsets2D<Au> {
        let inner = &self.0;
        SideOffsets2D::new(
            inner.0.to_used_value(containing_block.height()),
            inner.1.to_used_value(containing_block.width()),
            inner.2.to_used_value(containing_block.height()),
            inner.3.to_used_value(containing_block.width()),
        )
    }
}
