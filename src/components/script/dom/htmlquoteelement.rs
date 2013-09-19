/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLQuoteElement {
    htmlelement: HTMLElement,
}

impl HTMLQuoteElement {
    pub fn Cite(&self) -> DOMString {
        None
    }

    pub fn SetCite(&self, _cite: &DOMString) -> ErrorResult {
        Ok(())
    }
}
