/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLLegendElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLLegendElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLLegendElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLLegendElement {
    pub htmlelement: HTMLElement,
}

impl HTMLLegendElementDerived for EventTarget {
    fn is_htmllegendelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLLegendElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLLegendElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLLegendElement {
        HTMLLegendElement {
            htmlelement: HTMLElement::new_inherited(HTMLLegendElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLLegendElement> {
        let element = HTMLLegendElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLLegendElementBinding::Wrap)
    }
}

impl HTMLLegendElement {
    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }
}
