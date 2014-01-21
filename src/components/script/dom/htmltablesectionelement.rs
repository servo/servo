/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTableSectionElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableSectionElementDerived;
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::Document;
use dom::element::HTMLTableSectionElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};

pub struct HTMLTableSectionElement {
    htmlelement: HTMLElement,
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
    pub fn new_inherited(localName: ~str, document: JSManaged<Document>) -> HTMLTableSectionElement {
        HTMLTableSectionElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableSectionElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: JSManaged<Document>) -> JSManaged<HTMLTableSectionElement> {
        let element = HTMLTableSectionElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLTableSectionElementBinding::Wrap)
    }
}

impl HTMLTableSectionElement {
    pub fn DeleteRow(&mut self, _index: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Ch(&self) -> DOMString {
        ~""
    }

    pub fn SetCh(&mut self, _ch: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn ChOff(&self) -> DOMString {
        ~""
    }

    pub fn SetChOff(&mut self, _ch_off: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn VAlign(&self) -> DOMString {
        ~""
    }

    pub fn SetVAlign(&mut self, _v_align: DOMString) -> ErrorResult {
        Ok(())
    }
}
