/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::{ElementTypeId, EventTargetTypeId};
use dom::bindings::codegen::InheritTypes::{HTMLElementTypeId, HTMLMediaElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLMediaElementTypeId, NodeTypeId};
use dom::bindings::utils::TopDOMClass;
use dom::document::Document;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLMediaElement {
    htmlelement: HTMLElement,
}

impl HTMLMediaElementDerived for EventTarget {
    fn is_htmlmediaelement(&self) -> bool {
        match *self.type_id() {
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMediaElement(_)))) => true,
            _ => false
        }
    }
}

impl HTMLMediaElement {
    pub fn new_inherited(type_id: HTMLMediaElementTypeId, tag_name: DOMString,
                         prefix: Option<DOMString>, document: &Document)
                         -> HTMLMediaElement {
        HTMLMediaElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLMediaElement(type_id), tag_name, prefix, document)
        }
    }

    #[inline]
    pub fn htmlelement(&self) -> &HTMLElement {
        &self.htmlelement
    }
}
