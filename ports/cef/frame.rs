/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use browser::ServoCefBrowserExtensions;
use eutil::Downcast;
use interfaces::{CefBrowser, CefFrame, CefStringVisitor, cef_frame_t, cef_string_visitor_t};
use types::{cef_string_t, cef_string_userfree_t};

use compositing::windowing::WindowEvent;
use std::cell::RefCell;

pub struct ServoCefFrame {
    pub title_visitor: RefCell<Option<CefStringVisitor>>,
    pub url: RefCell<String>,

    /// A reference to the browser.
    pub browser: RefCell<Option<CefBrowser>>,
}

impl ServoCefFrame {
    pub fn new() -> ServoCefFrame {
        ServoCefFrame {
            title_visitor: RefCell::new(None),
            url: RefCell::new(String::new()),
            browser: RefCell::new(None),
        }
    }
}

full_cef_class_impl! {
    ServoCefFrame : CefFrame, cef_frame_t {
        fn load_url(&this, url: *const cef_string_t [&[u16]],) -> () {{
            let this = this.downcast();
            let url = String::from_utf16(url).unwrap();
            *this.url.borrow_mut() = url.clone();
            let event = WindowEvent::LoadUrl(url);
            this.browser.borrow_mut().as_mut().unwrap().send_window_event(event);
        }}
        fn get_url(&this,) -> cef_string_userfree_t {{
            let this = this.downcast();
            // FIXME(https://github.com/rust-lang/rust/issues/23338)
            let url = this.url.borrow();
            (*url).clone()
        }}
        fn get_text(&this, visitor: *mut cef_string_visitor_t [CefStringVisitor],) -> () {{
            let this = this.downcast();
            *this.title_visitor.borrow_mut() = Some(visitor);
            this.browser.borrow().as_ref().unwrap().get_title_for_main_frame();
        }}
    }
}

pub trait ServoCefFrameExtensions {
    fn set_browser(&self, browser: CefBrowser);
    fn set_url(&self, url: &[u16]);
    fn load(&self);
}

impl ServoCefFrameExtensions for CefFrame {
    fn set_browser(&self, browser: CefBrowser) {
        *self.downcast().browser.borrow_mut() = Some(browser)
    }
    fn set_url(&self, url: &[u16]) {
        let frame = self.downcast();
        *frame.url.borrow_mut() = String::from_utf16(url).unwrap();
    }
    fn load(&self) {
        let event = WindowEvent::LoadUrl(self.downcast().url.borrow().clone());
        self.downcast().browser.borrow_mut().as_mut().unwrap().send_window_event(event);
    }
}
