/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLStyleElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLStyleElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLStyleElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLStyleElement {
    htmlelement: HTMLElement,
}

impl HTMLStyleElementDerived for EventTarget {
    fn is_htmlstyleelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLStyleElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLStyleElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLStyleElement {
        HTMLStyleElement {
            htmlelement: HTMLElement::new_inherited(HTMLStyleElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLStyleElement> {
        let element = HTMLStyleElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLStyleElementBinding::Wrap)
    }
}

impl HTMLStyleElement {
    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&self, _disabled: bool) {
    }

    pub fn Media(&self) -> DOMString {
        ~""
    }

    pub fn SetMedia(&mut self, _media: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Scoped(&self) -> bool {
        false
    }

    pub fn SetScoped(&self, _scoped: bool) -> ErrorResult {
        Ok(())
    }
}
