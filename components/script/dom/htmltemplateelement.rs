/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTemplateElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTemplateElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

#[dom_struct]
pub struct HTMLTemplateElement {
    htmlelement: HTMLElement,
}

impl HTMLTemplateElementDerived for EventTarget {
    fn is_htmltemplateelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTemplateElement)))
    }
}

impl HTMLTemplateElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLTemplateElement {
        HTMLTemplateElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLTemplateElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLTemplateElement> {
        let element = HTMLTemplateElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTemplateElementBinding::Wrap)
    }
}

