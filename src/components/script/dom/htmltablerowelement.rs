/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLTableRowElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTableRowElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLTableRowElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTableRowElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTableRowElementDerived for EventTarget {
    fn is_htmltablerowelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLTableRowElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLTableRowElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLTableRowElement {
        HTMLTableRowElement {
            htmlelement: HTMLElement::new_inherited(HTMLTableRowElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLTableRowElement> {
        let element = HTMLTableRowElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLTableRowElementBinding::Wrap)
    }
}

pub trait HTMLTableRowElementMethods {
    fn RowIndex(&self) -> i32;
    fn GetRowIndex(&self) -> i32;
    fn SectionRowIndex(&self) -> i32;
    fn GetSectionRowIndex(&self) -> i32;
    fn DeleteCell(&mut self, _index: i32) -> ErrorResult;
    fn Align(&self) -> DOMString;
    fn SetAlign(&self, _align: DOMString) -> ErrorResult;
    fn Ch(&self) -> DOMString;
    fn SetCh(&self, _ch: DOMString) -> ErrorResult;
    fn ChOff(&self) -> DOMString;
    fn SetChOff(&self, _ch_off: DOMString) -> ErrorResult;
    fn VAlign(&self) -> DOMString;
    fn SetVAlign(&self, _v_align: DOMString) -> ErrorResult;
    fn BgColor(&self) -> DOMString;
    fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult;
}

impl<'a> HTMLTableRowElementMethods for JSRef<'a, HTMLTableRowElement> {
    fn RowIndex(&self) -> i32 {
        0
    }

    fn GetRowIndex(&self) -> i32 {
        0
    }

    fn SectionRowIndex(&self) -> i32 {
        0
    }

    fn GetSectionRowIndex(&self) -> i32 {
        0
    }

    fn DeleteCell(&mut self, _index: i32) -> ErrorResult {
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

    fn BgColor(&self) -> DOMString {
        "".to_owned()
    }

    fn SetBgColor(&self, _bg_color: DOMString) -> ErrorResult {
        Ok(())
    }
}
