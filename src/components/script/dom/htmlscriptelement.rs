/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLScriptElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLScriptElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLScriptElement {
    htmlelement: HTMLElement,
}

impl HTMLScriptElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLScriptElement {
        HTMLScriptElement {
            htmlelement: HTMLElement::new_inherited(HTMLScriptElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode {
        let element = HTMLScriptElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLScriptElementBinding::Wrap)
    }
}

impl HTMLScriptElement {
    pub fn Src(&self) -> DOMString {
        self.htmlelement.element.get_url_attribute("src")
    }

    pub fn SetSrc(&mut self, _src: DOMString) -> ErrorResult {
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
