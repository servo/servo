/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLMapElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLMapElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeTypeId};
use servo_util::str::DOMString;

#[dom_struct]
pub struct HTMLMapElement {
    htmlelement: HTMLElement
}

impl HTMLMapElementDerived for EventTarget {
    fn is_htmlmapelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLMapElement))
    }
}

impl HTMLMapElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLMapElement {
        HTMLMapElement {
            htmlelement: HTMLElement::new_inherited(ElementTypeId::HTMLMapElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLMapElement> {
        let element = HTMLMapElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLMapElementBinding::Wrap)
    }
}

impl Reflectable for HTMLMapElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
