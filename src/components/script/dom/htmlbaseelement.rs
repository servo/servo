/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLBaseElement {
    htmlelement: HTMLElement
}

impl HTMLBaseElement {
    pub fn Href(&self) -> DOMString {
        None
    }

    pub fn SetHref(&self, _href: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Target(&self) -> DOMString {
        None
    }

    pub fn SetTarget(&self, _target: &DOMString) -> ErrorResult {
        Ok(())
    }
}
