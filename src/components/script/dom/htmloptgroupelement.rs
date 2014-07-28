/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLOptGroupElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLOptGroupElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLOptGroupElement {
    pub htmlelement: HTMLElement
}

impl HTMLOptGroupElementDerived for EventTarget {
    fn is_htmloptgroupelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLOptGroupElementTypeId))
    }
}

impl HTMLOptGroupElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLOptGroupElement {
        HTMLOptGroupElement {
            htmlelement: HTMLElement::new_inherited(HTMLOptGroupElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLOptGroupElement> {
        let element = HTMLOptGroupElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLOptGroupElementBinding::Wrap)
    }
}

impl Reflectable for HTMLOptGroupElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
