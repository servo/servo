/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLTrackElement {
    parent: HTMLElement,
}

impl HTMLTrackElement {
    pub fn Kind(&self) -> DOMString {
        null_string
    }

    pub fn SetKind(&mut self, _kind: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Src(&self) -> DOMString {
        null_string
    }

    pub fn SetSrc(&mut self, _src: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Srclang(&self) -> DOMString {
        null_string
    }

    pub fn SetSrclang(&mut self, _srclang: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Label(&self) -> DOMString {
        null_string
    }

    pub fn SetLabel(&mut self, _label: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Default(&self) -> bool {
        false
    }

    pub fn SetDefault(&mut self, _default: bool, _rv: &mut ErrorResult) {
    }

    pub fn ReadyState(&self) -> u16 {
        0
    }
}
