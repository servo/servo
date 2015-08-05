/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableCaptionElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableCaptionElementDerived;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLTableCaptionElement {
    htmlelement: HTMLElement
}

impl HTMLTableCaptionElementDerived for EventTarget {
    fn is_htmltablecaptionelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableCaptionElement)))
    }
}

impl HTMLTableCaptionElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLTableCaptionElement {
        HTMLTableCaptionElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLTableCaptionElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLTableCaptionElement> {
        let element = HTMLTableCaptionElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTableCaptionElementBinding::Wrap)
    }
}

