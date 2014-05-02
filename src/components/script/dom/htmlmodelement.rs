/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLModElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLModElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLModElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLModElement {
    pub htmlelement: HTMLElement
}

impl HTMLModElementDerived for EventTarget {
    fn is_htmlmodelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLModElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLModElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLModElement {
        HTMLModElement {
            htmlelement: HTMLElement::new_inherited(HTMLModElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLModElement> {
        let element = HTMLModElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLModElementBinding::Wrap)
    }
}

impl HTMLModElement {
    pub fn Cite(&self) -> DOMString {
        ~""
    }

    pub fn SetCite(&mut self, _cite: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn DateTime(&self) -> DOMString {
        ~""
    }

    pub fn SetDateTime(&mut self, _datetime: DOMString) -> ErrorResult {
        Ok(())
    }
}
