/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLTableCellElement {
    parent: HTMLElement,
}

impl HTMLTableCellElement {
    pub fn ColSpan(&self) -> u32 {
        0
    }

    pub fn SetColSpan(&self, _col_span: u32, _rv: &mut ErrorResult) {
    }

    pub fn RowSpan(&self) -> u32 {
        0
    }

    pub fn SetRowSpan(&self, _col_span: u32, _rv: &mut ErrorResult) {
    }

    pub fn Headers(&self) -> DOMString {
        null_string
    }

    pub fn SetHeaders(&self, _headers: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn CellIndex(&self) -> i32 {
        0
    }

    pub fn GetCellIndex(&self, _cell_index: i32, _rv: &mut ErrorResult) {
    }

    pub fn Abbr(&self) -> DOMString {
        null_string
    }

    pub fn SetAbbr(&self, _abbr: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Scope(&self) -> DOMString {
        null_string
    }

    pub fn SetScope(&self, _abbr: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Align(&self) -> DOMString {
        null_string
    }

    pub fn SetAlign(&self, _align: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Axis(&self) -> DOMString {
        null_string
    }

    pub fn SetAxis(&self, _axis: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Height(&self) -> DOMString {
        null_string
    }

    pub fn SetHeight(&self, _height: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Width(&self) -> DOMString {
        null_string
    }

    pub fn SetWidth(&self, _width: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Ch(&self) -> DOMString {
        null_string
    }

    pub fn SetCh(&self, _ch: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn ChOff(&self) -> DOMString {
        null_string
    }

    pub fn SetChOff(&self, _ch_off: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn NoWrap(&self) -> bool {
        false
    }

    pub fn SetNoWrap(&self, _no_wrap: bool, _rv: &mut ErrorResult) {
    }

    pub fn VAlign(&self) -> DOMString {
        null_string
    }

    pub fn SetVAlign(&self, _valign: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn BgColor(&self) -> DOMString {
        null_string
    }

    pub fn SetBgColor(&self, _bg_color: &DOMString, _rv: &mut ErrorResult) {
    }
}
