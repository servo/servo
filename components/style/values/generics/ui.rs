/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Generic values for UI properties.

use crate::values::specified::ui::CursorKind;
use std::fmt::{self, Write};
use style_traits::{CssWriter, ToCss};

/// A generic value for the `cursor` property.
///
/// https://drafts.csswg.org/css-ui/#cursor
#[derive(
    Clone,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToComputedValue,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C)]
pub struct GenericCursor<Image> {
    /// The parsed images for the cursor.
    pub images: crate::OwnedSlice<Image>,
    /// The kind of the cursor [default | help | ...].
    pub keyword: CursorKind,
}

pub use self::GenericCursor as Cursor;

impl<Image> Cursor<Image> {
    /// Set `cursor` to `auto`
    #[inline]
    pub fn auto() -> Self {
        Self {
            images: Default::default(),
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
#[derive(Clone, Debug, MallocSizeOf, PartialEq, ToComputedValue, ToResolvedValue, ToShmem)]
#[repr(C)]
pub struct GenericCursorImage<Image, Number> {
    /// The url to parse images from.
    pub image: Image,
    /// Whether the image has a hotspot or not.
    pub has_hotspot: bool,
    /// The x coordinate.
    pub hotspot_x: Number,
    /// The y coordinate.
    pub hotspot_y: Number,
}

pub use self::GenericCursorImage as CursorImage;

impl<Image: ToCss, Number: ToCss> ToCss for CursorImage<Image, Number> {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        self.image.to_css(dest)?;
        if self.has_hotspot {
            dest.write_char(' ')?;
            self.hotspot_x.to_css(dest)?;
            dest.write_char(' ')?;
            self.hotspot_y.to_css(dest)?;
        }
        Ok(())
    }
}

/// A generic value for `scrollbar-color` property.
///
/// https://drafts.csswg.org/css-scrollbars-1/#scrollbar-color
#[derive(
    Animate,
    Clone,
    ComputeSquaredDistance,
    Copy,
    Debug,
    MallocSizeOf,
    PartialEq,
    SpecifiedValueInfo,
    ToAnimatedValue,
    ToAnimatedZero,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
)]
#[repr(C, u8)]
pub enum GenericScrollbarColor<Color> {
    /// `auto`
    Auto,
    /// `<color>{2}`
    Colors {
        /// First `<color>`, for color of the scrollbar thumb.
        thumb: Color,
        /// Second `<color>`, for color of the scrollbar track.
        track: Color,
    },
}

pub use self::GenericScrollbarColor as ScrollbarColor;

impl<Color> Default for ScrollbarColor<Color> {
    #[inline]
    fn default() -> Self {
        ScrollbarColor::Auto
    }
}
