/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDirectoryElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDirectoryElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLDirectoryElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLDirectoryElement {
    pub htmlelement: HTMLElement
}

impl HTMLDirectoryElementDerived for EventTarget {
    fn is_htmldirectoryelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLDirectoryElementTypeId))
    }
}

impl HTMLDirectoryElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLDirectoryElement {
        HTMLDirectoryElement {
            htmlelement: HTMLElement::new_inherited(HTMLDirectoryElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLDirectoryElement> {
        let element = HTMLDirectoryElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLDirectoryElementBinding::Wrap)
    }
}

impl Reflectable for HTMLDirectoryElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
