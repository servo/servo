/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLParagraphElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLParagraphElementDerived;
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::Document;
use dom::element::HTMLParagraphElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};

pub struct HTMLParagraphElement {
    htmlelement: HTMLElement
}

impl HTMLParagraphElementDerived for EventTarget {
    fn is_htmlparagraphelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLParagraphElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLParagraphElement {
    pub fn new_inherited(localName: ~str, document: JSManaged<Document>) -> HTMLParagraphElement {
        HTMLParagraphElement {
            htmlelement: HTMLElement::new_inherited(HTMLParagraphElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: JSManaged<Document>) -> JSManaged<HTMLParagraphElement> {
        let element = HTMLParagraphElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLParagraphElementBinding::Wrap)
    }
}

impl HTMLParagraphElement {
    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }
}
