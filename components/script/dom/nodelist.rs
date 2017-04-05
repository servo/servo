/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::NodeListBinding;
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::js::{JS, MutNullableJS, Root, RootedReference};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::node::{ChildrenMutation, Node};
use dom::window::Window;
use dom_struct::dom_struct;
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
    #[allow(unrooted_must_root)]
    pub fn new_inherited(list_type: NodeListType) -> NodeList {
        NodeList {
            reflector_: Reflector::new(),
            list_type: list_type,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(window: &Window, list_type: NodeListType) -> Root<NodeList> {
        reflect_dom_object(box NodeList::new_inherited(list_type),
                           window,
                           NodeListBinding::Wrap)
    }

    pub fn new_simple_list<T>(window: &Window, iter: T) -> Root<NodeList>
                              where T: Iterator<Item=Root<Node>> {
        NodeList::new(window, NodeListType::Simple(iter.map(|r| JS::from_ref(&*r)).collect()))
    }

    pub fn new_child_list(window: &Window, node: &Node) -> Root<NodeList> {
        NodeList::new(window, NodeListType::Children(ChildrenList::new(node)))
    }

    pub fn empty(window: &Window) -> Root<NodeList> {
        NodeList::new(window, NodeListType::Simple(vec![]))
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
                elems.get(index as usize).map(|node| Root::from_ref(&**node))
            },
            NodeListType::Children(ref list) => list.item(index),
        }
    }

    // https://dom.spec.whatwg.org/#dom-nodelist-item
    fn IndexedGetter(&self, index: u32) -> Option<Root<Node>> {
        self.Item(index)
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

    pub fn as_simple_list(&self) -> &Vec<JS<Node>> {
        if let NodeListType::Simple(ref list) = self.list_type {
            list
        } else {
            panic!("called as_simple_list() on a children node list")
        }
    }

    pub fn iter(&self) -> NodeListIterator {
        NodeListIterator {
            nodes: self,
            offset: 0,
        }
    }
}

#[derive(JSTraceable, HeapSizeOf)]
#[must_root]
pub struct ChildrenList {
    node: JS<Node>,
    #[ignore_heap_size_of = "Defined in rust-mozjs"]
    last_visited: MutNullableJS<Node>,
    last_index: Cell<u32>,
}

impl ChildrenList {
    pub fn new(node: &Node) -> ChildrenList {
        let last_visited = node.GetFirstChild();
        ChildrenList {
            node: JS::from_ref(node),
            last_visited: MutNullableJS::new(last_visited.r()),
            last_index: Cell::new(0u32),
        }
    }

    pub fn len(&self) -> u32 {
        self.node.children_count()
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
            return self.node.GetFirstChild();
        }
        let last_index = self.last_index.get();
        if index == last_index {
            // Item is last visited child, no need to update last visited.
            return Some(self.last_visited.get().unwrap());
        }
        let last_visited = if index - 1u32 == last_index {
            // Item is last visited's next sibling.
            self.last_visited.get().unwrap().GetNextSibling().unwrap()
        } else if last_index > 0 && index == last_index - 1u32 {
            // Item is last visited's previous sibling.
            self.last_visited.get().unwrap().GetPreviousSibling().unwrap()
        } else if index > last_index {
            if index == len - 1u32 {
                // Item is parent's last child, not worth updating last visited.
                return Some(self.node.GetLastChild().unwrap());
            }
            if index <= last_index + (len - last_index) / 2u32 {
                // Item is closer to the last visited child and follows it.
                self.last_visited.get().unwrap()
                                 .inclusively_following_siblings()
                                 .nth((index - last_index) as usize).unwrap()
            } else {
                // Item is closer to parent's last child and obviously
                // precedes it.
                self.node.GetLastChild().unwrap()
                    .inclusively_preceding_siblings()
                    .nth((len - index - 1u32) as usize).unwrap()
            }
        } else if index >= last_index / 2u32 {
            // Item is closer to the last visited child and precedes it.
            self.last_visited.get().unwrap()
                             .inclusively_preceding_siblings()
                             .nth((last_index - index) as usize).unwrap()
        } else {
            // Item is closer to parent's first child and obviously follows it.
            debug_assert!(index < last_index / 2u32);
            self.node.GetFirstChild().unwrap()
                     .inclusively_following_siblings()
                     .nth(index as usize)
                     .unwrap()
        };
        self.last_visited.set(Some(&last_visited));
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
                list.last_visited.set(Some(added[index as usize]));
            } else if index / 2u32 >= len {
                // If last index is twice as large as the number of added nodes,
                // updating only it means that less nodes will be traversed if
                // caller is traversing the node list linearly.
                list.last_index.set(len + index);
            } else {
                // If last index is not twice as large but still larger,
                // it's better to update it to the number of added nodes.
                list.last_visited.set(Some(next));
                list.last_index.set(len);
            }
        }

        fn replace(list: &ChildrenList,
                   prev: Option<&Node>,
                   removed: &Node,
                   added: &[&Node],
                   next: Option<&Node>) {
            let index = list.last_index.get();
            if removed == &*list.last_visited.get().unwrap() {
                let visited = match (prev, added, next) {
                    (None, _, None) => {
                        // Such cases where parent had only one child should
                        // have been changed into ChildrenMutation::ReplaceAll
                        // by ChildrenMutation::replace().
                        unreachable!()
                    },
                    (_, &[node, ..], _) => node,
                    (_, &[], Some(next)) => next,
                    (Some(prev), &[], None) => {
                        list.last_index.set(index - 1u32);
                        prev
                    },
                };
                list.last_visited.set(Some(visited));
            } else if added.len() != 1 {
                // The replaced child isn't the last visited one, and there are
                // 0 or more than 1 nodes to replace it. Special care must be
                // given to update the state of that ChildrenList.
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
                    self.last_visited.set(Some(added[index as usize]));
                } else {
                    // Setting last visited to parent's last child serves no purpose,
                    // so the middle is arbitrarily chosen here in case the caller
                    // wants random access.
                    let middle = len / 2;
                    self.last_visited.set(Some(added[middle]));
                    self.last_index.set(middle as u32);
                }
            },
        }
    }

    fn reset(&self) {
        self.last_visited.set(self.node.GetFirstChild().r());
        self.last_index.set(0u32);
    }
}

pub struct NodeListIterator<'a> {
    nodes: &'a NodeList,
    offset: u32,
}

impl<'a> Iterator for NodeListIterator<'a> {
    type Item = Root<Node>;

    fn next(&mut self) -> Option<Root<Node>> {
        let result = self.nodes.Item(self.offset);
        self.offset = self.offset + 1;
        result
    }
}
