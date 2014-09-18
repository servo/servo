/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLUListElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLUListElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLUListElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
#[must_root]
pub struct HTMLUListElement {
    pub htmlelement: HTMLElement
}

impl HTMLUListElementDerived for EventTarget {
    fn is_htmlulistelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLUListElementTypeId))
    }
}

impl HTMLUListElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLUListElement {
        HTMLUListElement {
            htmlelement: HTMLElement::new_inherited(HTMLUListElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLUListElement> {
        let element = HTMLUListElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLUListElementBinding::Wrap)
    }
}

impl Reflectable for HTMLUListElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
