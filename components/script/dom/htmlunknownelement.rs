/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLUnknownElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLUnknownElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeTypeId};
use servo_util::str::DOMString;

#[dom_struct]
pub struct HTMLUnknownElement {
    htmlelement: HTMLElement
}

impl HTMLUnknownElementDerived for EventTarget {
    fn is_htmlunknownelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLUnknownElement))
    }
}

impl HTMLUnknownElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLUnknownElement {
        HTMLUnknownElement {
            htmlelement: HTMLElement::new_inherited(ElementTypeId::HTMLUnknownElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLUnknownElement> {
        let element = HTMLUnknownElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLUnknownElementBinding::Wrap)
    }
}

impl Reflectable for HTMLUnknownElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
