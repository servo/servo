/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLProgressElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLProgressElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::{ErrorResult, Fallible};
use dom::document::Document;
use dom::element::HTMLProgressElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLProgressElement {
    pub htmlelement: HTMLElement,
}

impl HTMLProgressElementDerived for EventTarget {
    fn is_htmlprogresselement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLProgressElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLProgressElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLProgressElement {
        HTMLProgressElement {
            htmlelement: HTMLElement::new_inherited(HTMLProgressElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLProgressElement> {
        let element = HTMLProgressElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLProgressElementBinding::Wrap)
    }
}

pub trait HTMLProgressElementMethods {
    fn Value(&self) -> f64;
    fn SetValue(&mut self, _value: f64) -> ErrorResult;
    fn Max(&self) -> f64;
    fn SetMax(&mut self, _max: f64) -> ErrorResult;
    fn Position(&self) -> f64;
    fn GetPositiom(&self) -> Fallible<f64>;
}

impl<'a> HTMLProgressElementMethods for JSRef<'a, HTMLProgressElement> {
    fn Value(&self) -> f64 {
        0f64
    }

    fn SetValue(&mut self, _value: f64) -> ErrorResult {
        Ok(())
    }

    fn Max(&self) -> f64 {
        0f64
    }

    fn SetMax(&mut self, _max: f64) -> ErrorResult {
        Ok(())
    }

    fn Position(&self) -> f64 {
        0f64
    }

    fn GetPositiom(&self) -> Fallible<f64> {
        Ok(0f64)
    }
}
