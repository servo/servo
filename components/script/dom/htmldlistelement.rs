/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLDListElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLDListElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLDListElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLDListElement {
    pub htmlelement: HTMLElement
}

impl HTMLDListElementDerived for EventTarget {
    fn is_htmldlistelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLDListElementTypeId))
    }
}

impl HTMLDListElement {
    fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLDListElement {
        HTMLDListElement {
            htmlelement: HTMLElement::new_inherited(HTMLDListElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLDListElement> {
        let element = HTMLDListElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLDListElementBinding::Wrap)
    }
}

impl Reflectable for HTMLDListElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
