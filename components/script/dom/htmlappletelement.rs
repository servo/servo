/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLAppletElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLAppletElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

#[dom_struct]
pub struct HTMLAppletElement {
    htmlelement: HTMLElement
}

impl HTMLAppletElementDerived for EventTarget {
    fn is_htmlappletelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAppletElement)))
    }
}

impl HTMLAppletElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLAppletElement {
        HTMLAppletElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLAppletElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLAppletElement> {
        let element = HTMLAppletElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLAppletElementBinding::Wrap)
    }
}

