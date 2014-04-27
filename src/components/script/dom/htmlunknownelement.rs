/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLUnknownElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLUnknownElementDerived;
use dom::bindings::js::JS;
use dom::document::Document;
use dom::element::HTMLUnknownElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLUnknownElement {
    pub htmlelement: HTMLElement
}

impl HTMLUnknownElementDerived for EventTarget {
    fn is_htmlunknownelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLUnknownElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLUnknownElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLUnknownElement {
        HTMLUnknownElement {
            htmlelement: HTMLElement::new_inherited(HTMLUnknownElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLUnknownElement> {
        let element = HTMLUnknownElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLUnknownElementBinding::Wrap)
    }
}
