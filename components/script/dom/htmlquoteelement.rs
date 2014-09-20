/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLQuoteElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLQuoteElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLQuoteElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
#[must_root]
pub struct HTMLQuoteElement {
    pub htmlelement: HTMLElement,
}

impl HTMLQuoteElementDerived for EventTarget {
    fn is_htmlquoteelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLQuoteElementTypeId))
    }
}

impl HTMLQuoteElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLQuoteElement {
        HTMLQuoteElement {
            htmlelement: HTMLElement::new_inherited(HTMLQuoteElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLQuoteElement> {
        let element = HTMLQuoteElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLQuoteElementBinding::Wrap)
    }
}

impl Reflectable for HTMLQuoteElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
