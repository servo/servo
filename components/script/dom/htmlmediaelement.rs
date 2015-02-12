/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::{JSRef};
use dom::bindings::codegen::InheritTypes::HTMLMediaElementDerived;
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::NodeTypeId;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLMediaElement {
    htmlelement: HTMLElement,
}

impl HTMLMediaElementDerived for EventTarget {
    fn is_htmlmediaelement(&self) -> bool {
        match *self.type_id() {
            EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMediaElement(_)))) => true,
            _ => false
        }
    }
}

impl HTMLMediaElement {
    pub fn new_inherited(type_id: HTMLMediaElementTypeId, tag_name: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLMediaElement {
        HTMLMediaElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLMediaElement(type_id), tag_name, prefix, document)
        }
    }

    #[inline]
    pub fn htmlelement<'a>(&'a self) -> &'a HTMLElement {
        &self.htmlelement
    }
}

#[derive(Copy, PartialEq, Debug)]
#[jstraceable]
pub enum HTMLMediaElementTypeId {
    HTMLAudioElement,
    HTMLVideoElement,
}

