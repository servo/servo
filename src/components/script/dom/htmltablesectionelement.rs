/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLTableSectionElement {
    htmlelement: HTMLElement,
}

impl HTMLTableSectionElement {
    pub fn DeleteRow(&mut self, _index: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Align(&self) -> DOMString {
        None
    }

    pub fn SetAlign(&mut self, _align: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Ch(&self) -> DOMString {
        None
    }

    pub fn SetCh(&mut self, _ch: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn ChOff(&self) -> DOMString {
        None
    }

    pub fn SetChOff(&mut self, _ch_off: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn VAlign(&self) -> DOMString {
        None
    }

    pub fn SetVAlign(&mut self, _v_align: &DOMString) -> ErrorResult {
        Ok(())
    }
}
