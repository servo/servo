/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::document::AbstractDocument;
use dom::htmlelement::HTMLElement;
use dom::windowproxy::WindowProxy;

pub struct HTMLFrameElement {
    parent: HTMLElement
}

impl HTMLFrameElement {
    pub fn Name(&self) -> DOMString {
        null_string
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Scrolling(&self) -> DOMString {
        null_string
    }

    pub fn SetScrolling(&mut self, _scrolling: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Src(&self) -> DOMString {
        null_string
    }

    pub fn SetSrc(&mut self, _src: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn FrameBorder(&self) -> DOMString {
        null_string
    }

    pub fn SetFrameBorder(&mut self, _frameborder: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn LongDesc(&self) -> DOMString {
        null_string
    }

    pub fn SetLongDesc(&mut self, _longdesc: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn NoResize(&self) -> bool {
        false
    }

    pub fn SetNoResize(&mut self, _no_resize: bool, _rv: &mut ErrorResult) {
    }

    pub fn GetContentDocument(&self) -> Option<AbstractDocument> {
        None
    }

    pub fn GetContentWindow(&self) -> Option<@mut WindowProxy> {
        None
    }

    pub fn MarginHeight(&self) -> DOMString {
        null_string
    }

    pub fn SetMarginHeight(&mut self, _height: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn MarginWidth(&self) -> DOMString {
        null_string
    }

    pub fn SetMarginWidth(&mut self, _height: &DOMString, _rv: &mut ErrorResult) {
    }
}