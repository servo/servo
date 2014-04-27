/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTableElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLTableElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTableElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTableElementDerived for EventTarget {
    fn is_htmltableelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLTableElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLTableElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLTableElement {
        HTMLTableElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLTableElement> {
        let element = HTMLTableElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLTableElementBinding::Wrap)
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

    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Border(&self) -> DOMString {
        ~""
    }

    pub fn SetBorder(&self, _border: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Frame(&self) -> DOMString {
        ~""
    }

    pub fn SetFrame(&self, _frame: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Rules(&self) -> DOMString {
        ~""
    }

    pub fn SetRules(&self, _rules: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Summary(&self) -> DOMString {
        ~""
    }

    pub fn SetSummary(&self, _summary: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self) -> DOMString {
        ~""
    }

    pub fn SetWidth(&self, _width: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn BgColor(&self) -> DOMString {
        ~""
    }

    pub fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn CellPadding(&self) -> DOMString {
        ~""
    }

    pub fn SetCellPadding(&self, _cell_padding: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn CellSpacing(&self) -> DOMString {
        ~""
    }

    pub fn SetCellSpacing(&self, _cell_spacing: DOMString) -> ErrorResult {
        Ok(())
    }
}
