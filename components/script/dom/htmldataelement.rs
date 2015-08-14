/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDataElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDataElementDerived;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLDataElement {
    htmlelement: HTMLElement
}

impl HTMLDataElementDerived for EventTarget {
    fn is_htmldataelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDataElement)))
    }
}

impl HTMLDataElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLDataElement {
        HTMLDataElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLDataElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLDataElement> {
        let element = HTMLDataElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLDataElementBinding::Wrap)
    }
}

