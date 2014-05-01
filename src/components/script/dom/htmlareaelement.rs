/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLAreaElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLAreaElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLAreaElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLAreaElement {
    pub htmlelement: HTMLElement
}

impl HTMLAreaElementDerived for EventTarget {
    fn is_htmlareaelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLAreaElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLAreaElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLAreaElement {
        HTMLAreaElement {
            htmlelement: HTMLElement::new_inherited(HTMLAreaElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLAreaElement> {
        let element = HTMLAreaElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLAreaElementBinding::Wrap)
    }
}

impl HTMLAreaElement {
    pub fn Alt(&self) -> DOMString {
        ~""
    }

    pub fn SetAlt(&self, _alt: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Coords(&self) -> DOMString {
        ~""
    }

    pub fn SetCoords(&self, _coords: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Shape(&self) -> DOMString {
        ~""
    }

    pub fn SetShape(&self, _shape: DOMString) -> ErrorResult {
        Ok(())
    }

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

    pub fn Download(&self) -> DOMString {
        ~""
    }

    pub fn SetDownload(&self, _download: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Ping(&self) -> DOMString {
        ~""
    }

    pub fn SetPing(&self, _ping: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn NoHref(&self) -> bool {
        false
    }

    pub fn SetNoHref(&mut self, _no_href: bool) -> ErrorResult {
        Ok(())
    }
}
