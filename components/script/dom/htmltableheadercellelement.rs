/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableHeaderCellElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableHeaderCellElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLTableHeaderCellElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmltablecellelement::HTMLTableCellElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
#[must_root]
pub struct HTMLTableHeaderCellElement {
    pub htmltablecellelement: HTMLTableCellElement,
}

impl HTMLTableHeaderCellElementDerived for EventTarget {
    fn is_htmltableheadercellelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTableHeaderCellElementTypeId))
    }
}

impl HTMLTableHeaderCellElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLTableHeaderCellElement {
        HTMLTableHeaderCellElement {
            htmltablecellelement: HTMLTableCellElement::new_inherited(HTMLTableHeaderCellElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLTableHeaderCellElement> {
        let element = HTMLTableHeaderCellElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLTableHeaderCellElementBinding::Wrap)
    }
}

impl Reflectable for HTMLTableHeaderCellElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmltablecellelement.reflector()
    }
}
