/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTemplateElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTemplateElementDerived;
use dom::bindings::jsmanaged::JSManaged;
use dom::document::Document;
use dom::element::HTMLTemplateElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};

pub struct HTMLTemplateElement {
    htmlelement: HTMLElement,
}

impl HTMLTemplateElementDerived for EventTarget {
    fn is_htmltemplateelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLTemplateElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLTemplateElement {
    pub fn new_inherited(localName: ~str, document: JSManaged<Document>) -> HTMLTemplateElement {
        HTMLTemplateElement {
            htmlelement: HTMLElement::new_inherited(HTMLTemplateElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: JSManaged<Document>) -> JSManaged<HTMLTemplateElement> {
        let element = HTMLTemplateElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLTemplateElementBinding::Wrap)
    }
}
