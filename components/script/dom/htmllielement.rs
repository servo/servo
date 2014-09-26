/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLLIElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLLIElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLLIElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLLIElement {
    pub htmlelement: HTMLElement,
}

impl HTMLLIElementDerived for EventTarget {
    fn is_htmllielement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLLIElementTypeId))
    }
}

impl HTMLLIElement {
    fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLLIElement {
        HTMLLIElement {
            htmlelement: HTMLElement::new_inherited(HTMLLIElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLLIElement> {
        let element = HTMLLIElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLLIElementBinding::Wrap)
    }
}

impl Reflectable for HTMLLIElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
