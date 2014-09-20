/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLHRElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLHRElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLHRElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
#[must_root]
pub struct HTMLHRElement {
    pub htmlelement: HTMLElement,
}

impl HTMLHRElementDerived for EventTarget {
    fn is_htmlhrelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLHRElementTypeId))
    }
}

impl HTMLHRElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLHRElement {
        HTMLHRElement {
            htmlelement: HTMLElement::new_inherited(HTMLHRElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLHRElement> {
        let element = HTMLHRElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLHRElementBinding::Wrap)
    }
}

impl Reflectable for HTMLHRElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
