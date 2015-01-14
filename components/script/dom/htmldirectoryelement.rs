/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDirectoryElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDirectoryElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use servo_util::str::DOMString;

#[dom_struct]
pub struct HTMLDirectoryElement {
    htmlelement: HTMLElement
}

impl HTMLDirectoryElementDerived for EventTarget {
    fn is_htmldirectoryelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDirectoryElement)))
    }
}

impl HTMLDirectoryElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLDirectoryElement {
        HTMLDirectoryElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLDirectoryElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLDirectoryElement> {
        let element = HTMLDirectoryElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLDirectoryElementBinding::Wrap)
    }
}

