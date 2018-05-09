/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Generic values for pointing properties.

use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};
use style_traits::cursor::CursorKind;

/// A generic value for the `caret-color` property.
#[derive(Animate, Clone, ComputeSquaredDistance, Copy, Debug, MallocSizeOf,
         PartialEq, SpecifiedValueInfo, ToAnimatedValue, ToAnimatedZero,
         ToComputedValue, ToCss)]
pub enum CaretColor<Color> {
    /// An explicit color.
    Color(Color),
    /// The keyword `auto`.
    Auto,
}

/// A generic value for the `cursor` property.
///
/// https://drafts.csswg.org/css-ui/#cursor
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue)]
pub struct Cursor<Image> {
    /// The parsed images for the cursor.
    pub images: Box<[Image]>,
    /// The kind of the cursor [default | help | ...].
    pub keyword: CursorKind,
}

impl<Image> Cursor<Image> {
    /// Set `cursor` to `auto`
    #[inline]
    pub fn auto() -> Self {
        Self {
            images: vec![].into_boxed_slice(),
            keyword: CursorKind::Auto,
        }
    }
}

impl<Image: ToCss> ToCss for Cursor<Image> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        for image in &*self.images {
            image.to_css(dest)?;
            dest.write_str(", ")?;
        }
        self.keyword.to_css(dest)
    }
}

/// A generic value for item of `image cursors`.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo,
         ToComputedValue)]
pub struct CursorImage<ImageUrl, Number> {
    /// The url to parse images from.
    pub url: ImageUrl,
    /// The <x> and <y> coordinates.
    pub hotspot: Option<(Number, Number)>,
}

impl<ImageUrl: ToCss, Number: ToCss> ToCss for CursorImage<ImageUrl, Number> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.url.to_css(dest)?;
        if let Some((ref x, ref y)) = self.hotspot {
            dest.write_str(" ")?;
            x.to_css(dest)?;
            dest.write_str(" ")?;
            y.to_css(dest)?;
        }
        Ok(())
    }
}
