/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::element::{HTMLImageElementTypeId, HTMLIframeElementTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::node::{Node, AbstractNode, ElementNodeTypeId};
use servo_util::str::DOMString;

use std::unstable::raw::Box;

pub trait VirtualMethods {
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods>;

    fn before_set_attr(&mut self, abstract_self: AbstractNode, name: DOMString, old_value: Option<DOMString>, new_value: DOMString) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().before_set_attr(abstract_self, name, old_value, new_value);
        }
    }

    fn after_set_attr(&mut self, abstract_self: AbstractNode, name: DOMString, value: DOMString) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().after_set_attr(abstract_self, name, value);
        }
    }

    fn before_remove_attr(&mut self, abstract_self: AbstractNode, name: DOMString, value: DOMString) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().before_remove_attr(abstract_self, name, value);
        }
    }

    fn after_remove_attr(&mut self, abstract_self: AbstractNode, name: DOMString) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().after_remove_attr(abstract_self, name);
        }
    }

    fn bind_to_tree(&mut self, abstract_self: AbstractNode) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().bind_to_tree(abstract_self);
        }
    }

    fn unbind_from_tree(&mut self, abstract_self: AbstractNode) {
        let s = self.super_type();
        if s.is_some() {
            s.unwrap().unbind_from_tree(abstract_self);
        }
   }
}

pub fn vtable_for(node: AbstractNode) -> &mut VirtualMethods {
    unsafe {
        match node.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => {
                let elem = node.raw_object() as *mut Box<HTMLImageElement>;
                &mut (*elem).data as &mut VirtualMethods
            }
            ElementNodeTypeId(HTMLIframeElementTypeId) => {
                let elem = node.raw_object() as *mut Box<HTMLIFrameElement>;
                &mut (*elem).data as &mut VirtualMethods
            }
            ElementNodeTypeId(_) => {
                let elem = node.raw_object() as *mut Box<HTMLElement>;
                &mut (*elem).data as &mut VirtualMethods
            }
            _ => {
                let elem = node.raw_object() as *mut Box<Node>;
                &mut (*elem).data as &mut VirtualMethods
            }
        }
    }
}