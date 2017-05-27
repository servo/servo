/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to backgrounds.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use values::generics::background::BackgroundSize as GenericBackgroundSize;
use values::specified::length::LengthOrPercentageOrAuto;

/// A specified value for the `background-size` property.
pub type BackgroundSize = GenericBackgroundSize<LengthOrPercentageOrAuto>;

impl Parse for BackgroundSize {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        if let Ok(width) = input.try(|i| LengthOrPercentageOrAuto::parse_non_negative(context, i)) {
            let height = input
                .try(|i| LengthOrPercentageOrAuto::parse_non_negative(context, i))
                .unwrap_or(LengthOrPercentageOrAuto::Auto);
            return Ok(GenericBackgroundSize::Explicit { width: width, height: height });
        }
        match_ignore_ascii_case! { &input.expect_ident()?,
            "cover" => Ok(GenericBackgroundSize::Cover),
            "contain" => Ok(GenericBackgroundSize::Contain),
            _ => Err(()),
        }
    }
}
