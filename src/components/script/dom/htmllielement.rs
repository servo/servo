/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLLIElement {
    parent: HTMLElement,
}

impl HTMLLIElement {
    pub fn Value(&self) -> i32 {
        0
    }

    pub fn SetValue(&mut self, _value: i32, _rv: &mut ErrorResult) {
    }

    pub fn Type(&self) -> DOMString {
        null_string
    }

    pub fn SetType(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }
}
