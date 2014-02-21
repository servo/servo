/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLMetaElementBinding;
use dom::bindings::utils::ErrorResult;
use dom::document::AbstractDocument;
use dom::element::HTMLMetaElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLMetaElement {
    htmlelement: HTMLElement,
}

impl HTMLMetaElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLMetaElement {
        HTMLMetaElement {
            htmlelement: HTMLElement::new_inherited(HTMLMetaElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLMetaElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLMetaElementBinding::Wrap)
    }
}

impl HTMLMetaElement {
    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn HttpEquiv(&self) -> DOMString {
        ~""
    }

    pub fn SetHttpEquiv(&mut self, _http_equiv: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Content(&self) -> DOMString {
        ~""
    }

    pub fn SetContent(&mut self, _content: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Scheme(&self) -> DOMString {
        ~""
    }

    pub fn SetScheme(&mut self, _scheme: DOMString) -> ErrorResult {
        Ok(())
    }
}
