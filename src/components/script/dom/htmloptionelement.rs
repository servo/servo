/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLOptionElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLOptionElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLOptionElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::HTMLFormElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLOptionElement {
    pub htmlelement: HTMLElement
}

impl HTMLOptionElementDerived for EventTarget {
    fn is_htmloptionelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLOptionElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLOptionElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLOptionElement {
        HTMLOptionElement {
            htmlelement: HTMLElement::new_inherited(HTMLOptionElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLOptionElement> {
        let element = HTMLOptionElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLOptionElementBinding::Wrap)
    }
}

impl HTMLOptionElement {
    pub fn Disabled(&self) -> bool {
        false
    }

    pub fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    pub fn GetForm(&self) -> Option<JS<HTMLFormElement>> {
        None
    }

    pub fn Label(&self) -> DOMString {
        ~""
    }

    pub fn SetLabel(&mut self, _label: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn DefaultSelected(&self) -> bool {
        false
    }

    pub fn SetDefaultSelected(&mut self, _default_selected: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Selected(&self) -> bool {
        false
    }

    pub fn SetSelected(&mut self, _selected: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Value(&self) -> DOMString {
        ~""
    }

    pub fn SetValue(&mut self, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Text(&self) -> DOMString {
        ~""
    }

    pub fn SetText(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Index(&self) -> i32 {
        0
    }
}
