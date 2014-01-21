/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLSourceElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLSourceElementDerived;
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::Document;
use dom::element::HTMLSourceElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};

pub struct HTMLSourceElement {
    htmlelement: HTMLElement
}

impl HTMLSourceElementDerived for EventTarget {
    fn is_htmlsourceelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLSourceElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLSourceElement {
    pub fn new_inherited(localName: ~str, document: JSManaged<Document>) -> HTMLSourceElement {
        HTMLSourceElement {
            htmlelement: HTMLElement::new_inherited(HTMLSourceElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: JSManaged<Document>) -> JSManaged<HTMLSourceElement> {
        let element = HTMLSourceElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLSourceElementBinding::Wrap)
    }
}

impl HTMLSourceElement {
    pub fn Src(&self) -> DOMString {
        ~""
    }
    
    pub fn SetSrc(&mut self, _src: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }
    
    pub fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Media(&self) -> DOMString {
        ~""
    }
    
    pub fn SetMedia(&mut self, _media: DOMString) -> ErrorResult {
        Ok(())
    }
}
