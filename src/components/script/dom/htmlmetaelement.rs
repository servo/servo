/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLMetaElement {
    parent: HTMLElement,
}

impl HTMLMetaElement {
    pub fn Name(&self) -> DOMString {
        null_string
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn HttpEquiv(&self) -> DOMString {
        null_string
    }

    pub fn SetHttpEquiv(&mut self, _http_equiv: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Content(&self) -> DOMString {
        null_string
    }

    pub fn SetContent(&mut self, _content: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Scheme(&self) -> DOMString {
        null_string
    }

    pub fn SetScheme(&mut self, _scheme: &DOMString, _rv: &mut ErrorResult) {
    }
}
