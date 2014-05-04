/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLParamElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLParamElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLParamElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLParamElement {
    pub htmlelement: HTMLElement
}

impl HTMLParamElementDerived for EventTarget {
    fn is_htmlparamelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLParamElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLParamElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLParamElement {
        HTMLParamElement {
            htmlelement: HTMLElement::new_inherited(HTMLParamElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLParamElement> {
        let element = HTMLParamElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLParamElementBinding::Wrap)
    }
}

pub trait HTMLParamElementMethods {
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, _name: DOMString) -> ErrorResult;
    fn Value(&self) -> DOMString;
    fn SetValue(&mut self, _value: DOMString) -> ErrorResult;
    fn Type(&self) -> DOMString;
    fn SetType(&mut self, _type: DOMString) -> ErrorResult;
    fn ValueType(&self) -> DOMString;
    fn SetValueType(&mut self, _value_type: DOMString) -> ErrorResult;
}

impl<'a> HTMLParamElementMethods for JSRef<'a, HTMLParamElement> {
    fn Name(&self) -> DOMString {
        "".to_owned()
    }

    fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Value(&self) -> DOMString {
        "".to_owned()
    }

    fn SetValue(&mut self, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Type(&self) -> DOMString {
        "".to_owned()
    }

    fn SetType(&mut self, _type: DOMString) -> ErrorResult {
        Ok(())
    }

    fn ValueType(&self) -> DOMString {
        "".to_owned()
    }

    fn SetValueType(&mut self, _value_type: DOMString) -> ErrorResult {
        Ok(())
    }
}
