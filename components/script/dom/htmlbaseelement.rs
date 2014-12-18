/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLBaseElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLBaseElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeTypeId};
use servo_util::str::DOMString;

#[dom_struct]
pub struct HTMLBaseElement {
    htmlelement: HTMLElement
}

impl HTMLBaseElementDerived for EventTarget {
    fn is_htmlbaseelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLBaseElement))
    }
}

impl HTMLBaseElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLBaseElement {
        HTMLBaseElement {
            htmlelement: HTMLElement::new_inherited(ElementTypeId::HTMLBaseElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLBaseElement> {
        let element = HTMLBaseElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLBaseElementBinding::Wrap)
    }
}

impl Reflectable for HTMLBaseElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
