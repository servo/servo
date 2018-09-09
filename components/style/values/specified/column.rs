/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for the column properties.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use style_traits::ParseError;
use values::generics::column::ColumnCount as GenericColumnCount;
use values::specified::PositiveInteger;

/// A specified type for `column-count` values.
pub type ColumnCount = GenericColumnCount<PositiveInteger>;

impl Parse for ColumnCount {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("auto")).is_ok() {
            return Ok(GenericColumnCount::Auto);
        }
        Ok(GenericColumnCount::Integer(PositiveInteger::parse(
            context, input,
        )?))
    }
}
