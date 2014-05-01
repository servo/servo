/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLOListElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLOListElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLOListElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLOListElement {
    pub htmlelement: HTMLElement,
}

impl HTMLOListElementDerived for EventTarget {
    fn is_htmlolistelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLOListElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLOListElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLOListElement {
        HTMLOListElement {
            htmlelement: HTMLElement::new_inherited(HTMLOListElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLOListElement> {
        let element = HTMLOListElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLOListElementBinding::Wrap)
    }
}

impl HTMLOListElement {
    pub fn Reversed(&self) -> bool {
        false
    }

    pub fn SetReversed(&self, _reversed: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Start(&self) -> i32 {
        0
    }

    pub fn SetStart(&mut self, _start: i32) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Compact(&self) -> bool {
        false
    }

    pub fn SetCompact(&self, _compact: bool) -> ErrorResult {
        Ok(())
    }
}
