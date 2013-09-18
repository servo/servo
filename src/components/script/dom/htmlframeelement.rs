/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::htmlelement::HTMLElement;
use dom::windowproxy::WindowProxy;

pub struct HTMLFrameElement {
    parent: HTMLElement
}

impl HTMLFrameElement {
    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Scrolling(&self) -> DOMString {
        None
    }

    pub fn SetScrolling(&mut self, _scrolling: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Src(&self) -> DOMString {
        None
    }

    pub fn SetSrc(&mut self, _src: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn FrameBorder(&self) -> DOMString {
        None
    }

    pub fn SetFrameBorder(&mut self, _frameborder: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn LongDesc(&self) -> DOMString {
        None
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
        None
    }

    pub fn SetMarginHeight(&mut self, _height: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn MarginWidth(&self) -> DOMString {
        None
    }

    pub fn SetMarginWidth(&mut self, _height: &DOMString, _rv: &mut ErrorResult) {
    }
}