/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLLinkElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLLinkElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLLinkElement {
    htmlelement: HTMLElement,
}

impl HTMLLinkElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLLinkElement {
        HTMLLinkElement {
            htmlelement: HTMLElement::new_inherited(HTMLLinkElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLLinkElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLLinkElementBinding::Wrap)
    }
}

impl HTMLLinkElement {
    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disable: bool) {
    }

    pub fn Href(&self) -> Option<DOMString> {
        None
    }

    pub fn SetHref(&mut self, _href: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn CrossOrigin(&self) -> Option<DOMString> {
        None
    }

    pub fn SetCrossOrigin(&mut self, _cross_origin: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Rel(&self) -> Option<DOMString> {
        None
    }

    pub fn SetRel(&mut self, _rel: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Media(&self) -> Option<DOMString> {
        None
    }

    pub fn SetMedia(&mut self, _media: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Hreflang(&self) -> Option<DOMString> {
        None
    }

    pub fn SetHreflang(&mut self, _href: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> Option<DOMString> {
        None
    }

    pub fn SetType(&mut self, _type: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Charset(&self) -> Option<DOMString> {
        None
    }

    pub fn SetCharset(&mut self, _charset: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Rev(&self) -> Option<DOMString> {
        None
    }

    pub fn SetRev(&mut self, _rev: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Target(&self) -> Option<DOMString> {
        None
    }

    pub fn SetTarget(&mut self, _target: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }
}
