/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified values for outline properties

use cssparser::Parser;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseErrorKind;
use style_traits::ParseError;
use values::specified::BorderStyle;

#[derive(Clone, Copy, Debug, Eq, MallocSizeOf, Ord)]
#[derive(PartialEq, PartialOrd, ToComputedValue, ToCss)]
/// <https://drafts.csswg.org/css-ui/#propdef-outline-style>
pub enum OutlineStyle {
    /// auto
    Auto,
    /// <border-style>
    Other(BorderStyle),
}

impl OutlineStyle {
    #[inline]
    /// Get default value as None
    pub fn none() -> OutlineStyle {
        OutlineStyle::Other(BorderStyle::None)
    }

    #[inline]
    /// Get value for None or Hidden
    pub fn none_or_hidden(&self) -> bool {
        match *self {
            OutlineStyle::Auto => false,
            OutlineStyle::Other(ref border_style) => border_style.none_or_hidden()
        }
    }
}

impl Parse for OutlineStyle {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<OutlineStyle, ParseError<'i>> {
        if let Ok(border_style) = input.try(|i| BorderStyle::parse(context, i)) {
            if let BorderStyle::Hidden = border_style {
                return Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent("hidden".into())));
            }

            return Ok(OutlineStyle::Other(border_style));
        }

        input.expect_ident_matching("auto")?;
        Ok(OutlineStyle::Auto)
    }
}
