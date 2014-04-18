/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLMeterElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLMeterElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLMeterElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLMeterElement {
    pub htmlelement: HTMLElement
}

impl HTMLMeterElementDerived for EventTarget {
    fn is_htmlmeterelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLMeterElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLMeterElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLMeterElement {
        HTMLMeterElement {
            htmlelement: HTMLElement::new_inherited(HTMLMeterElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLMeterElement> {
        let element = HTMLMeterElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLMeterElementBinding::Wrap)
    }
}

pub trait HTMLMeterElementMethods {
    fn Value(&self) -> f64;
    fn SetValue(&mut self, _value: f64) -> ErrorResult;
    fn Min(&self) -> f64;
    fn SetMin(&mut self, _min: f64) -> ErrorResult;
    fn Max(&self) -> f64;
    fn SetMax(&mut self, _max: f64) -> ErrorResult;
    fn Low(&self) -> f64;
    fn SetLow(&mut self, _low: f64) -> ErrorResult;
    fn High(&self) -> f64;
    fn SetHigh(&mut self, _high: f64) -> ErrorResult;
    fn Optimum(&self) -> f64;
    fn SetOptimum(&mut self, _optimum: f64) -> ErrorResult;
}

impl<'a> HTMLMeterElementMethods for JSRef<'a, HTMLMeterElement> {
    fn Value(&self) -> f64 {
        0.0
    }

    fn SetValue(&mut self, _value: f64) -> ErrorResult {
        Ok(())
    }

    fn Min(&self) -> f64 {
        0.0
    }

    fn SetMin(&mut self, _min: f64) -> ErrorResult {
        Ok(())
    }

    fn Max(&self) -> f64 {
        0.0
    }

    fn SetMax(&mut self, _max: f64) -> ErrorResult {
        Ok(())
    }

    fn Low(&self) -> f64 {
        0.0
    }

    fn SetLow(&mut self, _low: f64) -> ErrorResult {
        Ok(())
    }

    fn High(&self) -> f64 {
        0.0
    }

    fn SetHigh(&mut self, _high: f64) -> ErrorResult {
        Ok(())
    }

    fn Optimum(&self) -> f64 {
        0.0
    }

    fn SetOptimum(&mut self, _optimum: f64) -> ErrorResult {
        Ok(())
    }
}

