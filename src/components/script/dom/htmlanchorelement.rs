/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLAnchorElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLAnchorElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLAnchorElement {
    htmlelement: HTMLElement
}

impl HTMLAnchorElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLAnchorElement {
        HTMLAnchorElement {
            htmlelement: HTMLElement::new_inherited(HTMLAnchorElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLAnchorElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLAnchorElementBinding::Wrap)
    }
}

impl HTMLAnchorElement {
    pub fn Href(&self) -> Option<DOMString> {
        None
    }

    pub fn SetHref(&mut self, _href: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Target(&self) -> Option<DOMString> {
        None
    }

    pub fn SetTarget(&self, _target: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Download(&self) -> Option<DOMString> {
        None
    }

    pub fn SetDownload(&self, _download: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Ping(&self) -> Option<DOMString> {
        None
    }

    pub fn SetPing(&self, _ping: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Rel(&self) -> Option<DOMString> {
        None
    }

    pub fn SetRel(&self, _rel: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Hreflang(&self) -> Option<DOMString> {
        None
    }

    pub fn SetHreflang(&self, _href_lang: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> Option<DOMString> {
        None
    }

    pub fn SetType(&mut self, _type: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Text(&self) -> Option<DOMString> {
        None
    }

    pub fn SetText(&mut self, _text: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Coords(&self) -> Option<DOMString> {
        None
    }

    pub fn SetCoords(&mut self, _coords: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Charset(&self) -> Option<DOMString> {
        None
    }

    pub fn SetCharset(&mut self, _charset: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Name(&self) -> Option<DOMString> {
        None
    }

    pub fn SetName(&mut self, _name: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Rev(&self) -> Option<DOMString> {
        None
    }

    pub fn SetRev(&mut self, _rev: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Shape(&self) -> Option<DOMString> {
        None
    }

    pub fn SetShape(&mut self, _shape: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }
}
