/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified values for outline properties

use crate::parser::{Parse, ParserContext};
use crate::values::specified::BorderStyle;
use cssparser::Parser;
use selectors::parser::SelectorParseErrorKind;
use style_traits::ParseError;

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Ord,
    PartialEq,
    PartialOrd,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
/// <https://drafts.csswg.org/css-ui/#propdef-outline-style>
pub enum OutlineStyle {
    /// auto
    Auto,
    /// <border-style>
    BorderStyle(BorderStyle),
}

impl OutlineStyle {
    #[inline]
    /// Get default value as None
    pub fn none() -> OutlineStyle {
        OutlineStyle::BorderStyle(BorderStyle::None)
    }

    #[inline]
    /// Get value for None or Hidden
    pub fn none_or_hidden(&self) -> bool {
        match *self {
            OutlineStyle::Auto => false,
            OutlineStyle::BorderStyle(ref style) => style.none_or_hidden(),
        }
    }
}

impl Parse for OutlineStyle {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<OutlineStyle, ParseError<'i>> {
        if let Ok(border_style) = input.try(BorderStyle::parse) {
            if let BorderStyle::Hidden = border_style {
                return Err(input
                    .new_custom_error(SelectorParseErrorKind::UnexpectedIdent("hidden".into())));
            }

            return Ok(OutlineStyle::BorderStyle(border_style));
        }

        input.expect_ident_matching("auto")?;
        Ok(OutlineStyle::Auto)
    }
}
