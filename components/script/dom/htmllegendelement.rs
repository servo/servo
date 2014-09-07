/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLLegendElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLLegendElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLLegendElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLLegendElement {
    pub htmlelement: HTMLElement,
}

impl HTMLLegendElementDerived for EventTarget {
    fn is_htmllegendelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLLegendElementTypeId))
    }
}

impl HTMLLegendElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLLegendElement {
        HTMLLegendElement {
            htmlelement: HTMLElement::new_inherited(HTMLLegendElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLLegendElement> {
        let element = HTMLLegendElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLLegendElementBinding::Wrap)
    }
}

impl Reflectable for HTMLLegendElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
