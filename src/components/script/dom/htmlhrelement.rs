/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLHRElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLHRElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node};

pub struct HTMLHRElement {
    htmlelement: HTMLElement,
}

impl HTMLHRElement {
    pub fn new_inherited(localName: DOMString, document: AbstractDocument) -> HTMLHRElement {
        HTMLHRElement {
            htmlelement: HTMLElement::new_inherited(HTMLHRElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: AbstractDocument) -> AbstractNode {
        let element = HTMLHRElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLHRElementBinding::Wrap)
    }
}

impl HTMLHRElement {
    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Color(&self) -> DOMString {
        ~""
    }

    pub fn SetColor(&mut self, _color: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn NoShade(&self) -> bool {
        false
    }

    pub fn SetNoShade(&self, _no_shade: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Size(&self) -> DOMString {
        ~""
    }

    pub fn SetSize(&mut self, _size: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self) -> DOMString {
        ~""
    }

    pub fn SetWidth(&mut self, _width: DOMString) -> ErrorResult {
        Ok(())
    }
}
