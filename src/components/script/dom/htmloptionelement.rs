/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLOptionElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLOptionElementDerived;
use dom::bindings::js::{JSRef, Temporary};
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
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLOptionElement {
        HTMLOptionElement {
            htmlelement: HTMLElement::new_inherited(HTMLOptionElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLOptionElement> {
        let element = HTMLOptionElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLOptionElementBinding::Wrap)
    }
}

pub trait HTMLOptionElementMethods {
    fn Disabled(&self) -> bool;
    fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult;
    fn GetForm(&self) -> Option<Temporary<HTMLFormElement>>;
    fn Label(&self) -> DOMString;
    fn SetLabel(&mut self, _label: DOMString) -> ErrorResult;
    fn DefaultSelected(&self) -> bool;
    fn SetDefaultSelected(&mut self, _default_selected: bool) -> ErrorResult;
    fn Selected(&self) -> bool;
    fn SetSelected(&mut self, _selected: bool) -> ErrorResult;
    fn Value(&self) -> DOMString;
    fn SetValue(&mut self, _value: DOMString) -> ErrorResult;
    fn Text(&self) -> DOMString;
    fn SetText(&mut self, _text: DOMString) -> ErrorResult;
    fn Index(&self) -> i32;
}

impl<'a> HTMLOptionElementMethods for JSRef<'a, HTMLOptionElement> {
    fn Disabled(&self) -> bool {
        false
    }

    fn SetDisabled(&mut self, _disabled: bool) -> ErrorResult {
        Ok(())
    }

    fn GetForm(&self) -> Option<Temporary<HTMLFormElement>> {
        None
    }

    fn Label(&self) -> DOMString {
        "".to_owned()
    }

    fn SetLabel(&mut self, _label: DOMString) -> ErrorResult {
        Ok(())
    }

    fn DefaultSelected(&self) -> bool {
        false
    }

    fn SetDefaultSelected(&mut self, _default_selected: bool) -> ErrorResult {
        Ok(())
    }

    fn Selected(&self) -> bool {
        false
    }

    fn SetSelected(&mut self, _selected: bool) -> ErrorResult {
        Ok(())
    }

    fn Value(&self) -> DOMString {
        "".to_owned()
    }

    fn SetValue(&mut self, _value: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Text(&self) -> DOMString {
        "".to_owned()
    }

    fn SetText(&mut self, _text: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Index(&self) -> i32 {
        0
    }
}
