/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLBodyElement {
    parent: HTMLElement
}

impl HTMLBodyElement {
    pub fn Text(&self) -> DOMString {
        None
    }

    pub fn SetText(&mut self, _text: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Link(&self) -> DOMString {
        None
    }

    pub fn SetLink(&self, _link: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn VLink(&self) -> DOMString {
        None
    }

    pub fn SetVLink(&self, _v_link: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn ALink(&self) -> DOMString {
        None
    }

    pub fn SetALink(&self, _a_link: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn BgColor(&self) -> DOMString {
        None
    }

    pub fn SetBgColor(&self, _bg_color: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Background(&self) -> DOMString {
        None
    }

    pub fn SetBackground(&self, _background: &DOMString, _rv: &mut ErrorResult) {
    }
}
