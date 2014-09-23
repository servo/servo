/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLOutputElementBinding;
use dom::bindings::codegen::Bindings::HTMLOutputElementBinding::HTMLOutputElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLOutputElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLOutputElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use dom::validitystate::ValidityState;
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
pub struct HTMLOutputElement {
    pub htmlelement: HTMLElement
}

impl HTMLOutputElementDerived for EventTarget {
    fn is_htmloutputelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLOutputElementTypeId))
    }
}

impl HTMLOutputElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLOutputElement {
        HTMLOutputElement {
            htmlelement: HTMLElement::new_inherited(HTMLOutputElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLOutputElement> {
        let element = HTMLOutputElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLOutputElementBinding::Wrap)
    }
}

impl<'a> HTMLOutputElementMethods for JSRef<'a, HTMLOutputElement> {
    fn Validity(self) -> Temporary<ValidityState> {
        let window = window_from_node(self).root();
        ValidityState::new(*window)
    }
}

impl Reflectable for HTMLOutputElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
