/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to backgrounds.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseError;
use style_traits::ParseError;
use values::generics::background::BackgroundSize as GenericBackgroundSize;
use values::specified::length::LengthOrPercentageOrAuto;

/// A specified value for the `background-size` property.
pub type BackgroundSize = GenericBackgroundSize<LengthOrPercentageOrAuto>;

impl Parse for BackgroundSize {
    fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        if let Ok(width) = input.try(|i| LengthOrPercentageOrAuto::parse_non_negative(context, i)) {
            let height = input
                .try(|i| LengthOrPercentageOrAuto::parse_non_negative(context, i))
                .unwrap_or(LengthOrPercentageOrAuto::Auto);
            return Ok(GenericBackgroundSize::Explicit { width: width, height: height });
        }
        let ident = input.expect_ident()?;
        (match_ignore_ascii_case! { &ident,
            "cover" => Ok(GenericBackgroundSize::Cover),
            "contain" => Ok(GenericBackgroundSize::Contain),
            _ => Err(()),
        }).map_err(|()| SelectorParseError::UnexpectedIdent(ident).into())
    }
}
