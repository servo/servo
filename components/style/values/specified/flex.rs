/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to flexbox.

use crate::parser::{Parse, ParserContext};
use crate::values::generics::flex::FlexBasis as GenericFlexBasis;
use cssparser::Parser;
use style_traits::ParseError;

/// The `width` value type.
#[cfg(feature = "servo")]
pub type Width = crate::values::specified::NonNegativeLengthOrPercentageOrAuto;

/// The `width` value type.
#[cfg(feature = "gecko")]
pub type Width = crate::values::specified::MozLength;

/// A specified value for the `flex-basis` property.
pub type FlexBasis = GenericFlexBasis<Width>;

impl Parse for FlexBasis {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if let Ok(width) = input.try(|i| Width::parse(context, i)) {
            return Ok(GenericFlexBasis::Width(width));
        }
        try_match_ident_ignore_ascii_case! { input,
            "content" => Ok(GenericFlexBasis::Content),
        }
    }
}

impl FlexBasis {
    /// `auto`
    #[inline]
    pub fn auto() -> Self {
        GenericFlexBasis::Width(Width::auto())
    }

    /// `0%`
    #[inline]
    pub fn zero_percent() -> Self {
        GenericFlexBasis::Width(Width::zero_percent())
    }
}
