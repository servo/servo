/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::InheritTypes::HTMLElementCast;
use dom::bindings::codegen::InheritTypes::HTMLIFrameElementCast;
use dom::bindings::codegen::InheritTypes::HTMLImageElementCast;
use dom::bindings::codegen::InheritTypes::HTMLObjectElementCast;
use dom::bindings::js::JS;
use dom::element::Element;
use dom::element::{HTMLImageElementTypeId, HTMLIframeElementTypeId, HTMLObjectElementTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlobjectelement::HTMLObjectElement;
use dom::node::{Node, ElementNodeTypeId};
use servo_util::str::DOMString;

use std::cast;

pub trait VirtualMethods {
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods>;

    fn after_set_attr(&mut self, abstract_self: &JS<Element>, name: DOMString, value: DOMString) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().after_set_attr(abstract_self, name, value);
        }
    }

    fn before_remove_attr(&mut self, abstract_self: &JS<Element>, name: DOMString, value: DOMString) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().before_remove_attr(abstract_self, name, value);
        }
    }
}

pub fn vtable_for<'a>(node: &'a mut JS<Node>) -> &'a mut VirtualMethods {
    unsafe {
        match node.get().type_id {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                let mut elem: JS<HTMLImageElement> = HTMLImageElementCast::to(node);
                cast::transmute_mut_region(elem.get_mut()) as &mut VirtualMethods
            }
            ElementNodeTypeId(HTMLIframeElementTypeId) => {
                let mut elem: JS<HTMLIFrameElement> = HTMLIFrameElementCast::to(node);
                cast::transmute_mut_region(elem.get_mut()) as &mut VirtualMethods
            }
            ElementNodeTypeId(HTMLObjectElementTypeId) => {
                let mut elem: JS<HTMLObjectElement> = HTMLObjectElementCast::to(node);
                cast::transmute_mut_region(elem.get_mut()) as &mut VirtualMethods
            }
            ElementNodeTypeId(_) => {
                let mut elem: JS<HTMLElement> = HTMLElementCast::to(node);
                cast::transmute_mut_region(elem.get_mut()) as &mut VirtualMethods
            }
            _ => {
                cast::transmute_mut_region(node.get_mut()) as &mut VirtualMethods
            }
        }
    }
}
