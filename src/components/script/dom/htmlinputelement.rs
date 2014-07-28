/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLInputElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLInputElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLInputElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLInputElement {
    pub htmlelement: HTMLElement,
}

impl HTMLInputElementDerived for EventTarget {
    fn is_htmlinputelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLInputElementTypeId))
    }
}

impl HTMLInputElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLInputElement {
        HTMLInputElement {
            htmlelement: HTMLElement::new_inherited(HTMLInputElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLInputElement> {
        let element = HTMLInputElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLInputElementBinding::Wrap)
    }
}

impl Reflectable for HTMLInputElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
