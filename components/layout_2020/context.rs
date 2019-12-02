/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use gfx::font_cache_thread::FontCacheThread;
use gfx::font_context::FontContext;
use msg::constellation_msg::PipelineId;
use std::cell::RefCell;
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

pub(crate) type LayoutFontContext = FontContext<FontCacheThread>;

thread_local!(static FONT_CONTEXT: RefCell<Option<LayoutFontContext>> = RefCell::new(None));

pub(crate) fn with_thread_local_font_context<F, R>(layout_context: &LayoutContext, f: F) -> R
where
    F: FnOnce(&mut LayoutFontContext) -> R,
{
    FONT_CONTEXT.with(|font_context| {
        f(font_context.borrow_mut().get_or_insert_with(|| {
            FontContext::new(layout_context.font_cache_thread.lock().unwrap().clone())
        }))
    })
}
