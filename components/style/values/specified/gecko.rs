/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for legacy Gecko-only properties.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use values::generics::gecko::ScrollSnapPoint as GenericScrollSnapPoint;
use values::specified::length::LengthOrPercentage;

/// A specified type for scroll snap points.
pub type ScrollSnapPoint = GenericScrollSnapPoint<LengthOrPercentage>;

impl Parse for ScrollSnapPoint {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
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
