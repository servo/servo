/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for table properties.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use style_traits::{StyleParseErrorKind, ParseError};

#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
/// span. for `<col span>` pres attr
pub struct XSpan(#[css(skip)] pub i32);

impl Parse for XSpan {
    // never parse it, only set via presentation attribute
    fn parse<'i, 't>(_: &ParserContext, input: &mut Parser<'i, 't>) -> Result<XSpan, ParseError<'i>> {
        Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError))
    }
}
