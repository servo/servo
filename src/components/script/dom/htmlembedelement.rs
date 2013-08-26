/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::document::AbstractDocument;
use dom::htmlelement::HTMLElement;

pub struct HTMLEmbedElement {
    parent: HTMLElement
}

impl HTMLEmbedElement {
    pub fn Src(&self) -> DOMString {
        null_string
    }

    pub fn SetSrc(&mut self, _src: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Type(&self) -> DOMString {
        null_string
    }

    pub fn SetType(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Width(&self) -> DOMString {
        null_string
    }

    pub fn SetWidth(&mut self, _width: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Height(&self) -> DOMString {
        null_string
    }

    pub fn SetHeight(&mut self, _height: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Align(&self) -> DOMString {
        null_string
    }

    pub fn SetAlign(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Name(&self) -> DOMString {
        null_string
    }

    pub fn SetName(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn GetSVGDocument(&self) -> Option<AbstractDocument> {
        None
    }
}
