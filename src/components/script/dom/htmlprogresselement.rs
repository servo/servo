/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLProgressElement {
    parent: HTMLElement,
}

impl HTMLProgressElement {
    pub fn Value(&self) -> f64 {
        0f64
    }

    pub fn SetValue(&mut self, _value: f64, _rv: &mut ErrorResult) {
    }

    pub fn Max(&self) -> f64 {
        0f64
    }

    pub fn SetMax(&mut self, _max: f64, _rv: &mut ErrorResult) {
    }

    pub fn Position(&self) -> f64 {
        0f64
    }

    pub fn GetPositiom(&self, _rv: &mut ErrorResult) -> f64 {
        0f64
    }
}
