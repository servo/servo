/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrHelpers};
use dom::bindings::codegen::Bindings::HTMLTableRowElementBinding;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLTableRowElementDerived};
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeTypeId};
use dom::virtualmethods::VirtualMethods;

use cssparser::RGBA;
use servo_util::str::{mod, DOMString};
use std::cell::Cell;

#[dom_struct]
pub struct HTMLTableRowElement {
    htmlelement: HTMLElement,
    background_color: Cell<Option<RGBA>>,
}

impl HTMLTableRowElementDerived for EventTarget {
    fn is_htmltablerowelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLTableRowElement))
    }
}

impl HTMLTableRowElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>)
                     -> HTMLTableRowElement {
        HTMLTableRowElement {
            htmlelement: HTMLElement::new_inherited(ElementTypeId::HTMLTableRowElement,
                                                    localName,
                                                    prefix,
                                                    document),
            background_color: Cell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>)
               -> Temporary<HTMLTableRowElement> {
        Node::reflect_node(box HTMLTableRowElement::new_inherited(localName, prefix, document),
                           document,
                           HTMLTableRowElementBinding::Wrap)
    }
}

