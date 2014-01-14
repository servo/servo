/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed by the layout task.

use extra::arc::MutexArc;
use layout::flow::LeafSet;

use geom::size::Size2D;
use gfx::font_context::FontContext;
use servo_msg::constellation_msg::ConstellationChan;
use servo_net::local_image_cache::LocalImageCache;
use servo_util::geometry::Au;

/// Data shared by all layout workers.
#[deriving(Clone)]
pub struct SharedLayoutInfo {
    /// The local image cache.
    image_cache: MutexArc<LocalImageCache>,

    /// The current screen size.
    screen_size: Size2D<Au>,

    /// A channel up to the constellation.
    constellation_chan: ConstellationChan,

    /// The set of leaf flows.
    leaf_set: MutexArc<LeafSet>,
}

/// Data specific to a layout worker.
pub struct LayoutContext {
    /// Shared layout info.
    shared: SharedLayoutInfo,

    /// The current font context.
    font_ctx: ~FontContext,
}

