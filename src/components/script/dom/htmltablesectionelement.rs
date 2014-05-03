/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLTableSectionElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableSectionElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLTableSectionElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTableSectionElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTableSectionElementDerived for EventTarget {
    fn is_htmltablesectionelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLTableSectionElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLTableSectionElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLTableSectionElement {
        HTMLTableSectionElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableSectionElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLTableSectionElement> {
        let element = HTMLTableSectionElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLTableSectionElementBinding::Wrap)
    }
}

pub trait HTMLTableSectionElementMethods {
    fn DeleteRow(&mut self, _index: i32) -> ErrorResult;
    fn Align(&self) -> DOMString;
    fn SetAlign(&mut self, _align: DOMString) -> ErrorResult;
    fn Ch(&self) -> DOMString;
    fn SetCh(&mut self, _ch: DOMString) -> ErrorResult;
    fn ChOff(&self) -> DOMString;
    fn SetChOff(&mut self, _ch_off: DOMString) -> ErrorResult;
    fn VAlign(&self) -> DOMString;
    fn SetVAlign(&mut self, _v_align: DOMString) -> ErrorResult;
}

impl<'a> HTMLTableSectionElementMethods for JSRef<'a, HTMLTableSectionElement> {
    fn DeleteRow(&mut self, _index: i32) -> ErrorResult {
        Ok(())
    }

    fn Align(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Ch(&self) -> DOMString {
        "".to_owned()
    }

    fn SetCh(&mut self, _ch: DOMString) -> ErrorResult {
        Ok(())
    }

    fn ChOff(&self) -> DOMString {
        "".to_owned()
    }

    fn SetChOff(&mut self, _ch_off: DOMString) -> ErrorResult {
        Ok(())
    }

    fn VAlign(&self) -> DOMString {
        "".to_owned()
    }

    fn SetVAlign(&mut self, _v_align: DOMString) -> ErrorResult {
        Ok(())
    }
}
