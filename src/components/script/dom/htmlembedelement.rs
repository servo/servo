/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLEmbedElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLEmbedElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLEmbedElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLEmbedElement {
    pub htmlelement: HTMLElement
}

impl HTMLEmbedElementDerived for EventTarget {
    fn is_htmlembedelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLEmbedElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLEmbedElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLEmbedElement {
        HTMLEmbedElement {
            htmlelement: HTMLElement::new_inherited(HTMLEmbedElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLEmbedElement> {
        let element = HTMLEmbedElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLEmbedElementBinding::Wrap)
    }
}

pub trait HTMLEmbedElementMethods {
    fn Src(&self) -> DOMString;
    fn SetSrc(&mut self, _src: DOMString) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&mut self, _type: DOMString) -> ErrorResult;
    fn Width(&self) -> DOMString;
    fn SetWidth(&mut self, _width: DOMString) -> ErrorResult;
    fn Height(&self) -> DOMString;
    fn SetHeight(&mut self, _height: DOMString) -> ErrorResult;
    fn Align(&self) -> DOMString;
    fn SetAlign(&mut self, _type: DOMString) -> ErrorResult;
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, _type: DOMString) -> ErrorResult;
    fn GetSVGDocument(&self) -> Option<Temporary<Document>>;
}

impl<'a> HTMLEmbedElementMethods for JSRef<'a, HTMLEmbedElement> {
    fn Src(&self) -> DOMString {
        "".to_owned()
    }

    fn SetSrc(&mut self, _src: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        "".to_owned()
    }

    fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Width(&self) -> DOMString {
        "".to_owned()
    }

    fn SetWidth(&mut self, _width: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Height(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHeight(&mut self, _height: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Align(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAlign(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Name(&self) -> DOMString {
        "".to_owned()
    }

    fn SetName(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    fn GetSVGDocument(&self) -> Option<Temporary<Document>> {
        None
    }
}
