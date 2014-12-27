/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLBRElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLBRElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeTypeId};
use servo_util::str::DOMString;

#[dom_struct]
pub struct HTMLBRElement {
    htmlelement: HTMLElement,
}

impl HTMLBRElementDerived for EventTarget {
    fn is_htmlbrelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLBRElement))
    }
}

impl HTMLBRElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLBRElement {
        HTMLBRElement {
            htmlelement: HTMLElement::new_inherited(ElementTypeId::HTMLBRElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLBRElement> {
        let element = HTMLBRElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLBRElementBinding::Wrap)
    }
}

