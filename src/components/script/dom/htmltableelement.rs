/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTableElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLTableElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLTableElement {
    htmlelement: HTMLElement,
}

impl HTMLTableElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLTableElement {
        HTMLTableElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLTableElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLTableElementBinding::Wrap)
    }
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

    pub fn Align(&self) -> Option<DOMString> {
        None
    }

    pub fn SetAlign(&self, _align: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Border(&self) -> Option<DOMString> {
        None
    }

    pub fn SetBorder(&self, _border: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Frame(&self) -> Option<DOMString> {
        None
    }

    pub fn SetFrame(&self, _frame: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Rules(&self) -> Option<DOMString> {
        None
    }

    pub fn SetRules(&self, _rules: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Summary(&self) -> Option<DOMString> {
        None
    }

    pub fn SetSummary(&self, _summary: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self) -> Option<DOMString> {
        None
    }

    pub fn SetWidth(&self, _width: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn BgColor(&self) -> Option<DOMString> {
        None
    }

    pub fn SetBgColor(&self, _bg_color: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn CellPadding(&self) -> Option<DOMString> {
        None
    }

    pub fn SetCellPadding(&self, _cell_padding: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn CellSpacing(&self) -> Option<DOMString> {
        None
    }

    pub fn SetCellSpacing(&self, _cell_spacing: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }
}
