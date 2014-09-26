/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableDataCellElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableDataCellElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLTableDataCellElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmltablecellelement::HTMLTableCellElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLTableDataCellElement {
    pub htmltablecellelement: HTMLTableCellElement,
}

impl HTMLTableDataCellElementDerived for EventTarget {
    fn is_htmltabledatacellelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTableDataCellElementTypeId))
    }
}

impl HTMLTableDataCellElement {
    fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLTableDataCellElement {
        HTMLTableDataCellElement {
            htmltablecellelement: HTMLTableCellElement::new_inherited(HTMLTableDataCellElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLTableDataCellElement> {
        let element = HTMLTableDataCellElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLTableDataCellElementBinding::Wrap)
    }
}

impl Reflectable for HTMLTableDataCellElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmltablecellelement.reflector()
    }
}
