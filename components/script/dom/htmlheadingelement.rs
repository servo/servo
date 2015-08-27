/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLHeadingElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLHeadingElementDerived;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

#[derive(JSTraceable, HeapSizeOf)]
pub enum HeadingLevel {
    Heading1,
    Heading2,
    Heading3,
    Heading4,
    Heading5,
    Heading6,
}

#[dom_struct]
pub struct HTMLHeadingElement {
    htmlelement: HTMLElement,
    level: HeadingLevel,
}

impl HTMLHeadingElementDerived for EventTarget {
    fn is_htmlheadingelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHeadingElement)))
    }
}

impl HTMLHeadingElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document,
                     level: HeadingLevel) -> HTMLHeadingElement {
        HTMLHeadingElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLHeadingElement, localName, prefix, document),
            level: level,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document,
               level: HeadingLevel) -> Root<HTMLHeadingElement> {
        let element = HTMLHeadingElement::new_inherited(localName, prefix, document, level);
        Node::reflect_node(box element, document, HTMLHeadingElementBinding::Wrap)
    }
}
