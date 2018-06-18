/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Resolution values:
//!
//! https://drafts.csswg.org/css-values/#resolution

use cssparser::{Parser, Token};
use parser::{Parse, ParserContext};
use style_traits::{ParseError, StyleParseErrorKind};
use values::CSSFloat;

/// A specified resolution.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToCss)]
pub enum Resolution {
    /// Dots per inch.
    #[css(dimension)]
    Dpi(CSSFloat),
    /// An alias unit for dots per pixel.
    #[css(dimension)]
    X(CSSFloat),
    /// Dots per pixel.
    #[css(dimension)]
    Dppx(CSSFloat),
    /// Dots per centimeter.
    #[css(dimension)]
    Dpcm(CSSFloat),
}

impl Resolution {
    /// Convert this resolution value to dppx units.
    pub fn to_dppx(&self) -> CSSFloat {
        match *self {
            Resolution::X(f) |
            Resolution::Dppx(f) => f,
            _ => self.to_dpi() / 96.0,
        }
    }

    /// Convert this resolution value to dpi units.
    pub fn to_dpi(&self) -> CSSFloat {
        match *self {
            Resolution::Dpi(f) => f,
            Resolution::X(f) |
            Resolution::Dppx(f) => f * 96.0,
            Resolution::Dpcm(f) => f * 2.54,
        }
    }
}

impl Parse for Resolution {
    fn parse<'i, 't>(
        _: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let (value, unit) = match *input.next()? {
            Token::Dimension {
                value, ref unit, ..
            } => (value, unit),
            ref t => return Err(location.new_unexpected_token_error(t.clone())),
        };

        if value <= 0. {
            return Err(location.new_custom_error(StyleParseErrorKind::UnspecifiedError));
        }

        match_ignore_ascii_case! { &unit,
            "dpi" => Ok(Resolution::Dpi(value)),
            "dppx" => Ok(Resolution::Dppx(value)),
            "dpcm" => Ok(Resolution::Dpcm(value)),
            "x" => Ok(Resolution::X(value)),
            _ => Err(location.new_custom_error(
                StyleParseErrorKind::UnexpectedDimension(unit.clone())
            )),
        }
    }
}
