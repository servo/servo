/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::DOMString;
use dom::htmlelement::HTMLElement;

pub struct HTMLLabelElement {
    htmlelement: HTMLElement,
}

impl HTMLLabelElement {
    pub fn HtmlFor(&self) -> DOMString {
        None
    }

    pub fn SetHtmlFor(&mut self, _html_for: &DOMString) {
    }
}
