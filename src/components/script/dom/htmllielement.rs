/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLLIElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLLIElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLLIElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLLIElement {
    pub htmlelement: HTMLElement,
}

impl HTMLLIElementDerived for EventTarget {
    fn is_htmllielement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLLIElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLLIElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLLIElement {
        HTMLLIElement {
            htmlelement: HTMLElement::new_inherited(HTMLLIElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLLIElement> {
        let element = HTMLLIElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLLIElementBinding::Wrap)
    }
}

impl HTMLLIElement {
    pub fn Value(&self) -> i32 {
        0
    }

    pub fn SetValue(&mut self, _value: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }
}
