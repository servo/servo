/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Computed values for Pointing properties.
//!
//! https://drafts.csswg.org/css-ui/#pointing-keyboard

#[cfg(feature = "gecko")]
use std::fmt;
#[cfg(feature = "gecko")]
use style_traits::ToCss;
use style_traits::cursor::CursorKind;
#[cfg(feature = "gecko")]
use values::specified::url::SpecifiedUrl;

/// The computed value for the `cursor` property.
/// Servo variant.
/// https://drafts.csswg.org/css-ui/#cursor
#[cfg(not(feature = "gecko"))]
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToCss)]
pub struct Cursor(pub CursorKind);

/// The computed value for the `cursor` property.
/// Gecko variant.
/// https://drafts.csswg.org/css-ui/#cursor
#[cfg(feature = "gecko")]
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct Cursor {
    /// The parsed images for the cursor.
    pub images: Vec<CursorImage>,
    /// The kind of the cursor [default | help | ...].
    pub keyword: CursorKind,
}

#[cfg(feature = "gecko")]
impl ToCss for Cursor {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        for url in &self.images {
            url.to_css(dest)?;
            dest.write_str(", ")?;
        }
        self.keyword.to_css(dest)
    }
}

/// The computed value for the `image cursors`.
#[cfg(feature = "gecko")]
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue)]
pub struct CursorImage {
    /// The url to parse images from.
    pub url: SpecifiedUrl,
    /// The <x> and <y> coordinates.
    pub hotspot: Option<(f32, f32)>,
}

#[cfg(feature = "gecko")]
impl ToCss for CursorImage {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write
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
