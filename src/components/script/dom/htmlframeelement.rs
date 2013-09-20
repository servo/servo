/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::htmlelement::HTMLElement;
use dom::windowproxy::WindowProxy;

pub struct HTMLFrameElement {
    htmlelement: HTMLElement
}

impl HTMLFrameElement {
    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Scrolling(&self) -> DOMString {
        None
    }

    pub fn SetScrolling(&mut self, _scrolling: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Src(&self) -> DOMString {
        None
    }

    pub fn SetSrc(&mut self, _src: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn FrameBorder(&self) -> DOMString {
        None
    }

    pub fn SetFrameBorder(&mut self, _frameborder: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn LongDesc(&self) -> DOMString {
        None
    }

    pub fn SetLongDesc(&mut self, _longdesc: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn NoResize(&self) -> bool {
        false
    }

    pub fn SetNoResize(&mut self, _no_resize: bool) -> ErrorResult {
        Ok(())
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

    pub fn SetMarginHeight(&mut self, _height: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn MarginWidth(&self) -> DOMString {
        None
    }

    pub fn SetMarginWidth(&mut self, _height: &DOMString) -> ErrorResult {
        Ok(())
    }
}
