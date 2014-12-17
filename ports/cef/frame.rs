/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use eutil::Downcast;
use interfaces::{CefFrame, CefStringVisitor, cef_frame_t, cef_string_visitor_t};
use types::{cef_string_t, cef_string_userfree_t};

use core;
use compositing::windowing::WindowEvent;
use std::cell::RefCell;

pub struct ServoCefFrame {
    pub title_visitor: RefCell<Option<CefStringVisitor>>,
    pub url: RefCell<String>,
}

impl ServoCefFrame {
    pub fn new() -> ServoCefFrame {
        ServoCefFrame {
            title_visitor: RefCell::new(None),
            url: RefCell::new(String::new()),
        }
    }
}

cef_class_impl! {
    ServoCefFrame : CefFrame, cef_frame_t {
        fn load_url(&this, url: *const cef_string_t) -> () {
            let this = this.downcast();
            *this.url.borrow_mut() = String::from_utf16(url).unwrap();
            core::send_window_event(WindowEvent::LoadUrl(String::from_utf16(url).unwrap()));
        }
        fn get_url(&this) -> cef_string_userfree_t {
            let this = this.downcast();
            (*this.url.borrow()).clone()
        }
        fn get_text(&this, visitor: *mut cef_string_visitor_t) -> () {
            let this = this.downcast();
            *this.title_visitor.borrow_mut() = Some(visitor);
            core::get_title_for_main_frame();
        }
    }
}

