/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLUListElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLUListElementDerived;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLUListElement {
    htmlelement: HTMLElement
}

impl HTMLUListElementDerived for EventTarget {
    fn is_htmlulistelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLUListElement)))
    }
}

impl HTMLUListElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> HTMLUListElement {
        HTMLUListElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLUListElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLUListElement> {
        let element = HTMLUListElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLUListElementBinding::Wrap)
    }
}

