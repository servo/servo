/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLTableElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTableElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTableElementDerived for EventTarget {
    fn is_htmltableelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTableElementTypeId))
    }
}

impl HTMLTableElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLTableElement {
        HTMLTableElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLTableElement> {
        let element = HTMLTableElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLTableElementBinding::Wrap)
    }
}

impl Reflectable for HTMLTableElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
