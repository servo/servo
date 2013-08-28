/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult, null_string};
use dom::htmlelement::HTMLElement;

pub struct HTMLParamElement {
    parent: HTMLElement
}

impl HTMLParamElement {
    pub fn Name(&self) -> DOMString {
        null_string
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Value(&self) -> DOMString {
        null_string
    }

    pub fn SetValue(&mut self, _value: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Type(&self) -> DOMString {
        null_string
    }

    pub fn SetType(&mut self, _type: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn ValueType(&self) -> DOMString {
        null_string
    }

    pub fn SetValueType(&mut self, _value_type: &DOMString, _rv: &mut ErrorResult) {
    }
}