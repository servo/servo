/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLAreaElementBinding;
use dom::bindings::utils::ErrorResult;
use dom::document::AbstractDocument;
use dom::element::HTMLAreaElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLAreaElement {
    htmlelement: HTMLElement
}

impl HTMLAreaElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLAreaElement {
        HTMLAreaElement {
            htmlelement: HTMLElement::new_inherited(HTMLAreaElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLAreaElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLAreaElementBinding::Wrap)
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
