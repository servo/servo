/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Data needed by the layout task.

use geom::rect::Rect;
use gfx::font_context::FontContext;
use gfx::geometry::Au;
use servo_net::local_image_cache::LocalImageCache;

/// Data needed by the layout task.
pub struct LayoutContext {
    font_ctx: @mut FontContext,
    image_cache: @mut LocalImageCache,
    screen_size: Rect<Au>
}
