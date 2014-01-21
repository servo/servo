/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLTableCaptionElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableCaptionElementDerived;
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::Document;
use dom::element::HTMLTableCaptionElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};

pub struct HTMLTableCaptionElement {
    htmlelement: HTMLElement
}

impl HTMLTableCaptionElementDerived for EventTarget {
    fn is_htmltablecaptionelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLTableCaptionElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLTableCaptionElement {
    pub fn new_inherited(localName: ~str, document: JSManaged<Document>) -> HTMLTableCaptionElement {
        HTMLTableCaptionElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableCaptionElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: JSManaged<Document>) -> JSManaged<HTMLTableCaptionElement> {
        let element = HTMLTableCaptionElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLTableCaptionElementBinding::Wrap)
    }
}

impl HTMLTableCaptionElement {
    pub fn Align(&self) -> DOMString {
        ~""
    }
    
    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }
}
