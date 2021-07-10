/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for <ratio>.
//!
//! [ratio]: https://drafts.csswg.org/css-values/#ratios

use crate::parser::{Parse, ParserContext};
use crate::values::generics::ratio::Ratio as GenericRatio;
use crate::values::specified::NonNegativeNumber;
use crate::One;
use cssparser::Parser;
use style_traits::ParseError;

/// A specified <ratio> value.
pub type Ratio = GenericRatio<NonNegativeNumber>;

impl Parse for Ratio {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let a = NonNegativeNumber::parse(context, input)?;
        let b = match input.try_parse(|input| input.expect_delim('/')) {
            Ok(()) => NonNegativeNumber::parse(context, input)?,
            _ => One::one(),
        };

        Ok(GenericRatio(a, b))
    }
}
