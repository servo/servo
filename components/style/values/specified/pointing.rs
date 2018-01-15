/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified values for Pointing properties.
//!
//! https://drafts.csswg.org/css-ui/#pointing-keyboard

use cssparser::Parser;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseErrorKind;
use style_traits::ParseError;
use style_traits::cursor::CursorKind;

/// The specified value for the `cursor` property.
///
/// https://drafts.csswg.org/css-ui/#cursor
pub use values::computed::pointing::Cursor;
#[cfg(feature = "gecko")]
pub use values::computed::pointing::CursorImage;
#[cfg(feature = "gecko")]
use values::specified::url::SpecifiedUrl;

impl Cursor {
    /// Set `cursor` to `auto`
    #[cfg(not(feature = "gecko"))]
    #[inline]
    pub fn auto() -> Self {
        Self {
            keyword: CursorKind::Auto,
        }
    }

    /// Set `cursor` to `auto`
    #[cfg(feature = "gecko")]
    #[inline]
    pub fn auto() -> Self {
        Self {
            images: vec![],
            keyword: CursorKind::Auto
        }
    }

    /// cursor: [auto | default | ...]
    #[cfg(not(feature = "gecko"))]
    pub fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        Ok(Self {
            keyword: CursorKind::parse(context, input)?,
        })
    }

    /// cursor: [<url> [<number> <number>]?]# [auto | default | ...]
    #[cfg(feature = "gecko")]
    pub fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        let mut images = vec![];
        loop {
            match input.try(|input| CursorImage::parse_image(context, input)) {
                Ok(mut image) => {
                    image.url.build_image_value();
                    images.push(image)
                }
                Err(_) => break,
            }
            input.expect_comma()?;
        }
        Ok(Self {
            images: images,
            keyword: CursorKind::parse(context, input)?,
        })
    }
}

impl Parse for CursorKind {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        #[allow(unused_imports)] use std::ascii::AsciiExt;
        use style_traits::cursor::CursorKind;
        let location = input.current_source_location();
        let ident = input.expect_ident()?;
        if ident.eq_ignore_ascii_case("auto") {
            Ok(CursorKind::Auto)
        } else {
            CursorKind::from_css_keyword(&ident)
                .map_err(|_| location.new_custom_error(
                        SelectorParseErrorKind::UnexpectedIdent(ident.clone())))
        }
    }
}

#[cfg(feature = "gecko")]
impl CursorImage {
    fn parse_image<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        Ok(Self {
            url: SpecifiedUrl::parse(context, input)?,
            hotspot: match input.try(|input| input.expect_number()) {
                Ok(number) => Some((number, input.expect_number()?)),
                Err(_) => None,
            },
        })
    }
}
