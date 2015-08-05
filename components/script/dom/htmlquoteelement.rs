/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLQuoteElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLQuoteElementDerived;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLQuoteElement {
    htmlelement: HTMLElement,
}

impl HTMLQuoteElementDerived for EventTarget {
    fn is_htmlquoteelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLQuoteElement)))
    }
}

impl HTMLQuoteElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLQuoteElement {
        HTMLQuoteElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLQuoteElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLQuoteElement> {
        let element = HTMLQuoteElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLQuoteElementBinding::Wrap)
    }
}

