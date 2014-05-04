/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLDListElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDListElementDerived;
use dom::bindings::js::{JSRef, Temporary};
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
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLDListElement {
        HTMLDListElement {
            htmlelement: HTMLElement::new_inherited(HTMLDListElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLDListElement> {
        let element = HTMLDListElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLDListElementBinding::Wrap)
    }
}

pub trait HTMLDListElementMethods {
    fn Compact(&self) -> bool;
    fn SetCompact(&mut self, _compact: bool) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&mut self, _type: DOMString) -> ErrorResult;
}

impl<'a> HTMLDListElementMethods for JSRef<'a, HTMLDListElement> {
    fn Compact(&self) -> bool {
        false
    }

    fn SetCompact(&mut self, _compact: bool) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        "".to_owned()
    }

    fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }
}

