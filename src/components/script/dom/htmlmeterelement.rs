/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLMeterElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLMeterElementDerived;
use dom::bindings::js::JS;
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
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLMeterElement {
        HTMLMeterElement {
            htmlelement: HTMLElement::new_inherited(HTMLMeterElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLMeterElement> {
        let element = HTMLMeterElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLMeterElementBinding::Wrap)
    }
}

impl HTMLMeterElement {
    pub fn Value(&self) -> f64 {
        0.0
    }

    pub fn SetValue(&mut self, _value: f64) -> ErrorResult {
        Ok(())
    }

    pub fn Min(&self) -> f64 {
        0.0
    }

    pub fn SetMin(&mut self, _min: f64) -> ErrorResult {
        Ok(())
    }

    pub fn Max(&self) -> f64 {
        0.0
    }

    pub fn SetMax(&mut self, _max: f64) -> ErrorResult {
        Ok(())
    }

    pub fn Low(&self) -> f64 {
        0.0
    }

    pub fn SetLow(&mut self, _low: f64) -> ErrorResult {
        Ok(())
    }

    pub fn High(&self) -> f64 {
        0.0
    }

    pub fn SetHigh(&mut self, _high: f64) -> ErrorResult {
        Ok(())
    }

    pub fn Optimum(&self) -> f64 {
        0.0
    }

    pub fn SetOptimum(&mut self, _optimum: f64) -> ErrorResult {
        Ok(())
    }
}
