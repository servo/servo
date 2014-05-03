/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLFontElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLFontElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLFontElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLFontElement {
    pub htmlelement: HTMLElement
}

impl HTMLFontElementDerived for EventTarget {
    fn is_htmlfontelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLFontElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLFontElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLFontElement {
        HTMLFontElement {
            htmlelement: HTMLElement::new_inherited(HTMLFontElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLFontElement> {
        let element = HTMLFontElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLFontElementBinding::Wrap)
    }
}

pub trait HTMLFontElementMethods {
    fn Color(&self) -> DOMString;
    fn SetColor(&mut self, _color: DOMString) -> ErrorResult;
    fn Face(&self) -> DOMString;
    fn SetFace(&mut self, _face: DOMString) -> ErrorResult;
    fn Size(&self) -> DOMString;
    fn SetSize(&mut self, _size: DOMString) -> ErrorResult;
}

impl<'a> HTMLFontElementMethods for JSRef<'a, HTMLFontElement> {
    fn Color(&self) -> DOMString {
        ~""
    }

    fn SetColor(&mut self, _color: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Face(&self) -> DOMString {
        ~""
    }

    fn SetFace(&mut self, _face: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Size(&self) -> DOMString {
        ~""
    }

    fn SetSize(&mut self, _size: DOMString) -> ErrorResult {
        Ok(())
    }
}
