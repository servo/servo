/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLParagraphElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLParagraphElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLParagraphElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
#[must_root]
pub struct HTMLParagraphElement {
    pub htmlelement: HTMLElement
}

impl HTMLParagraphElementDerived for EventTarget {
    fn is_htmlparagraphelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLParagraphElementTypeId))
    }
}

impl HTMLParagraphElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLParagraphElement {
        HTMLParagraphElement {
            htmlelement: HTMLElement::new_inherited(HTMLParagraphElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLParagraphElement> {
        let element = HTMLParagraphElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLParagraphElementBinding::Wrap)
    }
}

impl Reflectable for HTMLParagraphElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
