/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for legacy Gecko-only properties.

use cssparser::{Parser, Token};
use gecko_bindings::structs;
use gecko_bindings::sugar::ns_css_value::ToNsCssValue;
use parser::{Parse, ParserContext};
use style_traits::{ParseError, StyleParseErrorKind};
use values::CSSFloat;
use values::computed;
use values::generics::gecko::ScrollSnapPoint as GenericScrollSnapPoint;
use values::generics::rect::Rect;
use values::specified::length::LengthOrPercentage;

/// A specified type for scroll snap points.
pub type ScrollSnapPoint = GenericScrollSnapPoint<LengthOrPercentage>;

impl Parse for ScrollSnapPoint {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("none")).is_ok() {
            return Ok(GenericScrollSnapPoint::None);
        }
        input.expect_function_matching("repeat")?;
        let length = input.parse_nested_block(|i| {
            LengthOrPercentage::parse_non_negative(context, i)
        })?;
        Ok(GenericScrollSnapPoint::Repeat(length))
    }
}

/// A component of an IntersectionObserverRootMargin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PixelOrPercentage {
    /// An absolute length in pixels (px)
    Px(CSSFloat),
    /// A percentage (%)
    Percentage(computed::Percentage),
}

impl Parse for PixelOrPercentage {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let token = input.next()?;
        let value = match *token {
            Token::Dimension { value, ref unit, .. } => {
                match_ignore_ascii_case! { unit,
                    "px" => Ok(PixelOrPercentage::Px(value)),
                    _ => Err(()),
                }
            }
            Token::Percentage { unit_value, .. } => {
                Ok(PixelOrPercentage::Percentage(
                    computed::Percentage(unit_value)
                ))
            }
            _ => Err(()),
        };
        value.map_err(|()| {
            location.new_custom_error(StyleParseErrorKind::UnspecifiedError)
        })
    }
}

impl ToNsCssValue for PixelOrPercentage {
    fn convert(self, nscssvalue: &mut structs::nsCSSValue) {
        match self {
            PixelOrPercentage::Px(px) => {
                unsafe { nscssvalue.set_px(px); }
            }
            PixelOrPercentage::Percentage(pc) => {
                unsafe { nscssvalue.set_percentage(pc.0); }
            }
        }
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
