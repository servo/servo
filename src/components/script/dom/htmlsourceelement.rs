/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLSourceElement {
    parent: HTMLElement
}

impl HTMLSourceElement {
    pub fn Src(&self) -> DOMString {
        None
    }
    
    pub fn SetSrc(&mut self, _src: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Type(&self) -> DOMString {
        None
    }
    
    pub fn SetType(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Media(&self) -> DOMString {
        None
    }
    
    pub fn SetMedia(&mut self, _media: &DOMString, _rv: &mut ErrorResult) {
    }
}
