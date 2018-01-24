/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Specified values for Pointing properties.
//!
//! https://drafts.csswg.org/css-ui/#pointing-keyboard

use style_traits::cursor::CursorKind;
#[cfg(feature = "gecko")]
use values::specified::url::SpecifiedUrl;

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
    pub url: SpecifiedUrl,
    /// The <x> and <y> coordinates.
    pub hotspot: Option<(f32, f32)>,
}
