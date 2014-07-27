/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTextAreaElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLTextAreaElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTextAreaElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTextAreaElementDerived for EventTarget {
    fn is_htmltextareaelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTextAreaElementTypeId))
    }
}

impl HTMLTextAreaElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLTextAreaElement {
        HTMLTextAreaElement {
            htmlelement: HTMLElement::new_inherited(HTMLTextAreaElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLTextAreaElement> {
        let element = HTMLTextAreaElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLTextAreaElementBinding::Wrap)
    }
}

impl Reflectable for HTMLTextAreaElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
