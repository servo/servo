/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLAnchorElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLAnchorElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLAnchorElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLAnchorElement {
    pub htmlelement: HTMLElement
}

impl HTMLAnchorElementDerived for EventTarget {
    fn is_htmlanchorelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLAnchorElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLAnchorElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLAnchorElement {
        HTMLAnchorElement {
            htmlelement: HTMLElement::new_inherited(HTMLAnchorElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLAnchorElement> {
        let element = HTMLAnchorElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLAnchorElementBinding::Wrap)
    }
}

pub trait HTMLAnchorElementMethods {
    fn Href(&self) -> DOMString;
    fn SetHref(&mut self, _href: DOMString) -> ErrorResult;
    fn Target(&self) -> DOMString;
    fn SetTarget(&self, _target: DOMString) -> ErrorResult;
    fn Download(&self) -> DOMString;
    fn SetDownload(&self, _download: DOMString) -> ErrorResult;
    fn Ping(&self) -> DOMString;
    fn SetPing(&self, _ping: DOMString) -> ErrorResult;
    fn Rel(&self) -> DOMString;
    fn SetRel(&self, _rel: DOMString) -> ErrorResult;
    fn Hreflang(&self) -> DOMString;
    fn SetHreflang(&self, _href_lang: DOMString) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&mut self, _type: DOMString) -> ErrorResult;
    fn Text(&self) -> DOMString;
    fn SetText(&mut self, _text: DOMString) -> ErrorResult;
    fn Coords(&self) -> DOMString;
    fn SetCoords(&mut self, _coords: DOMString) -> ErrorResult;
    fn Charset(&self) -> DOMString;
    fn SetCharset(&mut self, _charset: DOMString) -> ErrorResult;
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, _name: DOMString) -> ErrorResult;
    fn Rev(&self) -> DOMString;
    fn SetRev(&mut self, _rev: DOMString) -> ErrorResult;
    fn Shape(&self) -> DOMString;
    fn SetShape(&mut self, _shape: DOMString) -> ErrorResult;
}

impl<'a> HTMLAnchorElementMethods for JSRef<'a, HTMLAnchorElement> {
    fn Href(&self) -> DOMString {
        ~""
    }

    fn SetHref(&mut self, _href: DOMString) -> ErrorResult {
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

    fn Rel(&self) -> DOMString {
        ~""
    }

    fn SetRel(&self, _rel: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Hreflang(&self) -> DOMString {
        ~""
    }

    fn SetHreflang(&self, _href_lang: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        ~""
    }

    fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Text(&self) -> DOMString {
        ~""
    }

    fn SetText(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Coords(&self) -> DOMString {
        ~""
    }

    fn SetCoords(&mut self, _coords: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Charset(&self) -> DOMString {
        ~""
    }

    fn SetCharset(&mut self, _charset: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Name(&self) -> DOMString {
        ~""
    }

    fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Rev(&self) -> DOMString {
        ~""
    }

    fn SetRev(&mut self, _rev: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Shape(&self) -> DOMString {
        ~""
    }

    fn SetShape(&mut self, _shape: DOMString) -> ErrorResult {
        Ok(())
    }
}
