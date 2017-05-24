/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed types for CSS borders.

use cssparser::Parser;
use parser::ParserContext;
use values::generics::rect::Rect;
use values::specified::length::LengthOrNumber;

/// A specified rectangle made of four `<length-or-number>` values.
pub type LengthOrNumberRect = Rect<LengthOrNumber>;

impl LengthOrNumberRect {
    /// Parses a `LengthOrNumberRect`, rejecting negative values.
    #[inline]
    pub fn parse_non_negative(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        Rect::parse_with(context, input, LengthOrNumber::parse_non_negative)
    }
}
