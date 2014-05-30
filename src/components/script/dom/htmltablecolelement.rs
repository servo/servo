/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLTableColElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableColElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLTableColElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTableColElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTableColElementDerived for EventTarget {
    fn is_htmltablecolelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTableColElementTypeId))
    }
}

impl HTMLTableColElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLTableColElement {
        HTMLTableColElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableColElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLTableColElement> {
        let element = HTMLTableColElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLTableColElementBinding::Wrap)
    }
}

pub trait HTMLTableColElementMethods {
    fn Span(&self) -> u32;
    fn SetSpan(&self, _span: u32) -> ErrorResult;
    fn Align(&self) -> DOMString;
    fn SetAlign(&self, _align: DOMString) -> ErrorResult;
    fn Ch(&self) -> DOMString;
    fn SetCh(&self, _ch: DOMString) -> ErrorResult;
    fn ChOff(&self) -> DOMString;
    fn SetChOff(&self, _ch_off: DOMString) -> ErrorResult;
    fn VAlign(&self) -> DOMString;
    fn SetVAlign(&self, _v_align: DOMString) -> ErrorResult;
    fn Width(&self) -> DOMString;
    fn SetWidth(&self, _width: DOMString) -> ErrorResult;
}

impl<'a> HTMLTableColElementMethods for JSRef<'a, HTMLTableColElement> {
    fn Span(&self) -> u32 {
        0
    }

    fn SetSpan(&self, _span: u32) -> ErrorResult {
        Ok(())
    }

    fn Align(&self) -> DOMString {
        "".to_owned()
    }

    fn SetAlign(&self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Ch(&self) -> DOMString {
        "".to_owned()
    }

    fn SetCh(&self, _ch: DOMString) -> ErrorResult {
        Ok(())
    }

    fn ChOff(&self) -> DOMString {
        "".to_owned()
    }

    fn SetChOff(&self, _ch_off: DOMString) -> ErrorResult {
        Ok(())
    }

    fn VAlign(&self) -> DOMString {
        "".to_owned()
    }

    fn SetVAlign(&self, _v_align: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Width(&self) -> DOMString {
        "".to_owned()
    }

    fn SetWidth(&self, _width: DOMString) -> ErrorResult {
        Ok(())
    }
}
