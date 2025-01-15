/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::gc::{RootedGuard, RootedVec};
use js::rust::HandleObject;
use style::attr::AttrValue;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLSlotElementBinding::{
    AssignedNodesOptions, HTMLSlotElementMethods,
};
use crate::dom::bindings::codegen::Bindings::NodeBinding::{GetRootNodeOptions, NodeMethods};
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRoot_Binding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{
    ShadowRootMode, SlotAssignmentMode,
};
use crate::dom::bindings::codegen::UnionTypes::ElementOrText;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{Node, ShadowIncluding};
use crate::dom::text::Text;
use crate::script_runtime::CanGc;

/// <https://html.spec.whatwg.org/multipage/scripting.html#the-slot-element>
#[dom_struct]
pub struct HTMLSlotElement {
    htmlelement: HTMLElement,

    // /// <https://html.spec.whatwg.org/multipage/scripting.html#dom-slot-name>
    // name: DOMString,
    /// <https://dom.spec.whatwg.org/#slot-assigned-nodes>
    assigned_nodes: RefCell<Vec<Slottable>>,

    /// <https://html.spec.whatwg.org/multipage/scripting.html#manually-assigned-nodes>
    manually_assigned_nodes: RefCell<Vec<Slottable>>,
}

impl HTMLSlotElementMethods<crate::DomTypeHolder> for HTMLSlotElement {
    // https://html.spec.whatwg.org/multipage/scripting.html#dom-slot-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-slot-name
    make_atomic_setter!(SetName, "name");

    /// <https://html.spec.whatwg.org/multipage/scripting.html#dom-slot-assignednodes>
    fn AssignedNodes(&self, options: &AssignedNodesOptions) -> Vec<DomRoot<Node>> {
        // Step 1. If options["flatten"] is false, then return this's assigned nodes.
        if !options.flatten {
            return self
                .assigned_nodes
                .borrow()
                .iter()
                .map(|slottable| slottable.node())
                .map(DomRoot::from_ref)
                .collect();
        }

        // Step 2. Return the result of finding flattened slottables with this.
        rooted_vec!(let mut flattened_slottables);
        self.find_flattened_slottables(&mut flattened_slottables);

        flattened_slottables
            .iter()
            .map(|slottable| DomRoot::from_ref(slottable.node()))
            .collect()
    }

    /// <https://html.spec.whatwg.org/multipage/scripting.html#dom-slot-assignedelements>
    fn AssignedElements(&self, options: &AssignedNodesOptions) -> Vec<DomRoot<Element>> {
        self.AssignedNodes(options)
            .into_iter()
            .flat_map(|node| node.downcast::<Element>().map(DomRoot::from_ref))
            .collect()
    }

    /// <https://html.spec.whatwg.org/multipage/scripting.html#dom-slot-assign>
    fn Assign(&self, nodes: Vec<ElementOrText>) {
        let cx = GlobalScope::get_cx();

        // Step 1. For each node of this's manually assigned nodes, set node's manual slot assignment to null.
        for slottable in self.manually_assigned_nodes.borrow().iter() {
            slottable.data().borrow_mut().manual_slot_assignment = None;
        }

        // Step 2. Let nodesSet be a new ordered set.
        rooted_vec!(let mut nodes_set);

        // Step 3. For each node of nodes:
        for element_or_text in nodes.into_iter() {
            rooted!(in(*cx) let node = match element_or_text {
                ElementOrText::Element(element) => Slottable::Element(Dom::from_ref(&element)),
                ElementOrText::Text(text) => Slottable::Text(Dom::from_ref(&text)),
            });

            // Step 3.1 If node's manual slot assignment refers to a slot,
            // then remove node from that slot's manually assigned nodes.
            if let Some(slot) = &node.data().borrow().manual_slot_assignment {
                let mut manually_assigned_nodes = slot.manually_assigned_nodes.borrow_mut();
                if let Some(position) = manually_assigned_nodes
                    .iter()
                    .position(|value| *value == *node)
                {
                    manually_assigned_nodes.remove(position);
                }
            }

            // Step 3.2 Set node's manual slot assignment to this.
            node.data().borrow_mut().manual_slot_assignment = Some(Dom::from_ref(self));

            // Step 3.3 Append node to nodesSet.
            if !nodes_set.contains(&*node) {
                nodes_set.push(node.clone());
            }
        }

        // Step 4. Set this's manually assigned nodes to nodesSet.
        *self.manually_assigned_nodes.borrow_mut() = nodes_set.iter().cloned().collect();

        // Step 5. Run assign slottables for a tree for this's root.
        self.upcast::<Node>()
            .GetRootNode(&GetRootNodeOptions::empty())
            .assign_slottables_for_a_tree();
    }
}

/// <https://dom.spec.whatwg.org/#concept-slotable>
#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
#[crown::unrooted_must_root_lint::must_root]
pub(crate) enum Slottable {
    Element(Dom<Element>),
    Text(Dom<Text>),
}

/// Data shared between all [slottables](https://dom.spec.whatwg.org/#concept-slotable)
///
/// Note that the [slottable name](https://dom.spec.whatwg.org/#slotable-name) is not
/// part of this. While the spec says that all slottables have a name, only Element's
/// can ever have a non-empty name, so they store it seperately
#[derive(Default, JSTraceable, MallocSizeOf)]
#[crown::must_root]
pub struct SlottableData {
    /// <https://dom.spec.whatwg.org/#slotable-assigned-slot>
    assigned_slot: Option<Dom<HTMLSlotElement>>,

    /// <https://dom.spec.whatwg.org/#slottable-manual-slot-assignment>
    manual_slot_assignment: Option<Dom<HTMLSlotElement>>,
}

impl HTMLSlotElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLSlotElement {
        HTMLSlotElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            assigned_nodes: Default::default(),
            manually_assigned_nodes: Default::default(),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLSlotElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLSlotElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
            can_gc,
        )
    }

    /// <https://dom.spec.whatwg.org/#find-flattened-slotables>
    fn find_flattened_slottables(&self, result: &mut RootedVec<Slottable>) {
        // Step 1. Let result be an empty list.
        debug_assert!(result.is_empty());

        // Step 2. If slot’s root is not a shadow root, then return result.
        if self.upcast::<Node>().containing_shadow_root().is_none() {
            return;
        };

        // Step 3. Let slottables be the result of finding slottables given slot.
        rooted_vec!(let mut slottables);
        self.find_slottables(&mut slottables);

        // Step 4. If slottables is the empty list, then append each slottable child of slot, in tree order, to slottables.
        if slottables.is_empty() {
            for child in self.upcast::<Node>().children() {
                if let Some(element) = child.downcast::<Element>() {
                    slottables.push(Slottable::Element(Dom::from_ref(element)));
                } else if let Some(text) = child.downcast::<Text>() {
                    slottables.push(Slottable::Text(Dom::from_ref(text)));
                }
            }
        }

        // Step 5. For each node in slottables:
        for slottable in slottables.iter() {
            // Step 5.1 If node is a slot whose root is a shadow root:
            // NOTE: Only elements can be slots
            let maybe_slot_element = match &slottable {
                Slottable::Element(element) => element.downcast::<HTMLSlotElement>(),
                Slottable::Text(_) => None,
            };
            match maybe_slot_element {
                Some(slot_element)
                    if slot_element
                        .upcast::<Node>()
                        .containing_shadow_root()
                        .is_some() =>
                {
                    // Step 5.1.1 Let temporaryResult be the result of finding flattened slottables given node.
                    rooted_vec!(let mut temporary_result);
                    slot_element.find_flattened_slottables(&mut temporary_result);

                    // Step 5.1.2 Append each slottable in temporaryResult, in order, to result.
                    result.extend_from_slice(&temporary_result);
                },
                // Step 5.2 Otherwise, append node to result.
                _ => {
                    result.push(slottable.clone());
                },
            };
        }

        // Step 6. Return result.
    }

    /// <https://dom.spec.whatwg.org/#find-slotables>
    ///
    /// To avoid rooting shenanigans, this writes the returned slottables
    /// into the `result` argument
    fn find_slottables(&self, result: &mut RootedVec<Slottable>) {
        let cx = GlobalScope::get_cx();

        // Step 1. Let result be an empty list.
        debug_assert!(result.is_empty());

        // Step 2. Let root be slot’s root.
        // Step 3. If root is not a shadow root, then return result.
        let Some(root) = self.upcast::<Node>().containing_shadow_root() else {
            return;
        };

        // Step 4. Let host be root’s host.
        let host = root.Host();

        // Step 5. If root’s slot assignment is "manual":
        if root.SlotAssignment() == SlotAssignmentMode::Manual {
            // Step 5.1 Let result be « ».
            // NOTE: redundant.

            // Step 5.2 For each slottable slottable of slot’s manually assigned nodes,
            // if slottable’s parent is host, append slottable to result.
            for slottable in self.manually_assigned_nodes.borrow().iter() {
                if slottable
                    .node()
                    .GetParentNode()
                    .is_some_and(|node| &*node == host.upcast::<Node>())
                {
                    result.push(slottable.clone());
                }
            }
        }
        // Step 6. Otherwise, for each slottable child slottable of host, in tree order:
        else {
            let mut for_slottable = |slottable: RootedGuard<Slottable>| {
                // Step 6.1 Let foundSlot be the result of finding a slot given slottable.
                let found_slot = slottable.find_a_slot(false);

                // Step 6.2 If foundSlot is slot, then append slottable to result.
                if found_slot.is_some_and(|found_slot| &*found_slot == self) {
                    result.push(slottable.clone());
                }
            };
            for child in host.upcast::<Node>().children() {
                if let Some(element) = child.downcast::<Element>() {
                    rooted!(in(*cx) let slottable = Slottable::Element(Dom::from_ref(element)));
                    for_slottable(slottable);
                    continue;
                }
                if let Some(text) = child.downcast::<Text>() {
                    rooted!(in(*cx) let slottable = Slottable::Text(Dom::from_ref(text)));
                    for_slottable(slottable);
                }
            }
        }

        // Step 7. Return result.
    }

    /// <https://dom.spec.whatwg.org/#assign-slotables>
    pub(crate) fn assign_slottables(&self) {
        // Step 1. Let slottables be the result of finding slottables for slot.
        rooted_vec!(let mut slottables);
        self.find_slottables(&mut slottables);

        // Step 2. TODO If slottables and slot’s assigned nodes are not identical, then run signal a slot change for slot.

        // Step 3. Set slot’s assigned nodes to slottables.
        *self.assigned_nodes.borrow_mut() = slottables.iter().cloned().collect();

        // Step 4. For each slottable in slottables, set slottable’s assigned slot to slot.
        for slottable in slottables.iter() {
            slottable.data().borrow_mut().assigned_slot = Some(Dom::from_ref(self));
        }
    }
}

impl Slottable {
    /// <https://dom.spec.whatwg.org/#find-a-slot>
    pub fn find_a_slot(&self, open_flag: bool) -> Option<DomRoot<HTMLSlotElement>> {
        // Step 1. If slottable’s parent is null, then return null.
        let parent = self.node().GetParentNode()?;

        // Step 2. Let shadow be slottable’s parent’s shadow root.
        // Step 3. If shadow is null, then return null.
        let shadow_root = parent
            .downcast::<Element>()
            .and_then(Element::shadow_root)?;

        // Step 4. If the open flag is set and shadow’s mode is not "open", then return null.
        if open_flag && shadow_root.Mode() != ShadowRootMode::Open {
            return None;
        }

        // Step 5. If shadow’s slot assignment is "manual", then return the slot in shadow’s descendants whose
        // manually assigned nodes contains slottable, if any; otherwise null.
        if shadow_root.SlotAssignment() == SlotAssignmentMode::Manual {
            for node in shadow_root
                .upcast::<Node>()
                .traverse_preorder(ShadowIncluding::No)
            {
                if let Some(slot) = node.downcast::<HTMLSlotElement>() {
                    if slot.manually_assigned_nodes.borrow().contains(self) {
                        return Some(DomRoot::from_ref(slot));
                    }
                }
            }
            return None;
        }

        // Step 6. Return the first slot in tree order in shadow’s descendants whose name is slottable’s name, if any; otherwise null.
        for node in shadow_root
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::No)
        {
            if let Some(slot) = node.downcast::<HTMLSlotElement>() {
                if slot.Name() == self.name() {
                    return Some(DomRoot::from_ref(slot));
                }
            }
        }
        None
    }

    /// Slottable name change steps from https://dom.spec.whatwg.org/#light-tree-slotables
    pub(crate) fn update_slot_name(&self, attr: &Attr, mutation: AttributeMutation) {
        debug_assert!(matches!(self, Self::Element(_)));

        // Step 1. If localName is slot and namespace is null:
        // NOTE: This is done by the caller
        let old_value = if let AttributeMutation::Set(old_name) = mutation {
            old_name.and_then(|attr| match &*attr {
                AttrValue::String(s) => Some(s.clone()),
                _ => None,
            })
        } else {
            None
        };
        let value = mutation.new_value(attr).and_then(|attr| match &*attr {
            AttrValue::String(s) => Some(s.clone()),
            _ => None,
        });

        // Step 1.1 If value is oldValue, then return.
        if value == old_value {
            return;
        }

        // Step 1.2 If value is null and oldValue is the empty string, then return.
        if value.is_none() && old_value.as_ref().is_some_and(|s| s.is_empty()) {
            return;
        }

        // Step 1.3 If value is the empty string and oldValue is null, then return.
        if old_value.is_none() && value.as_ref().is_some_and(|s| s.is_empty()) {
            return;
        }

        // Step 1.4 If value is null or the empty string, then set element’s name to the empty string.
        if !value.as_ref().is_some_and(|s| !s.is_empty()) {
            self.set_name(DOMString::new());
        }
        // Step 1.5 Otherwise, set element’s name to value.
        else {
            self.set_name(DOMString::from(value.unwrap_or_default()));
        }

        // Step 1.6 If element is assigned, then run assign slottables for element’s assigned slot.
        if let Some(assigned_slot) = self
            .data()
            .borrow()
            .assigned_slot
            .as_ref()
            .map(|slot| slot.as_rooted())
        {
            assigned_slot.assign_slottables();
        }

        // Step 1.7 Run assign a slot for element.
        self.assign_a_slot();
    }

    /// <https://dom.spec.whatwg.org/#assign-a-slot>
    pub(crate) fn assign_a_slot(&self) {
        // Step 1. Let slot be the result of finding a slot with slottable.
        let slot = self.find_a_slot(false);

        // Step 2. If slot is non-null, then run assign slottables for slot.
        if let Some(slot) = slot {
            slot.assign_slottables();
        }
    }

    fn node(&self) -> &Node {
        match self {
            Self::Element(element) => element.upcast(),
            Self::Text(text) => text.upcast(),
        }
    }

    fn data(&self) -> &RefCell<SlottableData> {
        match self {
            Self::Element(element) => element.slottable_data(),
            Self::Text(text) => text.slottable_data(),
        }
    }

    fn set_name(&self, name: DOMString) {
        // NOTE: Only elements have non-empty names
        let Self::Element(element) = self else {
            return;
        };
        *element.slottable_name().borrow_mut() = name;
    }

    fn name(&self) -> DOMString {
        // NOTE: Only elements have non-empty names
        let Self::Element(element) = self else {
            return DOMString::new();
        };

        element.slottable_name().borrow().clone()
    }
}

impl js::gc::Rootable for Slottable {}

impl js::gc::Initialize for Slottable {
    #[allow(unsafe_code)]
    unsafe fn initial() -> Option<Self> {
        None
    }
}
