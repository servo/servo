/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLPreElement {
    htmlelement: HTMLElement,
}

impl HTMLPreElement {
    pub fn Width(&self) -> i32 {
        0
    }

    pub fn SetWidth(&mut self, _width: i32) -> ErrorResult {
        Ok(())
    }
}
