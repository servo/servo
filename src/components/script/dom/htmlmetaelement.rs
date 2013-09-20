/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLMetaElement {
    htmlelement: HTMLElement,
}

impl HTMLMetaElement {
    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn HttpEquiv(&self) -> DOMString {
        None
    }

    pub fn SetHttpEquiv(&mut self, _http_equiv: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Content(&self) -> DOMString {
        None
    }

    pub fn SetContent(&mut self, _content: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Scheme(&self) -> DOMString {
        None
    }

    pub fn SetScheme(&mut self, _scheme: &DOMString) -> ErrorResult {
        Ok(())
    }
}
