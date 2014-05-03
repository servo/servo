/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLAreaElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLAreaElementDerived;
use dom::bindings::js::{JSRef, Temporary};
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
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLAreaElement {
        HTMLAreaElement {
            htmlelement: HTMLElement::new_inherited(HTMLAreaElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLAreaElement> {
        let element = HTMLAreaElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLAreaElementBinding::Wrap)
    }
}

pub trait HTMLAreaElementMethods {
    fn Alt(&self) -> DOMString;
    fn SetAlt(&self, _alt: DOMString) -> ErrorResult;
    fn Coords(&self) -> DOMString;
    fn SetCoords(&self, _coords: DOMString) -> ErrorResult;
    fn Shape(&self) -> DOMString;
    fn SetShape(&self, _shape: DOMString) -> ErrorResult;
    fn Href(&self) -> DOMString;
    fn SetHref(&self, _href: DOMString) -> ErrorResult;
    fn Target(&self) -> DOMString;
    fn SetTarget(&self, _target: DOMString) -> ErrorResult;
    fn Download(&self) -> DOMString;
    fn SetDownload(&self, _download: DOMString) -> ErrorResult;
    fn Ping(&self) -> DOMString;
    fn SetPing(&self, _ping: DOMString) -> ErrorResult;
    fn NoHref(&self) -> bool;
    fn SetNoHref(&mut self, _no_href: bool) -> ErrorResult;
}

impl<'a> HTMLAreaElementMethods for JSRef<'a, HTMLAreaElement> {
    fn Alt(&self) -> DOMString {
        ~""
    }

    fn SetAlt(&self, _alt: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Coords(&self) -> DOMString {
        ~""
    }

    fn SetCoords(&self, _coords: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Shape(&self) -> DOMString {
        ~""
    }

    fn SetShape(&self, _shape: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Href(&self) -> DOMString {
        ~""
    }

    fn SetHref(&self, _href: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Target(&self) -> DOMString {
        ~""
    }

    fn SetTarget(&self, _target: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Download(&self) -> DOMString {
        ~""
    }

    fn SetDownload(&self, _download: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Ping(&self) -> DOMString {
        ~""
    }

    fn SetPing(&self, _ping: DOMString) -> ErrorResult {
        Ok(())
    }

    fn NoHref(&self) -> bool {
        false
    }

    fn SetNoHref(&mut self, _no_href: bool) -> ErrorResult {
        Ok(())
    }
}
