/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLOptGroupElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLOptGroupElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLOptGroupElement {
    htmlelement: HTMLElement
}

impl HTMLOptGroupElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLOptGroupElement {
        HTMLOptGroupElement {
            htmlelement: HTMLElement::new_inherited(HTMLOptGroupElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLOptGroupElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLOptGroupElementBinding::Wrap)
    }
}

impl HTMLOptGroupElement {
    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Label(&self) -> DOMString {
        ~""
    }

    pub fn SetLabel(&mut self, _label: &DOMString) -> ErrorResult {
        Ok(())
    }
}
