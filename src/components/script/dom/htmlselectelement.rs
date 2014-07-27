/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLSelectElementBinding;
use dom::bindings::codegen::Bindings::HTMLSelectElementBinding::HTMLSelectElementMethods;
use dom::bindings::codegen::InheritTypes::HTMLSelectElementDerived;
use dom::bindings::codegen::UnionTypes::HTMLElementOrLong::HTMLElementOrLong;
use dom::bindings::codegen::UnionTypes::HTMLOptionElementOrHTMLOptGroupElement::HTMLOptionElementOrHTMLOptGroupElement;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLSelectElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId, window_from_node};
use dom::validitystate::ValidityState;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLSelectElement {
    pub htmlelement: HTMLElement
}

impl HTMLSelectElementDerived for EventTarget {
    fn is_htmlselectelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLSelectElementTypeId))
    }
}

impl HTMLSelectElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLSelectElement {
        HTMLSelectElement {
            htmlelement: HTMLElement::new_inherited(HTMLSelectElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLSelectElement> {
        let element = HTMLSelectElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLSelectElementBinding::Wrap)
    }
}

impl<'a> HTMLSelectElementMethods for JSRef<'a, HTMLSelectElement> {
    fn Validity(&self) -> Temporary<ValidityState> {
        let window = window_from_node(self).root();
        ValidityState::new(&*window)
    }

    // Note: this function currently only exists for test_union.html.
    fn Add(&self, _element: HTMLOptionElementOrHTMLOptGroupElement, _before: Option<HTMLElementOrLong>) {
    }
}

impl Reflectable for HTMLSelectElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
