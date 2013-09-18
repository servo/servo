/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLScriptElement {
    parent: HTMLElement,
}

impl HTMLScriptElement {
    pub fn Src(&self) -> DOMString {
        None
    }

    pub fn SetSrc(&mut self, _src: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Type(&self) -> DOMString {
        None
    }

    pub fn SetType(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Charset(&self) -> DOMString {
        None
    }

    pub fn SetCharset(&mut self, _charset: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Async(&self) -> bool {
        false
    }

    pub fn SetAsync(&self, _async: bool, _rv: &mut ErrorResult) {
    }

    pub fn Defer(&self) -> bool {
        false
    }

    pub fn SetDefer(&self, _defer: bool, _rv: &mut ErrorResult) {
    }

    pub fn CrossOrigin(&self) -> DOMString {
        None
    }

    pub fn SetCrossOrigin(&mut self, _cross_origin: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Text(&self) -> DOMString {
        None
    }

    pub fn SetText(&mut self, _text: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Event(&self) -> DOMString {
        None
    }

    pub fn SetEvent(&mut self, _event: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn HtmlFor(&self) -> DOMString {
        None
    }

    pub fn SetHtmlFor(&mut self, _html_for: &DOMString, _rv: &mut ErrorResult) {
    }
}
