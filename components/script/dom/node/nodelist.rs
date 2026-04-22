/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use stylo_atoms::Atom;

use crate::dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlformelement::HTMLFormElement;
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
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn new_inherited(list_type: NodeListType) -> NodeList {
        NodeList {
            reflector_: Reflector::new(),
            list_type,
        }
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn new(
        window: &Window,
        list_type: NodeListType,
        can_gc: CanGc,
    ) -> DomRoot<NodeList> {
        reflect_dom_object(Box::new(NodeList::new_inherited(list_type)), window, can_gc)
    }

    pub(crate) fn new_simple_list<T>(window: &Window, iter: T, can_gc: CanGc) -> DomRoot<NodeList>
    where
        T: Iterator<Item = DomRoot<Node>>,
    {
        NodeList::new(
            window,
            NodeListType::Simple(iter.map(|r| Dom::from_ref(&*r)).collect()),
            can_gc,
        )
    }

    pub(crate) fn new_simple_list_slice(
        window: &Window,
        slice: &[&Node],
        can_gc: CanGc,
    ) -> DomRoot<NodeList> {
        NodeList::new(
            window,
            NodeListType::Simple(slice.iter().map(|r| Dom::from_ref(*r)).collect()),
            can_gc,
        )
    }

    pub(crate) fn new_child_list(window: &Window, node: &Node, can_gc: CanGc) -> DomRoot<NodeList> {
        NodeList::new(
            window,
            NodeListType::Children(ChildrenList::new(node)),
            can_gc,
        )
    }

    pub(crate) fn new_labels_list(
        window: &Window,
        element: &HTMLElement,
        can_gc: CanGc,
    ) -> DomRoot<NodeList> {
        NodeList::new(
            window,
            NodeListType::Labels(LabelsList::new(element)),
            can_gc,
        )
    }

    pub(crate) fn new_elements_by_name_list(
        window: &Window,
        document: &Document,
        name: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<NodeList> {
        NodeList::new(
            window,
            NodeListType::ElementsByName(ElementsByNameList::new(document, name)),
            can_gc,
        )
    }

    pub(crate) fn empty(window: &Window, can_gc: CanGc) -> DomRoot<NodeList> {
        NodeList::new(window, NodeListType::Simple(vec![]), can_gc)
    }
}

impl NodeListMethods<crate::DomTypeHolder> for NodeList {
    /// <https://dom.spec.whatwg.org/#dom-nodelist-length>
    fn Length(&self) -> u32 {
        match self.list_type {
            NodeListType::Simple(ref elems) => elems.len() as u32,
            NodeListType::Children(ref list) => list.len(),
            NodeListType::Labels(ref list) => list.len(),
            NodeListType::Radio(ref list) => list.len(),
            NodeListType::ElementsByName(ref list) => list.len(),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-nodelist-item>
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

    /// <https://dom.spec.whatwg.org/#dom-nodelist-item>
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
    cached_children: RefCell<Option<Vec<Dom<Node>>>>,
}

impl ChildrenList {
    pub(crate) fn new(node: &Node) -> ChildrenList {
        ChildrenList {
            node: Dom::from_ref(node),
            cached_children: RefCell::new(None),
        }
    }

    pub(crate) fn len(&self) -> u32 {
        self.node.children_count()
    }

    pub(crate) fn item(&self, index: u32) -> Option<DomRoot<Node>> {
        self.cached_children
            .borrow_mut()
            .get_or_insert_with(|| {
                self.node
                    .children()
                    .map(|child| Dom::from_ref(&*child))
                    .collect()
            })
            .get(index as usize)
            .map(|child| DomRoot::from_ref(&**child))
    }

    pub(crate) fn children_changed(&self, mutation: &ChildrenMutation) {
        match mutation {
            ChildrenMutation::Append { .. } |
            ChildrenMutation::Insert { .. } |
            ChildrenMutation::Prepend { .. } |
            ChildrenMutation::Replace { .. } |
            ChildrenMutation::ReplaceAll { .. } => *self.cached_children.borrow_mut() = None,
            ChildrenMutation::ChangeText => {},
        }
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
