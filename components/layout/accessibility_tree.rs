/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::VecDeque;
use std::fmt::Debug;
use std::iter::repeat;
use std::sync::atomic::AtomicU64;
use std::sync::{LazyLock, atomic};

use accesskit::{NodeId, Role};
use bitflags::bitflags;
use layout_api::{AccessibilityDamage, LayoutElement, LayoutNode, LayoutNodeType};
use log::trace;
use rustc_hash::{FxHashMap, FxHashSet};
use script::layout_dom::ServoLayoutNode;
use servo_base::Epoch;
use servo_base::print_tree::PrintTree;
use servo_config::opts::{self, DiagnosticsLogging, DiagnosticsLoggingOption};
use servo_config::pref;
use style::dom::OpaqueNode;
use web_atoms::{LocalName, local_name};

use crate::ArcRefCell;
use crate::cell::WeakRefCell;

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

/// Changes which have occurred during the current update.
struct AccessibilityUpdate {
    /// Nodes whose internal data has changed within the current update.
    changed_nodes: FxHashSet<NodeId>,
    /// Nodes that changed their relation to the tree within the current update.
    tree_changes: FxHashMap<NodeId, TreeChange>,
    /// Damage to nodes caused by changes in the accessibility tree.
    unresolved_local_damage: FxHashMap<NodeId, LocalAccessibilityDamage>,
    /// Counters to track how many nodes we've checked for changes or updated in this tree update.
    counters: UpdateCounters,
    /// Nodes which were removed from the DOM tree since the last reflow, which were rooted in
    /// `AccessibilityData`. Only set if `pref::expensive_accessibility_test_assertions_enabled`
    /// is set.
    rooted_nodes: Option<FxHashSet<OpaqueNode>>,
}

#[derive(Debug, Default)]
pub struct UpdateCounters {
    pub update_node_and_descendants_from_dom_node: u32,
    pub update_node_local: u32,
    pub nodes_in_tree_update: u32,
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
    /// Whether this node has been updated in the current tree update. This is reset to `false`
    /// when the node is added to the [`AccessibilityUpdate`] - see [`AccessibilityUpdate::add()`].
    updated: bool,
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
    pub(super) fn update_tree<'dom>(
        &mut self,
        root_dom_node: &ServoLayoutNode<'dom>,
        mut damage_from_dom: VecDeque<(ServoLayoutNode<'dom>, AccessibilityDamage)>,
        rooted_nodes: Option<FxHashSet<OpaqueNode>>,
    ) -> (Option<accesskit::TreeUpdate>, UpdateCounters) {
        let mut update = AccessibilityUpdate::new(rooted_nodes);

        self.ensure_root_node(root_dom_node, &mut damage_from_dom, &mut update);

        self.apply_changes_from_dom_tree(damage_from_dom, &mut update);

        // FIXME: This assumes any local subtree damage always propagates up to the root. This is
        // currently true, but we might be able to improve at stopping propagation.
        self.resolve_local_damage_for_node_and_subtree(self.assert_root_node(), &mut update);

        update.finalize(self)
    }

    /// Get the node corresponding to the root DOM node, and set it as this tree's root. If the root
    /// node is newly created, which probably means this accessibility tree is newly created, append
    /// an `AccessibilityDamage::REBUILD` value for it to `damage_from_dom`.
    fn ensure_root_node<'dom>(
        &mut self,
        root_dom_node: &ServoLayoutNode<'dom>,
        damage_from_dom: &mut VecDeque<(ServoLayoutNode<'dom>, AccessibilityDamage)>,
        update: &mut AccessibilityUpdate,
    ) {
        let (root_id, root_node) = self.get_or_create_node(root_dom_node, update);
        if update.is_new(&root_id) {
            damage_from_dom.push_front((*root_dom_node, AccessibilityDamage::Rebuild));
        }
        self.root_node = Some(root_node);
    }

    /// Get the root node for this tree, asserting that it has been set.
    fn assert_root_node(&self) -> ArcRefCell<AccessibilityNode> {
        self.root_node.clone().expect("Root node was asserted")
    }

    /// For each DOM node in `damage_from_dom`, update the corresponding accessibility node based on
    /// its `AccessibilityDamage`. If any [`LocalDamage`] results from the update, propagate
    /// [`LocalDamage::SUBTREE_CHANGED`] to its ancestors.
    fn apply_changes_from_dom_tree<'dom>(
        &mut self,
        mut damage_from_dom: VecDeque<(ServoLayoutNode<'dom>, AccessibilityDamage)>,
        update: &mut AccessibilityUpdate,
    ) {
        while let Some((dom_node, dom_node_damage)) = damage_from_dom.pop_front() {
            let Some((_, node)) = self.node_for_dom_node(&dom_node) else {
                // If we don't have a node for this DOM node yet, it will be created and populated
                // when it's added to its parent node.
                continue;
            };
            self.update_node_and_descendants_from_dom_node(
                node.clone(),
                &dom_node,
                dom_node_damage,
                update,
            );
        }

        self.propagate_subtree_damage_to_ancestors(update);
    }

    /// After applying changes from the DOM tree, mark the ancestors of any changed nodes with
    /// [`LocalDamage::SubtreeChanged`].
    fn propagate_subtree_damage_to_ancestors(&mut self, update: &mut AccessibilityUpdate) {
        for (node_id, damage) in update.unresolved_local_damage.clone() {
            if damage.is_empty() {
                continue;
            }
            let node = self.assert_node_for_id(&node_id);

            let mut parent_node = node.borrow().parent();
            while let Some(node) = parent_node {
                let node = node.borrow();
                let existing_damage = update.unresolved_local_damage.entry(node.id).or_default();

                // If we encounter a node which already has `SubtreeChanged` damage, we know that
                // all of its ancestors have it too, so we can bail out early from our ancestor walk
                if existing_damage.contains(LocalAccessibilityDamage::SubtreeChanged) {
                    break;
                }

                existing_damage.insert(LocalAccessibilityDamage::SubtreeChanged);
                parent_node = node.parent();
            }
        }
    }

    /// Update the given AccessibilityNode from its corresponding DOM node and
    /// ['AccessibilityDamage'].
    /// If it has new children, those will be recursively populated here.
    // Any changed nodes will be added to the given [`AccessibilityUpdate`].
    fn update_node_and_descendants_from_dom_node(
        &mut self,
        node: ArcRefCell<AccessibilityNode>,
        dom_node: &ServoLayoutNode<'_>,
        dom_damage: AccessibilityDamage,
        update: &mut AccessibilityUpdate,
    ) -> LocalAccessibilityDamage {
        update.counters.update_node_and_descendants_from_dom_node += 1;

        let weak_node = node.downgrade();
        let mut node = node.borrow_mut();
        let mut local_damage = LocalAccessibilityDamage::empty();

        local_damage.insert(node.update_node_from_dom_node(dom_node, dom_damage));
        local_damage.insert(
            node.update_descendants_from_dom_node(weak_node, dom_node, dom_damage, self, update),
        );

        if node.updated {
            update.add(&mut node);
        }

        if !local_damage.is_empty() {
            update
                .unresolved_local_damage
                .entry(node.id)
                .or_default()
                .insert(local_damage);
        }

        local_damage
    }

    /// Update the given node and, where necessary, its descendants, based on damage propagated
    /// within the accessibility as a result of changes made based on DOM tree changes.
    /// For example, if a node's descendants changed as a result of the DOM tree changing, its
    /// computed text may also have changed, so it would have had
    /// [`LocalAccessibilityDamage::SubtreeChanged`] set when changes from the DOM tree were
    /// applied. That damage is resolved here.
    fn resolve_local_damage_for_node_and_subtree(
        &mut self,
        node: ArcRefCell<AccessibilityNode>,
        update: &mut AccessibilityUpdate,
    ) {
        let mut node = node.borrow_mut();
        let Some(&local_damage) = update.unresolved_local_damage.get(&node.id) else {
            return;
        };

        node.update_node_local(local_damage, update);

        if local_damage.contains(LocalAccessibilityDamage::SubtreeChanged) {
            for child in node.children() {
                self.resolve_local_damage_for_node_and_subtree(child.clone(), update);
            }
        }

        if node.updated {
            update.add(&mut node);
        }

        update.unresolved_local_damage.remove(&node.id);
    }

    fn get_or_create_node(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
        update: &mut AccessibilityUpdate,
    ) -> (NodeId, ArcRefCell<AccessibilityNode>) {
        let id = self.get_or_create_id_for_opaque(dom_node.opaque());
        let node = self.get_or_create_node_with_id(id, update);

        {
            let mut node = node.borrow_mut();
            node.opaque_node = Some(dom_node.opaque());
            if let Some(dom_element) = dom_node.as_element() {
                let local_name = dom_element.local_name().to_ascii_lowercase();
                node.set_html_tag(&local_name);
            }
        }

        (id, node)
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

    fn node_for_dom_node(
        &self,
        dom_node: &ServoLayoutNode<'_>,
    ) -> Option<(NodeId, ArcRefCell<AccessibilityNode>)> {
        let id = self.existing_id_for_opaque(dom_node.opaque())?;
        Some((id, self.node_for_id(id)?))
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
        if let Some(rooted_nodes) = std::mem::take(&mut update.rooted_nodes) {
            self.assert_removed_nodes_were_rooted(&update, rooted_nodes);
        }

        let mut ids_to_remove: Vec<_> = update
            .tree_changes
            .drain()
            .filter_map(|(id, change)| match change {
                TreeChange::PendingMove => {
                    unreachable!(
                        "Pending move found for node id {id:?} when draining tree state changes"
                    );
                },
                TreeChange::Removed => Some(id),
                TreeChange::New => None,
                TreeChange::Moved => None,
            })
            .collect();

        while let Some(id) = ids_to_remove.pop() {
            update.unresolved_local_damage.remove(&id);
            let Some(node) = self.nodes.remove(&id) else {
                panic!("Node for id {id:?} was already removed");
            };
            if let Some(opaque_node) = self.id_to_opaque_node.remove(&id) {
                self.opaque_node_to_id.remove(&opaque_node);
            }
            ids_to_remove.extend(node.borrow().child_ids());
        }

        // We should have resolved all damage in nodes still in the tree by this point, and any
        // nodes not in the tree should have been removed from this map in the loop above.
        assert!(
            update.unresolved_local_damage.is_empty(),
            "Damage not empty: {:?}",
            update.unresolved_local_damage
        );

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
    /// removed during this update was rooted, and any leftover rooted nodes were never known to the
    /// accessibility tree.
    fn assert_removed_nodes_were_rooted(
        &mut self,
        update: &AccessibilityUpdate,
        mut rooted_nodes: FxHashSet<OpaqueNode>,
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

        for leftover_node in rooted_nodes {
            assert!(
                !self.opaque_node_to_id.contains_key(&leftover_node),
                "Found node removed from DOM tree but not accessibility tree"
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
            nodes.extend(node.children().iter().cloned().zip(repeat(weak_node)));
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

fn role_from_dom_node(dom_node: &ServoLayoutNode<'_>) -> Role {
    if let Some(dom_element) = dom_node.as_element() {
        let local_name = dom_element.local_name().to_ascii_lowercase();
        *HTML_ELEMENT_ROLE_MAPPINGS
            .get(&local_name)
            .unwrap_or(&Role::GenericContainer)
    } else if dom_node.type_id() == Some(LayoutNodeType::Text) {
        Role::TextRun
    } else {
        Role::GenericContainer
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
            updated: true,
        }
    }

    /// Update this node's [`Self::children`] from its corresponding DOM node. If any children are
    /// newly added to the tree, populate them and recursively populate their children.
    fn update_descendants_from_dom_node<'dom>(
        &mut self,
        weak_self: WeakRefCell<Self>,
        dom_node: &ServoLayoutNode<'dom>,
        dom_damage: AccessibilityDamage,
        tree: &mut AccessibilityTree,
        update: &mut AccessibilityUpdate,
    ) -> LocalAccessibilityDamage {
        if !dom_damage.contains(AccessibilityDamage::Children) {
            return LocalAccessibilityDamage::empty();
        }

        let mut remaining_dom_children = dom_node.flat_tree_children().peekable();
        let mut old_child_ids = self.child_ids().iter().peekable();
        let mut unchanged_count = 0usize;

        // For the first `unchanged_count` children, no action is necessary.
        while let Some(&old_id) = old_child_ids.peek() &&
            let Some(dom_child) = remaining_dom_children.peek()
        {
            if tree.existing_id_for_opaque(dom_child.opaque()) == Some(*old_id) {
                unchanged_count += 1;
                old_child_ids.next();
                remaining_dom_children.next();
            } else {
                break;
            }
        }

        // If we iterated over all the dom children without finding any changes, we're done.
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
        for dom_child in remaining_dom_children {
            let (new_id, new_child) = tree.get_or_create_node(&dom_child, update);
            if update.is_new(&new_id) {
                tree.update_node_and_descendants_from_dom_node(
                    new_child.clone(),
                    &dom_child,
                    AccessibilityDamage::Rebuild,
                    update,
                );
            } else {
                update.set_tree_state_change(new_id, TreeChange::PendingMove);
            }

            // Update self.child_nodes in place.
            self.child_nodes.push(new_child.clone());
            new_child_ids.push(new_id);

            let mut new_child = new_child.borrow_mut();
            new_child.parent_node = Some(weak_self.clone());
        }

        // We can't update the AccessKit node's `children` in place, so we build up the full list
        // and then set it here.
        self.accesskit_node.set_children(new_child_ids);
        self.updated = true;

        LocalAccessibilityDamage::SubtreeChanged
    }

    /// Update this node's properties from its corresponding DOM node.
    fn update_node_from_dom_node(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
        dom_damage: AccessibilityDamage,
    ) -> LocalAccessibilityDamage {
        let mut local_damage = LocalAccessibilityDamage::empty();
        if !dom_damage.contains(AccessibilityDamage::Text) {
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

    /// Update this node's properties based on changes already made to the accessibility tree.
    /// For example, if there were nodes added or removed in its subtree, its computed text may have
    /// changed, so that will be recomputed here.
    /// If any changes are made, add this node to the given [`AccessibilityUpdate`].
    fn update_node_local(
        &mut self,
        local_damage: LocalAccessibilityDamage,
        update: &mut AccessibilityUpdate,
    ) -> LocalAccessibilityDamage {
        update.counters.update_node_local += 1;

        let mut new_damage = LocalAccessibilityDamage::empty();
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
        let mut children = VecDeque::from_iter(self.children().iter().cloned());
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
                    for node in child.children().iter().rev() {
                        children.push_front(node.clone());
                    }
                },
            }
        }
        Some(text.trim().to_owned())
    }

    fn print(&self, print_tree: &mut PrintTree) {
        if self.children().is_empty() {
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

    fn children(&self) -> &Vec<ArcRefCell<AccessibilityNode>> {
        &self.child_nodes
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
        self.updated = true;
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
        self.updated = true;
        LocalAccessibilityDamage::TextChanged
    }

    fn clear_label(&mut self) -> LocalAccessibilityDamage {
        if self.accesskit_node.label().is_none() {
            return LocalAccessibilityDamage::empty();
        }
        self.accesskit_node.clear_label();
        self.updated = true;
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
        self.updated = true;
    }

    fn value(&self) -> Option<&str> {
        self.accesskit_node.value()
    }

    fn set_value(&mut self, value: &str) -> LocalAccessibilityDamage {
        if Some(value) == self.accesskit_node.value() {
            return LocalAccessibilityDamage::empty();
        }
        self.accesskit_node.set_value(value);
        self.updated = true;
        LocalAccessibilityDamage::TextChanged
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

        let children_ids: Vec<_> = self
            .children()
            .iter()
            .map(|child| child.borrow().id)
            .collect();
        assert_eq!(
            children_ids,
            self.child_ids(),
            "children() IDs didn't match child_ids() for {self:?}"
        );
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
        if !self.child_ids().is_empty() {
            write!(f, "\nchildren: {:?}", self.child_ids())?;
        }
        Ok(())
    }
}

impl AccessibilityUpdate {
    fn new(rooted_nodes: Option<FxHashSet<OpaqueNode>>) -> Self {
        Self {
            changed_nodes: FxHashSet::default(),
            tree_changes: FxHashMap::default(),
            unresolved_local_damage: FxHashMap::default(),
            counters: UpdateCounters::default(),
            rooted_nodes,
        }
    }

    fn add(&mut self, node: &mut AccessibilityNode) {
        self.changed_nodes.insert(node.id);

        node.updated = false;
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
            assert!(self.unresolved_local_damage.is_empty());
            return (None, self.counters);
        }

        let changed_nodes = std::mem::take(&mut self.changed_nodes);
        let mut counters = std::mem::take(&mut self.counters);

        tree.drop_removed_nodes(self);

        let changed_nodes: Vec<_> = changed_nodes
            .into_iter()
            .filter_map(|id| Some((id, tree.node_for_id(id)?.borrow().accesskit_node.clone())))
            .collect();

        counters.nodes_in_tree_update = changed_nodes.len().try_into().unwrap_or_default();

        let accesskit_tree = accesskit::Tree::new(root_node_id);
        let tree_update = accesskit::TreeUpdate {
            // Filter out any nodes which were both changed and removed.
            nodes: changed_nodes,
            tree: Some(accesskit_tree),
            focus: NodeId(1),
            tree_id: tree.tree_id,
        };

        (Some(tree_update), counters)
    }
}

#[cfg(test)]
#[test]
fn test_accessibility_update_add_some_nodes_twice() {
    let mut tree = AccessibilityTree::new(accesskit::TreeId::ROOT, Epoch::default());
    let mut root_update = AccessibilityUpdate::new(None);

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

    let mut update = AccessibilityUpdate::new(None);

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

/// <https://w3c.github.io/aria/#namefromcontent>
static NAME_FROM_CONTENTS_ROLES: LazyLock<FxHashSet<Role>> =
    LazyLock::new(|| [(Role::Heading)].into_iter().collect());
