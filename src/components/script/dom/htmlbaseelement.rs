/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLBaseElement {
    parent: HTMLElement
}

impl HTMLBaseElement {
    pub fn Href(&self) -> DOMString {
        null_string
    }

    pub fn SetHref(&self, _href: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Target(&self) -> DOMString {
        null_string
    }

    pub fn SetTarget(&self, _target: &DOMString, _rv: &mut ErrorResult) {
    }
}
