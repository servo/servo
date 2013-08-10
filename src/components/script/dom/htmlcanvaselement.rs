/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLCanvasElement {
    parent: HTMLElement,
}

impl HTMLCanvasElement {
    pub fn Width(&self) -> u32 {
        0
    }

    pub fn SetWidth(&mut self, _width: u32, _rv: &mut ErrorResult) {
    }

    pub fn Height(&self) -> u32 {
        0
    }

    pub fn SetHeight(&mut self, _height: u32, _rv: &mut ErrorResult) {
    }
}
