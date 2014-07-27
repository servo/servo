/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLOptionElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLOptionElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLOptionElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLOptionElement {
    pub htmlelement: HTMLElement
}

impl HTMLOptionElementDerived for EventTarget {
    fn is_htmloptionelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLOptionElementTypeId))
    }
}

impl HTMLOptionElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLOptionElement {
        HTMLOptionElement {
            htmlelement: HTMLElement::new_inherited(HTMLOptionElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLOptionElement> {
        let element = HTMLOptionElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLOptionElementBinding::Wrap)
    }
}

impl Reflectable for HTMLOptionElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
