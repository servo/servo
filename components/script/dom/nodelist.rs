/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NodeListBinding;
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, Rootable, Temporary};
use dom::bindings::trace::RootedVec;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::node::{Node, NodeHelpers};
use dom::window::Window;

#[jstraceable]
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

    pub fn new(window: JSRef<Window>,
               list_type: NodeListType) -> Temporary<NodeList> {
        reflect_dom_object(box NodeList::new_inherited(list_type),
                           GlobalRef::Window(window), NodeListBinding::Wrap)
    }

    pub fn new_simple_list(window: JSRef<Window>, elements: &RootedVec<JS<Node>>) -> Temporary<NodeList> {
        NodeList::new(window, NodeListType::Simple((**elements).clone()))
    }

    pub fn new_child_list(window: JSRef<Window>, node: JSRef<Node>) -> Temporary<NodeList> {
        NodeList::new(window, NodeListType::Children(JS::from_rooted(node)))
    }
}

impl<'a> NodeListMethods for JSRef<'a, NodeList> {
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
    fn Item(self, index: u32) -> Option<Temporary<Node>> {
        match self.list_type {
            _ if index >= self.Length() => None,
            NodeListType::Simple(ref elems) => Some(Temporary::from_rooted(elems[index as usize].clone())),
            NodeListType::Children(ref node) => {
                let node = node.root();
                node.r().children().nth(index as usize)
            }
        }
    }

    fn IndexedGetter(self, index: u32, found: &mut bool) -> Option<Temporary<Node>> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}

