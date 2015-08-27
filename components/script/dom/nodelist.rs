/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeListBinding;
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::node::{ChildrenMutation, Node};
use dom::window::Window;

use std::cell::Cell;

#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
pub enum NodeListType {
    Simple(Vec<JS<Node>>),
    Children(ChildrenList),
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
        NodeList::new(window, NodeListType::Children(ChildrenList::new(node)))
    }
}

impl NodeListMethods for NodeList {
    // https://dom.spec.whatwg.org/#dom-nodelist-length
    fn Length(&self) -> u32 {
        match self.list_type {
            NodeListType::Simple(ref elems) => elems.len() as u32,
            NodeListType::Children(ref list) => list.len(),
        }
    }

    // https://dom.spec.whatwg.org/#dom-nodelist-item
    fn Item(&self, index: u32) -> Option<Root<Node>> {
        match self.list_type {
            NodeListType::Simple(ref elems) => {
                elems.get(index as usize).map(|node| Root::from_rooted(*node))
            },
            NodeListType::Children(ref list) => list.item(index),
        }
    }

    // https://dom.spec.whatwg.org/#dom-nodelist-item
    fn IndexedGetter(&self, index: u32, found: &mut bool) -> Option<Root<Node>> {
        let item = self.Item(index);
        *found = item.is_some();
        item
    }
}


impl NodeList {
    pub fn as_children_list(&self) -> &ChildrenList {
        if let NodeListType::Children(ref list) = self.list_type {
            list
        } else {
            panic!("called as_children_list() on a simple node list")
        }
    }
}

#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
pub struct ChildrenList {
    node: JS<Node>,
    #[ignore_heap_size_of = "Defined in rust-mozjs"]
    last_visited: MutNullableHeap<JS<Node>>,
    last_index: Cell<u32>,
}

impl ChildrenList {
    fn new(node: &Node) -> ChildrenList {
        let last_visited = node.GetFirstChild();
        ChildrenList {
            node: JS::from_ref(node),
            last_visited:
                MutNullableHeap::new(last_visited.as_ref().map(JS::from_rooted)),
            last_index: Cell::new(0u32),
        }
    }

    pub fn len(&self) -> u32 {
        self.node.root().children_count()
    }

    pub fn item(&self, index: u32) -> Option<Root<Node>> {
        // This always start traversing the children from the closest element
        // among parent's first and last children and the last visited one.
        let len = self.len() as u32;
        if index >= len {
            return None;
        }
        if index == 0u32 {
            // Item is first child if any, not worth updating last visited.
            return self.node.root().GetFirstChild();
        }
        let last_index = self.last_index.get();
        if index == last_index {
            // Item is last visited child, no need to update last visited.
            return Some(self.last_visited.get().unwrap().root());
        }
        let last_visited = if index - 1u32 == last_index {
            // Item is last visited's next sibling.
            self.last_visited.get().unwrap().root().GetNextSibling().unwrap()
        } else if last_index > 0 && index == last_index - 1u32 {
            // Item is last visited's previous sibling.
            self.last_visited.get().unwrap().root().GetPreviousSibling().unwrap()
        } else if index > last_index {
            if index == len - 1u32 {
                // Item is parent's last child, not worth updating last visited.
                return Some(self.node.root().GetLastChild().unwrap());
            }
            if index <= last_index + (len - last_index) / 2u32 {
                // Item is closer to the last visited child and follows it.
                self.last_visited.get().unwrap().root()
                                 .inclusively_following_siblings()
                                 .nth((index - last_index) as usize).unwrap()
            } else {
                // Item is closer to parent's last child and obviously
                // precedes it.
                self.node.root().GetLastChild().unwrap()
                    .inclusively_preceding_siblings()
                    .nth((len - index - 1u32) as usize).unwrap()
            }
        } else if index >= last_index / 2u32 {
            // Item is closer to the last visited child and precedes it.
            self.last_visited.get().unwrap().root()
                             .inclusively_preceding_siblings()
                             .nth((last_index - index) as usize).unwrap()
        } else {
            // Item is closer to parent's first child and obviously follows it.
            debug_assert!(index < last_index / 2u32);
            self.node.root().GetFirstChild().unwrap()
                     .inclusively_following_siblings()
                     .nth(index as usize)
                     .unwrap()
        };
        self.last_visited.set(Some(JS::from_rooted(&last_visited)));
        self.last_index.set(index);
        Some(last_visited)
    }

    pub fn children_changed(&self, mutation: &ChildrenMutation) {
        fn prepend(list: &ChildrenList, added: &[&Node], next: &Node) {
            let len = added.len() as u32;
            if len == 0u32 {
                return;
            }
            let index = list.last_index.get();
            if index < len {
                list.last_visited.set(Some(JS::from_ref(added[index as usize])));
            } else if index / 2u32 >= len {
                // If last index is twice as large as the number of added nodes,
                // updating only it means that less nodes will be traversed if
                // caller is traversing the node list linearly.
                list.last_index.set(len + index);
            } else {
                // If last index is not twice as large but still larger,
                // it's better to update it to the number of added nodes.
                list.last_visited.set(Some(JS::from_ref(next)));
                list.last_index.set(len);
            }
        }

        fn replace(list: &ChildrenList,
                   prev: Option<&Node>,
                   removed: &Node,
                   added: &[&Node],
                   next: Option<&Node>) {
            let index = list.last_index.get();
            if removed == &*list.last_visited.get().unwrap().root() {
                let visited = match (prev, added, next) {
                    (None, _, None) => {
                        // Such cases where parent had only one child should
                        // have been changed into ChildrenMutation::ReplaceAll
                        // by ChildrenMutation::replace().
                        unreachable!()
                    },
                    (_, [node, ..], _) => node,
                    (_, [], Some(next)) => next,
                    (Some(prev), [], None) => {
                        list.last_index.set(index - 1u32);
                        prev
                    },
                };
                list.last_visited.set(Some(JS::from_ref(visited)));
            } else {
                match (prev, next) {
                    (Some(_), None) => {},
                    (None, Some(next)) => {
                        list.last_index.set(index - 1);
                        prepend(list, added, next);
                    },
                    (Some(_), Some(_)) => {
                        list.reset();
                    },
                    (None, None) => unreachable!(),
                }
            }
        }

        match *mutation {
            ChildrenMutation::Append { .. } => {},
            ChildrenMutation::Insert { .. } => {
                self.reset();
            },
            ChildrenMutation::Prepend { added, next } => {
                prepend(self, added, next);
            },
            ChildrenMutation::Replace { prev, removed, added, next } => {
                replace(self, prev, removed, added, next);
            },
            ChildrenMutation::ReplaceAll { added, .. } => {
                let len = added.len();
                let index = self.last_index.get();
                if len == 0 {
                    self.last_visited.set(None);
                    self.last_index.set(0u32);
                } else if index < len as u32 {
                    self.last_visited.set(Some(JS::from_ref(added[index as usize])));
                } else {
                    // Setting last visited to parent's last child serves no purpose,
                    // so the middle is arbitrarily chosen here in case the caller
                    // wants random access.
                    let middle = len / 2;
                    self.last_visited.set(Some(JS::from_ref(added[middle])));
                    self.last_index.set(middle as u32);
                }
            },
        }
    }

    fn reset(&self) {
        self.last_visited.set(
            self.node.root().GetFirstChild().map(|node| JS::from_rooted(&node)));
        self.last_index.set(0u32);
    }
}
