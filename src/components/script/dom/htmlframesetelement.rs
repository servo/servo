/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLFrameSetElement {
    htmlelement: HTMLElement
}

impl HTMLFrameSetElement {
    pub fn Cols(&self) -> DOMString {
        None
    }

    pub fn SetCols(&mut self, _cols: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Rows(&self) -> DOMString {
        None
    }

    pub fn SetRows(&mut self, _rows: &DOMString) -> ErrorResult {
        Ok(())
    }
}
