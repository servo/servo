/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLHeadElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLHeadElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::element::HTMLHeadElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLHeadElement {
    pub htmlelement: HTMLElement
}

impl HTMLHeadElementDerived for EventTarget {
    fn is_htmlheadelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLHeadElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLHeadElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLHeadElement {
        HTMLHeadElement {
            htmlelement: HTMLElement::new_inherited(HTMLHeadElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLHeadElement> {
        let element = HTMLHeadElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLHeadElementBinding::Wrap)
    }
}

pub trait HTMLHeadElementMethods {
}
