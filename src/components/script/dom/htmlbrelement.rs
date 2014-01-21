/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLBRElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLBRElementDerived;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::bindings::jsmanaged::JSManaged;
use dom::document::Document;
use dom::element::HTMLBRElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};

pub struct HTMLBRElement {
    htmlelement: HTMLElement,
}

impl HTMLBRElementDerived for EventTarget {
    fn is_htmlbrelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLBRElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLBRElement {
    pub fn new_inherited(localName: ~str, document: JSManaged<Document>) -> HTMLBRElement {
        HTMLBRElement {
            htmlelement: HTMLElement::new_inherited(HTMLBRElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: JSManaged<Document>) -> JSManaged<HTMLBRElement> {
        let element = HTMLBRElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLBRElementBinding::Wrap)
    }
}

impl HTMLBRElement {
    pub fn Clear(&self) -> DOMString {
        ~""
    }

    pub fn SetClear(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }
}
