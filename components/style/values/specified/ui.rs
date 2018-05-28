/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified types for UI properties.

use cssparser::Parser;
use parser::{Parse, ParserContext};
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};
use style_traits::cursor::CursorKind;
use values::{Auto, Either};
use values::generics::ui as generics;
use values::specified::Number;
use values::specified::color::Color;
use values::specified::url::SpecifiedImageUrl;

/// auto | <color>
pub type ColorOrAuto = Either<Color, Auto>;

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

/// Specified value of `-moz-force-broken-image-icon`
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue)]
pub struct MozForceBrokenImageIcon(pub bool);

impl MozForceBrokenImageIcon {
    /// Return initial value of -moz-force-broken-image-icon which is false.
    #[inline]
    pub fn false_value() -> MozForceBrokenImageIcon {
        MozForceBrokenImageIcon(false)
    }
}

impl Parse for MozForceBrokenImageIcon {
    fn parse<'i, 't>(
        _context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<MozForceBrokenImageIcon, ParseError<'i>> {
        // We intentionally don't support calc values here.
        match input.expect_integer()? {
            0 => Ok(MozForceBrokenImageIcon(false)),
            1 => Ok(MozForceBrokenImageIcon(true)),
            _ => Err(input.new_custom_error(StyleParseErrorKind::UnspecifiedError)),
        }
    }
}

impl ToCss for MozForceBrokenImageIcon {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        dest.write_str(if self.0 { "1" } else { "0" })
    }
}

impl From<u8> for MozForceBrokenImageIcon {
    fn from(bits: u8) -> MozForceBrokenImageIcon {
        MozForceBrokenImageIcon(bits == 1)
    }
}

impl From<MozForceBrokenImageIcon> for u8 {
    fn from(v: MozForceBrokenImageIcon) -> u8 {
        if v.0 {
            1
        } else {
            0
        }
    }
}
