/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref, RefCell};

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::gc::RootedVec;
use js::rust::HandleObject;
use script_bindings::codegen::InheritTypes::{CharacterDataTypeId, NodeTypeId};

use crate::ScriptThread;
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
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::node::{BindContext, Node, NodeDamage, NodeTraits, ShadowIncluding, UnbindContext};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

/// <https://html.spec.whatwg.org/multipage/#the-slot-element>
#[dom_struct]
pub(crate) struct HTMLSlotElement {
    htmlelement: HTMLElement,

    /// <https://dom.spec.whatwg.org/#slot-assigned-nodes>
    assigned_nodes: RefCell<Vec<Slottable>>,

    /// <https://html.spec.whatwg.org/multipage/#manually-assigned-nodes>
    manually_assigned_nodes: RefCell<Vec<Slottable>>,

    /// Whether there is a queued signal change for this element
    ///
    /// Necessary to avoid triggering too many slotchange events
    is_in_agents_signal_slots: Cell<bool>,
}

impl HTMLSlotElementMethods<crate::DomTypeHolder> for HTMLSlotElement {
    // https://html.spec.whatwg.org/multipage/#dom-slot-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-slot-name
    make_atomic_setter!(SetName, "name");

    /// <https://html.spec.whatwg.org/multipage/#dom-slot-assignednodes>
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

    /// <https://html.spec.whatwg.org/multipage/#dom-slot-assignedelements>
    fn AssignedElements(&self, options: &AssignedNodesOptions) -> Vec<DomRoot<Element>> {
        self.AssignedNodes(options)
            .into_iter()
            .flat_map(|node| node.downcast::<Element>().map(DomRoot::from_ref))
            .collect()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-slot-assign>
    fn Assign(&self, nodes: Vec<ElementOrText>) {
        let cx = GlobalScope::get_cx();

        // Step 1. For each node of this's manually assigned nodes, set node's manual slot assignment to null.
        for slottable in self.manually_assigned_nodes.borrow().iter() {
            slottable.set_manual_slot_assignment(None);
        }

        // Step 2. Let nodesSet be a new ordered set.
        rooted_vec!(let mut nodes_set);

        // Step 3. For each node of nodes:
        for element_or_text in nodes.into_iter() {
            rooted!(in(*cx) let node = match element_or_text {
                ElementOrText::Element(element) => Slottable(Dom::from_ref(element.upcast())),
                ElementOrText::Text(text) => Slottable(Dom::from_ref(text.upcast())),
            });

            // Step 3.1 If node's manual slot assignment refers to a slot,
            // then remove node from that slot's manually assigned nodes.
            if let Some(slot) = node.manual_slot_assignment() {
                let mut manually_assigned_nodes = slot.manually_assigned_nodes.borrow_mut();
                if let Some(position) = manually_assigned_nodes
                    .iter()
                    .position(|value| *value == *node)
                {
                    manually_assigned_nodes.remove(position);
                }
            }

            // Step 3.2 Set node's manual slot assignment to this.
            node.set_manual_slot_assignment(Some(self));

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
///
/// The contained node is assumed to be either `Element` or `Text`
///
/// This field is public to make it easy to construct slottables.
/// As such, it is possible to put Nodes that are not slottables
/// in there. Using a [Slottable] like this will quickly lead to
/// a panic.
#[derive(Clone, JSTraceable, MallocSizeOf, PartialEq)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[repr(transparent)]
pub(crate) struct Slottable(pub Dom<Node>);
/// Data shared between all [slottables](https://dom.spec.whatwg.org/#concept-slotable)
///
/// Note that the [slottable name](https://dom.spec.whatwg.org/#slotable-name) is not
/// part of this. While the spec says that all slottables have a name, only Element's
/// can ever have a non-empty name, so they store it seperately
#[derive(Default, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub struct SlottableData {
    /// <https://dom.spec.whatwg.org/#slotable-assigned-slot>
    pub(crate) assigned_slot: Option<Dom<HTMLSlotElement>>,

    /// <https://dom.spec.whatwg.org/#slottable-manual-slot-assignment>
    pub(crate) manual_slot_assignment: Option<Dom<HTMLSlotElement>>,
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
            is_in_agents_signal_slots: Default::default(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
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

    pub(crate) fn has_assigned_nodes(&self) -> bool {
        !self.assigned_nodes.borrow().is_empty()
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

        // Step 4. If slottables is the empty list, then append each slottable
        // child of slot, in tree order, to slottables.
        if slottables.is_empty() {
            for child in self.upcast::<Node>().children() {
                let is_slottable = matches!(
                    child.type_id(),
                    NodeTypeId::Element(_) |
                        NodeTypeId::CharacterData(CharacterDataTypeId::Text(_))
                );
                if is_slottable {
                    slottables.push(Slottable(child.as_traced()));
                }
            }
        }

        // Step 5. For each node in slottables:
        for slottable in slottables.iter() {
            // Step 5.1 If node is a slot whose root is a shadow root:
            match slottable.0.downcast::<HTMLSlotElement>() {
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
            for child in host.upcast::<Node>().children() {
                let is_slottable = matches!(
                    child.type_id(),
                    NodeTypeId::Element(_) |
                        NodeTypeId::CharacterData(CharacterDataTypeId::Text(_))
                );
                if is_slottable {
                    rooted!(in(*cx) let slottable = Slottable(child.as_traced()));
                    // Step 6.1 Let foundSlot be the result of finding a slot given slottable.
                    let found_slot = slottable.find_a_slot(false);

                    // Step 6.2 If foundSlot is slot, then append slottable to result.
                    if found_slot.is_some_and(|found_slot| &*found_slot == self) {
                        result.push(slottable.clone());
                    }
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

        // Step 2. If slottables and slot’s assigned nodes are not identical,
        // then run signal a slot change for slot.
        let slots_are_identical = self.assigned_nodes.borrow().iter().eq(slottables.iter());
        if !slots_are_identical {
            self.signal_a_slot_change();
        }

        // NOTE: This is not written in the spec, which is likely a bug (https://github.com/whatwg/dom/issues/1352)
        // If we don't disconnect the old slottables from this slot then they'll stay implictly
        // connected, which causes problems later on
        for slottable in self.assigned_nodes().iter() {
            slottable.set_assigned_slot(None);
        }

        // Step 3. Set slot’s assigned nodes to slottables.
        *self.assigned_nodes.borrow_mut() = slottables.iter().cloned().collect();

        // Step 4. For each slottable in slottables, set slottable’s assigned slot to slot.
        for slottable in slottables.iter() {
            slottable.set_assigned_slot(Some(self));
        }
    }

    /// <https://dom.spec.whatwg.org/#signal-a-slot-change>
    pub(crate) fn signal_a_slot_change(&self) {
        self.upcast::<Node>().dirty(NodeDamage::ContentOrHeritage);

        if self.is_in_agents_signal_slots.get() {
            return;
        }
        self.is_in_agents_signal_slots.set(true);

        // Step 1. Append slot to slot’s relevant agent’s signal slots.
        ScriptThread::add_signal_slot(self);

        // Step 2. Queue a mutation observer microtask.
        let mutation_observers = ScriptThread::mutation_observers();
        mutation_observers.queue_mutation_observer_microtask(ScriptThread::microtask_queue());
    }

    pub(crate) fn remove_from_signal_slots(&self) {
        debug_assert!(self.is_in_agents_signal_slots.get());
        self.is_in_agents_signal_slots.set(false);
    }

    /// Returns the slot's assigned nodes if the root's slot assignment mode
    /// is "named", or the manually assigned nodes otherwise
    pub(crate) fn assigned_nodes(&self) -> Ref<'_, [Slottable]> {
        Ref::map(self.assigned_nodes.borrow(), Vec::as_slice)
    }
}

impl Slottable {
    /// <https://dom.spec.whatwg.org/#find-a-slot>
    pub(crate) fn find_a_slot(&self, open_flag: bool) -> Option<DomRoot<HTMLSlotElement>> {
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

        // Step 6. Return the first slot in tree order in shadow’s descendants whose
        // name is slottable’s name, if any; otherwise null.
        shadow_root.slot_for_name(&self.name())
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
        &self.0
    }

    pub(crate) fn assigned_slot(&self) -> Option<DomRoot<HTMLSlotElement>> {
        self.node().assigned_slot()
    }

    pub(crate) fn set_assigned_slot(&self, assigned_slot: Option<&HTMLSlotElement>) {
        self.node().set_assigned_slot(assigned_slot);
    }

    pub(crate) fn set_manual_slot_assignment(
        &self,
        manually_assigned_slot: Option<&HTMLSlotElement>,
    ) {
        self.node()
            .set_manual_slot_assignment(manually_assigned_slot);
    }

    pub(crate) fn manual_slot_assignment(&self) -> Option<DomRoot<HTMLSlotElement>> {
        self.node().manual_slot_assignment()
    }

    fn name(&self) -> DOMString {
        // NOTE: Only elements have non-empty names
        let Some(element) = self.0.downcast::<Element>() else {
            return DOMString::new();
        };

        element.get_string_attribute(&local_name!("slot"))
    }
}

impl VirtualMethods for HTMLSlotElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    /// <https://dom.spec.whatwg.org/#shadow-tree-slots>
    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);

        if attr.local_name() == &local_name!("name") && attr.namespace() == &ns!() {
            if let Some(shadow_root) = self.containing_shadow_root() {
                // Shadow roots keep a list of slot descendants, so we need to tell it
                // about our name change
                let old_value = match mutation {
                    AttributeMutation::Set(old) => old
                        .map(|value| value.to_string().into())
                        .unwrap_or_default(),
                    AttributeMutation::Removed => attr.value().to_string().into(),
                };

                shadow_root.unregister_slot(old_value, self);
                shadow_root.register_slot(self);
            }

            // Changing the name might cause slot assignments to change
            self.upcast::<Node>()
                .GetRootNode(&GetRootNodeOptions::empty())
                .assign_slottables_for_a_tree()
        }
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context, can_gc);
        }

        if !context.tree_is_in_a_shadow_tree {
            return;
        }

        self.containing_shadow_root()
            .expect("not in a shadow tree")
            .register_slot(self);
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.unbind_from_tree(context, can_gc);
        }

        if let Some(shadow_root) = self.containing_shadow_root() {
            shadow_root.unregister_slot(self.Name(), self);
        }
    }
}

impl js::gc::Rootable for Slottable {}

impl js::gc::Initialize for Slottable {
    #[allow(unsafe_code)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    unsafe fn initial() -> Option<Self> {
        None
    }
}
