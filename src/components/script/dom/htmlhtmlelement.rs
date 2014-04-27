/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLHtmlElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLHtmlElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLHtmlElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLHtmlElement {
    pub htmlelement: HTMLElement
}

impl HTMLHtmlElementDerived for EventTarget {
    fn is_htmlhtmlelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLHtmlElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLHtmlElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLHtmlElement {
        HTMLHtmlElement {
            htmlelement: HTMLElement::new_inherited(HTMLHtmlElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLHtmlElement> {
        let element = HTMLHtmlElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLHtmlElementBinding::Wrap)
    }
}

impl HTMLHtmlElement {
    pub fn Version(&self) -> DOMString {
        ~""
    }

    pub fn SetVersion(&mut self, _version: DOMString) -> ErrorResult {
        Ok(())
    }
}
