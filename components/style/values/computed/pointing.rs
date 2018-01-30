/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values for Pointing properties.
//!
//! https://drafts.csswg.org/css-ui/#pointing-keyboard

use cssparser::Parser;
use parser::{Parse, ParserContext};
use selectors::parser::SelectorParseErrorKind;
#[cfg(feature = "gecko")]
use std::fmt::{self, Write};
#[cfg(feature = "gecko")]
use style_traits::{CssWriter, ToCss};
use style_traits::ParseError;
use style_traits::cursor::CursorKind;

/// The computed value for the `cursor` property.
///
/// https://drafts.csswg.org/css-ui/#cursor
pub use values::specified::pointing::Cursor;
#[cfg(feature = "gecko")]
pub use values::specified::pointing::CursorImage;
#[cfg(feature = "gecko")]
use values::specified::url::SpecifiedUrl;

impl Cursor {
    /// Set `cursor` to `auto`
    #[cfg(feature = "servo")]
    #[inline]
    pub fn auto() -> Self {
        Cursor(CursorKind::Auto)
    }

    /// Set `cursor` to `auto`
    #[cfg(feature = "gecko")]
    #[inline]
    pub fn auto() -> Self {
        Self {
            images: vec![].into_boxed_slice(),
            keyword: CursorKind::Auto
        }
    }
}

impl Parse for Cursor {
    /// cursor: [auto | default | ...]
    #[cfg(feature = "servo")]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        Ok(Cursor(CursorKind::parse(context, input)?))
    }

    /// cursor: [<url> [<number> <number>]?]# [auto | default | ...]
    #[cfg(feature = "gecko")]
    fn parse<'i, 't>(
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
            images: images.into_boxed_slice(),
            keyword: CursorKind::parse(context, input)?,
        })
    }
}

#[cfg(feature = "gecko")]
impl ToCss for Cursor {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        for url in &*self.images {
            url.to_css(dest)?;
            dest.write_str(", ")?;
        }
        self.keyword.to_css(dest)
    }
}

impl Parse for CursorKind {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let ident = input.expect_ident()?;
        CursorKind::from_css_keyword(&ident)
            .map_err(|_| location.new_custom_error(
                    SelectorParseErrorKind::UnexpectedIdent(ident.clone())))
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
            // FIXME(emilio): Should use Number::parse to handle calc() correctly.
            hotspot: match input.try(|input| input.expect_number()) {
                Ok(number) => Some((number, input.expect_number()?)),
                Err(_) => None,
            },
        })
    }
}

#[cfg(feature = "gecko")]
impl ToCss for CursorImage {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.url.to_css(dest)?;
        if let Some((x, y)) = self.hotspot {
            dest.write_str(" ")?;
            x.to_css(dest)?;
            dest.write_str(" ")?;
            y.to_css(dest)?;
        }
        Ok(())
    }
}
