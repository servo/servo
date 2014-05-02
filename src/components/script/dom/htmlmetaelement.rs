/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLMetaElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLMetaElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLMetaElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLMetaElement {
    pub htmlelement: HTMLElement,
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
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLMetaElement {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(HTMLMetaElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLMetaElement> {
        let element = HTMLMetaElement::new_inherited(localName, document.clone());
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
