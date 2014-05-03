/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLLinkElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLLinkElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLLinkElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLLinkElement {
    pub htmlelement: HTMLElement,
}

impl HTMLLinkElementDerived for EventTarget {
    fn is_htmllinkelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLLinkElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLLinkElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLLinkElement {
        HTMLLinkElement {
            htmlelement: HTMLElement::new_inherited(HTMLLinkElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLLinkElement> {
        let element = HTMLLinkElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLLinkElementBinding::Wrap)
    }
}

pub trait HTMLLinkElementMethods {
    fn Disabled(&self) -> bool;
    fn SetDisabled(&mut self, _disable: bool);
    fn Href(&self) -> DOMString;
    fn SetHref(&mut self, _href: DOMString) -> ErrorResult;
    fn CrossOrigin(&self) -> DOMString;
    fn SetCrossOrigin(&mut self, _cross_origin: DOMString) -> ErrorResult;
    fn Rel(&self) -> DOMString;
    fn SetRel(&mut self, _rel: DOMString) -> ErrorResult;
    fn Media(&self) -> DOMString;
    fn SetMedia(&mut self, _media: DOMString) -> ErrorResult;
    fn Hreflang(&self) -> DOMString;
    fn SetHreflang(&mut self, _href: DOMString) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&mut self, _type: DOMString) -> ErrorResult;
    fn Charset(&self) -> DOMString;
    fn SetCharset(&mut self, _charset: DOMString) -> ErrorResult;
    fn Rev(&self) -> DOMString;
    fn SetRev(&mut self, _rev: DOMString) -> ErrorResult;
    fn Target(&self) -> DOMString;
    fn SetTarget(&mut self, _target: DOMString) -> ErrorResult;
}

impl<'a> HTMLLinkElementMethods for JSRef<'a, HTMLLinkElement> {
    fn Disabled(&self) -> bool {
        false
    }

    fn SetDisabled(&mut self, _disable: bool) {
    }

    fn Href(&self) -> DOMString {
        ~""
    }

    fn SetHref(&mut self, _href: DOMString) -> ErrorResult {
        Ok(())
    }

    fn CrossOrigin(&self) -> DOMString {
        ~""
    }

    fn SetCrossOrigin(&mut self, _cross_origin: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Rel(&self) -> DOMString {
        ~""
    }

    fn SetRel(&mut self, _rel: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Media(&self) -> DOMString {
        ~""
    }

    fn SetMedia(&mut self, _media: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Hreflang(&self) -> DOMString {
        ~""
    }

    fn SetHreflang(&mut self, _href: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        ~""
    }

    fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Charset(&self) -> DOMString {
        ~""
    }

    fn SetCharset(&mut self, _charset: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Rev(&self) -> DOMString {
        ~""
    }

    fn SetRev(&mut self, _rev: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Target(&self) -> DOMString {
        ~""
    }

    fn SetTarget(&mut self, _target: DOMString) -> ErrorResult {
        Ok(())
    }
}
