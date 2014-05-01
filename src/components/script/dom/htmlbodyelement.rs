/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLBodyElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLBodyElementDerived;
use dom::bindings::error::ErrorResult;
use dom::bindings::js::JS;
use dom::document::Document;
use dom::element::HTMLBodyElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLBodyElement {
    pub htmlelement: HTMLElement
}

impl HTMLBodyElementDerived for EventTarget {
    fn is_htmlbodyelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLBodyElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLBodyElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLBodyElement {
        HTMLBodyElement {
            htmlelement: HTMLElement::new_inherited(HTMLBodyElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLBodyElement> {
        let element = HTMLBodyElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLBodyElementBinding::Wrap)
    }
}

impl HTMLBodyElement {
    pub fn Text(&self) -> DOMString {
        ~""
    }

    pub fn SetText(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Link(&self) -> DOMString {
        ~""
    }

    pub fn SetLink(&self, _link: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn VLink(&self) -> DOMString {
        ~""
    }

    pub fn SetVLink(&self, _v_link: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn ALink(&self) -> DOMString {
        ~""
    }

    pub fn SetALink(&self, _a_link: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn BgColor(&self) -> DOMString {
        ~""
    }

    pub fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Background(&self) -> DOMString {
        ~""
    }

    pub fn SetBackground(&self, _background: DOMString) -> ErrorResult {
        Ok(())
    }
}
