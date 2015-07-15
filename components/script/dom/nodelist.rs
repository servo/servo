/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NodeListBinding;
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::node::{Node, NodeHelpers};
use dom::window::Window;

#[derive(JSTraceable)]
#[must_root]
pub enum NodeListType {
    Simple(Vec<JS<Node>>),
    Children(JS<Node>)
}

// https://dom.spec.whatwg.org/#interface-nodelist
#[dom_struct]
pub struct NodeList {
    reflector_: Reflector,
    list_type: NodeListType,
}

impl NodeList {
    fn new_inherited(list_type: NodeListType) -> NodeList {
        NodeList {
            reflector_: Reflector::new(),
            list_type: list_type,
        }
    }

    pub fn new(window: &Window,
               list_type: NodeListType) -> Root<NodeList> {
        reflect_dom_object(box NodeList::new_inherited(list_type),
                           GlobalRef::Window(window), NodeListBinding::Wrap)
    }

    pub fn new_simple_list<T>(window: &Window, iter: T)
                              -> Root<NodeList>
                              where T: Iterator<Item=Root<Node>> {
        NodeList::new(window, NodeListType::Simple(iter.map(|r| JS::from_rooted(&r)).collect()))
    }

    pub fn new_child_list(window: &Window, node: &Node) -> Root<NodeList> {
        NodeList::new(window, NodeListType::Children(JS::from_ref(node)))
    }
}

impl<'a> NodeListMethods for &'a NodeList {
    // https://dom.spec.whatwg.org/#dom-nodelist-length
    fn Length(self) -> u32 {
        match self.list_type {
            NodeListType::Simple(ref elems) => elems.len() as u32,
            NodeListType::Children(ref node) => {
                let node = node.root();
                node.r().children().count() as u32
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-nodelist-item
    fn Item(self, index: u32) -> Option<Root<Node>> {
        match self.list_type {
            _ if index >= self.Length() => None,
            NodeListType::Simple(ref elems) => Some(elems[index as usize].root()),
            NodeListType::Children(ref node) => {
                let node = node.root();
                node.r().children().nth(index as usize)
            }
        }
    }

    fn IndexedGetter(self, index: u32, found: &mut bool) -> Option<Root<Node>> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}

