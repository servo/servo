/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLUListElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLUListElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLUListElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLUListElement {
    pub htmlelement: HTMLElement
}

impl HTMLUListElementDerived for EventTarget {
    fn is_htmlulistelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLUListElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLUListElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLUListElement {
        HTMLUListElement {
            htmlelement: HTMLElement::new_inherited(HTMLUListElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLUListElement> {
        let element = HTMLUListElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLUListElementBinding::Wrap)
    }
}

impl HTMLUListElement {
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
