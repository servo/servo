/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLSourceElementBinding;
use dom::bindings::utils::ErrorResult;
use dom::document::AbstractDocument;
use dom::element::HTMLSourceElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};
use servo_util::str::DOMString;

pub struct HTMLSourceElement {
    htmlelement: HTMLElement
}

impl HTMLSourceElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLSourceElement {
        HTMLSourceElement {
            htmlelement: HTMLElement::new_inherited(HTMLSourceElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLSourceElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLSourceElementBinding::Wrap)
    }
}

impl HTMLSourceElement {
    pub fn Src(&self) -> DOMString {
        ~""
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

    pub fn Media(&self) -> DOMString {
        ~""
    }
    
    pub fn SetMedia(&mut self, _media: DOMString) -> ErrorResult {
        Ok(())
    }
}
