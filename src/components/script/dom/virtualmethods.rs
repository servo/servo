/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::utils::DOMString;
use dom::element::{HTMLImageElementTypeId, HTMLIframeElementTypeId};
use dom::htmlelement::HTMLElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::node::{Node, AbstractNode, ElementNodeTypeId};

use std::cast;
use std::unstable::raw::Box;

pub trait VirtualMethods {
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods>;

    fn after_set_attr(&mut self, name: DOMString, value: DOMString) {
        self.super_type().map(|s| s.after_set_attr(name.clone(), value.clone()));
    }

    fn after_remove_attr(&mut self, name: DOMString) {
        self.super_type().map(|s| s.after_remove_attr(name.clone()));
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