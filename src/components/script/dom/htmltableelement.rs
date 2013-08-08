/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, null_string, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLTableElement {
    parent: HTMLElement,
}

impl HTMLTableElement {

    pub fn DeleteCaption(&self) {
    }

    pub fn DeleteTHead(&self) {
    }

    pub fn DeleteTFoot(&self) {
    }

    pub fn DeleteRow(&mut self, _index: i32, _rv: &mut ErrorResult) {
    }

    pub fn Sortable(&self) -> bool {
        false
    }

    pub fn SetSortable(&self, _sortable: bool) {
    }

    pub fn StopSorting(&self) {
    }

    pub fn Align(&self) -> DOMString {
        null_string
    }

    pub fn SetAlign(&self, _align: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Border(&self) -> DOMString {
        null_string
    }

    pub fn SetBorder(&self, _border: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Frame(&self) -> DOMString {
        null_string
    }

    pub fn SetFrame(&self, _frame: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Rules(&self) -> DOMString {
        null_string
    }

    pub fn SetRules(&self, _rules: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Summary(&self) -> DOMString {
        null_string
    }

    pub fn SetSummary(&self, _summary: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn Width(&self) -> DOMString {
        null_string
    }

    pub fn SetWidth(&self, _width: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn BgColor(&self) -> DOMString {
        null_string
    }

    pub fn SetBgColor(&self, _bg_color: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn CellPadding(&self) -> DOMString {
        null_string
    }

    pub fn SetCellPadding(&self, _cell_padding: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn CellSpacing(&self) -> DOMString {
        null_string
    }

    pub fn SetCellSpacing(&self, _cell_spacing: &DOMString, _rv: &mut ErrorResult) {
    }
}
