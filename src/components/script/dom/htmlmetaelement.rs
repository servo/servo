/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLMetaElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLMetaElementDerived;
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::Document;
use dom::element::HTMLMetaElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};

pub struct HTMLMetaElement {
    htmlelement: HTMLElement,
}

impl HTMLMetaElementDerived for EventTarget {
    fn is_htmlmetaelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLMetaElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLMetaElement {
    pub fn new_inherited(localName: ~str, document: JSManaged<Document>) -> HTMLMetaElement {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(HTMLMetaElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: JSManaged<Document>) -> JSManaged<HTMLMetaElement> {
        let element = HTMLMetaElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLMetaElementBinding::Wrap)
    }
}

impl HTMLMetaElement {
    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn HttpEquiv(&self) -> DOMString {
        ~""
    }

    pub fn SetHttpEquiv(&mut self, _http_equiv: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Content(&self) -> DOMString {
        ~""
    }

    pub fn SetContent(&mut self, _content: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Scheme(&self) -> DOMString {
        ~""
    }

    pub fn SetScheme(&mut self, _scheme: DOMString) -> ErrorResult {
        Ok(())
    }
}
