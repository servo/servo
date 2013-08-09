/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLTableRowElement {
    parent: HTMLElement,
}

impl HTMLTableRowElement {
    pub fn RowIndex(&self) -> i32 {
        0
    }

    pub fn GetRowIndex(&self) -> i32 {
        0
    }

    pub fn SectionRowIndex(&self) -> i32 {
        0
    }

    pub fn GetSectionRowIndex(&self) -> i32 {
        0
    }

    pub fn DeleteCell(&mut self, _index: i32, _rv: &mut ErrorResult) {
    }

    pub fn Align(&self) -> DOMString {
        null_string
    }

    pub fn SetAlign(&self, _align: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Ch(&self) -> DOMString {
        null_string
    }

    pub fn SetCh(&self, _ch: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn ChOff(&self) -> DOMString {
        null_string
    }

    pub fn SetChOff(&self, _ch_off: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn VAlign(&self) -> DOMString {
        null_string
    }

    pub fn SetVAlign(&self, _v_align: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn BgColor(&self) -> DOMString {
        null_string
    }

    pub fn SetBgColor(&self, _bg_color: &DOMString, _rv: &mut ErrorResult) {
    }
}
