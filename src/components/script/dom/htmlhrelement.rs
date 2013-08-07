/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLHRElement {
    parent: HTMLElement,
}

impl HTMLHRElement {
    pub fn Align(&self) -> DOMString {
        null_string
    }

    pub fn SetAlign(&mut self, _align: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Color(&self) -> DOMString {
        null_string
    }

    pub fn SetColor(&mut self, _color: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn NoShade(&self) -> bool {
        false
    }

    pub fn SetNoShade(&self, _no_shade: bool, _rv: &mut ErrorResult) {
    }

    pub fn Size(&self) -> DOMString {
        null_string
    }

    pub fn SetSize(&mut self, _size: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Width(&self) -> DOMString {
        null_string
    }

    pub fn SetWidth(&mut self, _width: &DOMString, _rv: &mut ErrorResult) {
    }
}
