/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLEmbedElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLEmbedElementDerived;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

#[dom_struct]
pub struct HTMLEmbedElement {
    htmlelement: HTMLElement
}

impl HTMLEmbedElementDerived for EventTarget {
    fn is_htmlembedelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLEmbedElement)))
    }
}

impl HTMLEmbedElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> HTMLEmbedElement {
        HTMLEmbedElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLEmbedElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLEmbedElement> {
        let element = HTMLEmbedElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLEmbedElementBinding::Wrap)
    }
}
