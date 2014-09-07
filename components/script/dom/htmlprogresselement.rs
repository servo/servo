/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLProgressElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLProgressElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLProgressElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLProgressElement {
    pub htmlelement: HTMLElement,
}

impl HTMLProgressElementDerived for EventTarget {
    fn is_htmlprogresselement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLProgressElementTypeId))
    }
}

impl HTMLProgressElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLProgressElement {
        HTMLProgressElement {
            htmlelement: HTMLElement::new_inherited(HTMLProgressElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLProgressElement> {
        let element = HTMLProgressElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLProgressElementBinding::Wrap)
    }
}

impl Reflectable for HTMLProgressElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
