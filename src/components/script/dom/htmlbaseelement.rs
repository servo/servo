/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLBaseElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLBaseElementDerived;
use dom::bindings::error::ErrorResult;
use dom::bindings::js::{JSRef, Temporary};
use dom::document::Document;
use dom::element::HTMLBaseElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLBaseElement {
    pub htmlelement: HTMLElement
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
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLBaseElement {
        HTMLBaseElement {
            htmlelement: HTMLElement::new_inherited(HTMLBaseElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLBaseElement> {
        let element = HTMLBaseElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLBaseElementBinding::Wrap)
    }
}

pub trait HTMLBaseElementMethods {
    fn Href(&self) -> DOMString;
    fn SetHref(&self, _href: DOMString) -> ErrorResult;
    fn Target(&self) -> DOMString;
    fn SetTarget(&self, _target: DOMString) -> ErrorResult;
}

impl<'a> HTMLBaseElementMethods for JSRef<'a, HTMLBaseElement> {
    fn Href(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHref(&self, _href: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Target(&self) -> DOMString {
        "".to_owned()
    }

    fn SetTarget(&self, _target: DOMString) -> ErrorResult {
        Ok(())
    }
}
