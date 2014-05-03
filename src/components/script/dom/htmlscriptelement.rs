/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLScriptElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLScriptElementDerived;
use dom::bindings::codegen::InheritTypes::ElementCast;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::{HTMLScriptElementTypeId, Element, AttributeHandlers};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLScriptElement {
    pub htmlelement: HTMLElement,
}

impl HTMLScriptElementDerived for EventTarget {
    fn is_htmlscriptelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLScriptElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLScriptElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLScriptElement {
        HTMLScriptElement {
            htmlelement: HTMLElement::new_inherited(HTMLScriptElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLScriptElement> {
        let element = HTMLScriptElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLScriptElementBinding::Wrap)
    }
}

pub trait HTMLScriptElementMethods {
    fn Src(&self) -> DOMString;
    fn SetSrc(&mut self, _src: DOMString) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&mut self, _type: DOMString) -> ErrorResult;
    fn Charset(&self) -> DOMString;
    fn SetCharset(&mut self, _charset: DOMString) -> ErrorResult;
    fn Async(&self) -> bool;
    fn SetAsync(&self, _async: bool) -> ErrorResult;
    fn Defer(&self) -> bool;
    fn SetDefer(&self, _defer: bool) -> ErrorResult;
    fn CrossOrigin(&self) -> DOMString;
    fn SetCrossOrigin(&mut self, _cross_origin: DOMString) -> ErrorResult;
    fn Text(&self) -> DOMString;
    fn SetText(&mut self, _text: DOMString) -> ErrorResult;
    fn Event(&self) -> DOMString;
    fn SetEvent(&mut self, _event: DOMString) -> ErrorResult;
    fn HtmlFor(&self) -> DOMString;
    fn SetHtmlFor(&mut self, _html_for: DOMString) -> ErrorResult;
}

impl<'a> HTMLScriptElementMethods for JSRef<'a, HTMLScriptElement> {
    fn Src(&self) -> DOMString {
        let element: &JSRef<Element> = ElementCast::from_ref(self);
        element.get_url_attribute("src")
    }

    fn SetSrc(&mut self, _src: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        "".to_owned()
    }

    fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Charset(&self) -> DOMString {
        "".to_owned()
    }

    fn SetCharset(&mut self, _charset: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Async(&self) -> bool {
        false
    }

    fn SetAsync(&self, _async: bool) -> ErrorResult {
        Ok(())
    }

    fn Defer(&self) -> bool {
        false
    }

    fn SetDefer(&self, _defer: bool) -> ErrorResult {
        Ok(())
    }

    fn CrossOrigin(&self) -> DOMString {
        "".to_owned()
    }

    fn SetCrossOrigin(&mut self, _cross_origin: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Text(&self) -> DOMString {
        "".to_owned()
    }

    fn SetText(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Event(&self) -> DOMString {
        "".to_owned()
    }

    fn SetEvent(&mut self, _event: DOMString) -> ErrorResult {
        Ok(())
    }

    fn HtmlFor(&self) -> DOMString {
        "".to_owned()
    }

    fn SetHtmlFor(&mut self, _html_for: DOMString) -> ErrorResult {
        Ok(())
    }
}
