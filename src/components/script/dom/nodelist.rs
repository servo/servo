/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NodeListBinding;
use dom::bindings::global::Window;
use dom::bindings::js::{JS, JSRef, Temporary};
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
    list_type: NodeListType,
    reflector_: Reflector,
}

impl NodeList {
    pub fn new_inherited(list_type: NodeListType) -> NodeList {
        NodeList {
            list_type: list_type,
            reflector_: Reflector::new(),
        }
    }

    pub fn new(window: &JSRef<Window>,
               list_type: NodeListType) -> Temporary<NodeList> {
        reflect_dom_object(box NodeList::new_inherited(list_type),
                           &Window(*window), NodeListBinding::Wrap)
    }

    pub fn new_simple_list(window: &JSRef<Window>, elements: Vec<JSRef<Node>>) -> Temporary<NodeList> {
        NodeList::new(window, Simple(elements.iter().map(|element| JS::from_rooted(element)).collect()))
    }

    pub fn new_child_list(window: &JSRef<Window>, node: &JSRef<Node>) -> Temporary<NodeList> {
        NodeList::new(window, Children(JS::from_rooted(node)))
    }
}

pub trait NodeListMethods {
    fn Length(&self) -> u32;
    fn Item(&self, index: u32) -> Option<Temporary<Node>>;
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Temporary<Node>>;
}

impl<'a> NodeListMethods for JSRef<'a, NodeList> {
    fn Length(&self) -> u32 {
        match self.list_type {
            Simple(ref elems) => elems.len() as u32,
            Children(ref node) => {
                let node = node.root();
                node.deref().children().count() as u32
            }
        }
    }

    fn Item(&self, index: u32) -> Option<Temporary<Node>> {
        match self.list_type {
            _ if index >= self.Length() => None,
            Simple(ref elems) => Some(Temporary::new(elems.get(index as uint).clone())),
            Children(ref node) => {
                let node = node.root();
                node.deref().children().nth(index as uint)
                                       .map(|child| Temporary::from_rooted(&child))
            }
        }
    }

    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Temporary<Node>> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}

impl Reflectable for NodeList {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}
