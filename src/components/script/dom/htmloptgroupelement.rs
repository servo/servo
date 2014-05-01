/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLOptGroupElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLOptGroupElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLOptGroupElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLOptGroupElement {
    pub htmlelement: HTMLElement
}

impl HTMLOptGroupElementDerived for EventTarget {
    fn is_htmloptgroupelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLOptGroupElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLOptGroupElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLOptGroupElement {
        HTMLOptGroupElement {
            htmlelement: HTMLElement::new_inherited(HTMLOptGroupElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLOptGroupElement> {
        let element = HTMLOptGroupElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLOptGroupElementBinding::Wrap)
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

    pub fn SetLabel(&mut self, _label: DOMString) -> ErrorResult {
        Ok(())
    }
}
