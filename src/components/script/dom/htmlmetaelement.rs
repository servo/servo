/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLMetaElement {
    parent: HTMLElement,
}

impl HTMLMetaElement {
    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn HttpEquiv(&self) -> DOMString {
        None
    }

    pub fn SetHttpEquiv(&mut self, _http_equiv: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Content(&self) -> DOMString {
        None
    }

    pub fn SetContent(&mut self, _content: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Scheme(&self) -> DOMString {
        None
    }

    pub fn SetScheme(&mut self, _scheme: &DOMString, _rv: &mut ErrorResult) {
    }
}
