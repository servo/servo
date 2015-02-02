/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLFontElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLFontElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use util::str::DOMString;

#[dom_struct]
pub struct HTMLFontElement {
    htmlelement: HTMLElement
}

impl HTMLFontElementDerived for EventTarget {
    fn is_htmlfontelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFontElement)))
    }
}

impl HTMLFontElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLFontElement {
        HTMLFontElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLFontElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLFontElement> {
        let element = HTMLFontElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLFontElementBinding::Wrap)
    }
}

