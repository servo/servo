/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLLIElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLLIElementDerived;
use dom::bindings::js::{JSRef, Temporary};
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
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLLIElementTypeId))
    }
}

impl HTMLLIElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLLIElement {
        HTMLLIElement {
            htmlelement: HTMLElement::new_inherited(HTMLLIElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLLIElement> {
        let element = HTMLLIElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLLIElementBinding::Wrap)
    }
}

pub trait HTMLLIElementMethods {
    fn Value(&self) -> i32;
    fn SetValue(&mut self, _value: i32) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&mut self, _type: DOMString) -> ErrorResult;
}

impl<'a> HTMLLIElementMethods for JSRef<'a, HTMLLIElement> {
    fn Value(&self) -> i32 {
        0
    }

    fn SetValue(&mut self, _value: i32) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        "".to_owned()
    }

    fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }
}
