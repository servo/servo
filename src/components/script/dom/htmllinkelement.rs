/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLLinkElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLLinkElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLLinkElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLLinkElement {
    pub htmlelement: HTMLElement,
}

impl HTMLLinkElementDerived for EventTarget {
    fn is_htmllinkelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLLinkElementTypeId))
    }
}

impl HTMLLinkElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLLinkElement {
        HTMLLinkElement {
            htmlelement: HTMLElement::new_inherited(HTMLLinkElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLLinkElement> {
        let element = HTMLLinkElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLLinkElementBinding::Wrap)
    }
}

impl Reflectable for HTMLLinkElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
