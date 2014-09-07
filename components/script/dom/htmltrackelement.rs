/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTrackElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTrackElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLTrackElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTrackElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTrackElementDerived for EventTarget {
    fn is_htmltrackelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTrackElementTypeId))
    }
}

impl HTMLTrackElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLTrackElement {
        HTMLTrackElement {
            htmlelement: HTMLElement::new_inherited(HTMLTrackElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLTrackElement> {
        let element = HTMLTrackElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLTrackElementBinding::Wrap)
    }
}

impl Reflectable for HTMLTrackElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
