/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::iter::repeat;
use std::sync::atomic::{self, AtomicU64};

use layout_api::{
    AccessibilityActionRequest, AccessibilityDamage, LayoutNode, node_id_from_scroll_id,
};
use paint_api::display_list::SpatialTreeNodeInfo;
use rustc_hash::{FxHashMap, FxHashSet};
use script::layout_dom::ServoLayoutNode;
use servo_base::Epoch;
use servo_base::print_tree::PrintTree;
use servo_config::opts::{self, DiagnosticsLogging, DiagnosticsLoggingOption};
use servo_config::pref;
use style::dom::OpaqueNode;
use webrender_api::ExternalScrollId;
use webrender_api::units::LayoutVector2D;

use crate::ArcRefCell;
use crate::accessibility::accessibility_node::{AccessibilityNode, DirtyState};
use crate::display_list::StackingContextTree;
use crate::layout_impl::LayoutThread;

mod accessibility_node;

use accesskit::{ActionRequest, NodeId, Role};

/// Everything the accessibility tree needs from layout in order to compute node bounds during an
/// update.
pub(crate) struct AccessibilityContext<'update> {
    pub(crate) layout_thread: &'update LayoutThread,
    pub(crate) stacking_context_tree: &'update StackingContextTree,
    pub(crate) focused_element: Option<OpaqueNode>,
    pub(crate) rooted_nodes_for_integrity_check: Option<FxHashSet<OpaqueNode>>,
}

/// All the [`AccessibilityDamage`] which comes from outside the accessibility tree itself.
pub(crate) type AccessibilityDamageMap<'a> =
    FxHashMap<OpaqueNode, (ServoLayoutNode<'a>, AccessibilityDamage)>;

/// Changes which have occurred during the current update, and data required to process the update.
pub(super) struct AccessibilityUpdate<'update> {
    /// Nodes whose internal data has changed within the current update.
    changed_nodes: FxHashSet<NodeId>,
    /// Sent with the initial [`accesskit::TreeUpdate`], and whenever the root node changes.
    /// This is a property of the update, rather than the tree, because it only needs to exist for
    /// updates where one of those two conditions is true.
    accesskit_tree: Option<accesskit::Tree>,
    /// Nodes that changed their relation to the tree within the current update.
    tree_changes: FxHashMap<NodeId, TreeChange>,
    /// Whether the tree's focused node has changed in this update.
    focused_node_changed: bool,
    /// Counters to track how many nodes we've checked for changes or updated in this tree update.
    counters: UpdateCounters,

    /// Map of [`NodeId`] to the [`AccessibilityDamage`] which was passed in for that node.
    damage_map: FxHashMap<NodeId, AccessibilityDamage>,
    /// Map of [`NodeId`] to the corresponding [`ServoLayoutNode`]. This is populated for nodes
    /// which have damage, including nodes which are newly added to the accessibility tree.
    dom_node_map: RefCell<FxHashMap<NodeId, ServoLayoutNode<'update>>>,
}

#[derive(Debug, Default)]
pub struct UpdateCounters {
    pub nodes_updated_from_dom: u32,
    pub nodes_updated_from_tree: u32,
    pub nodes_updated_bounds: u32,
    pub nodes_in_tree_update: u32,
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
    /// The node which has focus.
    focused_node_id: Option<NodeId>,
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
    /// If any nodes were scrolled since the last update, they are tracked here so that the next
    /// update can update the tree accordingly.
    pending_scroll_updates: FxHashMap<ExternalScrollId, LayoutVector2D>,
    /// Sent to the embedder alongside each [`accesskit::TreeUpdate`], so that the embedder can
    /// drop updates from documents which have been navigated away from.
    embedder_epoch: Epoch,
    /// Pending actions which have been processed from [`accesskit::ActionRequest`]s to retrieve the
    /// [`OpaqueNode`] for the corresponding DOM node.
    /// Any [`OpaqueNode`] in this list corresponds to an [`AccessibilityNode`] which is still in
    /// the tree immediately after the tree has been updated, and therefore should correspond to a
    /// live DOM node.
    pending_actions: Vec<AccessibilityActionRequest>,
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
            focused_node_id: None,
            opaque_node_to_id: FxHashMap::default(),
            id_to_opaque_node: FxHashMap::default(),
            tree_id,
            root_node: None,
            pending_scroll_updates: FxHashMap::default(),
            embedder_epoch,
            pending_actions: vec![],
            debug: opts::get().debug.clone(),
        }
    }

    /// Update this tree based on the current state of the given DOM tree, and if anything changed,
    /// return an [`accesskit::TreeUpdate`] representing what changed.
    pub(super) fn update_tree<'update>(
        &mut self,
        root_dom_node: &ServoLayoutNode<'update>,
        damage_from_dom: AccessibilityDamageMap<'update>,
        action_requests: Vec<ActionRequest>,
        context: AccessibilityContext<'update>,
    ) -> (Option<accesskit::TreeUpdate>, UpdateCounters) {
        let mut update = AccessibilityUpdate::new(damage_from_dom, self);

        self.ensure_root_node(root_dom_node, &context, &mut update);

        self.apply_changes_from_dom_tree(&context, &mut update);

        self.update_focused_node(context.focused_element, &mut update);

        self.handle_pending_scroll_updates(&mut update);

        update.finalize(
            self,
            context.rooted_nodes_for_integrity_check,
            action_requests,
        )
    }

    /// Add all given scroll updates to [`Self::pending_scroll_updates`].
    /// See [`Self::handle_pending_scroll_updates()`].
    pub(super) fn add_pending_scroll_updates(
        &mut self,
        scroll_states: FxHashMap<ExternalScrollId, LayoutVector2D>,
    ) {
        self.pending_scroll_updates.extend(scroll_states);
    }

    /// Add the given scroll update to [`Self::pending_scroll_updates`].
    /// See [`Self::handle_pending_scroll_updates()`].
    pub(super) fn add_pending_scroll_update(
        &mut self,
        external_scroll_id: ExternalScrollId,
        offset: LayoutVector2D,
    ) {
        self.pending_scroll_updates
            .insert(external_scroll_id, offset);
    }

    /// Get the node corresponding to the root DOM node, and set it as this tree's root. If the root
    /// node is newly created, which probably means this accessibility tree is newly created, append
    /// an `AccessibilityDamage::Rebuild` value for it to `damage_from_dom`.
    fn ensure_root_node<'update>(
        &mut self,
        root_dom_node: &ServoLayoutNode<'update>,
        context: &AccessibilityContext<'update>,
        update: &mut AccessibilityUpdate<'update>,
    ) {
        let (root_id, root_node) = self.get_or_create_node(root_dom_node, update);
        if update.is_new(&root_id) {
            // We're going to rebuild the whole tree, so ignore any incoming damage.
            update.clear_damage();
            update.insert_damage(root_id, AccessibilityDamage::Rebuild);
            update.insert_dom_node(root_id, *root_dom_node);
            self.populate_pending_scroll_updates_from_scroll_tree(context);

            update.accesskit_tree = Some(accesskit::Tree::new(root_id));
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
        let hidden = false;
        let local_damage = damage_root.borrow_mut().update_subtree(
            damage_root.clone(),
            AccessibilityDamage::empty(),
            hidden,
            context,
            self,
            update,
        );

        damage_root.borrow().update_ancestors(local_damage, update);
    }

    /// Read all scroll offsets directly from the scroll tree, and use them to populate
    /// [`Self::pending_scroll_updates`].
    /// This will clear any previous pending scroll updates, as the scroll tree contains all scroll
    /// information.
    fn populate_pending_scroll_updates_from_scroll_tree(&mut self, context: &AccessibilityContext) {
        let scroll_tree = &context.stacking_context_tree.paint_info.scroll_tree;
        let scroll_updates = scroll_tree
            .nodes
            .iter()
            .filter_map(|node| match node.info {
                SpatialTreeNodeInfo::Scroll(ref info) => {
                    let offset = info.offset;
                    Some((info.external_id, offset))
                },
                _ => None,
            })
            .collect();
        self.pending_scroll_updates = scroll_updates;
    }

    /// For each entry in [`Self::pending_scroll_updates`], set the scroll offset on the
    /// [`AccessibilityNode`] corresponding to its [`ExternalScrollId`], if any.
    /// This sets a transformation on every direct child of the scrolled node.
    ///
    /// This should be called after the tree has been updated, so that we can be sure not to miss
    /// any newly-added nodes.
    fn handle_pending_scroll_updates(&mut self, update: &mut AccessibilityUpdate) {
        let pending_scroll_updates = std::mem::take(&mut self.pending_scroll_updates);
        for (opaque, offset) in
            pending_scroll_updates
                .into_iter()
                .filter_map(|(scroll_id, translate)| {
                    if scroll_id.is_root() {
                        let root_node_opaque =
                            self.root_node.as_ref()?.clone().borrow().opaque_node()?;
                        return Some((root_node_opaque, translate));
                    }
                    let node_id = node_id_from_scroll_id(scroll_id.0 as usize);
                    let opaque = OpaqueNode(node_id);
                    Some((opaque, translate))
                })
        {
            let Some(node) = self.node_for_opaque(opaque) else {
                continue;
            };
            node.borrow_mut().set_scroll_offset(offset, update);
        }
    }

    /// Given an iterator of `NodeId`s corresponding to nodes which have received some damage from
    /// the DOM:
    /// - mark each node as [`DirtyState::Dirty`];
    /// - mark all of each node's ancestors as [`DirtyState::HasDirtyDescendants`];
    /// - find the lowest common ancestor node of all the damaged nodes;
    /// - remove the [`DirtyState::HasDirtyDescendants`] flag on nodes between the common ancestor
    ///   and the root;
    /// - return the common ancestor.
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
            first_node.add_dirty_state(DirtyState::HasDamage);
            common_ancestors.push(first_node.id());
            common_ancestors.extend(first_node.ancestors().map(|ancestor| {
                let mut ancestor = ancestor.borrow_mut();
                ancestor.add_dirty_state(DirtyState::DescendantHasDamage);
                ancestor.id()
            }));
            common_ancestors.reverse();
        }

        let mut truncate_ancestors = |node: &AccessibilityNode| -> bool {
            if node.dirty_state().descendant_has_damage() {
                if let Some(pos) = common_ancestors.iter().position(|&id| id == node.id()) {
                    common_ancestors.truncate(pos + 1);
                }
                return true;
            }
            false
        };

        for node_id in dirty_node_ids {
            let node = self.assert_node_for_id(node_id);
            let mut node = node.borrow_mut();
            node.add_dirty_state(DirtyState::HasDamage);

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

                ancestor.add_dirty_state(DirtyState::DescendantHasDamage);
            }
        }

        let lowest_common_ancestor = common_ancestors.pop();

        for ancestor_id in common_ancestors {
            let ancestor = self.assert_node_for_id(&ancestor_id);
            ancestor
                .borrow_mut()
                .remove_dirty_state(DirtyState::DescendantHasDamage);
        }

        lowest_common_ancestor
    }

    fn update_focused_node(
        &mut self,
        focused_element: Option<OpaqueNode>,
        update: &mut AccessibilityUpdate,
    ) {
        let mut focused_node_id = None;
        if let Some(focused_element) = focused_element &&
            let Some(node_id) = self.existing_id_for_opaque(focused_element) &&
            let Some(focused_node) = self.node_for_id(node_id)
        {
            let focused_node = focused_node.borrow();
            if focused_node.role() == Role::GenericContainer {
                // Avoid moving focus to a node which is effectively hidden from accessibility, but
                // don't reset focus to the root node either.
                focused_node_id = self.focused_node_id;
            } else {
                focused_node_id = Some(focused_node.id());
            }
        }

        if focused_node_id == self.focused_node_id {
            return;
        }

        update.focused_node_changed = true;
        self.focused_node_id = focused_node_id;
    }

    pub(super) fn focused_node_id(&self) -> Option<NodeId> {
        self.focused_node_id
    }

    fn get_or_create_node(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
        update: &mut AccessibilityUpdate,
    ) -> (NodeId, ArcRefCell<AccessibilityNode>) {
        let id = self.get_or_create_id_for_opaque(dom_node.opaque());
        let node_ref = self.get_or_create_node_with_id(id, update);

        if update.is_new(&id) {
            let mut node = node_ref.borrow_mut();
            node.initialize_from_dom_node(dom_node);
            update.insert_damage(id, AccessibilityDamage::Rebuild);
            node.add_dirty_state(DirtyState::HasDamage);
        }

        (id, node_ref)
    }

    pub(crate) fn accesskit_node_for_dom_node(
        &self,
        dom_node: &ServoLayoutNode,
    ) -> Option<accesskit::Node> {
        let node = self.node_for_opaque(dom_node.opaque())?;
        Some(node.borrow().clone_accesskit_node())
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

    fn node_for_opaque(&self, opaque: OpaqueNode) -> Option<ArcRefCell<AccessibilityNode>> {
        self.nodes
            .get(&self.existing_id_for_opaque(opaque)?)
            .cloned()
    }

    /// Consume the [`AccessibilityUpdate`] by deleting all nodes it detected as being removed from
    /// the tree.
    fn drop_removed_nodes(
        &mut self,
        mut update: AccessibilityUpdate,
        mut rooted_nodes_for_integrity_check: Option<FxHashSet<OpaqueNode>>,
    ) {
        if let Some(rooted_nodes) = rooted_nodes_for_integrity_check.as_mut() {
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

        if let Some(rooted_nodes) = rooted_nodes_for_integrity_check {
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

    pub(crate) fn take_pending_actions(&mut self) -> Vec<AccessibilityActionRequest> {
        std::mem::take(&mut self.pending_actions)
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
                seen_node_ids.insert(node.id()),
                "Tree contains {:?} in multiple places",
                node.id()
            );

            node.assert_integrity(expected_parent);

            // assert_node_for_id() here double-checks that the node hasn't been incorrectly evicted
            // from the map while it's still retained as a child node.
            let weak_node = Some(self.assert_node_for_id(&node.id()).downgrade());
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
        root_node.borrow().print(&mut print_tree, self);
        print_tree.end_level();
    }
}

impl<'update> AccessibilityUpdate<'update> {
    fn new(dom_damage: AccessibilityDamageMap<'update>, tree: &AccessibilityTree) -> Self {
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
            accesskit_tree: None,
            tree_changes: FxHashMap::default(),
            focused_node_changed: false,
            counters: UpdateCounters::default(),
            damage_map,
            dom_node_map: RefCell::new(dom_node_map),
        }
    }

    fn add(&mut self, node: &mut AccessibilityNode) {
        self.changed_nodes.insert(node.id());
        node.remove_dirty_state(DirtyState::Updated);
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
        rooted_nodes_for_integrity_check: Option<FxHashSet<OpaqueNode>>,
        action_requests: Vec<ActionRequest>,
    ) -> (Option<accesskit::TreeUpdate>, UpdateCounters) {
        let mut tree_update = None;
        let mut counters = std::mem::take(&mut self.counters);
        if !self.changed_nodes.is_empty() ||
            self.accesskit_tree.is_some() ||
            self.focused_node_changed
        {
            let changed_nodes = std::mem::take(&mut self.changed_nodes);
            let accesskit_tree = std::mem::take(&mut self.accesskit_tree);

            tree.drop_removed_nodes(self, rooted_nodes_for_integrity_check);

            let changed_nodes: Vec<_> = changed_nodes
                .into_iter()
                .filter_map(|id| Some((id, tree.node_for_id(id)?.borrow().clone_accesskit_node())))
                .collect();

            counters.nodes_in_tree_update = changed_nodes.len().try_into().unwrap_or_default();

            let focus = tree.focused_node_id.unwrap_or(
                tree.root_node
                    .as_ref()
                    .expect("Root node must be set")
                    .borrow()
                    .id(),
            );
            tree_update = Some(accesskit::TreeUpdate {
                // Filter out any nodes which were both changed and removed.
                nodes: changed_nodes,
                tree: accesskit_tree,
                focus,
                tree_id: tree.tree_id,
            });
        } else {
            assert!(self.tree_changes.is_empty());
        }

        for action in action_requests {
            assert_eq!(
                action.target_tree, tree.tree_id,
                "Got action with wrong tree ID: {action:?}"
            );
            let Some(&opaque) = tree.id_to_opaque_node.get(&action.target_node) else {
                // If the action is on a node which has been dropped, silently drop the action.
                continue;
            };
            let dom_action_request = AccessibilityActionRequest {
                action: action.action,
                target: opaque,
                data: action.data,
            };
            tree.pending_actions.push(dom_action_request);
        }

        (tree_update, counters)
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

#[cfg(test)]
#[test]
fn test_accessibility_update_add_some_nodes_twice() {
    let mut tree = AccessibilityTree::new(accesskit::TreeId::ROOT, Epoch::default());
    let mut root_update = AccessibilityUpdate::new(AccessibilityDamageMap::default(), &tree);

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
        root_node.set_children_for_testing(child_node_ids, child_nodes);
    }

    let mut update = AccessibilityUpdate::new(AccessibilityDamageMap::default(), &tree);

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

    let (tree_update, _) = update.finalize(&mut tree, None, vec![]);
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
            tree: None,
            tree_id: accesskit::TreeId::ROOT,
            focus: NodeId(2),
        }
    );
}
