/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLPreElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLPreElementDerived;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLPreElement {
    htmlelement: HTMLElement,
}

impl HTMLPreElementDerived for EventTarget {
    fn is_htmlpreelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLPreElement)))
    }
}

impl HTMLPreElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLPreElement {
        HTMLPreElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLPreElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLPreElement> {
        let element = HTMLPreElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLPreElementBinding::Wrap)
    }
}

