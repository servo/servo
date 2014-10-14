/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::js::{JSRef};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::bindings::codegen::InheritTypes::HTMLMediaElementDerived;
use dom::document::Document;
use dom::element::{ElementTypeId, HTMLAudioElementTypeId, HTMLVideoElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::ElementNodeTypeId;
use servo_util::str::DOMString;

#[jstraceable]
#[must_root]
#[privatize]
pub struct HTMLMediaElement {
    htmlelement: HTMLElement,
}

impl HTMLMediaElementDerived for EventTarget {
    fn is_htmlmediaelement(&self) -> bool {
        match *self.type_id() {
            NodeTargetTypeId(ElementNodeTypeId(HTMLVideoElementTypeId)) |
            NodeTargetTypeId(ElementNodeTypeId(HTMLAudioElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLMediaElement {
    pub fn new_inherited(type_id: ElementTypeId, tag_name: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLMediaElement {
        HTMLMediaElement {
            htmlelement: HTMLElement::new_inherited(type_id, tag_name, prefix, document)
        }
    }

    #[inline]
    pub fn htmlelement<'a>(&'a self) -> &'a HTMLElement {
        &self.htmlelement
    }
}

impl Reflectable for HTMLMediaElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
