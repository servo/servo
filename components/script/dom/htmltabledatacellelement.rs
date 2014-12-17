/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableDataCellElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableDataCellElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmltablecellelement::HTMLTableCellElement;
use dom::node::{Node, NodeTypeId};
use servo_util::str::DOMString;

#[dom_struct]
pub struct HTMLTableDataCellElement {
    htmltablecellelement: HTMLTableCellElement,
}

impl HTMLTableDataCellElementDerived for EventTarget {
    fn is_htmltabledatacellelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLTableDataCellElement))
    }
}

impl HTMLTableDataCellElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLTableDataCellElement {
        HTMLTableDataCellElement {
            htmltablecellelement: HTMLTableCellElement::new_inherited(ElementTypeId::HTMLTableDataCellElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>)
               -> Temporary<HTMLTableDataCellElement> {
        Node::reflect_node(box HTMLTableDataCellElement::new_inherited(localName,
                                                                       prefix,
                                                                       document),
                           document,
                           HTMLTableDataCellElementBinding::Wrap)
    }
}

impl Reflectable for HTMLTableDataCellElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmltablecellelement.reflector()
    }
}
