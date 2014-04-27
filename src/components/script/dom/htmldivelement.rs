/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDivElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDivElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLDivElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLDivElement {
    pub htmlelement: HTMLElement
}

impl HTMLDivElementDerived for EventTarget {
    fn is_htmldivelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLDivElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLDivElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLDivElement {
        HTMLDivElement {
            htmlelement: HTMLElement::new_inherited(HTMLDivElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLDivElement> {
        let element = HTMLDivElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLDivElementBinding::Wrap)
    }
}

impl HTMLDivElement {
    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }
}
