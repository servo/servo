/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLModElement {
    htmlelement: HTMLElement
}

impl HTMLModElement {
    pub fn Cite(&self) -> DOMString {
        None
    }

    pub fn SetCite(&mut self, _cite: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn DateTime(&self) -> DOMString {
        None
    }

    pub fn SetDateTime(&mut self, _datetime: &DOMString) -> ErrorResult {
        Ok(())
    }
}
