/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use interfaces::{CefBrowser, CefRenderHandler};
use types::cef_paint_element_type_t::PET_VIEW;

use std::ptr;

pub trait CefRenderHandlerExtensions {
    fn paint(&self, browser: CefBrowser);
}

impl CefRenderHandlerExtensions for CefRenderHandler {
    fn paint(&self, browser: CefBrowser) {
        self.on_paint(browser, PET_VIEW, 0, ptr::null(), &mut (), 0, 0)
    }
}

