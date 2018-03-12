/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified values for Pointing properties.
//!
//! https://drafts.csswg.org/css-ui/#pointing-keyboard

use cssparser::Parser;
use parser::{Parse, ParserContext};
use style_traits::ParseError;
use style_traits::cursor::CursorKind;
use values::generics::pointing::CaretColor as GenericCaretColor;
use values::specified::color::Color;
#[cfg(feature = "gecko")]
use values::specified::url::SpecifiedImageUrl;

/// The specified value for the `cursor` property.
///
/// https://drafts.csswg.org/css-ui/#cursor
#[cfg(feature = "servo")]
#[derive(Clone, Copy, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub struct Cursor(pub CursorKind);

/// The specified value for the `cursor` property.
///
/// https://drafts.csswg.org/css-ui/#cursor
#[cfg(feature = "gecko")]
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct Cursor {
    /// The parsed images for the cursor.
    pub images: Box<[CursorImage]>,
    /// The kind of the cursor [default | help | ...].
    pub keyword: CursorKind,
}

/// The specified value for the `image cursors`.
#[cfg(feature = "gecko")]
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct CursorImage {
    /// The url to parse images from.
    pub url: SpecifiedImageUrl,
    /// The <x> and <y> coordinates.
    pub hotspot: Option<(f32, f32)>,
}

/// A specified value for the `caret-color` property.
pub type CaretColor = GenericCaretColor<Color>;

impl Parse for CaretColor {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("auto")).is_ok() {
            return Ok(GenericCaretColor::Auto);
        }
        Ok(GenericCaretColor::Color(Color::parse(context, input)?))
    }
}
