/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableHeaderCellElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableHeaderCellElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmltablecellelement::HTMLTableCellElement;
use dom::node::{Node, NodeTypeId};
use servo_util::str::DOMString;

#[dom_struct]
pub struct HTMLTableHeaderCellElement {
    htmltablecellelement: HTMLTableCellElement,
}

impl HTMLTableHeaderCellElementDerived for EventTarget {
    fn is_htmltableheadercellelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLTableHeaderCellElement))
    }
}

impl HTMLTableHeaderCellElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLTableHeaderCellElement {
        HTMLTableHeaderCellElement {
            htmltablecellelement: HTMLTableCellElement::new_inherited(ElementTypeId::HTMLTableHeaderCellElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLTableHeaderCellElement> {
        let element = HTMLTableHeaderCellElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTableHeaderCellElementBinding::Wrap)
    }
}

impl Reflectable for HTMLTableHeaderCellElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmltablecellelement.reflector()
    }
}
