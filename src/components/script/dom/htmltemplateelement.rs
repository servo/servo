/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLTemplateElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTemplateElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::element::HTMLTemplateElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTemplateElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTemplateElementDerived for EventTarget {
    fn is_htmltemplateelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTemplateElementTypeId))
    }
}

impl HTMLTemplateElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLTemplateElement {
        HTMLTemplateElement {
            htmlelement: HTMLElement::new_inherited(HTMLTemplateElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLTemplateElement> {
        let element = HTMLTemplateElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLTemplateElementBinding::Wrap)
    }
}

pub trait HTMLTemplateElementMethods {
}
