/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLDataElement {
    htmlelement: HTMLElement
}

impl HTMLDataElement {
    pub fn Value(&self) -> DOMString {
        None
    }

    pub fn SetValue(&mut self, _value: &DOMString) -> ErrorResult {
        Ok(())
    }
}
