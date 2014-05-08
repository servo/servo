/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLMetaElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLMetaElementDerived;
use dom::bindings::js::{JSRef, Temporary};
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
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLMetaElementTypeId))
    }
}

impl HTMLMetaElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLMetaElement {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(HTMLMetaElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLMetaElement> {
        let element = HTMLMetaElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLMetaElementBinding::Wrap)
    }
}

pub trait HTMLMetaElementMethods {
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, _name: DOMString) -> ErrorResult;
    fn HttpEquiv(&self) -> DOMString;
    fn SetHttpEquiv(&mut self, _http_equiv: DOMString) -> ErrorResult;
    fn Content(&self) -> DOMString;
    fn SetContent(&mut self, _content: DOMString) -> ErrorResult;
    fn Scheme(&self) -> DOMString;
    fn SetScheme(&mut self, _scheme: DOMString) -> ErrorResult;
}

impl<'a> HTMLMetaElementMethods for JSRef<'a, HTMLMetaElement> {
    fn Name(&self) -> DOMString {
        "".to_owned()
    }

    fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    fn HttpEquiv(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHttpEquiv(&mut self, _http_equiv: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Content(&self) -> DOMString {
        "".to_owned()
    }

    fn SetContent(&mut self, _content: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Scheme(&self) -> DOMString {
        "".to_owned()
    }

    fn SetScheme(&mut self, _scheme: DOMString) -> ErrorResult {
        Ok(())
    }
}
