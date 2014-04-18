/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLAudioElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLAudioElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::element::HTMLAudioElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlmediaelement::HTMLMediaElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLAudioElement {
    pub htmlmediaelement: HTMLMediaElement
}

impl HTMLAudioElementDerived for EventTarget {
    fn is_htmlaudioelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLAudioElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLAudioElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLAudioElement {
        HTMLAudioElement {
            htmlmediaelement: HTMLMediaElement::new_inherited(HTMLAudioElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLAudioElement> {
        let element = HTMLAudioElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLAudioElementBinding::Wrap)
    }
}

pub trait HTMLAudioElementMethods {
}
