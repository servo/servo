/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::HTMLFrameElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLFrameElementDerived;
use dom::bindings::js::{JSRef, Temporary};
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
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLFrameElementTypeId))
    }
}

impl HTMLFrameElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLFrameElement {
        HTMLFrameElement {
            htmlelement: HTMLElement::new_inherited(HTMLFrameElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLFrameElement> {
        let element = HTMLFrameElement::new_inherited(localName, document);
        Node::reflect_node(~element, document, HTMLFrameElementBinding::Wrap)
    }
}

pub trait HTMLFrameElementMethods {
    fn Name(&self) -> DOMString;
    fn SetName(&mut self, _name: DOMString) -> ErrorResult;
    fn Scrolling(&self) -> DOMString;
    fn SetScrolling(&mut self, _scrolling: DOMString) -> ErrorResult;
    fn Src(&self) -> DOMString;
    fn SetSrc(&mut self, _src: DOMString) -> ErrorResult;
    fn FrameBorder(&self) -> DOMString;
    fn SetFrameBorder(&mut self, _frameborder: DOMString) -> ErrorResult;
    fn LongDesc(&self) -> DOMString;
    fn SetLongDesc(&mut self, _longdesc: DOMString) -> ErrorResult;
    fn NoResize(&self) -> bool;
    fn SetNoResize(&mut self, _no_resize: bool) -> ErrorResult;
    fn GetContentDocument(&self) -> Option<Temporary<Document>>;
    fn GetContentWindow(&self) -> Option<Temporary<Window>>;
    fn MarginHeight(&self) -> DOMString;
    fn SetMarginHeight(&mut self, _height: DOMString) -> ErrorResult;
    fn MarginWidth(&self) -> DOMString;
    fn SetMarginWidth(&mut self, _height: DOMString) -> ErrorResult;
}

impl<'a> HTMLFrameElementMethods for JSRef<'a, HTMLFrameElement> {
    fn Name(&self) -> DOMString {
        "".to_owned()
    }

    fn SetName(&mut self, _name: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Scrolling(&self) -> DOMString {
        "".to_owned()
    }

    fn SetScrolling(&mut self, _scrolling: DOMString) -> ErrorResult {
        Ok(())
    }

    fn Src(&self) -> DOMString {
        "".to_owned()
    }

    fn SetSrc(&mut self, _src: DOMString) -> ErrorResult {
        Ok(())
    }

    fn FrameBorder(&self) -> DOMString {
        "".to_owned()
    }

    fn SetFrameBorder(&mut self, _frameborder: DOMString) -> ErrorResult {
        Ok(())
    }

    fn LongDesc(&self) -> DOMString {
        "".to_owned()
    }

    fn SetLongDesc(&mut self, _longdesc: DOMString) -> ErrorResult {
        Ok(())
    }

    fn NoResize(&self) -> bool {
        false
    }

    fn SetNoResize(&mut self, _no_resize: bool) -> ErrorResult {
        Ok(())
    }

    fn GetContentDocument(&self) -> Option<Temporary<Document>> {
        None
    }

    fn GetContentWindow(&self) -> Option<Temporary<Window>> {
        None
    }

    fn MarginHeight(&self) -> DOMString {
        "".to_owned()
    }

    fn SetMarginHeight(&mut self, _height: DOMString) -> ErrorResult {
        Ok(())
    }

    fn MarginWidth(&self) -> DOMString {
        "".to_owned()
    }

    fn SetMarginWidth(&mut self, _height: DOMString) -> ErrorResult {
        Ok(())
    }
}
