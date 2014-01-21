/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLMainElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLMainElementDerived;
use dom::bindings::jsmanaged::JSManaged;
use dom::document::Document;
use dom::element::HTMLMainElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};

pub struct HTMLMainElement {
    htmlelement: HTMLElement
}

impl HTMLMainElementDerived for EventTarget {
    fn is_htmlmainelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLMainElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLMainElement {
    pub fn new_inherited(localName: ~str, document: JSManaged<Document>) -> HTMLMainElement {
        HTMLMainElement {
            htmlelement: HTMLElement::new_inherited(HTMLMainElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: JSManaged<Document>) -> JSManaged<HTMLMainElement> {
        let element = HTMLMainElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLMainElementBinding::Wrap)
    }
}
