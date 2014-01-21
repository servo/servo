/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDataElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDataElementDerived;
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::Document;
use dom::element::HTMLDataElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};

pub struct HTMLDataElement {
    htmlelement: HTMLElement
}

impl HTMLDataElementDerived for EventTarget {
    fn is_htmldataelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLDataElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLDataElement {
    pub fn new_inherited(localName: ~str, document: JSManaged<Document>) -> HTMLDataElement {
        HTMLDataElement {
            htmlelement: HTMLElement::new_inherited(HTMLDataElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: JSManaged<Document>) -> JSManaged<HTMLDataElement> {
        let element = HTMLDataElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLDataElementBinding::Wrap)
    }
}

impl HTMLDataElement {
    pub fn Value(&self) -> DOMString {
        ~""
    }

    pub fn SetValue(&mut self, _value: DOMString) -> ErrorResult {
        Ok(())
    }
}
