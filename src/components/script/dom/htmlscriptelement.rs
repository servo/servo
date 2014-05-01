/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLScriptElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLScriptElementDerived;
use dom::bindings::codegen::InheritTypes::ElementCast;
use dom::bindings::js::JS;
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
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLScriptElement {
        HTMLScriptElement {
            htmlelement: HTMLElement::new_inherited(HTMLScriptElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLScriptElement> {
        let element = HTMLScriptElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLScriptElementBinding::Wrap)
    }
}

impl HTMLScriptElement {
    pub fn Src(&self, abstract_self: &JS<HTMLScriptElement>) -> DOMString {
        let element: JS<Element> = ElementCast::from(abstract_self);
        element.get_url_attribute("src")
    }

    pub fn SetSrc(&mut self, _abstract_self: &JS<HTMLScriptElement>, _src: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        ~""
    }

    pub fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Charset(&self) -> DOMString {
        ~""
    }

    pub fn SetCharset(&mut self, _charset: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Async(&self) -> bool {
        false
    }

    pub fn SetAsync(&self, _async: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Defer(&self) -> bool {
        false
    }

    pub fn SetDefer(&self, _defer: bool) -> ErrorResult {
        Ok(())
    }

    pub fn CrossOrigin(&self) -> DOMString {
        ~""
    }

    pub fn SetCrossOrigin(&mut self, _cross_origin: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Text(&self) -> DOMString {
        ~""
    }

    pub fn SetText(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Event(&self) -> DOMString {
        ~""
    }

    pub fn SetEvent(&mut self, _event: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn HtmlFor(&self) -> DOMString {
        ~""
    }

    pub fn SetHtmlFor(&mut self, _html_for: DOMString) -> ErrorResult {
        Ok(())
    }
}
