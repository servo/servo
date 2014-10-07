/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLUnknownElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLUnknownElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLUnknownElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLUnknownElement {
    pub htmlelement: HTMLElement
}

impl HTMLUnknownElementDerived for EventTarget {
    fn is_htmlunknownelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLUnknownElementTypeId))
    }
}

impl HTMLUnknownElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLUnknownElement {
        HTMLUnknownElement {
            htmlelement: HTMLElement::new_inherited(HTMLUnknownElementTypeId, localName, prefix, document)
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
