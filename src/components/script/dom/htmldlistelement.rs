/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLDListElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDListElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLDListElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLDListElement {
    pub htmlelement: HTMLElement
}

impl HTMLDListElementDerived for EventTarget {
    fn is_htmldlistelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLDListElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLDListElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLDListElement {
        HTMLDListElement {
            htmlelement: HTMLElement::new_inherited(HTMLDListElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLDListElement> {
        let element = HTMLDListElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLDListElementBinding::Wrap)
    }
}

impl HTMLDListElement {
    pub fn Compact(&self) -> bool {
        false
    }

    pub fn SetCompact(&mut self, _compact: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }
}
