/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableHeaderCellElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableHeaderCellElementDerived;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::htmlelement::HTMLElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmltablecellelement::{HTMLTableCellElement, HTMLTableCellElementTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLTableHeaderCellElement {
    htmltablecellelement: HTMLTableCellElement,
}

impl HTMLTableHeaderCellElementDerived for EventTarget {
    fn is_htmltableheadercellelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(
                                                   ElementTypeId::HTMLElement(
                                                   HTMLElementTypeId::HTMLTableCellElement(
                                                   HTMLTableCellElementTypeId::HTMLTableHeaderCellElement))))
    }
}

impl HTMLTableHeaderCellElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLTableHeaderCellElement {
        HTMLTableHeaderCellElement {
            htmltablecellelement: HTMLTableCellElement::new_inherited(
                HTMLTableCellElementTypeId::HTMLTableHeaderCellElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLTableHeaderCellElement> {
        let element = HTMLTableHeaderCellElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTableHeaderCellElementBinding::Wrap)
    }
}

