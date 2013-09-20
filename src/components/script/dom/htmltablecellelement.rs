/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLTableCellElement {
    htmlelement: HTMLElement,
}

impl HTMLTableCellElement {
    pub fn ColSpan(&self) -> u32 {
        0
    }

    pub fn SetColSpan(&self, _col_span: u32) -> ErrorResult {
        Ok(())
    }

    pub fn RowSpan(&self) -> u32 {
        0
    }

    pub fn SetRowSpan(&self, _col_span: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Headers(&self) -> DOMString {
        None
    }

    pub fn SetHeaders(&self, _headers: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn CellIndex(&self) -> i32 {
        0
    }

    pub fn GetCellIndex(&self, _cell_index: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Abbr(&self) -> DOMString {
        None
    }

    pub fn SetAbbr(&self, _abbr: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Scope(&self) -> DOMString {
        None
    }

    pub fn SetScope(&self, _abbr: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Align(&self) -> DOMString {
        None
    }

    pub fn SetAlign(&self, _align: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Axis(&self) -> DOMString {
        None
    }

    pub fn SetAxis(&self, _axis: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Height(&self) -> DOMString {
        None
    }

    pub fn SetHeight(&self, _height: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self) -> DOMString {
        None
    }

    pub fn SetWidth(&self, _width: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Ch(&self) -> DOMString {
        None
    }

    pub fn SetCh(&self, _ch: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn ChOff(&self) -> DOMString {
        None
    }

    pub fn SetChOff(&self, _ch_off: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn NoWrap(&self) -> bool {
        false
    }

    pub fn SetNoWrap(&self, _no_wrap: bool) -> ErrorResult {
        Ok(())
    }

    pub fn VAlign(&self) -> DOMString {
        None
    }

    pub fn SetVAlign(&self, _valign: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn BgColor(&self) -> DOMString {
        None
    }

    pub fn SetBgColor(&self, _bg_color: &DOMString) -> ErrorResult {
        Ok(())
    }
}
