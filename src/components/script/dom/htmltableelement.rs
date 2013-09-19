/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::{DOMString, ErrorResult};
use dom::htmlelement::HTMLElement;

pub struct HTMLTableElement {
    htmlelement: HTMLElement,
}

impl HTMLTableElement {

    pub fn DeleteCaption(&self) {
    }

    pub fn DeleteTHead(&self) {
    }

    pub fn DeleteTFoot(&self) {
    }

    pub fn DeleteRow(&mut self, _index: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Sortable(&self) -> bool {
        false
    }

    pub fn SetSortable(&self, _sortable: bool) {
    }

    pub fn StopSorting(&self) {
    }

    pub fn Align(&self) -> DOMString {
        None
    }

    pub fn SetAlign(&self, _align: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Border(&self) -> DOMString {
        None
    }

    pub fn SetBorder(&self, _border: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Frame(&self) -> DOMString {
        None
    }

    pub fn SetFrame(&self, _frame: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Rules(&self) -> DOMString {
        None
    }

    pub fn SetRules(&self, _rules: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Summary(&self) -> DOMString {
        None
    }

    pub fn SetSummary(&self, _summary: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self) -> DOMString {
        None
    }

    pub fn SetWidth(&self, _width: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn BgColor(&self) -> DOMString {
        None
    }

    pub fn SetBgColor(&self, _bg_color: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn CellPadding(&self) -> DOMString {
        None
    }

    pub fn SetCellPadding(&self, _cell_padding: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn CellSpacing(&self) -> DOMString {
        None
    }

    pub fn SetCellSpacing(&self, _cell_spacing: &DOMString) -> ErrorResult {
        Ok(())
    }
}
