/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLFontElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLFontElementDerived;
use dom::bindings::js::JS;
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
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLFontElement {
        HTMLFontElement {
            htmlelement: HTMLElement::new_inherited(HTMLFontElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLFontElement> {
        let element = HTMLFontElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLFontElementBinding::Wrap)
    }
}

impl HTMLFontElement {
    pub fn Color(&self) -> DOMString {
        ~""
    }

    pub fn SetColor(&mut self, _color: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Face(&self) -> DOMString {
        ~""
    }

    pub fn SetFace(&mut self, _face: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Size(&self) -> DOMString {
        ~""
    }

    pub fn SetSize(&mut self, _size: DOMString) -> ErrorResult {
        Ok(())
    }
}
