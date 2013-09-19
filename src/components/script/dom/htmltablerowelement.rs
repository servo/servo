/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLTableRowElement {
    htmlelement: HTMLElement,
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

    pub fn DeleteCell(&mut self, _index: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Align(&self) -> DOMString {
        None
    }

    pub fn SetAlign(&self, _align: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Ch(&self) -> DOMString {
        None
    }

    pub fn SetCh(&self, _ch: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn ChOff(&self) -> DOMString {
        None
    }

    pub fn SetChOff(&self, _ch_off: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn VAlign(&self) -> DOMString {
        None
    }

    pub fn SetVAlign(&self, _v_align: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn BgColor(&self) -> DOMString {
        None
    }

    pub fn SetBgColor(&self, _bg_color: &DOMString) -> ErrorResult {
        Ok(())
    }
}
