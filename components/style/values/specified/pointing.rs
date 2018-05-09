/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified values for Pointing properties.
//!
//! https://drafts.csswg.org/css-ui/#pointing-keyboard

use cssparser::Parser;
use parser::{Parse, ParserContext};
use style_traits::{ParseError, StyleParseErrorKind};
use style_traits::cursor::CursorKind;
use values::generics::pointing as generics;
use values::specified::Number;
use values::specified::color::Color;
use values::specified::url::SpecifiedImageUrl;

/// A specified value for the `caret-color` property.
pub type CaretColor = generics::CaretColor<Color>;

impl Parse for CaretColor {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("auto")).is_ok() {
            return Ok(generics::CaretColor::Auto);
        }
        Ok(generics::CaretColor::Color(Color::parse(context, input)?))
    }
}

/// A specified value for the `cursor` property.
pub type Cursor = generics::Cursor<CursorImage>;

/// A specified value for item of `image cursors`.
pub type CursorImage = generics::CursorImage<SpecifiedImageUrl, Number>;

impl Parse for Cursor {
    /// cursor: [<url> [<number> <number>]?]# [auto | default | ...]
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let mut images = vec![];
        loop {
            match input.try(|input| CursorImage::parse(context, input)) {
                Ok(image) => images.push(image),
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

impl Parse for CursorKind {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let location = input.current_source_location();
        let ident = input.expect_ident()?;
        CursorKind::from_css_keyword(&ident).map_err(|_| {
            location.new_custom_error(StyleParseErrorKind::UnspecifiedError)
        })
    }
}

impl Parse for CursorImage {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        Ok(Self {
            url: SpecifiedImageUrl::parse(context, input)?,
            hotspot: match input.try(|input| Number::parse(context, input)) {
                Ok(number) => Some((number, Number::parse(context, input)?)),
                Err(_) => None,
            },
        })
    }
}
