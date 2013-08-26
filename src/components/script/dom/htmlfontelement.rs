/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLFontElement {
    parent: HTMLElement
}

impl HTMLFontElement {
    pub fn Color(&self) -> DOMString {
        null_string
    }

    pub fn SetColor(&mut self, _color: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Face(&self) -> DOMString {
        null_string
    }

    pub fn SetFace(&mut self, _face: &DOMString, _rv: &mut ErrorResult) {
    }
    
    pub fn Size(&self) -> DOMString {
        null_string
    }

    pub fn SetSize(&mut self, _size: &DOMString, _rv: &mut ErrorResult) {
    }
}
