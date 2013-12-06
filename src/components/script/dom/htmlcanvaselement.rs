/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::HTMLCanvasElementBinding;
use dom::bindings::codegen::InheritTypes::{HTMLCanvasElementDerived, HTMLElementCast};
use dom::bindings::codegen::InheritTypes::ElementCast;
use dom::bindings::js::JS;
use dom::canvasrenderingcontext2d::CanvasRenderingContext2D;
use dom::document::Document;
use dom::element::{Element, HTMLCanvasElementTypeId, AttributeHandlers};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, ElementNodeTypeId};
use dom::virtualmethods::VirtualMethods;
use servo_util::str::DOMString;

use std::num;

#[deriving(Encodable)]
pub struct HTMLCanvasElement {
    htmlelement: HTMLElement,
    context: Option<JS<CanvasRenderingContext2D>>,
    width: u32,
    height: u32,
  }

impl HTMLCanvasElementDerived for EventTarget {
    fn is_htmlcanvaselement(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(ElementNodeTypeId(HTMLCanvasElementTypeId)) => true,
            _ => false
        }
    }
}

impl HTMLCanvasElement {
    pub fn new_inherited(localName: DOMString, document: JS<Document>) -> HTMLCanvasElement {
        HTMLCanvasElement {
            htmlelement: HTMLElement::new_inherited(HTMLCanvasElementTypeId, localName, document),
            context: None,
            width: 100,
            height: 100,
       }
    }

    pub fn new(localName: DOMString, document: &JS<Document>) -> JS<HTMLCanvasElement> {
        let element = HTMLCanvasElement::new_inherited(localName, document.clone());
        Node::reflect_node(~element, document, HTMLCanvasElementBinding::Wrap)
    }
}

impl HTMLCanvasElement {
    pub fn Width(&self, _abstract_self: &JS<HTMLCanvasElement>) -> u32 {
        self.width
    }

    pub fn SetWidth(&self, abstract_self: &mut JS<HTMLCanvasElement>, width: u32) {
        let mut elem: JS<Element> = ElementCast::from(abstract_self);
        elem.set_uint_attribute("width", width)
    }

    pub fn Height(&self, _abstract_self: &JS<HTMLCanvasElement>) -> u32 {
        self.height
    }

    pub fn SetHeight(&mut self, abstract_self: &mut JS<HTMLCanvasElement>, height: u32) {
        let mut elem: JS<Element> = ElementCast::from(abstract_self);
        elem.set_uint_attribute("height", height)
    }

    pub fn GetContext(&mut self, id: DOMString) -> Option<JS<CanvasRenderingContext2D>> {
        if "2d" != id {
            return None;
        }

        if self.context.is_none() {
            self.context = Some(CanvasRenderingContext2D::new(&self.htmlelement.element.node.owner_doc().get().window));
        }
        self.context.clone()
     }
}

impl VirtualMethods for JS<HTMLCanvasElement> {
    fn super_type(&self) -> Option<~VirtualMethods:> {
        let node: JS<HTMLElement> = HTMLElementCast::from(self);
        Some(~node as ~VirtualMethods:)
    }

    fn after_set_attr(&mut self, name: DOMString, value: DOMString) {
        match self.super_type() {
            Some(ref mut s) => s.after_set_attr(name.clone(), value.clone()),
            _ => (),
        }

        let recreate = match name.as_slice() {
            "width" => {
                self.get_mut().width = num::from_str_radix(value, 10).unwrap();
                true
            }
            "height" => {
                self.get_mut().height = num::from_str_radix(value, 10).unwrap();
                true
            }
            _ => false,
        };

        if recreate {
            let (width, height) = (self.get().width, self.get().height);
            match self.get_mut().context {
                Some(ref mut context) => context.get_mut().recreate(width, height),
                None => ()
            }
        }
    }
}
