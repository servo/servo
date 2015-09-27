/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableDataCellElementBinding;
use dom::bindings::codegen::InheritTypes::{ElementTypeId, EventTargetTypeId};
use dom::bindings::codegen::InheritTypes::{HTMLElementTypeId, HTMLTableDataCellElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLTableCellElementTypeId, NodeTypeId};
use dom::bindings::js::Root;
use dom::bindings::utils::TopDOMClass;
use dom::document::Document;
use dom::eventtarget::EventTarget;
use dom::htmltablecellelement::HTMLTableCellElement;
use dom::node::Node;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLTableDataCellElement {
    htmltablecellelement: HTMLTableCellElement,
}

impl HTMLTableDataCellElementDerived for EventTarget {
    fn is_htmltabledatacellelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(
                                                   ElementTypeId::HTMLElement(
                                                   HTMLElementTypeId::HTMLTableCellElement(
                                                   HTMLTableCellElementTypeId::HTMLTableDataCellElement))))
    }
}

impl HTMLTableDataCellElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLTableDataCellElement {
        HTMLTableDataCellElement {
            htmltablecellelement:
                HTMLTableCellElement::new_inherited(
                    HTMLTableCellElementTypeId::HTMLTableDataCellElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: &Document)
               -> Root<HTMLTableDataCellElement> {
        Node::reflect_node(box HTMLTableDataCellElement::new_inherited(localName,
                                                                       prefix,
                                                                       document),
                           document,
                           HTMLTableDataCellElementBinding::Wrap)
    }
}
