/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLPreElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLPreElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLPreElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLPreElement {
    pub htmlelement: HTMLElement,
}

impl HTMLPreElementDerived for EventTarget {
    fn is_htmlpreelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLPreElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLPreElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLPreElement {
        HTMLPreElement {
            htmlelement: HTMLElement::new_inherited(HTMLPreElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLPreElement> {
        let element = HTMLPreElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLPreElementBinding::Wrap)
    }
}

pub trait HTMLPreElementMethods {
    fn Width(&self) -> i32;
    fn SetWidth(&mut self, _width: i32) -> ErrorResult;
}

impl<'a> HTMLPreElementMethods for JSRef<'a, HTMLPreElement> {
    fn Width(&self) -> i32 {
        0
    }

    fn SetWidth(&mut self, _width: i32) -> ErrorResult {
        Ok(())
    }
}
