/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to flexbox.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use style_traits::ParseError;
use values::generics::flex::FlexBasis as GenericFlexBasis;
use values::specified::length::LengthOrPercentage;

/// A specified value for the `flex-basis` property.
pub type FlexBasis = GenericFlexBasis<LengthOrPercentage>;

impl Parse for FlexBasis {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>)
    -> Result<Self, ParseError<'i>> {
        if let Ok(length) = input.try(|i| LengthOrPercentage::parse_non_negative(context, i)) {
            return Ok(GenericFlexBasis::Length(length));
        }
        try_match_ident_ignore_ascii_case! { input.expect_ident()?,
            "auto" => Ok(GenericFlexBasis::Auto),
            "content" => Ok(GenericFlexBasis::Content),
        }
    }
}
