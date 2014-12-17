/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NodeListBinding;
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::global;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::node::{Node, NodeHelpers};
use dom::window::Window;

#[jstraceable]
#[must_root]
pub enum NodeListType {
    Simple(Vec<JS<Node>>),
    Children(JS<Node>)
}

#[dom_struct]
pub struct NodeList {
    list_type: NodeListType,
    reflector_: Reflector,
}

impl NodeList {
    fn new_inherited(list_type: NodeListType) -> NodeList {
        NodeList {
            list_type: list_type,
            reflector_: Reflector::new(),
        }
    }

    pub fn new(window: JSRef<Window>,
               list_type: NodeListType) -> Temporary<NodeList> {
        reflect_dom_object(box NodeList::new_inherited(list_type),
                           global::Window(window), NodeListBinding::Wrap)
    }

    pub fn new_simple_list(window: JSRef<Window>, elements: Vec<JSRef<Node>>) -> Temporary<NodeList> {
        NodeList::new(window, NodeListType::Simple(elements.iter().map(|element| JS::from_rooted(*element)).collect()))
    }

    pub fn new_child_list(window: JSRef<Window>, node: JSRef<Node>) -> Temporary<NodeList> {
        NodeList::new(window, NodeListType::Children(JS::from_rooted(node)))
    }
}

impl<'a> NodeListMethods for JSRef<'a, NodeList> {
    fn Length(self) -> u32 {
        match self.list_type {
            NodeListType::Simple(ref elems) => elems.len() as u32,
            NodeListType::Children(ref node) => {
                let node = node.root();
                node.children().count() as u32
            }
        }
    }

    fn Item(self, index: u32) -> Option<Temporary<Node>> {
        match self.list_type {
            _ if index >= self.Length() => None,
            NodeListType::Simple(ref elems) => Some(Temporary::new(elems[index as uint].clone())),
            NodeListType::Children(ref node) => {
                let node = node.root();
                node.children().nth(index as uint)
                                       .map(|child| Temporary::from_rooted(child))
            }
        }
    }

    fn IndexedGetter(self, index: u32, found: &mut bool) -> Option<Temporary<Node>> {
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
