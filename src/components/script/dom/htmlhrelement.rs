/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLHRElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLHRElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLHRElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLHRElement {
    pub htmlelement: HTMLElement,
}

impl HTMLHRElementDerived for EventTarget {
    fn is_htmlhrelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLHRElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLHRElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLHRElement {
        HTMLHRElement {
            htmlelement: HTMLElement::new_inherited(HTMLHRElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLHRElement> {
        let element = HTMLHRElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLHRElementBinding::Wrap)
    }
}

pub trait HTMLHRElementMethods {
    fn Align(&self) -> DOMString;
    fn SetAlign(&mut self, _align: DOMString) -> ErrorResult;
    fn Color(&self) -> DOMString;
    fn SetColor(&mut self, _color: DOMString) -> ErrorResult;
    fn NoShade(&self) -> bool;
    fn SetNoShade(&self, _no_shade: bool) -> ErrorResult;
    fn Size(&self) -> DOMString;
    fn SetSize(&mut self, _size: DOMString) -> ErrorResult;
    fn Width(&self) -> DOMString;
    fn SetWidth(&mut self, _width: DOMString) -> ErrorResult;
}

impl<'a> HTMLHRElementMethods for JSRef<'a, HTMLHRElement> {
    fn Align(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Color(&self) -> DOMString {
        "".to_owned()
    }

    fn SetColor(&mut self, _color: DOMString) -> ErrorResult {
        Ok(())
    }

    fn NoShade(&self) -> bool {
        false
    }

    fn SetNoShade(&self, _no_shade: bool) -> ErrorResult {
        Ok(())
    }

    fn Size(&self) -> DOMString {
        "".to_owned()
    }

    fn SetSize(&mut self, _size: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Width(&self) -> DOMString {
        "".to_owned()
    }

    fn SetWidth(&mut self, _width: DOMString) -> ErrorResult {
        Ok(())
    }
}
