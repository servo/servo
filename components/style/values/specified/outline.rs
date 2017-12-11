/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified values for outline properties

use cssparser::Parser;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseErrorKind;
use style_traits::ParseError;
use values::{Auto, Either};
use values::specified::BorderStyle;

/// <https://drafts.csswg.org/css-ui/#propdef-outline-style>
pub type OutlineStyle = Either<Auto, BorderStyle>;

impl OutlineStyle {
    #[inline]
    /// Get default value as None
    pub fn second() -> OutlineStyle {
        Either::Second(BorderStyle::None)
    }

    #[inline]
    /// Get value for None or Hidden
    pub fn none_or_hidden(&self) -> bool {
        match *self {
            Either::First(ref _auto) => false,
            Either::Second(ref border_style) => border_style.none_or_hidden()
        }
    }

    /// Parse outline-style
    pub fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<OutlineStyle, ParseError<'i>> {
        if let Ok(result) = input.try(|i| BorderStyle::parse(context, i)) {
            match result {
                BorderStyle::Hidden => {
                    Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent("hidden".into())))
                },
                _ => Ok(Either::Second(result))
            }
        } else {
            Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent("hidden".into())))
        }
    }
}
