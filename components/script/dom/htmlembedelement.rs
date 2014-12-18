/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLEmbedElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLEmbedElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeTypeId};
use servo_util::str::DOMString;

#[dom_struct]
pub struct HTMLEmbedElement {
    htmlelement: HTMLElement
}

impl HTMLEmbedElementDerived for EventTarget {
    fn is_htmlembedelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLEmbedElement))
    }
}

impl HTMLEmbedElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLEmbedElement {
        HTMLEmbedElement {
            htmlelement: HTMLElement::new_inherited(ElementTypeId::HTMLEmbedElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLEmbedElement> {
        let element = HTMLEmbedElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLEmbedElementBinding::Wrap)
    }
}

impl Reflectable for HTMLEmbedElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
