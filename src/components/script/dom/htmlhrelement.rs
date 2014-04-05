/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLHRElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLHRElementDerived;
use dom::bindings::js::JS;
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
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLHRElement {
        HTMLHRElement {
            htmlelement: HTMLElement::new_inherited(HTMLHRElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLHRElement> {
        let element = HTMLHRElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLHRElementBinding::Wrap)
    }
}

impl HTMLHRElement {
    pub fn Align(&self) -> DOMString {
        ~""
    }

    pub fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Color(&self) -> DOMString {
        ~""
    }

    pub fn SetColor(&mut self, _color: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn NoShade(&self) -> bool {
        false
    }

    pub fn SetNoShade(&self, _no_shade: bool) -> ErrorResult {
        Ok(())
    }

    pub fn Size(&self) -> DOMString {
        ~""
    }

    pub fn SetSize(&mut self, _size: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Width(&self) -> DOMString {
        ~""
    }

    pub fn SetWidth(&mut self, _width: DOMString) -> ErrorResult {
        Ok(())
    }
}
