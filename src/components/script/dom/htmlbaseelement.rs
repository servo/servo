/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLBaseElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLBaseElementDerived;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::bindings::jsmanaged::JSManaged;
use dom::document::Document;
use dom::element::HTMLBaseElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};

pub struct HTMLBaseElement {
    htmlelement: HTMLElement
}

impl HTMLBaseElementDerived for EventTarget {
    fn is_htmlbaseelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLBaseElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLBaseElement {
    pub fn new_inherited(localName: ~str, document: JSManaged<Document>) -> HTMLBaseElement {
        HTMLBaseElement {
            htmlelement: HTMLElement::new_inherited(HTMLBaseElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: JSManaged<Document>) -> JSManaged<HTMLBaseElement> {
        let element = HTMLBaseElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLBaseElementBinding::Wrap)
    }
}

impl HTMLBaseElement {
    pub fn Href(&self) -> DOMString {
        ~""
    }

    pub fn SetHref(&self, _href: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Target(&self) -> DOMString {
        ~""
    }

    pub fn SetTarget(&self, _target: DOMString) -> ErrorResult {
        Ok(())
    }
}
