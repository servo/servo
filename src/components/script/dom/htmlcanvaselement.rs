/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLCanvasElementBinding;
use dom::bindings::codegen::CanvasRenderingContext2DBinding;
use std::libc::*;
use dom::canvasrenderingcontext2d::CanvasRenderingContext2D;
use dom::window::Window;
use dom::bindings::utils::{ErrorResult};
use dom::document::AbstractDocument;
use dom::element::HTMLCanvasElementTypeId;
use dom::htmlelement::HTMLElement;
use dom::bindings::utils::{DOMString, ErrorResult, Fallible};
use dom::node::{ElementNodeTypeId, Node, ScriptView, AbstractNode};
use dom::element::Element;
use dom::element::ElementTypeId;
use dom::element::{Element, ElementTypeId};

pub struct HTMLCanvasElement {
    htmlelement: HTMLElement,
  }

impl HTMLCanvasElement {
    pub fn new_inherited(type_id: ElementTypeId, localName: ~str, document: AbstractDocument) -> HTMLCanvasElement {
        HTMLCanvasElement {
            htmlelement: HTMLElement::new_inherited(HTMLCanvasElementTypeId, localName, document),
       }
    }

    pub fn new(localName: ~str, document: AbstractDocument) -> AbstractNode<ScriptView> {
        let element = HTMLCanvasElement::new_inherited(HTMLCanvasElementTypeId, localName, document);
        Node::reflect_node(@mut element, document, HTMLCanvasElementBinding::Wrap)
    }
}

impl HTMLCanvasElement {
    pub fn Width(&self) -> u32 {
        0
    }

    pub fn SetWidth(&mut self, _width: u32) -> ErrorResult {
        Ok(())
    }

    pub fn Height(&self) -> u32 {
        0
    }

    pub fn SetHeight(&mut self, _height: u32) -> ErrorResult {
        Ok(())
    }
  
    pub fn GetContext(&self, _id: DOMString) -> @mut CanvasRenderingContext2D {
    
          let canvas = CanvasRenderingContext2D::new(self.htmlelement.element.node.owner_doc().document().window);
	  canvas
     }
}
