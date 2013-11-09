/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLFrameElementBinding;
use dom::bindings::utils::{DOMString, ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLFrameElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::node::{AbstractNode, Node, ScriptView};
use dom::windowproxy::WindowProxy;

pub struct HTMLFrameElement {
    htmlelement: HTMLElement
}

impl HTMLFrameElement {
    pub fn new_inherited(localName: ~str, document: AbstractDocument) -> HTMLFrameElement {
        HTMLFrameElement {
            htmlelement: HTMLElement::new_inherited(HTMLFrameElementTypeId, localName, document)
        }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLFrameElement::new_inherited(localName, document);
        Node::reflect_node(@mut element, document, HTMLFrameElementBinding::Wrap)
    }
}

impl HTMLFrameElement {
    pub fn Name(&self) -> Option<DOMString> {
        None
    }

    pub fn SetName(&mut self, _name: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Scrolling(&self) -> Option<DOMString> {
        None
    }

    pub fn SetScrolling(&mut self, _scrolling: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn Src(&self) -> Option<DOMString> {
        None
    }

    pub fn SetSrc(&mut self, _src: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn FrameBorder(&self) -> Option<DOMString> {
        None
    }

    pub fn SetFrameBorder(&mut self, _frameborder: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn LongDesc(&self) -> Option<DOMString> {
        None
    }

    pub fn SetLongDesc(&mut self, _longdesc: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn NoResize(&self) -> bool {
        false
    }

    pub fn SetNoResize(&mut self, _no_resize: bool) -> ErrorResult {
        Ok(())
    }

    pub fn GetContentDocument(&self) -> Option<AbstractDocument> {
        None
    }

    pub fn GetContentWindow(&self) -> Option<@mut WindowProxy> {
        None
    }

    pub fn MarginHeight(&self) -> Option<DOMString> {
        None
    }

    pub fn SetMarginHeight(&mut self, _height: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }

    pub fn MarginWidth(&self) -> Option<DOMString> {
        None
    }

    pub fn SetMarginWidth(&mut self, _height: &Option<DOMString>) -> ErrorResult {
        Ok(())
    }
}
