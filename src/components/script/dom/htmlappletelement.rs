/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLAppletElement {
    parent: HTMLElement
}

impl HTMLAppletElement {
    pub fn Align(&self) -> DOMString {
        null_string
    }

    pub fn SetAlign(&mut self, _align: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Alt(&self) -> DOMString {
        null_string
    }

    pub fn SetAlt(&self, _alt: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Archive(&self) -> DOMString {
        null_string
    }

    pub fn SetArchive(&self, _archive: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Code(&self) -> DOMString {
        null_string
    }

    pub fn SetCode(&self, _code: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn CodeBase(&self) -> DOMString {
        null_string
    }

    pub fn SetCodeBase(&self, _code_base: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Height(&self) -> DOMString {
        null_string
    }

    pub fn SetHeight(&self, _height: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Hspace(&self) -> u32 {
        0
    }

    pub fn SetHspace(&mut self, _hspace: u32, _rv: &mut ErrorResult) {
    }

    pub fn Name(&self) -> DOMString {
        null_string
    }

    pub fn SetName(&mut self, _name: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Object(&self) -> DOMString {
        null_string
    }

    pub fn SetObject(&mut self, _object: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Vspace(&self) -> u32 {
        0
    }

    pub fn SetVspace(&mut self, _vspace: u32, _rv: &mut ErrorResult) {
    }

    pub fn Width(&self) -> DOMString {
        null_string
    }

    pub fn SetWidth(&mut self, _width: &DOMString, _rv: &mut ErrorResult) {
    }
}
