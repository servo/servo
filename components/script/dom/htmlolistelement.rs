/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLOListElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLOListElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLOListElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
#[must_root]
pub struct HTMLOListElement {
    pub htmlelement: HTMLElement,
}

impl HTMLOListElementDerived for EventTarget {
    fn is_htmlolistelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLOListElementTypeId))
    }
}

impl HTMLOListElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLOListElement {
        HTMLOListElement {
            htmlelement: HTMLElement::new_inherited(HTMLOListElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLOListElement> {
        let element = HTMLOListElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLOListElementBinding::Wrap)
    }
}

impl Reflectable for HTMLOListElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
