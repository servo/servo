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
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLLinkElementTypeId))
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
        Node::reflect_node(box element, document, HTMLLinkElementBinding::Wrap)
    }
}

pub trait HTMLLinkElementMethods {
    fn Disabled(&self) -> bool;
    fn SetDisabled(&self, _disable: bool);
    fn Href(&self) -> DOMString;
    fn SetHref(&self, _href: DOMString) -> ErrorResult;
    fn CrossOrigin(&self) -> DOMString;
    fn SetCrossOrigin(&self, _cross_origin: DOMString) -> ErrorResult;
    fn Rel(&self) -> DOMString;
    fn SetRel(&self, _rel: DOMString) -> ErrorResult;
    fn Media(&self) -> DOMString;
    fn SetMedia(&self, _media: DOMString) -> ErrorResult;
    fn Hreflang(&self) -> DOMString;
    fn SetHreflang(&self, _href: DOMString) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&self, _type: DOMString) -> ErrorResult;
    fn Charset(&self) -> DOMString;
    fn SetCharset(&self, _charset: DOMString) -> ErrorResult;
    fn Rev(&self) -> DOMString;
    fn SetRev(&self, _rev: DOMString) -> ErrorResult;
    fn Target(&self) -> DOMString;
    fn SetTarget(&self, _target: DOMString) -> ErrorResult;
}

impl<'a> HTMLLinkElementMethods for JSRef<'a, HTMLLinkElement> {
    fn Disabled(&self) -> bool {
        false
    }

    fn SetDisabled(&self, _disable: bool) {
    }

    fn Href(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHref(&self, _href: DOMString) -> ErrorResult {
        Ok(())
    }

    fn CrossOrigin(&self) -> DOMString {
        "".to_owned()
    }

    fn SetCrossOrigin(&self, _cross_origin: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Rel(&self) -> DOMString {
        "".to_owned()
    }

    fn SetRel(&self, _rel: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Media(&self) -> DOMString {
        "".to_owned()
    }

    fn SetMedia(&self, _media: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Hreflang(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHreflang(&self, _href: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        "".to_owned()
    }

    fn SetType(&self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Charset(&self) -> DOMString {
        "".to_owned()
    }

    fn SetCharset(&self, _charset: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Rev(&self) -> DOMString {
        "".to_owned()
    }

    fn SetRev(&self, _rev: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Target(&self) -> DOMString {
        "".to_owned()
    }

    fn SetTarget(&self, _target: DOMString) -> ErrorResult {
        Ok(())
    }
}
