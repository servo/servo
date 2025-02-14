/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Copy of Stylo Gecko's [`style::values::specified::gecko::IntersectionObserverRootMargin`] implementation.
//! TODO: expose the object to servo as well in Stylo

use std::fmt;

use cssparser::{match_ignore_ascii_case, Parser, Token};
use style::parser::{Parse, ParserContext};
use style::values::computed::{self, Length, LengthPercentage};
use style::values::generics::rect::Rect;
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
pub struct IntersectionObserverRootMargin(pub Rect<LengthPercentage>);

impl Parse for IntersectionObserverRootMargin {
    fn parse<'i>(
        context: &ParserContext,
        input: &mut Parser<'i, '_>,
    ) -> Result<Self, ParseError<'i>> {
        use style::Zero;
        if input.is_exhausted() {
            // If there are zero elements in tokens, set tokens to ["0px"].
            return Ok(IntersectionObserverRootMargin(Rect::all(
                LengthPercentage::zero(),
            )));
        }
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
