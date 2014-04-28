/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::NodeListBinding;
use dom::bindings::js::JS;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::node::{Node, NodeHelpers};
use dom::window::Window;

#[deriving(Encodable)]
pub enum NodeListType {
    Simple(Vec<JS<Node>>),
    Children(JS<Node>)
}

#[deriving(Encodable)]
pub struct NodeList {
    pub list_type: NodeListType,
    pub reflector_: Reflector,
    pub window: JS<Window>
}

impl NodeList {
    pub fn new_inherited(window: JS<Window>,
                         list_type: NodeListType) -> NodeList {
        NodeList {
            list_type: list_type,
            reflector_: Reflector::new(),
            window: window
        }
    }

    pub fn new(window: &JS<Window>,
               list_type: NodeListType) -> JS<NodeList> {
        reflect_dom_object(~NodeList::new_inherited(window.clone(), list_type),
                           window, NodeListBinding::Wrap)
    }

    pub fn new_simple_list(window: &JS<Window>, elements: Vec<JS<Node>>) -> JS<NodeList> {
        NodeList::new(window, Simple(elements))
    }

    pub fn new_child_list(window: &JS<Window>, node: &JS<Node>) -> JS<NodeList> {
        NodeList::new(window, Children(node.clone()))
    }

    pub fn Length(&self) -> u32 {
        match self.list_type {
            Simple(ref elems) => elems.len() as u32,
            Children(ref node) => node.children().len() as u32
        }
    }

    pub fn Item(&self, index: u32) -> Option<JS<Node>> {
        match self.list_type {
            _ if index >= self.Length() => None,
            Simple(ref elems) => Some(elems.get(index as uint).clone()),
            Children(ref node) => node.children().nth(index as uint)
        }
    }

    pub fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<JS<Node>> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}

impl Reflectable for NodeList {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}
