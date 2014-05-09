/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLTrackElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLTrackElementDerived;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLTrackElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLTrackElement {
    pub htmlelement: HTMLElement,
}

impl HTMLTrackElementDerived for EventTarget {
    fn is_htmltrackelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLTrackElementTypeId))
    }
}

impl HTMLTrackElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLTrackElement {
        HTMLTrackElement {
            htmlelement: HTMLElement::new_inherited(HTMLTrackElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLTrackElement> {
        let element = HTMLTrackElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLTrackElementBinding::Wrap)
    }
}

pub trait HTMLTrackElementMethods {
    fn Kind(&self) -> DOMString;
    fn SetKind(&mut self, _kind: DOMString) -> ErrorResult;
    fn Src(&self) -> DOMString;
    fn SetSrc(&mut self, _src: DOMString) -> ErrorResult;
    fn Srclang(&self) -> DOMString;
    fn SetSrclang(&mut self, _srclang: DOMString) -> ErrorResult;
    fn Label(&self) -> DOMString;
    fn SetLabel(&mut self, _label: DOMString) -> ErrorResult;
    fn Default(&self) -> bool;
    fn SetDefault(&mut self, _default: bool) -> ErrorResult;
    fn ReadyState(&self) -> u16;
}

impl<'a> HTMLTrackElementMethods for JSRef<'a, HTMLTrackElement> {
    fn Kind(&self) -> DOMString {
        "".to_owned()
    }

    fn SetKind(&mut self, _kind: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Src(&self) -> DOMString {
        "".to_owned()
    }

    fn SetSrc(&mut self, _src: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Srclang(&self) -> DOMString {
        "".to_owned()
    }

    fn SetSrclang(&mut self, _srclang: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Label(&self) -> DOMString {
        "".to_owned()
    }

    fn SetLabel(&mut self, _label: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Default(&self) -> bool {
        false
    }

    fn SetDefault(&mut self, _default: bool) -> ErrorResult {
        Ok(())
    }

    fn ReadyState(&self) -> u16 {
        0
    }
}
