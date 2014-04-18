/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLAppletElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLAppletElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLAppletElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLAppletElement {
    pub htmlelement: HTMLElement
}

impl HTMLAppletElementDerived for EventTarget {
    fn is_htmlappletelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLAppletElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLAppletElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLAppletElement {
        HTMLAppletElement {
            htmlelement: HTMLElement::new_inherited(HTMLAppletElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLAppletElement> {
        let element = HTMLAppletElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLAppletElementBinding::Wrap)
    }
}

pub trait HTMLAppletElementMethods {
    fn Align(&self) -> DOMString;
    fn SetAlign(&mut self, _align: DOMString) -> ErrorResult;
    fn Alt(&self) -> DOMString;
    fn SetAlt(&self, _alt: DOMString) -> ErrorResult;
    fn Archive(&self) -> DOMString;
    fn SetArchive(&self, _archive: DOMString) -> ErrorResult;
    fn Code(&self) -> DOMString;
    fn SetCode(&self, _code: DOMString) -> ErrorResult;
    fn CodeBase(&self) -> DOMString;
    fn SetCodeBase(&self, _code_base: DOMString) -> ErrorResult;
    fn Height(&self) -> DOMString;
    fn SetHeight(&self, _height: DOMString) -> ErrorResult;
    fn Hspace(&self) -> u32;
    fn SetHspace(&mut self, _hspace: u32) -> ErrorResult;
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, _name: DOMString) -> ErrorResult;
    fn Object(&self) -> DOMString;
    fn SetObject(&mut self, _object: DOMString) -> ErrorResult;
    fn Vspace(&self) -> u32;
    fn SetVspace(&mut self, _vspace: u32) -> ErrorResult;
    fn Width(&self) -> DOMString;
    fn SetWidth(&mut self, _width: DOMString) -> ErrorResult;
}

impl<'a> HTMLAppletElementMethods for JSRef<'a, HTMLAppletElement> {
    fn Align(&self) -> DOMString {
        ~""
    }

    fn SetAlign(&mut self, _align: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Alt(&self) -> DOMString {
        ~""
    }

    fn SetAlt(&self, _alt: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Archive(&self) -> DOMString {
        ~""
    }

    fn SetArchive(&self, _archive: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Code(&self) -> DOMString {
        ~""
    }

    fn SetCode(&self, _code: DOMString) -> ErrorResult {
        Ok(())
    }

    fn CodeBase(&self) -> DOMString {
        ~""
    }

    fn SetCodeBase(&self, _code_base: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Height(&self) -> DOMString {
        ~""
    }

    fn SetHeight(&self, _height: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Hspace(&self) -> u32 {
        0
    }

    fn SetHspace(&mut self, _hspace: u32) -> ErrorResult {
        Ok(())
    }

    fn Name(&self) -> DOMString {
        ~""
    }

    fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Object(&self) -> DOMString {
        ~""
    }

    fn SetObject(&mut self, _object: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Vspace(&self) -> u32 {
        0
    }

    fn SetVspace(&mut self, _vspace: u32) -> ErrorResult {
        Ok(())
    }

    fn Width(&self) -> DOMString {
        ~""
    }

    fn SetWidth(&mut self, _width: DOMString) -> ErrorResult {
        Ok(())
    }
}
