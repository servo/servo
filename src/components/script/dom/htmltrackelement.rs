/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLTrackElement {
    htmlelement: HTMLElement,
}

impl HTMLTrackElement {
    pub fn Kind(&self) -> DOMString {
        None
    }

    pub fn SetKind(&mut self, _kind: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Src(&self) -> DOMString {
        None
    }

    pub fn SetSrc(&mut self, _src: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Srclang(&self) -> DOMString {
        None
    }

    pub fn SetSrclang(&mut self, _srclang: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Label(&self) -> DOMString {
        None
    }

    pub fn SetLabel(&mut self, _label: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Default(&self) -> bool {
        false
    }

    pub fn SetDefault(&mut self, _default: bool) -> ErrorResult {
        Ok(())
    }

    pub fn ReadyState(&self) -> u16 {
        0
    }
}
