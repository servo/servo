/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::iter::repeat;
use std::sync::atomic::AtomicU64;
use std::sync::{LazyLock, atomic};

use accesskit::{NodeId, Role};
use app_units::Au;
use bitflags::bitflags;
use euclid::Rect;
use layout_api::{AccessibilityDamage, BoxAreaType, LayoutElement, LayoutNode, LayoutNodeType};
use log::trace;
use rustc_hash::{FxHashMap, FxHashSet};
use script::layout_dom::{ServoLayoutElement, ServoLayoutNode};
use servo_base::Epoch;
use servo_base::print_tree::PrintTree;
use servo_config::opts::{self, DiagnosticsLogging, DiagnosticsLoggingOption};
use servo_config::pref;
use style::Atom;
use style::dom::OpaqueNode;
use style_traits::CSSPixel;
use web_atoms::{LocalName, local_name, ns};

use crate::ArcRefCell;
use crate::cell::WeakRefCell;
use crate::display_list::StackingContextTree;
use crate::layout_impl::LayoutThread;
use crate::query::process_box_area_request;

bitflags! {
    /// Damage which was caused by changes to the accessibility tree. These changes can cause other
    /// properties to need to be re-computed based on the updated values, either on the same node or
    /// on other nodes.
    #[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
    struct LocalAccessibilityDamage: u16 {
        /// This node's children changed, and/or any node in its subtree changed.
        const SubtreeChanged = 0b0001;
        /// This node's computed role changed.
        const RoleChanged = 0b0010;
        /// This node's computed label or text value (for a text node) changed.
        const TextChanged = 0b0100;
    }
}

/// Everything the accessibility tree needs from layout in order to compute node bounds during an
/// update.
pub(super) struct AccessibilityContext<'update> {
    pub(super) layout_thread: &'update LayoutThread,
    pub(super) stacking_context_tree: &'update StackingContextTree,
}

/// All the [`AccessibilityDamage`] which comes from outside the accessibility tree itself.
pub(super) type AccessibilityDamageMap<'a> =
    FxHashMap<OpaqueNode, (ServoLayoutNode<'a>, AccessibilityDamage)>;

/// Convert a rectangle as layout reports it into the one [`accesskit`] wants.
fn au_rect_to_accesskit_rect(rect: Rect<Au, CSSPixel>) -> accesskit::Rect {
    accesskit::Rect::new(
        rect.min_x().to_f64_px(),
        rect.min_y().to_f64_px(),
        rect.max_x().to_f64_px(),
        rect.max_y().to_f64_px(),
    )
}

/// Changes which have occurred during the current update, and data required to process the update.
struct AccessibilityUpdate<'update> {
    /// Nodes whose internal data has changed within the current update.
    changed_nodes: FxHashSet<NodeId>,
    /// Nodes that changed their relation to the tree within the current update.
    tree_changes: FxHashMap<NodeId, TreeChange>,
    /// Counters to track how many nodes we've checked for changes or updated in this tree update.
    counters: UpdateCounters,

    /// Map of [`NodeId`] to the [`AccessibilityDamage`] which was passed in for that node.
    damage_map: FxHashMap<NodeId, AccessibilityDamage>,
    /// Map of [`NodeId`] to the corresponding [`ServoLayoutNode`]. This is populated for nodes
    /// which have damage, including nodes which are newly added to the accessibility tree.
    dom_node_map: RefCell<FxHashMap<NodeId, ServoLayoutNode<'update>>>,

    /// Nodes which were removed from the DOM tree since the last reflow, which were rooted in
    /// `AccessibilityData`. Only set if `pref::expensive_accessibility_test_assertions_enabled`
    /// is set.
    rooted_nodes: Option<FxHashSet<OpaqueNode>>,
}

#[derive(Debug, Default)]
pub struct UpdateCounters {
    pub nodes_updated_from_dom: u32,
    pub nodes_updated_from_tree: u32,
    pub nodes_updated_bounds: u32,
    pub nodes_in_tree_update: u32,
}

bitflags! {
    /// Flags tracking an [`AccessibilityNode`]'s dirty state during an update. All flags which are
    /// set during the update should be unset by the end of the update.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct DirtyState : u16 {
        /// At least one descendant of this node has unresolved damage from the DOM tree.
        const DescendantHasDamage = 0b0001;
        /// This node has unresolved damage from the DOM tree.
        const HasDamage = 0b0010;
        /// This node's data changed, but it hasn't yet been added to the [`AccessibilityUpdate`].
        const Updated = 0b0100;
    }
}

struct AccessibilityNode {
    /// The unique ID for the node. This is used both as a key in [`AccessibilityTree`]'s cache of
    /// nodes, and as an identifier in [`accesskit`] datastructures: [`accesskit::Node`]s,
    /// [`accesskit::TreeUpdate`]s and [`accesskit::ActionRequest`]s.
    id: NodeId,
    /// The computed [`accesskit::Node`] data. This will be copied and serialized into a
    /// [`accesskit::TreeUpdate`] whenever it is changed during an update.
    accesskit_node: accesskit::Node,
    /// This node's parent, if any.
    parent_node: Option<WeakRefCell<AccessibilityNode>>,
    /// All this node's children.
    child_nodes: Vec<ArcRefCell<AccessibilityNode>>,
    /// The [`OpaqueNode`] for the DOM node which corresponds to this accessibility node, if any.
    /// An accessibility node may not correspond to a DOM node if it corresponds to a
    /// pseudo-element, or in a test.
    opaque_node: Option<OpaqueNode>,
    /// Any dirty state for the current update.
    dirty_state: DirtyState,
}

/// A retained, internal representation of the accessibility tree for a document.
///
/// [`accesskit`] only provides interchange types for tree updates and action requests, so we need
/// to define our own representation for incremental tree building.
#[derive(Debug)]
pub struct AccessibilityTree {
    /// All nodes currently in the tree as of the most recent update. New nodes are added and stale
    /// nodes are pruned during [`AccessibilityTree::update_tree()`].
    nodes: FxHashMap<NodeId, ArcRefCell<AccessibilityNode>>,
    /// A map to allow retrieving the [`AccessibilityNode`] which corresponds to a particular DOM
    /// node, if any.
    ///
    /// This must be kept in sync with [`Self::id_to_opaque_node`].
    opaque_node_to_id: FxHashMap<OpaqueNode, NodeId>,
    /// A map to retrieve the `OpaqueNode` corresponding to a particular [`AccessibilityNode`], if
    /// any.
    ///
    /// This must be kept in sync with [`Self::opaque_node_to_id`].
    id_to_opaque_node: FxHashMap<NodeId, OpaqueNode>,
    /// Sent with each [`accesskit::TreeUpdate`]. This allows this tree to be
    /// [grafted](https://docs.rs/accesskit/latest/accesskit/struct.Node.html#method.tree_id) into
    /// an application's tree.
    tree_id: accesskit::TreeId,
    /// This node's ID is sent with each [`accesskit::TreeUpdate`] to identify the root node.
    /// Also used for any complete tree walk, such as in [`Self::assert_integrity()`] and
    /// [`Self::print()`].
    root_node: Option<ArcRefCell<AccessibilityNode>>,
    /// Sent to the embedder alongside each [`accesskit::TreeUpdate`], so that the embedder can
    /// drop updates from documents which have been navigated away from.
    embedder_epoch: Epoch,
    /// Debug options, copied from configuration to this `AccessibilityTree` in order
    /// to avoid having to constantly access the thread-safe global options.
    debug: DiagnosticsLogging,
}

/// Tracks changes to a node's relation to the tree within an update.
///
/// This is used to remove nodes from the accessibility tree's cache when they are no longer in the
/// tree.
#[derive(Debug, PartialEq, Copy, Clone)]
enum TreeChange {
    /// The node was newly created in this update.
    New,

    /// The node has been re-parented in this update.
    Moved,

    /// The node has been added to its new parent, but not yet removed from its old
    /// parent.
    ///
    /// When a node is moved within the tree, it must be both removed from its old parent
    /// and added to its new parent within the same update. This may happen in either
    /// order, depending on the relative positions of the node before and after it moves.
    ///
    /// - If a node's new parent is updated before its old parent, the node will be in a
    ///   `TreeChange::PendingMove` state until its old parent is updated. We expect that it
    ///   must later be removed from its old parent, at which point its state will be updated to
    ///   `TreeChange::Moved`.
    /// - If a node's old parent is updated before its new parent, the node will be first
    ///   `TreeChange::Removed` and then `TreeChange::Moved`.
    ///
    /// At the end of the update, we assert that there are no pending moves remaining.
    PendingMove,

    /// The node is no longer a child of its previous parent.
    Removed,
}

impl AccessibilityTree {
    /// See [`Self::tree_id`] and [`Self::embedder_epoch`] for explanations of the parameters.
    pub(super) fn new(tree_id: accesskit::TreeId, embedder_epoch: Epoch) -> Self {
        Self {
            nodes: FxHashMap::default(),
            opaque_node_to_id: FxHashMap::default(),
            id_to_opaque_node: FxHashMap::default(),
            tree_id,
            root_node: None,
            embedder_epoch,
            debug: opts::get().debug.clone(),
        }
    }

    /// Update this tree based on the current state of the given DOM tree, and if anything changed,
    /// return an [`accesskit::TreeUpdate`] representing what changed.
    pub(super) fn update_tree<'update>(
        &mut self,
        root_dom_node: &ServoLayoutNode<'update>,
        damage_from_dom: AccessibilityDamageMap<'update>,
        context: AccessibilityContext<'update>,
        rooted_nodes: Option<FxHashSet<OpaqueNode>>,
    ) -> (Option<accesskit::TreeUpdate>, UpdateCounters) {
        let mut update = AccessibilityUpdate::new(damage_from_dom, rooted_nodes, self);

        self.ensure_root_node(root_dom_node, &mut update);

        self.apply_changes_from_dom_tree(&context, &mut update);

        update.finalize(self)
    }

    /// Get the node corresponding to the root DOM node, and set it as this tree's root. If the root
    /// node is newly created, which probably means this accessibility tree is newly created, append
    /// an `AccessibilityDamage::Rebuild` value for it to `damage_from_dom`.
    fn ensure_root_node<'update>(
        &mut self,
        root_dom_node: &ServoLayoutNode<'update>,
        update: &mut AccessibilityUpdate<'update>,
    ) {
        let (root_id, root_node) = self.get_or_create_node(root_dom_node, update);
        if update.is_new(&root_id) {
            // We're going to rebuild the whole tree, so ignore any incoming damage.
            update.clear_damage();
            update.insert_damage(root_id, AccessibilityDamage::Rebuild);
            update.insert_dom_node(root_id, *root_dom_node);
        } else {
            // TODO(#47162, #47161) This hack is necessary because we don't collect accessibility damage
            // from layout, and we don't handle scrolling properly.
            update.insert_damage(root_id, AccessibilityDamage::Subtree);
            update.insert_dom_node(root_id, *root_dom_node);
        }

        self.root_node = Some(root_node);
    }

    /// Update all nodes with damage tracked in `update` based on their `AccessibilityDamage`. If
    /// any [`LocalAccessibilityDamage`] results from the update, propagate
    /// [`LocalAccessibilityDamage::SubtreeChanged`] to its ancestors.
    fn apply_changes_from_dom_tree(
        &mut self,
        context: &AccessibilityContext,
        update: &mut AccessibilityUpdate,
    ) {
        let Some(damage_root_id) = self.mark_nodes_and_ancestors_dirty(update) else {
            return;
        };
        let damage_root = self.assert_node_for_id(&damage_root_id);
        let local_damage =
            damage_root
                .borrow_mut()
                .update_subtree(damage_root.clone(), context, self, update);

        damage_root.borrow().update_ancestors(local_damage, update);
    }

    /// Given an iterator of `NodeId`s corresponding to nodes which have received some damage from
    /// the DOM:
    /// - mark each node as `dirty`;
    /// - mark all of each node's ancestors as `has_dirty_descendants`;
    /// - return the lowest common ancestor node of all the damaged nodes.
    fn mark_nodes_and_ancestors_dirty(
        &mut self,
        update: &mut AccessibilityUpdate,
    ) -> Option<NodeId> {
        let mut dirty_node_ids = update.damage_map.keys();

        // An ordered list of common ancestors for the nodes seen so far, from shallowest to
        // deepest. At the end of the loop, the lowest common ancestor is the last node in this vec.
        let mut common_ancestors: Vec<NodeId> = Vec::new();

        {
            // Initialize the list of potential common ancestors.
            let node_id = dirty_node_ids.next()?;
            update.collect_dom_node_ancestors(node_id, self);
            let first_node = self.assert_node_for_id(node_id);
            let mut first_node = first_node.borrow_mut();
            first_node.dirty_state |= DirtyState::HasDamage;
            common_ancestors.push(first_node.id);
            common_ancestors.extend(first_node.ancestors().map(|ancestor| {
                let mut ancestor = ancestor.borrow_mut();
                ancestor.dirty_state |= DirtyState::DescendantHasDamage;
                ancestor.id
            }));
            common_ancestors.reverse();
        }

        let mut truncate_ancestors = |node: &AccessibilityNode| -> bool {
            if node.dirty_state.descendant_has_damage() {
                if let Some(pos) = common_ancestors.iter().position(|&id| id == node.id) {
                    common_ancestors.truncate(pos + 1);
                }
                return true;
            }
            false
        };

        for node_id in dirty_node_ids {
            let node = self.assert_node_for_id(node_id);
            let mut node = node.borrow_mut();
            node.dirty_state |= DirtyState::HasDamage;

            if truncate_ancestors(&node) {
                continue;
            }

            for ancestor in node.ancestors() {
                let mut ancestor = ancestor.borrow_mut();

                // If we find an ancestor we've already seen, discard any potential ancestors deeper
                // than this one, and go on to the next dirty node.
                if truncate_ancestors(&ancestor) {
                    break;
                }

                ancestor.dirty_state |= DirtyState::DescendantHasDamage;
            }
        }

        common_ancestors.pop()
    }

    /// Get the [`AccessibilityNode`] corresponding to the given DOM node.
    /// If there is no existing [`AccessibilityNode`] for this DOM node, it will be created and
    /// marked as having [`AccessibilityDamage::Rebuild`] in `update`.
    fn get_or_create_node(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
        update: &mut AccessibilityUpdate,
    ) -> (NodeId, ArcRefCell<AccessibilityNode>) {
        let id = self.get_or_create_id_for_opaque(dom_node.opaque());
        let node_ref = self.get_or_create_node_with_id(id, update);

        if update.is_new(&id) {
            let mut node = node_ref.borrow_mut();
            node.opaque_node = Some(dom_node.opaque());
            if let Some(dom_element) = dom_node.as_element() {
                let local_name = dom_element.local_name().to_ascii_lowercase();
                node.set_html_tag(&local_name);
            }
            update.insert_damage(id, AccessibilityDamage::Rebuild);
            node.dirty_state |= DirtyState::HasDamage;
        }

        (id, node_ref)
    }

    fn get_or_create_node_with_id(
        &mut self,
        id: NodeId,
        update: &mut AccessibilityUpdate,
    ) -> ArcRefCell<AccessibilityNode> {
        if let Some(node) = self.nodes.get(&id) {
            return node.clone();
        }

        let node = ArcRefCell::new(AccessibilityNode::new(id));
        update.set_tree_state_change(id, TreeChange::New);
        self.nodes.insert(id, node.clone());

        node
    }

    fn node_for_id(&self, id: NodeId) -> Option<ArcRefCell<AccessibilityNode>> {
        self.nodes.get(&id).cloned()
    }

    fn assert_node_for_id(&self, id: &NodeId) -> ArcRefCell<AccessibilityNode> {
        let Some(node) = self.nodes.get(id) else {
            panic!("{id:?} does not exist in tree");
        };
        node.clone()
    }

    /// Consume the [`AccessibilityUpdate`] by deleting all nodes it detected as being removed from
    /// the tree.
    fn drop_removed_nodes(&mut self, mut update: AccessibilityUpdate) {
        let mut rooted_nodes = std::mem::take(&mut update.rooted_nodes);
        if let Some(rooted_nodes) = rooted_nodes.as_mut() {
            self.assert_removed_nodes_were_rooted(&update, rooted_nodes);
        }

        let mut ids_to_remove: Vec<_> = update
            .tree_changes
            .iter()
            .filter_map(|(id, change)| match change {
                TreeChange::Removed => Some(id),
                TreeChange::PendingMove => None,
                TreeChange::New => None,
                TreeChange::Moved => None,
            })
            .cloned()
            .collect();

        while let Some(id) = ids_to_remove.pop() {
            if update.tree_changes.get(&id) == Some(&TreeChange::PendingMove) {
                // Mark the move as completed by marking the node as removed from its old position.
                update.set_tree_state_change(id, TreeChange::Removed);

                // Since this node is actually moved, don't continue removing its subtree.
                continue;
            }

            if let Some(opaque_node) = self.id_to_opaque_node.remove(&id) {
                self.opaque_node_to_id.remove(&opaque_node);
            }
            let node = self.nodes.remove(&id).expect("Node {id:?} already removed");
            ids_to_remove.extend(node.borrow().child_ids());
        }

        update
            .tree_changes
            .drain()
            .for_each(|(id, change)| match change {
                TreeChange::PendingMove => unreachable!(
                    "Pending move found for node id {id:?} when draining tree state changes"
                ),
                TreeChange::Removed => (),
                TreeChange::New => (),
                TreeChange::Moved => (),
            });

        if let Some(rooted_nodes) = rooted_nodes {
            self.assert_remaining_rooted_nodes_not_in_tree(rooted_nodes);
        }

        if self
            .debug
            .is_enabled(DiagnosticsLoggingOption::AccessibilityTree)
        {
            self.print();
        }

        if pref!(expensive_accessibility_test_assertions_enabled) {
            self.assert_integrity();
        }
    }

    /// If we got `rooted_nodes` from the document's `AccessibilityData`, assert that every node we
    /// marked as `TreeChange::Removed` during this update was rooted.
    fn assert_removed_nodes_were_rooted(
        &mut self,
        update: &AccessibilityUpdate,
        rooted_nodes: &mut FxHashSet<OpaqueNode>,
    ) {
        debug_assert!(pref!(expensive_accessibility_test_assertions_enabled));
        for (id, change) in update.tree_changes.iter() {
            if change == &TreeChange::Removed {
                let Some(&opaque_node) = self.id_to_opaque_node.get(id) else {
                    panic!("No opaque node found for removed node: id {id:?}");
                };
                assert!(
                    rooted_nodes.remove(&opaque_node),
                    "Node removed from accessibility tree wasn't rooted: id {id:?}"
                );
            };
        }
    }

    /// If we got `rooted_nodes` from the document's `AccessibilityData`, assert that any nodes
    /// which were rooted but not marked as `TreeChange::Removed` are no longer in the tree after
    /// dropping all nodes which were removed from the tree. They may have been part of a subtree
    /// which was marked `TreeChange::Removed` on an ancestor node, or may have never made it into
    /// the accessibility tree to begin with.
    fn assert_remaining_rooted_nodes_not_in_tree(&self, rooted_nodes: FxHashSet<OpaqueNode>) {
        for leftover_node in rooted_nodes {
            assert!(
                !self.opaque_node_to_id.contains_key(&leftover_node),
                "Found node removed from DOM tree but not accessibility tree: {:#x}",
                leftover_node.0
            );
        }
    }

    fn get_or_create_id_for_opaque(&mut self, opaque: OpaqueNode) -> NodeId {
        let id = self.opaque_node_to_id.entry(opaque).or_insert_with(|| {
            static LAST_ID: AtomicU64 = AtomicU64::new(0);
            let id = LAST_ID.fetch_add(1, atomic::Ordering::SeqCst).into();
            self.id_to_opaque_node.insert(id, opaque);
            id
        });
        *id
    }

    fn existing_id_for_opaque(&self, opaque: OpaqueNode) -> Option<NodeId> {
        self.opaque_node_to_id.get(&opaque).cloned()
    }

    pub(crate) fn embedder_epoch(&self) -> Epoch {
        self.embedder_epoch
    }

    /// Assert that the tree is a tree without any dangling references or orphaned nodes.
    ///
    /// For accessibility tests only, because it’s expensive.
    fn assert_integrity(&self) {
        debug_assert!(pref!(expensive_accessibility_test_assertions_enabled));
        let Some(root_node) = self.root_node.clone() else {
            return;
        };

        // Traverse the tree from the given root.
        // `nodes` is a Vec of pairs of nodes and their expected parents.
        let mut nodes = vec![(root_node, None)];
        let mut seen_node_ids = FxHashSet::default();
        while let Some((node, expected_parent)) = nodes.pop() {
            let node = node.borrow();

            // If this fails, then the tree is not a tree at all.
            assert!(
                seen_node_ids.insert(node.id),
                "Tree contains {:?} in multiple places",
                node.id
            );

            node.assert_integrity(expected_parent);

            // assert_node_for_id() here double-checks that the node hasn't been incorrectly evicted
            // from the map while it's still retained as a child node.
            let weak_node = Some(self.assert_node_for_id(&node.id).downgrade());
            nodes.extend(node.children().cloned().zip(repeat(weak_node)));
        }

        // If this fails, then the tree has orphaned nodes (a leak).
        // If a node has been incorrectly removed from the map, that will be caught above.
        assert_eq!(seen_node_ids, self.nodes.keys().copied().collect());
    }

    fn print(&self) {
        let Some(root_node) = self.root_node.clone() else {
            return;
        };

        let mut print_tree = PrintTree::new("Accessibility Tree");
        root_node.borrow().print(&mut print_tree);
        print_tree.end_level();
    }
}

/// <https://w3c.github.io/aria/#host_general_role>
fn role_from_role_attribute(dom_element: &ServoLayoutElement<'_>) -> Option<Role> {
    let role_attribute = dom_element.attribute(&ns!(), &local_name!("role"))?;
    role_attribute
        .as_tokens()
        .iter()
        .filter_map(|role_name_in_attribute| SUPPORTED_ARIA_ROLES.get(role_name_in_attribute))
        .next()
        .cloned()
}

fn role_from_dom_node(dom_node: &ServoLayoutNode<'_>) -> Role {
    if let Some(dom_element) = dom_node.as_element() {
        role_from_role_attribute(&dom_element).unwrap_or_else(|| {
            let local_name = dom_element.local_name().to_ascii_lowercase();
            *HTML_ELEMENT_ROLE_MAPPINGS
                .get(&local_name)
                .unwrap_or(&Role::GenericContainer)
        })
    } else if dom_node.type_id() == Some(LayoutNodeType::Text) {
        Role::TextRun
    } else {
        Role::GenericContainer
    }
}

struct AccessibilityNodeIterator<I>
where
    I: Fn(&AccessibilityNode) -> Option<ArcRefCell<AccessibilityNode>>,
{
    next_value: Option<ArcRefCell<AccessibilityNode>>,
    next_fn: I,
}

impl<I> AccessibilityNodeIterator<I>
where
    I: Fn(&AccessibilityNode) -> Option<ArcRefCell<AccessibilityNode>>,
{
    fn new(next_value: Option<ArcRefCell<AccessibilityNode>>, next_fn: I) -> Self {
        AccessibilityNodeIterator {
            next_value,
            next_fn,
        }
    }
}

impl<I> Iterator for AccessibilityNodeIterator<I>
where
    I: Fn(&AccessibilityNode) -> Option<ArcRefCell<AccessibilityNode>>,
{
    type Item = ArcRefCell<AccessibilityNode>;

    fn next(&mut self) -> Option<Self::Item> {
        let next_value = self.next_value.take();
        self.next_value = next_value
            .as_ref()
            .and_then(|node| (self.next_fn)(&node.borrow()));
        next_value
    }
}

impl AccessibilityNode {
    fn new(id: NodeId) -> Self {
        Self::new_with_role(id, Role::Unknown)
    }

    fn new_with_role(id: NodeId, role: Role) -> Self {
        Self {
            id,
            accesskit_node: accesskit::Node::new(role),
            parent_node: None,
            child_nodes: vec![],
            opaque_node: None,
            dirty_state: DirtyState::empty(),
        }
    }

    /// Update this node and its subtree based on damage from the DOM.
    ///
    /// - First, if this node has damage from the DOM to be resolved, update the node from the DOM
    ///   tree, recursively populating any new children.
    /// - Next, recursively call this method for any children which are dirty, or have dirty
    ///   descendants.
    /// - Finally, update any properties on this node which are may have changed due to other
    ///   changes in the tree.
    ///
    /// At the end of this method, both `has_dirty_descendants` and `is_dirty` should be false for
    /// this node and all its descendants.
    fn update_subtree<'update>(
        &mut self,
        ref_self: ArcRefCell<Self>,
        context: &AccessibilityContext,
        tree: &mut AccessibilityTree,
        update: &mut AccessibilityUpdate<'update>,
    ) -> LocalAccessibilityDamage {
        let mut local_damage = LocalAccessibilityDamage::empty();

        let damage = self.compute_damage(update);

        if let Some(dom_node) = update.take_dom_node(&self.id) {
            // TODO(#47162, #47161): Once we handle scrolling properly and have a way of tracking
            // damage from layout, we won't need to update every node.
            local_damage.insert(self.update_properties_and_children_from_dom_node(
                ref_self, &dom_node, damage, tree, update,
            ));
            self.update_bounds_from_dom_node(&dom_node, context, update);
            self.dirty_state -= DirtyState::HasDamage;
        }

        for child_node in self.children() {
            let child_node_ref = child_node.clone();
            let mut child_node = child_node.borrow_mut();
            let child_local_damage =
                child_node.update_subtree(child_node_ref, context, tree, update);
            if !child_local_damage.is_empty() {
                local_damage.insert(LocalAccessibilityDamage::SubtreeChanged);
            }
        }
        self.dirty_state -= DirtyState::DescendantHasDamage;

        local_damage.insert(self.update_node_local(local_damage, update));

        if self.dirty_state.updated() {
            update.add(self);
        }

        local_damage
    }

    /// Update each of this node's ancestors based on changes which have already been applied in the
    /// tree.
    fn update_ancestors(
        &self,
        local_damage: LocalAccessibilityDamage,
        update: &mut AccessibilityUpdate,
    ) {
        if local_damage.is_empty() {
            return;
        }
        for node in self.ancestors() {
            let mut node = node.borrow_mut();
            node.update_node_local(LocalAccessibilityDamage::SubtreeChanged, update);
            node.dirty_state -= DirtyState::DescendantHasDamage;
            if node.dirty_state.updated() {
                update.add(&mut node);
            }
        }
    }

    /// Update the given [`AccessibilityNode`] from its corresponding DOM node and
    /// [`AccessibilityDamage`].
    /// If it has new children, those will be created here, but not yet populated.
    // Any changed nodes will be added to the given [`AccessibilityUpdate`].
    fn update_properties_and_children_from_dom_node<'update>(
        &mut self,
        ref_self: ArcRefCell<Self>,
        dom_node: &ServoLayoutNode<'update>,
        dom_damage: AccessibilityDamage,
        tree: &mut AccessibilityTree,
        update: &mut AccessibilityUpdate<'update>,
    ) -> LocalAccessibilityDamage {
        let mut local_damage = LocalAccessibilityDamage::empty();

        // TODO(#47162): We currently need to walk the children each time so that we always find the
        // DOM node for each accessibility node, since we update bounds on every node.
        // Once this is no longer true, we can check dom_damage and potentially early return here.

        update.counters.nodes_updated_from_dom += 1;

        local_damage.insert(self.update_properties_from_dom_node(dom_node, dom_damage));
        local_damage.insert(
            self.update_children_from_dom_node(ref_self, dom_node, dom_damage, tree, update),
        );

        local_damage
    }

    /// Update this node's [`Self::children`] from its corresponding DOM node.
    /// If it has new children, those will be created here, but not yet populated.
    fn update_children_from_dom_node<'update>(
        &mut self,
        ref_self: ArcRefCell<AccessibilityNode>,
        dom_node: &ServoLayoutNode<'update>,
        _dom_damage: AccessibilityDamage,
        tree: &mut AccessibilityTree,
        update: &mut AccessibilityUpdate<'update>,
    ) -> LocalAccessibilityDamage {
        // TODO(#47162): We currently need to walk the children each time so that we always find the
        // DOM node for each accessibility node, since we update bounds on every node.
        // Once this is no longer true, we can check _dom_damage and potentially early return here.

        let mut remaining_dom_children = dom_node.flat_tree_children().peekable();
        let mut old_child_ids = self.child_ids().iter().peekable();
        let mut unchanged_count = 0usize;

        // Iterate over existing children and DOM children while they match. No action is necessary
        // for these nodes.
        while let Some(&old_id) = old_child_ids.peek() &&
            let Some(dom_child) = remaining_dom_children.peek()
        {
            if tree.existing_id_for_opaque(dom_child.opaque()) == Some(*old_id) {
                update.insert_dom_node(*old_id, *dom_child);
                unchanged_count += 1;
                old_child_ids.next();
                remaining_dom_children.next();
            } else {
                break;
            }
        }

        // If we iterated over all the DOM children without finding any changes, we're done.
        if old_child_ids.peek().is_none() && remaining_dom_children.peek().is_none() {
            return LocalAccessibilityDamage::empty();
        }

        // Remove all child nodes after the first `unchanged_count`.
        self.child_nodes.truncate(unchanged_count);
        let mut new_child_ids = Vec::from(self.child_ids());
        for removed_child_id in new_child_ids.split_off(unchanged_count) {
            update.set_tree_state_change(removed_child_id, TreeChange::Removed);
        }

        // Then, (re-)add all the remaining DOM children. Note that this means that some children
        // may end up being "Moved" even though they haven't changed parents, and may even be in the
        // same position as previously.
        let weak_self = ref_self.downgrade();
        for dom_child in remaining_dom_children {
            let (child_id, child_ref) = tree.get_or_create_node(&dom_child, update);
            // TODO(#47162): Since we need to update bounds for all nodes, we need to ensure every
            // AccessibilityNode has a corresponding DOM node available to be retrieved from the
            // AccessibilityUpdate. Once we no longer update bounds on all nodes, we won't need to
            // add all nodes like this.
            update.insert_dom_node(child_id, dom_child);

            // Update self.child_nodes in place.
            self.child_nodes.push(child_ref.clone());
            new_child_ids.push(child_id);

            let mut child = child_ref.borrow_mut();
            child.parent_node = Some(weak_self.clone());

            if update.is_new(&child_id) {
                self.dirty_state |= DirtyState::DescendantHasDamage;
            } else {
                update.set_tree_state_change(child_id, TreeChange::PendingMove);
            }

            self.dirty_state
                .propagate_descendant_has_damage(child.dirty_state);
        }

        // We can't update the AccessKit node's `children` in place, so we build up the full list
        // and then set it here.
        self.accesskit_node.set_children(new_child_ids);
        self.dirty_state |= DirtyState::Updated;

        LocalAccessibilityDamage::SubtreeChanged
    }

    /// Update this node's properties from its corresponding DOM node.
    fn update_properties_from_dom_node(
        &mut self,
        dom_node: &ServoLayoutNode,
        dom_damage: AccessibilityDamage,
    ) -> LocalAccessibilityDamage {
        let mut local_damage = LocalAccessibilityDamage::empty();
        if !dom_damage.contains(AccessibilityDamage::Node) {
            return local_damage;
        }
        local_damage.insert(self.set_role(role_from_dom_node(dom_node)));
        if dom_node.type_id() == Some(LayoutNodeType::Text) {
            let text_content = dom_node.text_content();
            trace!("node text content = {text_content:?}");
            // FIXME: this should take into account editing selection units (grapheme clusters?)
            local_damage.insert(self.set_value(&text_content));
        }

        local_damage
    }

    /// Update this node's bounds from the current layout geometry.
    fn update_bounds_from_dom_node(
        &mut self,
        dom_node: &ServoLayoutNode,
        context: &AccessibilityContext,
        update: &mut AccessibilityUpdate,
    ) {
        update.counters.nodes_updated_bounds += 1;

        // Border box with transforms, matching getBoundingClientRect(). Bounds are in CSS pixels,
        // relative to the viewport origin; the embedder's graft node carries the transform that
        // composes them into AccessKit's coordinate space (see the "Coordinates" section of
        // <https://docs.rs/accesskit/latest/accesskit/struct.Node.html>).
        let bounds = process_box_area_request(
            context.layout_thread,
            context.stacking_context_tree,
            *dom_node,
            BoxAreaType::Border,
            false, /* exclude_transform_and_inline */
        )
        .map(au_rect_to_accesskit_rect);

        // For now only nodes with a box of their own get bounds; anything else, including
        // `display: none` content, gets its bounds cleared. That leaves two kinds of nodes
        // without geometry which assistive technology would like to have some:
        //
        // TODO(accessibility): A text node never has bounds of its own: `LayoutBox::Text` has no
        // `LayoutBoxBase`, and `Fragment::Text` has no box area, so the query above always returns
        // `None` for one. Text nodes should get the union of the rectangles of their own
        // `Fragment::Text` fragments, once `cumulative_box_area_rect()` can handle those. See
        // #47164.
        //
        // TODO(accessibility): A `display: contents` element generates no box either. Other
        // engines (Blink, WebKit, Gecko) compute its bounds as the union of the bounding boxes of
        // its rendered descendants. See #47163.
        match bounds {
            Some(bounds) => self.set_bounds(bounds),
            None => self.clear_bounds(),
        }
    }

    /// Update this node's properties based on changes already made to the accessibility tree.
    /// For example, if there were nodes added or removed in its subtree, its computed text may have
    /// changed, so that will be recomputed here.
    /// If any changes are made, add this node to the given [`AccessibilityUpdate`].
    fn update_node_local(
        &mut self,
        local_damage: LocalAccessibilityDamage,
        update: &mut AccessibilityUpdate,
    ) -> LocalAccessibilityDamage {
        let mut new_damage = LocalAccessibilityDamage::empty();
        if local_damage.is_empty() {
            return new_damage;
        }
        update.counters.nodes_updated_from_tree += 1;

        if local_damage.contains(LocalAccessibilityDamage::SubtreeChanged) ||
            local_damage.contains(LocalAccessibilityDamage::RoleChanged)
        {
            if let Some(text) = self.label_from_descendants() {
                new_damage.insert(self.set_label(text.as_str()));
            } else {
                new_damage.insert(self.clear_label());
            }
        }

        new_damage
    }

    fn label_from_descendants(&self) -> Option<String> {
        if !NAME_FROM_CONTENTS_ROLES.contains(&self.role()) {
            return None;
        }
        let mut children = VecDeque::from_iter(self.children().cloned());
        let mut text = String::new();
        while let Some(child) = children.pop_front() {
            let child = child.borrow();
            match child.role() {
                Role::TextRun => {
                    if let Some(child_text) = child.value() {
                        text.push_str(child_text);
                    }
                },
                _ => {
                    for node in child.children().rev() {
                        children.push_front(node.clone());
                    }
                },
            }
        }
        Some(text.trim().to_owned())
    }

    fn print(&self, print_tree: &mut PrintTree) {
        if self.child_nodes.is_empty() {
            print_tree.add_item(format!("{self:?}"));
            return;
        }

        print_tree.new_level(format!("{self:?}"));

        for child in self.children() {
            child.borrow().print(print_tree);
        }
        print_tree.end_level();
    }

    fn parent(&self) -> Option<ArcRefCell<AccessibilityNode>> {
        self.parent_node.as_ref().and_then(|weak| weak.upgrade())
    }

    // TODO: use macros to generate getter/setter methods.

    fn children(&self) -> impl DoubleEndedIterator<Item = &ArcRefCell<AccessibilityNode>> {
        self.child_nodes.iter()
    }

    fn ancestors(&self) -> impl Iterator<Item = ArcRefCell<AccessibilityNode>> {
        AccessibilityNodeIterator::new(self.parent(), |node| node.parent_node.clone()?.upgrade())
    }

    fn child_ids(&self) -> &[NodeId] {
        self.accesskit_node.children()
    }

    fn role(&self) -> Role {
        self.accesskit_node.role()
    }

    fn set_role(&mut self, role: Role) -> LocalAccessibilityDamage {
        if role == self.accesskit_node.role() {
            return LocalAccessibilityDamage::empty();
        }
        self.accesskit_node.set_role(role);
        self.dirty_state |= DirtyState::Updated;
        LocalAccessibilityDamage::RoleChanged
    }

    fn label(&self) -> Option<&str> {
        self.accesskit_node.label()
    }

    fn set_label(&mut self, label: &str) -> LocalAccessibilityDamage {
        if Some(label) == self.accesskit_node.label() {
            return LocalAccessibilityDamage::empty();
        }
        self.accesskit_node.set_label(label);
        self.dirty_state |= DirtyState::Updated;
        LocalAccessibilityDamage::TextChanged
    }

    fn clear_label(&mut self) -> LocalAccessibilityDamage {
        if self.accesskit_node.label().is_none() {
            return LocalAccessibilityDamage::empty();
        }
        self.accesskit_node.clear_label();
        self.dirty_state |= DirtyState::Updated;
        LocalAccessibilityDamage::TextChanged
    }

    fn html_tag(&self) -> Option<&str> {
        self.accesskit_node.html_tag()
    }

    fn set_html_tag(&mut self, html_tag: &str) {
        if Some(html_tag) == self.accesskit_node.html_tag() {
            return;
        }
        self.accesskit_node.set_html_tag(html_tag);
        self.dirty_state |= DirtyState::Updated;
    }

    fn value(&self) -> Option<&str> {
        self.accesskit_node.value()
    }

    fn set_value(&mut self, value: &str) -> LocalAccessibilityDamage {
        if Some(value) == self.accesskit_node.value() {
            return LocalAccessibilityDamage::empty();
        }
        self.accesskit_node.set_value(value);
        self.dirty_state |= DirtyState::Updated;
        LocalAccessibilityDamage::TextChanged
    }

    fn bounds(&self) -> Option<accesskit::Rect> {
        self.accesskit_node.bounds()
    }

    fn set_bounds(&mut self, bounds: accesskit::Rect) {
        if Some(bounds) == self.accesskit_node.bounds() {
            return;
        }
        self.accesskit_node.set_bounds(bounds);
        self.dirty_state |= DirtyState::Updated;
    }

    fn clear_bounds(&mut self) {
        if self.accesskit_node.bounds().is_none() {
            return;
        }
        self.accesskit_node.clear_bounds();
        self.dirty_state |= DirtyState::Updated;
    }

    fn assert_integrity(&self, expected_parent: Option<WeakRefCell<AccessibilityNode>>) {
        debug_assert!(pref!(expensive_accessibility_test_assertions_enabled));

        if let Some(actual_parent) = &self.parent_node {
            let expected = expected_parent.expect("Actual parent but no expected parent");
            let expected = expected.upgrade().expect("Expected parent was dropped");
            let actual = actual_parent.upgrade().expect("Actual parent was dropped");
            assert!(actual.ptr_eq(&expected));
        } else {
            assert!(
                expected_parent.is_none(),
                "Expected parent but no actual parent"
            );
        }

        assert!(
            self.dirty_state.is_empty(),
            "{self:?} has dirty state {:?}",
            self.dirty_state
        );

        let children_ids: Vec<_> = self.children().map(|child| child.borrow().id).collect();
        assert_eq!(
            children_ids,
            self.child_ids(),
            "children() IDs didn't match child_ids() for {self:?}"
        );
    }

    fn compute_damage(&self, update: &mut AccessibilityUpdate) -> AccessibilityDamage {
        let mut damage = AccessibilityDamage::empty();

        if self.dirty_state.has_damage() {
            damage |= update.take_damage(&self.id);
        }

        damage
    }
}

impl Debug for AccessibilityNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}", self.id, self.role())?;
        if let Some(html_tag) = self.html_tag() {
            write!(f, " (html_tag: {html_tag:?})")?;
        }
        if let Some(label) = self.label() {
            write!(f, "\nlabel: {label:?}")?;
        }
        if let Some(bounds) = self.bounds() {
            write!(f, "\nbounds: {bounds:?}")?;
        }
        if !self.child_ids().is_empty() {
            write!(f, "\nchildren: {:?}", self.child_ids())?;
        }
        Ok(())
    }
}

impl<'update> AccessibilityUpdate<'update> {
    fn new(
        dom_damage: AccessibilityDamageMap<'update>,
        rooted_nodes: Option<FxHashSet<OpaqueNode>>,
        tree: &AccessibilityTree,
    ) -> Self {
        let damage_map = dom_damage
            .iter()
            .filter_map(|(&opaque, &(_dom_node, damage))| {
                let id = tree.existing_id_for_opaque(opaque)?;
                Some((id, damage))
            })
            .collect();
        let dom_node_map = dom_damage
            .into_iter()
            .filter_map(|(opaque, (dom_node, _damage))| {
                let id = tree.existing_id_for_opaque(opaque)?;
                Some((id, dom_node))
            })
            .collect();
        Self {
            changed_nodes: FxHashSet::default(),
            tree_changes: FxHashMap::default(),
            counters: UpdateCounters::default(),
            damage_map,
            dom_node_map: RefCell::new(dom_node_map),
            rooted_nodes,
        }
    }

    fn add(&mut self, node: &mut AccessibilityNode) {
        self.changed_nodes.insert(node.id);
        node.dirty_state -= DirtyState::Updated;
    }

    fn set_tree_state_change(&mut self, node_id: NodeId, change: TreeChange) {
        let old_change = self.tree_changes.get(&node_id);

        assert!(
            change != TreeChange::Moved,
            "Incoming change must never be Moved"
        );

        let resolved_change = old_change
            .map(|old_change| match (old_change, change) {
                (TreeChange::PendingMove, TreeChange::Removed) => TreeChange::Moved,
                (TreeChange::Removed, TreeChange::PendingMove) => TreeChange::Moved,
                _ => {
                    unreachable!("Logically impossible state change: {old_change:?} → {change:?}")
                },
            })
            .unwrap_or(change);

        self.tree_changes.insert(node_id, resolved_change);
    }

    fn is_new(&mut self, node_id: &NodeId) -> bool {
        self.tree_changes.get(node_id) == Some(&TreeChange::New)
    }

    /// Consume this `AccessibilityUpdate`, producing an [`accesskit::TreeUpdate`] if there have
    /// been any changes to `tree`.
    /// This will pass `self` into [`AccessibilityTree::remove_stale_nodes()`] to consume
    /// [`Self::tree_changes`].
    fn finalize(
        mut self,
        tree: &mut AccessibilityTree,
    ) -> (Option<accesskit::TreeUpdate>, UpdateCounters) {
        let root_node_id = tree
            .root_node
            .clone()
            .expect("AccessibilityUpdate::finalize() called but no root_node set in tree")
            .borrow()
            .id;

        if self.changed_nodes.is_empty() {
            assert!(self.tree_changes.is_empty());
            return (None, self.counters);
        }

        let changed_nodes = std::mem::take(&mut self.changed_nodes);
        let mut counters = std::mem::take(&mut self.counters);

        tree.drop_removed_nodes(self);

        // Filter out any nodes which were both changed and removed.
        let changed_nodes: Vec<_> = changed_nodes
            .into_iter()
            .filter_map(|id| Some((id, tree.node_for_id(id)?.borrow().accesskit_node.clone())))
            .collect();

        counters.nodes_in_tree_update = changed_nodes.len().try_into().unwrap_or_default();

        let accesskit_tree = accesskit::Tree::new(root_node_id);
        let tree_update = accesskit::TreeUpdate {
            nodes: changed_nodes,
            tree: Some(accesskit_tree),
            focus: NodeId(1),
            tree_id: tree.tree_id,
        };

        (Some(tree_update), counters)
    }

    fn clear_damage(&mut self) {
        self.damage_map.clear();
    }

    fn insert_damage(&mut self, node_id: NodeId, damage: AccessibilityDamage) {
        self.damage_map.insert(node_id, damage);
    }

    fn insert_dom_node(&self, node_id: NodeId, dom_node: ServoLayoutNode<'update>) {
        self.dom_node_map.borrow_mut().insert(node_id, dom_node);
    }

    fn take_damage(&mut self, node_id: &NodeId) -> AccessibilityDamage {
        self.damage_map
            .remove(node_id)
            .unwrap_or(AccessibilityDamage::empty())
    }

    fn take_dom_node(&mut self, node_id: &NodeId) -> Option<ServoLayoutNode<'update>> {
        self.dom_node_map.borrow_mut().remove(node_id)
    }

    #[expect(unsafe_code)]
    fn collect_dom_node_ancestors(&self, node_id: &NodeId, tree: &AccessibilityTree) {
        let mut dom_node_map = self.dom_node_map.borrow_mut();
        let dom_node = dom_node_map
            .get(node_id)
            .expect("collect_dom_node_ancestors should be called for a known DOM node");
        let mut parent = unsafe { dom_node.dangerous_flat_tree_parent() };
        while let Some(node) = parent {
            if let Some(node_id) = tree.existing_id_for_opaque(node.opaque()) {
                dom_node_map.insert(node_id, node);
            }
            parent = unsafe { node.dangerous_flat_tree_parent() };
        }
    }
}

impl DirtyState {
    fn updated(&self) -> bool {
        self.contains(DirtyState::Updated)
    }

    fn has_damage(&self) -> bool {
        self.contains(DirtyState::HasDamage)
    }

    fn descendant_has_damage(&self) -> bool {
        self.contains(DirtyState::DescendantHasDamage)
    }

    fn propagate_descendant_has_damage(&mut self, child_dirty_state: DirtyState) {
        if child_dirty_state.self_or_descendant_has_damage() {
            self.insert(DirtyState::DescendantHasDamage)
        }
    }

    fn self_or_descendant_has_damage(&self) -> bool {
        self.intersects(DirtyState::HasDamage | DirtyState::DescendantHasDamage)
    }
}

#[cfg(test)]
#[test]
fn test_accessibility_update_add_some_nodes_twice() {
    let mut tree = AccessibilityTree::new(accesskit::TreeId::ROOT, Epoch::default());
    let mut root_update = AccessibilityUpdate::new(AccessibilityDamageMap::default(), None, &tree);

    let root_node = tree.get_or_create_node_with_id(NodeId(2), &mut root_update);
    tree.root_node = Some(root_node.clone());

    let nodes: Vec<_> = [
        (3, Role::GenericContainer),
        (4, Role::Heading),
        (5, Role::Paragraph),
    ]
    .into_iter()
    .map(|(id, role)| {
        let id = NodeId(id);
        let node = tree.get_or_create_node_with_id(id, &mut root_update);
        node.borrow_mut().set_role(role);
        (id, node)
    })
    .collect();

    {
        let (child_node_ids, child_nodes): (Vec<_>, Vec<_>) = nodes.iter().cloned().unzip();
        let mut root_node = root_node.borrow_mut();
        root_node.accesskit_node.set_children(child_node_ids);
        root_node.child_nodes = child_nodes;
    }

    let mut update = AccessibilityUpdate::new(AccessibilityDamageMap::default(), None, &tree);

    {
        let node_3 = tree.assert_node_for_id(&NodeId(3));
        let mut node_3 = node_3.borrow_mut();
        let node_4 = tree.assert_node_for_id(&NodeId(4));
        let mut node_4 = node_4.borrow_mut();
        let node_5 = tree.assert_node_for_id(&NodeId(5));
        let mut node_5 = node_5.borrow_mut();

        update.add(&mut node_5);
        update.add(&mut node_3);
        update.add(&mut node_4);
        update.add(&mut node_4);

        node_3.set_role(Role::ScrollView);
        update.add(&mut node_3);
    }

    let (tree_update, _) = update.finalize(&mut tree);
    let mut tree_update = tree_update.expect("finalize should produce a tree update");
    tree_update.nodes.sort_by_key(|(node_id, _node)| *node_id);
    assert_eq!(
        tree_update,
        accesskit::TreeUpdate {
            nodes: vec![
                (NodeId(3), accesskit::Node::new(Role::ScrollView)),
                (NodeId(4), accesskit::Node::new(Role::Heading)),
                (NodeId(5), accesskit::Node::new(Role::Paragraph)),
            ],
            tree: Some(accesskit::Tree {
                root: NodeId(2),
                toolkit_name: None,
                toolkit_version: None
            }),
            tree_id: accesskit::TreeId::ROOT,
            focus: NodeId(1),
        }
    );
}

static HTML_ELEMENT_ROLE_MAPPINGS: LazyLock<FxHashMap<LocalName, Role>> = LazyLock::new(|| {
    [
        (local_name!("article"), Role::Article),
        (local_name!("aside"), Role::Complementary),
        (local_name!("body"), Role::RootWebArea),
        (local_name!("footer"), Role::ContentInfo),
        (local_name!("h1"), Role::Heading),
        (local_name!("h2"), Role::Heading),
        (local_name!("h3"), Role::Heading),
        (local_name!("h4"), Role::Heading),
        (local_name!("h5"), Role::Heading),
        (local_name!("h6"), Role::Heading),
        (local_name!("header"), Role::Banner),
        (local_name!("hr"), Role::Splitter),
        (local_name!("main"), Role::Main),
        (local_name!("nav"), Role::Navigation),
        (local_name!("p"), Role::Paragraph),
    ]
    .into_iter()
    .collect()
});

/// A map from role names allowed in the 'role' attribute of an HTML element to the corresponding
/// [`Role`] in AccessKit.
///
/// This is currently just the roles that don't have any [supported][1] or [required][2] properties
/// and also don't require an [accessible name][3].
/// [1]: https://w3c.github.io/aria/#supportedState
/// [2]: https://w3c.github.io/aria/#requiredState
/// [3]: https://w3c.github.io/aria/#namefromauthor
static SUPPORTED_ARIA_ROLES: LazyLock<FxHashMap<Atom, Role>> = LazyLock::new(|| {
    [
        (Atom::from("alert"), Role::Alert),
        (Atom::from("banner"), Role::Banner),
        (Atom::from("blockquote"), Role::Blockquote),
        (Atom::from("caption"), Role::Caption),
        (Atom::from("code"), Role::Code),
        (Atom::from("complementary"), Role::Complementary),
        (Atom::from("contentinfo"), Role::ContentInfo),
        (Atom::from("definition"), Role::Definition),
        (Atom::from("deletion"), Role::ContentDeletion),
        (Atom::from("directory"), Role::Unknown),
        (Atom::from("document"), Role::Document),
        (Atom::from("emphasis"), Role::Emphasis),
        (Atom::from("feed"), Role::Feed),
        (Atom::from("figure"), Role::Figure),
        (Atom::from("generic"), Role::GenericContainer),
        (Atom::from("insertion"), Role::ContentInsertion),
        (Atom::from("list"), Role::List),
        (Atom::from("log"), Role::Log),
        (Atom::from("main"), Role::Main),
        (Atom::from("math"), Role::Math),
        (Atom::from("navigation"), Role::Navigation),
        (Atom::from("none"), Role::GenericContainer),
        (Atom::from("note"), Role::Note),
        (Atom::from("paragraph"), Role::Paragraph),
        (Atom::from("presentation"), Role::GenericContainer),
        (Atom::from("rowgroup"), Role::RowGroup),
        (Atom::from("search"), Role::Search),
        (Atom::from("status"), Role::Status),
        (Atom::from("strong"), Role::Strong),
        // (Atom::from("subscript"), Role::Subscript), // no corresponding accesskit role.
        // (Atom::from("superscript"), Role::Superscript), // no corresponding accesskit role.
        (Atom::from("term"), Role::Term),
        (Atom::from("time"), Role::Time),
        (Atom::from("timer"), Role::Timer),
    ]
    .into_iter()
    .collect()
});

/// <https://w3c.github.io/aria/#namefromcontent>
static NAME_FROM_CONTENTS_ROLES: LazyLock<FxHashSet<Role>> =
    LazyLock::new(|| [(Role::Heading)].into_iter().collect());
