/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLAppletElement {
    htmlelement: HTMLElement
}

impl HTMLAppletElement {
    pub fn Align(&self) -> DOMString {
        None
    }

    pub fn SetAlign(&mut self, _align: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Alt(&self) -> DOMString {
        None
    }

    pub fn SetAlt(&self, _alt: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Archive(&self) -> DOMString {
        None
    }

    pub fn SetArchive(&self, _archive: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Code(&self) -> DOMString {
        None
    }

    pub fn SetCode(&self, _code: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn CodeBase(&self) -> DOMString {
        None
    }

    pub fn SetCodeBase(&self, _code_base: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Height(&self) -> DOMString {
        None
    }

    pub fn SetHeight(&self, _height: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Hspace(&self) -> u32 {
        0
    }

    pub fn SetHspace(&mut self, _hspace: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Object(&self) -> DOMString {
        None
    }

    pub fn SetObject(&mut self, _object: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Vspace(&self) -> u32 {
        0
    }

    pub fn SetVspace(&mut self, _vspace: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self) -> DOMString {
        None
    }

    pub fn SetWidth(&mut self, _width: &DOMString) -> ErrorResult {
        Ok(())
    }
}
