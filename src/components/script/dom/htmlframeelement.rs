/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLFrameElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLFrameElementDerived;
use dom::bindings::js::JS;
use dom::bindings::error::ErrorResult;
use dom::document::Document;
use dom::element::HTMLFrameElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use dom::window::Window;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLFrameElement {
    pub htmlelement: HTMLElement
}

impl HTMLFrameElementDerived for EventTarget {
    fn is_htmlframeelement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLFrameElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLFrameElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLFrameElement {
        HTMLFrameElement {
            htmlelement: HTMLElement::new_inherited(HTMLFrameElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLFrameElement> {
        let element = HTMLFrameElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLFrameElementBinding::Wrap)
    }
}

impl HTMLFrameElement {
    pub fn Name(&self) -> DOMString {
        ~""
    }

    pub fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Scrolling(&self) -> DOMString {
        ~""
    }

    pub fn SetScrolling(&mut self, _scrolling: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn Src(&self) -> DOMString {
        ~""
    }

    pub fn SetSrc(&mut self, _src: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn FrameBorder(&self) -> DOMString {
        ~""
    }

    pub fn SetFrameBorder(&mut self, _frameborder: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn LongDesc(&self) -> DOMString {
        ~""
    }

    pub fn SetLongDesc(&mut self, _longdesc: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn NoResize(&self) -> bool {
        false
    }

    pub fn SetNoResize(&mut self, _no_resize: bool) -> ErrorResult {
        Ok(())
    }

    pub fn GetContentDocument(&self) -> Option<JS<Document>> {
        None
    }

    pub fn GetContentWindow(&self) -> Option<JS<Window>> {
        None
    }

    pub fn MarginHeight(&self) -> DOMString {
        ~""
    }

    pub fn SetMarginHeight(&mut self, _height: DOMString) -> ErrorResult {
        Ok(())
    }

    pub fn MarginWidth(&self) -> DOMString {
        ~""
    }

    pub fn SetMarginWidth(&mut self, _height: DOMString) -> ErrorResult {
        Ok(())
    }
}
