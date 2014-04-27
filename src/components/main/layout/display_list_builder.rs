/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Constructs display lists from boxes.

use layout::context::LayoutContext;

use geom::{Point2D, Rect, Size2D};
use gfx::render_task::RenderLayer;
use gfx;
use servo_util::geometry::Au;
use servo_util::smallvec::SmallVec0;
use style;

/// Manages the information needed to construct the display list.
pub struct DisplayListBuilder<'a> {
    pub ctx: &'a LayoutContext,

    /// A list of render layers that we've built up, root layer not included.
    pub layers: SmallVec0<RenderLayer>,

    /// The dirty rect.
    pub dirty: Rect<Au>,
}

/// Information needed at each step of the display list building traversal.
pub struct DisplayListBuildingInfo {
    /// The size of the containing block for relatively-positioned descendants.
    pub relative_containing_block_size: Size2D<Au>,
    /// The position and size of the absolute containing block.
    pub absolute_containing_block_position: Point2D<Au>,
    /// Whether the absolute containing block forces positioned descendants to be layerized.
    pub layers_needed_for_positioned_flows: bool,
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

