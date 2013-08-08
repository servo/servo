/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLOListElement {
    parent: HTMLElement,
}

impl HTMLOListElement {
    pub fn Reversed(&self) -> bool {
        false
    }

    pub fn SetReversed(&self, _reversed: bool, _rv: &mut ErrorResult) {
    }

    pub fn Start(&self) -> i32 {
        0
    }

    pub fn SetStart(&mut self, _start: i32, _rv: &mut ErrorResult) {
    }

    pub fn Type(&self) -> DOMString {
        null_string
    }

    pub fn SetType(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Compact(&self) -> bool {
        false
    }

    pub fn SetCompact(&self, _compact: bool, _rv: &mut ErrorResult) {
    }
}