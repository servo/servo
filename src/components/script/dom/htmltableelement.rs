/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLTableElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableElementDerived;
use dom::bindings::js::{JSRef, Temporary};
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
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLTableElement {
        HTMLTableElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLTableElement> {
        let element = HTMLTableElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLTableElementBinding::Wrap)
    }
}

pub trait HTMLTableElementMethods {
    fn DeleteCaption(&self);
    fn DeleteTHead(&self);
    fn DeleteTFoot(&self);
    fn DeleteRow(&mut self, _index: i32) -> ErrorResult;
    fn Sortable(&self) -> bool;
    fn SetSortable(&self, _sortable: bool);
    fn StopSorting(&self);
    fn Align(&self) -> DOMString;
    fn SetAlign(&self, _align: DOMString) -> ErrorResult;
    fn Border(&self) -> DOMString;
    fn SetBorder(&self, _border: DOMString) -> ErrorResult;
    fn Frame(&self) -> DOMString;
    fn SetFrame(&self, _frame: DOMString) -> ErrorResult;
    fn Rules(&self) -> DOMString;
    fn SetRules(&self, _rules: DOMString) -> ErrorResult;
    fn Summary(&self) -> DOMString;
    fn SetSummary(&self, _summary: DOMString) -> ErrorResult;
    fn Width(&self) -> DOMString;
    fn SetWidth(&self, _width: DOMString) -> ErrorResult;
    fn BgColor(&self) -> DOMString;
    fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult;
    fn CellPadding(&self) -> DOMString;
    fn SetCellPadding(&self, _cell_padding: DOMString) -> ErrorResult;
    fn CellSpacing(&self) -> DOMString;
    fn SetCellSpacing(&self, _cell_spacing: DOMString) -> ErrorResult;
}

impl<'a> HTMLTableElementMethods for JSRef<'a, HTMLTableElement> {
    fn DeleteCaption(&self) {
    }

    fn DeleteTHead(&self) {
    }

    fn DeleteTFoot(&self) {
    }

    fn DeleteRow(&mut self, _index: i32) -> ErrorResult {
        Ok(())
    }

    fn Sortable(&self) -> bool {
        false
    }

    fn SetSortable(&self, _sortable: bool) {
    }

    fn StopSorting(&self) {
    }

    fn Align(&self) -> DOMString {
        ~""
    }

    fn SetAlign(&self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Border(&self) -> DOMString {
        ~""
    }

    fn SetBorder(&self, _border: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Frame(&self) -> DOMString {
        ~""
    }

    fn SetFrame(&self, _frame: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Rules(&self) -> DOMString {
        ~""
    }

    fn SetRules(&self, _rules: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Summary(&self) -> DOMString {
        ~""
    }

    fn SetSummary(&self, _summary: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Width(&self) -> DOMString {
        ~""
    }

    fn SetWidth(&self, _width: DOMString) -> ErrorResult {
        Ok(())
    }

    fn BgColor(&self) -> DOMString {
        ~""
    }

    fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult {
        Ok(())
    }

    fn CellPadding(&self) -> DOMString {
        ~""
    }

    fn SetCellPadding(&self, _cell_padding: DOMString) -> ErrorResult {
        Ok(())
    }

    fn CellSpacing(&self) -> DOMString {
        ~""
    }

    fn SetCellSpacing(&self, _cell_spacing: DOMString) -> ErrorResult {
        Ok(())
    }
}
