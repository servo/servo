/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLTableSectionElement {
    parent: HTMLElement,
}

impl HTMLTableSectionElement {
    pub fn DeleteRow(&mut self, _index: i32, _rv: &mut ErrorResult) {
    }

    pub fn Align(&self) -> DOMString {
        null_string
    }

    pub fn SetAlign(&mut self, _align: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Ch(&self) -> DOMString {
        null_string
    }

    pub fn SetCh(&mut self, _ch: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn ChOff(&self) -> DOMString {
        null_string
    }

    pub fn SetChOff(&mut self, _ch_off: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn VAlign(&self) -> DOMString {
        null_string
    }

    pub fn SetVAlign(&mut self, _v_align: &DOMString, _rv: &mut ErrorResult) {
    }
}
