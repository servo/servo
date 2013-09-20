/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLHRElement {
    htmlelement: HTMLElement,
}

impl HTMLHRElement {
    pub fn Align(&self) -> DOMString {
        None
    }

    pub fn SetAlign(&mut self, _align: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Color(&self) -> DOMString {
        None
    }

    pub fn SetColor(&mut self, _color: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn NoShade(&self) -> bool {
        false
    }

    pub fn SetNoShade(&self, _no_shade: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Size(&self) -> DOMString {
        None
    }

    pub fn SetSize(&mut self, _size: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self) -> DOMString {
        None
    }

    pub fn SetWidth(&mut self, _width: &DOMString) -> ErrorResult {
        Ok(())
    }
}
