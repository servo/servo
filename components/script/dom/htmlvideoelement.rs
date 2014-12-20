/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLVideoElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLVideoElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlmediaelement::HTMLMediaElement;
use dom::node::{Node, NodeTypeId};
use servo_util::str::DOMString;

#[dom_struct]
pub struct HTMLVideoElement {
    htmlmediaelement: HTMLMediaElement
}

impl HTMLVideoElementDerived for EventTarget {
    fn is_htmlvideoelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLVideoElement))
    }
}

impl HTMLVideoElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLVideoElement {
        HTMLVideoElement {
            htmlmediaelement: HTMLMediaElement::new_inherited(ElementTypeId::HTMLVideoElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLVideoElement> {
        let element = HTMLVideoElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLVideoElementBinding::Wrap)
    }
}

impl Reflectable for HTMLVideoElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlmediaelement.reflector()
    }
}
