/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLSpanElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLSpanElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLSpanElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
#[must_root]
pub struct HTMLSpanElement {
    pub htmlelement: HTMLElement
}

impl HTMLSpanElementDerived for EventTarget {
    fn is_htmlspanelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLSpanElementTypeId))
    }
}

impl HTMLSpanElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLSpanElement {
        HTMLSpanElement {
            htmlelement: HTMLElement::new_inherited(HTMLSpanElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLSpanElement> {
        let element = HTMLSpanElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLSpanElementBinding::Wrap)
    }
}

impl Reflectable for HTMLSpanElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
