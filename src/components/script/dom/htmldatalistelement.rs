/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDataListElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDataListElementDerived;
use dom::bindings::jsmanaged::JSManaged;
use dom::document::Document;
use dom::element::HTMLDataListElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};

pub struct HTMLDataListElement {
    htmlelement: HTMLElement
}

impl HTMLDataListElementDerived for EventTarget {
    fn is_htmldatalistelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLDataListElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLDataListElement {
    pub fn new_inherited(localName: ~str, document: JSManaged<Document>) -> HTMLDataListElement {
        HTMLDataListElement {
            htmlelement: HTMLElement::new_inherited(HTMLDataListElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: JSManaged<Document>) -> JSManaged<HTMLDataListElement> {
        let element = HTMLDataListElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLDataListElementBinding::Wrap)
    }
}

impl HTMLDataListElement {
    pub fn Options(&self) -> JSManaged<HTMLCollection> {
        let window = self.htmlelement.element.node.owner_doc().value().window;
        HTMLCollection::new(window, ~[])
    }
}
