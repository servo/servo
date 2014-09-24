/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTableColElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableColElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLTableColElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLTableColElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTableColElementDerived for EventTarget {
    fn is_htmltablecolelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTableColElementTypeId))
    }
}

impl HTMLTableColElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLTableColElement {
        HTMLTableColElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableColElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLTableColElement> {
        let element = HTMLTableColElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLTableColElementBinding::Wrap)
    }
}

impl Reflectable for HTMLTableColElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
