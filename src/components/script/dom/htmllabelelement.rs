/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string};
use dom::htmlelement::HTMLElement;

pub struct HTMLLabelElement {
    parent: HTMLElement,
}

impl HTMLLabelElement {
    pub fn HtmlFor(&self) -> DOMString {
        null_string
    }

    pub fn SetHtmlFor(&mut self, _html_for: &DOMString) {
    }
}
