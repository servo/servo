/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed by the layout task.

use layout::flow::LeafSet;

use extra::arc::MutexArc;
use geom::rect::Rect;
use gfx::font_context::FontContext;
use servo_net::local_image_cache::LocalImageCache;
use servo_util::geometry::Au;

/// Data needed by the layout task.
#[deriving(Clone)]
pub struct LayoutContext {
    /// The font context.
    font_ctx: MutexArc<FontContext>,

    /// The image cache.
    image_cache: MutexArc<LocalImageCache>,

    /// The size of the viewport.
    screen_size: Rect<Au>,

    /// The set of leaves.
    leaf_set: MutexArc<LeafSet>,
}

// Ensures that layout context remains sendable. *Do not* remove this unless you know what you are
// doing.
fn stay_sendable_please<T:Send>(_: T) {
    let x: Option<LayoutContext> = None;
    stay_sendable_please(x)
}

