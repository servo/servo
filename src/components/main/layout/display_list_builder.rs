/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Constructs display lists from boxes.

use layout::box_::Box;
use layout::context::LayoutContext;
use layout::util::OpaqueNode;

use gfx;
use style;

pub trait ExtraDisplayListData {
    fn new(box_: &Box) -> Self;
}

pub type Nothing = ();

impl ExtraDisplayListData for OpaqueNode {
    fn new(box_: &Box) -> OpaqueNode {
        box_.node
    }
}

impl ExtraDisplayListData for Nothing {
    fn new(_: &Box) -> Nothing {
        ()
    }
}

/// A builder object that manages display list builder should mainly hold information about the
/// initial request and desired result--for example, whether the `DisplayList` is to be used for
/// painting or hit testing. This can affect which boxes are created.
///
/// Right now, the builder isn't used for much, but it establishes the pattern we'll need once we
/// support display-list-based hit testing and so forth.
pub struct DisplayListBuilder<'a> {
    ctx: &'a LayoutContext,
}

//
// Miscellaneous useful routines
//

/// Allows a CSS color to be converted into a graphics color.
pub trait ToGfxColor {
    /// Converts a CSS color to a graphics color.
    fn to_gfx_color(&self) -> gfx::color::Color;
}

impl ToGfxColor for style::computed_values::RGBA {
    fn to_gfx_color(&self) -> gfx::color::Color {
        gfx::color::rgba(self.red, self.green, self.blue, self.alpha)
    }
}

