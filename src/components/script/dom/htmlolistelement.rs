/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLOListElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLOListElementDerived;
use dom::bindings::js::{JSRef, Temporary};
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
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLOListElement {
        HTMLOListElement {
            htmlelement: HTMLElement::new_inherited(HTMLOListElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLOListElement> {
        let element = HTMLOListElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLOListElementBinding::Wrap)
    }
}

pub trait HTMLOListElementMethods {
    fn Reversed(&self) -> bool;
    fn SetReversed(&self, _reversed: bool) -> ErrorResult;
    fn Start(&self) -> i32;
    fn SetStart(&mut self, _start: i32) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&mut self, _type: DOMString) -> ErrorResult;
    fn Compact(&self) -> bool;
    fn SetCompact(&self, _compact: bool) -> ErrorResult;
}

impl<'a> HTMLOListElementMethods for JSRef<'a, HTMLOListElement> {
    fn Reversed(&self) -> bool {
        false
    }

    fn SetReversed(&self, _reversed: bool) -> ErrorResult {
        Ok(())
    }

    fn Start(&self) -> i32 {
        0
    }

    fn SetStart(&mut self, _start: i32) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        ~""
    }

    fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Compact(&self) -> bool {
        false
    }

    fn SetCompact(&self, _compact: bool) -> ErrorResult {
        Ok(())
    }
}
