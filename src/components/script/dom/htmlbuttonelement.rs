/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLButtonElementBinding;
use dom::bindings::codegen::Bindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLButtonElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLButtonElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use dom::validitystate::ValidityState;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLButtonElement {
    pub htmlelement: HTMLElement
}

impl HTMLButtonElementDerived for EventTarget {
    fn is_htmlbuttonelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLButtonElementTypeId))
    }
}

impl HTMLButtonElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLButtonElement {
        HTMLButtonElement {
            htmlelement: HTMLElement::new_inherited(HTMLButtonElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLButtonElement> {
        let element = HTMLButtonElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLButtonElementBinding::Wrap)
    }
}

impl<'a> HTMLButtonElementMethods for JSRef<'a, HTMLButtonElement> {
    fn Validity(&self) -> Temporary<ValidityState> {
        let window = window_from_node(self).root();
        ValidityState::new(&*window)
    }
}

impl Reflectable for HTMLButtonElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
