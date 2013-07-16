/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Constructs display lists from render boxes.

use layout::box::RenderBox;
use layout::context::LayoutContext;

use gfx;
use newcss;

/// Extra display list data is either nothing (if the display list is to be rendered) or the
/// originating render box (if the display list is generated for hit testing).
pub trait ExtraDisplayListData {
    fn new(box: RenderBox) -> Self;
}

/// The type representing the lack of extra display list data. This is used when sending display
/// list data off to be rendered.
pub type Nothing = ();

impl ExtraDisplayListData for Nothing {
    fn new(_: RenderBox) -> Nothing {
        ()
    }
}

impl ExtraDisplayListData for RenderBox {
    fn new(box: RenderBox) -> RenderBox {
        box
    }
}

/// A builder object that manages display list builder should mainly hold information about the
/// initial request and desired result--for example, whether the `DisplayList` is to be used for
/// painting or hit testing. This can affect which boxes are created.
///
/// Right now, the builder isn't used for much, but it establishes the pattern we'll need once we
/// support display-list-based hit testing and so forth.
pub struct DisplayListBuilder<'self> {
    ctx:  &'self LayoutContext,
}

//
// Miscellaneous useful routines
//

/// Allows a CSS color to be converted into a graphics color.
pub trait ToGfxColor {
    /// Converts a CSS color to a graphics color.
    fn to_gfx_color(&self) -> gfx::color::Color;
}

impl ToGfxColor for newcss::color::Color {
    fn to_gfx_color(&self) -> gfx::color::Color {
        gfx::color::rgba(self.red, self.green, self.blue, self.alpha)
    }
}

