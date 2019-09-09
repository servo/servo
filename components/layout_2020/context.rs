/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use gfx::font_cache_thread::FontCacheThread;
use msg::constellation_msg::PipelineId;
use std::sync::Mutex;
use style::context::SharedStyleContext;

pub struct LayoutContext<'a> {
    pub id: PipelineId,
    pub style_context: SharedStyleContext<'a>,
    pub font_cache_thread: Mutex<FontCacheThread>,
}

impl<'a> LayoutContext<'a> {
    #[inline(always)]
    pub fn shared_context(&self) -> &SharedStyleContext {
        &self.style_context
    }
}
