/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for UI properties.

use crate::parser::{Parse, ParserContext};
use crate::values::generics::ui as generics;
use crate::values::specified::color::Color;
use crate::values::specified::url::SpecifiedImageUrl;
use crate::values::specified::Number;
use cssparser::Parser;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ParseError, StyleParseErrorKind, ToCss};

/// A specified value for the `cursor` property.
pub type Cursor = generics::GenericCursor<CursorImage>;

/// A specified value for item of `image cursors`.
pub type CursorImage = generics::GenericCursorImage<SpecifiedImageUrl, Number>;

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
            images: images.into(),
            keyword: CursorKind::parse(input)?,
        })
    }
}

impl Parse for CursorImage {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        use crate::Zero;

        let url = SpecifiedImageUrl::parse(context, input)?;
        let mut has_hotspot = false;
        let mut hotspot_x = Number::zero();
        let mut hotspot_y = Number::zero();

        if let Ok(x) = input.try(|input| Number::parse(context, input)) {
            has_hotspot = true;
            hotspot_x = x;
            hotspot_y = Number::parse(context, input)?;
        }

        Ok(Self { url, has_hotspot, hotspot_x, hotspot_y })
    }
}

/// Specified value of `-moz-force-broken-image-icon`
#[derive(
    Clone,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
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

/// A specified value for `scrollbar-color` property
pub type ScrollbarColor = generics::ScrollbarColor<Color>;

impl Parse for ScrollbarColor {
    fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        if input.try(|i| i.expect_ident_matching("auto")).is_ok() {
            return Ok(generics::ScrollbarColor::Auto);
        }
        Ok(generics::ScrollbarColor::Colors {
            thumb: Color::parse(context, input)?,
            track: Color::parse(context, input)?,
        })
    }
}

/// The specified value for the `user-select` property.
///
/// https://drafts.csswg.org/css-ui-4/#propdef-user-select
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum UserSelect {
    Auto,
    Text,
    #[parse(aliases = "-moz-none")]
    None,
    /// Force selection of all children.
    All,
}

/// The keywords allowed in the Cursor property.
///
/// https://drafts.csswg.org/css-ui-4/#propdef-cursor
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    FromPrimitive,
    MallocSizeOf,
    Parse,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(u8)]
pub enum CursorKind {
    None,
    Default,
    Pointer,
    ContextMenu,
    Help,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    #[parse(aliases = "-moz-grab")]
    Grab,
    #[parse(aliases = "-moz-grabbing")]
    Grabbing,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
    AllScroll,
    #[parse(aliases = "-moz-zoom-in")]
    ZoomIn,
    #[parse(aliases = "-moz-zoom-out")]
    ZoomOut,
    Auto,
}
