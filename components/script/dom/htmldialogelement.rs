/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDialogElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDialogElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

#[dom_struct]
pub struct HTMLDialogElement {
    htmlelement: HTMLElement,
}

impl HTMLDialogElementDerived for EventTarget {
    fn is_htmldialogelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLDialogElement)))
    }
}

impl HTMLDialogElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLDialogElement {
        HTMLDialogElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLDialogElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLDialogElement> {
        let element = HTMLDialogElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLDialogElementBinding::Wrap)
    }
}

