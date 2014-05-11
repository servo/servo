/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLUnknownElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLUnknownElementDerived;
use dom::bindings::js::{JSRef, Temporary};
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
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLUnknownElementTypeId))
    }
}

impl HTMLUnknownElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLUnknownElement {
        HTMLUnknownElement {
            htmlelement: HTMLElement::new_inherited(HTMLUnknownElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLUnknownElement> {
        let element = HTMLUnknownElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLUnknownElementBinding::Wrap)
    }
}

pub trait HTMLUnknownElementMethods {
}
