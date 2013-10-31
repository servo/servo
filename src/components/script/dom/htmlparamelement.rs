/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLParamElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLParamElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};

pub struct HTMLParamElement {
    htmlelement: HTMLElement
}

impl HTMLParamElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLParamElement {
        HTMLParamElement {
            htmlelement: HTMLElement::new(HTMLParamElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLParamElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLParamElementBinding::Wrap)
    }
}

impl HTMLParamElement {
    pub fn Name(&self) -> DOMString {
        None
    }

    pub fn SetName(&mut self, _name: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Value(&self) -> DOMString {
        None
    }

    pub fn SetValue(&mut self, _value: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Type(&self) -> DOMString {
        None
    }

    pub fn SetType(&mut self, _type: &DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn ValueType(&self) -> DOMString {
        None
    }

    pub fn SetValueType(&mut self, _value_type: &DOMString) -> ErrorResult {
        Ok(())
    }
}
