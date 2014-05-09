/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLMainElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLMainElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::element::HTMLMainElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLMainElement {
    pub htmlelement: HTMLElement
}

impl HTMLMainElementDerived for EventTarget {
    fn is_htmlmainelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLMainElementTypeId))
    }
}

impl HTMLMainElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLMainElement {
        HTMLMainElement {
            htmlelement: HTMLElement::new_inherited(HTMLMainElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLMainElement> {
        let element = HTMLMainElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLMainElementBinding::Wrap)
    }
}

pub trait HTMLMainElementMethods {
}
