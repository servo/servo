/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLSpanElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLSpanElementDerived;
use dom::bindings::js::JS;
use dom::document::Document;
use dom::element::HTMLSpanElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLSpanElement {
    pub htmlelement: HTMLElement
}

impl HTMLSpanElementDerived for EventTarget {
    fn is_htmlspanelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLSpanElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLSpanElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLSpanElement {
        HTMLSpanElement {
            htmlelement: HTMLElement::new_inherited(HTMLSpanElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLSpanElement> {
        let element = HTMLSpanElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLSpanElementBinding::Wrap)
    }
}
