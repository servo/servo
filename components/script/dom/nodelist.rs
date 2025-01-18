/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use servo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::HTMLFormElement;
use crate::dom::node::{ChildrenMutation, Node};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) enum NodeListType {
    Simple(Vec<Dom<Node>>),
    Children(ChildrenList),
    Labels(LabelsList),
    Radio(RadioList),
    ElementsByName(ElementsByNameList),
}

// https://dom.spec.whatwg.org/#interface-nodelist
#[dom_struct]
pub(crate) struct NodeList {
    reflector_: Reflector,
    list_type: NodeListType,
}

impl NodeList {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(list_type: NodeListType) -> NodeList {
        NodeList {
            reflector_: Reflector::new(),
            list_type,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(window: &Window, list_type: NodeListType) -> DomRoot<NodeList> {
        reflect_dom_object(
            Box::new(NodeList::new_inherited(list_type)),
            window,
            CanGc::note(),
        )
    }

    pub(crate) fn new_simple_list<T>(window: &Window, iter: T) -> DomRoot<NodeList>
    where
        T: Iterator<Item = DomRoot<Node>>,
    {
        NodeList::new(
            window,
            NodeListType::Simple(iter.map(|r| Dom::from_ref(&*r)).collect()),
        )
    }

    pub(crate) fn new_simple_list_slice(window: &Window, slice: &[&Node]) -> DomRoot<NodeList> {
        NodeList::new(
            window,
            NodeListType::Simple(slice.iter().map(|r| Dom::from_ref(*r)).collect()),
        )
    }

    pub(crate) fn new_child_list(window: &Window, node: &Node) -> DomRoot<NodeList> {
        NodeList::new(window, NodeListType::Children(ChildrenList::new(node)))
    }

    pub(crate) fn new_labels_list(window: &Window, element: &HTMLElement) -> DomRoot<NodeList> {
        NodeList::new(window, NodeListType::Labels(LabelsList::new(element)))
    }

    pub(crate) fn new_elements_by_name_list(
        window: &Window,
        document: &Document,
        name: DOMString,
    ) -> DomRoot<NodeList> {
        NodeList::new(
            window,
            NodeListType::ElementsByName(ElementsByNameList::new(document, name)),
        )
    }

    pub(crate) fn empty(window: &Window) -> DomRoot<NodeList> {
        NodeList::new(window, NodeListType::Simple(vec![]))
    }
}

impl NodeListMethods<crate::DomTypeHolder> for NodeList {
    // https://dom.spec.whatwg.org/#dom-nodelist-length
    fn Length(&self) -> u32 {
        match self.list_type {
            NodeListType::Simple(ref elems) => elems.len() as u32,
            NodeListType::Children(ref list) => list.len(),
            NodeListType::Labels(ref list) => list.len(),
            NodeListType::Radio(ref list) => list.len(),
            NodeListType::ElementsByName(ref list) => list.len(),
        }
    }

    // https://dom.spec.whatwg.org/#dom-nodelist-item
    fn Item(&self, index: u32) -> Option<DomRoot<Node>> {
        match self.list_type {
            NodeListType::Simple(ref elems) => elems
                .get(index as usize)
                .map(|node| DomRoot::from_ref(&**node)),
            NodeListType::Children(ref list) => list.item(index),
            NodeListType::Labels(ref list) => list.item(index),
            NodeListType::Radio(ref list) => list.item(index),
            NodeListType::ElementsByName(ref list) => list.item(index),
        }
    }

    // https://dom.spec.whatwg.org/#dom-nodelist-item
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Node>> {
        self.Item(index)
    }
}

impl NodeList {
    pub(crate) fn as_children_list(&self) -> &ChildrenList {
        if let NodeListType::Children(ref list) = self.list_type {
            list
        } else {
            panic!("called as_children_list() on a non-children node list")
        }
    }

    pub(crate) fn as_radio_list(&self) -> &RadioList {
        if let NodeListType::Radio(ref list) = self.list_type {
            list
        } else {
            panic!("called as_radio_list() on a non-radio node list")
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = DomRoot<Node>> + '_ {
        let len = self.Length();
        // There is room for optimization here in non-simple cases,
        // as calling Item repeatedly on a live list can involve redundant work.
        (0..len).flat_map(move |i| self.Item(i))
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ChildrenList {
    node: Dom<Node>,
    #[ignore_malloc_size_of = "Defined in rust-mozjs"]
    last_visited: MutNullableDom<Node>,
    last_index: Cell<u32>,
}

impl ChildrenList {
    pub(crate) fn new(node: &Node) -> ChildrenList {
        let last_visited = node.GetFirstChild();
        ChildrenList {
            node: Dom::from_ref(node),
            last_visited: MutNullableDom::new(last_visited.as_deref()),
            last_index: Cell::new(0u32),
        }
    }

    pub(crate) fn len(&self) -> u32 {
        self.node.children_count()
    }

    pub(crate) fn item(&self, index: u32) -> Option<DomRoot<Node>> {
        // This always start traversing the children from the closest element
        // among parent's first and last children and the last visited one.
        let len = self.len();
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
            self.last_visited
                .get()
                .unwrap()
                .GetPreviousSibling()
                .unwrap()
        } else if index > last_index {
            if index == len - 1u32 {
                // Item is parent's last child, not worth updating last visited.
                return Some(self.node.GetLastChild().unwrap());
            }
            if index <= last_index + (len - last_index) / 2u32 {
                // Item is closer to the last visited child and follows it.
                self.last_visited
                    .get()
                    .unwrap()
                    .inclusively_following_siblings()
                    .nth((index - last_index) as usize)
                    .unwrap()
            } else {
                // Item is closer to parent's last child and obviously
                // precedes it.
                self.node
                    .GetLastChild()
                    .unwrap()
                    .inclusively_preceding_siblings()
                    .nth((len - index - 1u32) as usize)
                    .unwrap()
            }
        } else if index >= last_index / 2u32 {
            // Item is closer to the last visited child and precedes it.
            self.last_visited
                .get()
                .unwrap()
                .inclusively_preceding_siblings()
                .nth((last_index - index) as usize)
                .unwrap()
        } else {
            // Item is closer to parent's first child and obviously follows it.
            debug_assert!(index < last_index / 2u32);
            self.node
                .GetFirstChild()
                .unwrap()
                .inclusively_following_siblings()
                .nth(index as usize)
                .unwrap()
        };
        self.last_visited.set(Some(&last_visited));
        self.last_index.set(index);
        Some(last_visited)
    }

    pub(crate) fn children_changed(&self, mutation: &ChildrenMutation) {
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

        fn replace(
            list: &ChildrenList,
            prev: Option<&Node>,
            removed: &Node,
            added: &[&Node],
            next: Option<&Node>,
        ) {
            let index = list.last_index.get();
            if removed == &*list.last_visited.get().unwrap() {
                let visited = match (prev, added, next) {
                    (None, _, None) => {
                        // Such cases where parent had only one child should
                        // have been changed into ChildrenMutation::ReplaceAll
                        // by ChildrenMutation::replace().
                        unreachable!()
                    },
                    (_, added, _) if !added.is_empty() => added[0],
                    (_, _, Some(next)) => next,
                    (Some(prev), _, None) => {
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
            ChildrenMutation::Replace {
                prev,
                removed,
                added,
                next,
            } => {
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
            ChildrenMutation::ChangeText => {},
        }
    }

    fn reset(&self) {
        self.last_visited.set(self.node.GetFirstChild().as_deref());
        self.last_index.set(0u32);
    }
}

// Labels lists: There might be room for performance optimization
// analogous to the ChildrenMutation case of a children list,
// in which we can keep information from an older access live
// if we know nothing has happened that would change it.
// However, label relationships can happen from further away
// in the DOM than parent-child relationships, so it's not as simple,
// and it's possible that tracking label moves would end up no faster
// than recalculating labels.
#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct LabelsList {
    element: Dom<HTMLElement>,
}

impl LabelsList {
    pub(crate) fn new(element: &HTMLElement) -> LabelsList {
        LabelsList {
            element: Dom::from_ref(element),
        }
    }

    pub(crate) fn len(&self) -> u32 {
        self.element.labels_count()
    }

    pub(crate) fn item(&self, index: u32) -> Option<DomRoot<Node>> {
        self.element.label_at(index)
    }
}

// Radio node lists: There is room for performance improvement here;
// a form is already aware of changes to its set of controls,
// so a radio list can cache and cache-invalidate its contents
// just by hooking into what the form already knows without a
// separate mutation observer. FIXME #25482
#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
pub(crate) enum RadioListMode {
    ControlsExceptImageInputs,
    Images,
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct RadioList {
    form: Dom<HTMLFormElement>,
    mode: RadioListMode,
    #[no_trace]
    name: Atom,
}

impl RadioList {
    pub(crate) fn new(form: &HTMLFormElement, mode: RadioListMode, name: Atom) -> RadioList {
        RadioList {
            form: Dom::from_ref(form),
            mode,
            name,
        }
    }

    pub(crate) fn len(&self) -> u32 {
        self.form.count_for_radio_list(self.mode, &self.name)
    }

    pub(crate) fn item(&self, index: u32) -> Option<DomRoot<Node>> {
        self.form.nth_for_radio_list(index, self.mode, &self.name)
    }
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct ElementsByNameList {
    document: Dom<Document>,
    name: DOMString,
}

impl ElementsByNameList {
    pub(crate) fn new(document: &Document, name: DOMString) -> ElementsByNameList {
        ElementsByNameList {
            document: Dom::from_ref(document),
            name,
        }
    }

    pub(crate) fn len(&self) -> u32 {
        self.document.elements_by_name_count(&self.name)
    }

    pub(crate) fn item(&self, index: u32) -> Option<DomRoot<Node>> {
        self.document
            .nth_element_by_name(index, &self.name)
            .map(|n| DomRoot::from_ref(&*n))
    }
}
