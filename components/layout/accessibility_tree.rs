/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::atomic::AtomicU64;
use std::sync::{LazyLock, atomic};

use accesskit::{NodeId, Role};
use layout_api::{LayoutElement, LayoutNode, LayoutNodeType};
use log::trace;
use rustc_hash::{FxHashMap, FxHashSet};
use script::layout_dom::ServoLayoutNode;
use servo_base::Epoch;
use servo_base::print_tree::PrintTree;
use servo_config::opts::{self, DiagnosticsLogging, DiagnosticsLoggingOption};
use servo_config::pref;
use style::dom::{NodeInfo, OpaqueNode};
use web_atoms::{LocalName, local_name};

use crate::ArcRefCell;

/// Changes which have occurred during the current update.
struct AccessibilityUpdate {
    /// Nodes whose internal data has changed within the current update.
    changed_nodes: FxHashSet<NodeId>,
    /// Nodes that changed their relation to the tree within the current update.
    tree_changes: FxHashMap<NodeId, TreeChange>,
}

struct AccessibilityNode {
    id: NodeId,
    accesskit_node: accesskit::Node,
    opaque_node: Option<OpaqueNode>,
    updated: bool,
}

/// A retained, internal representation of the accessibility tree for a document.
///
/// [`accesskit`] only provides interchange types for tree updates and action requests, so we need
/// to define our own representation for incremental tree building.
#[derive(Debug)]
pub struct AccessibilityTree {
    nodes: FxHashMap<NodeId, ArcRefCell<AccessibilityNode>>,
    opaque_node_to_id: FxHashMap<OpaqueNode, NodeId>,
    tree_id: accesskit::TreeId,
    root_node_id: Option<accesskit::NodeId>,
    epoch: Epoch,
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
    ///   `TreeChange::Moved(Move::Pending)` state until its old parent is updated. We expect that it
    ///   must later be removed from its old parent, at which point its state will be updated to
    ///   `TreeChange::Moved(Move::Complete)`.
    /// - If a node's old parent is updated before its new parent, the node will be first
    ///   `TreeChange::Removed` and then `TreeChange ::Moved(Move::Complete)`.
    ///
    /// At the end of the update, we assert that there are no pending moves remaining.
    PendingMove,

    /// The node is no longer a child of its previous parent.
    Removed,
}

impl AccessibilityTree {
    pub(super) fn new(tree_id: accesskit::TreeId, epoch: Epoch) -> Self {
        Self {
            nodes: FxHashMap::default(),
            opaque_node_to_id: FxHashMap::default(),
            tree_id,
            root_node_id: None,
            epoch,
            debug: opts::get().debug.clone(),
        }
    }

    /// Update this tree based on the current state of the given DOM tree, and if anything changed,
    /// return an [`accesskit::TreeUpdate`] representing what changed.
    pub(super) fn update_tree(
        &mut self,
        root_dom_node: &ServoLayoutNode<'_>,
    ) -> Option<accesskit::TreeUpdate> {
        let mut update = AccessibilityUpdate::new();
        let root_node = self.get_or_create_node(root_dom_node, &mut update);
        let root_node_id = root_node.borrow().id;
        self.root_node_id = Some(root_node_id);

        self.update_node_and_descendants(root_dom_node, &mut update);

        update.finalize(self)
    }

    /// Update this tree starting at the given DOM node, adding any changed nodes to the given
    /// [`AccessibilityUpdate`].
    fn update_node_and_descendants(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
        update: &mut AccessibilityUpdate,
    ) -> bool {
        let node = self.assert_node_for_dom_node(dom_node);
        let mut node = node.borrow_mut();

        // TODO: read accessibility damage (right now, assume damage is complete)
        let any_descendant_updated = node.update_descendants(dom_node, self, update);

        node.update_node(dom_node, self, any_descendant_updated);

        if node.updated {
            update.add(&mut node);
            return true;
        }

        any_descendant_updated
    }

    fn get_or_create_node(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
        update: &mut AccessibilityUpdate,
    ) -> ArcRefCell<AccessibilityNode> {
        let id = self.id_for_opaque(dom_node.opaque());

        let node = self.nodes.entry(id).or_insert_with(|| {
            update.tree_changes.insert(id, TreeChange::New);
            ArcRefCell::new(AccessibilityNode::new(id))
        });

        let mut new_node = node.borrow_mut();

        new_node.opaque_node = Some(dom_node.opaque());
        if let Some(dom_element) = dom_node.as_element() {
            let local_name = dom_element.local_name().to_ascii_lowercase();
            new_node.set_html_tag(&local_name);
        }

        node.clone()
    }

    fn node_for_dom_node(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
    ) -> Option<ArcRefCell<AccessibilityNode>> {
        let id = self.id_for_opaque(dom_node.opaque());
        self.nodes.get(&id).cloned()
    }

    fn assert_node_for_dom_node(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
    ) -> ArcRefCell<AccessibilityNode> {
        let id = self.id_for_opaque(dom_node.opaque());
        let node = self.assert_node_for_id(id);
        assert!(node.borrow().opaque_node == Some(dom_node.opaque()));
        node
    }

    fn assert_node_for_id(&self, id: NodeId) -> ArcRefCell<AccessibilityNode> {
        let Some(node) = self.nodes.get(&id) else {
            panic!("{id:?} does not exist in tree");
        };
        node.clone()
    }

    /// Remove the node with the given ID from the cache and return it, if it exists.
    fn remove_node(&mut self, id: NodeId) -> Option<ArcRefCell<AccessibilityNode>> {
        let node = self.nodes.remove(&id)?;
        if let Some(opaque_node) = node.borrow().opaque_node {
            self.opaque_node_to_id.remove(&opaque_node);
        }
        Some(node)
    }

    fn id_for_opaque(&mut self, opaque: OpaqueNode) -> NodeId {
        let id = self.opaque_node_to_id.entry(opaque).or_insert_with(|| {
            static LAST_ID: AtomicU64 = AtomicU64::new(0);
            LAST_ID.fetch_add(1, atomic::Ordering::SeqCst).into()
        });
        *id
    }

    pub(crate) fn epoch(&self) -> Epoch {
        self.epoch
    }

    /// Assert that the tree is a tree without any dangling references or orphaned nodes.
    ///
    /// For accessibility tests only, because it’s expensive.
    fn assert_integrity(&self) {
        let Some(root_node_id) = self.root_node_id else {
            return;
        };

        assert!(pref!(expensive_accessibility_test_assertions_enabled));
        // Traverse the tree from the given root.
        let mut node_ids = vec![root_node_id];
        let mut seen_node_ids = FxHashSet::default();
        while let Some(node_id) = node_ids.pop() {
            // If this fails, then the tree is not a tree at all.
            assert!(
                seen_node_ids.insert(node_id),
                "Tree contains {node_id:?} in multiple places"
            );
            // If this fails, then the tree has dangling references.
            let node = self.assert_node_for_id(node_id);
            let node = node.borrow();
            node_ids.extend(node.children().iter().rev());
        }
        // If this fails, then the tree has orphaned nodes (a leak).
        // Dangling references are already caught in the loop above.
        assert_eq!(seen_node_ids, self.nodes.keys().copied().collect());
    }

    fn print(&self) {
        let Some(root_node_id) = self.root_node_id else {
            return;
        };

        let mut print_tree = PrintTree::new("Accessibility Tree".to_string());
        let node = self.assert_node_for_id(root_node_id);
        node.borrow().print(self, &mut print_tree);
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
            opaque_node: None,
            updated: true,
        }
    }

    fn update_descendants<'dom>(
        &mut self,
        dom_node: &ServoLayoutNode<'dom>,
        tree: &mut AccessibilityTree,
        update: &mut AccessibilityUpdate,
    ) -> bool {
        let mut any_descendant_updated = false;
        let mut newly_created = FxHashSet::default();
        let new_children: Vec<_> = dom_node
            .flat_tree_children()
            .map(|dom_child| {
                let child_node_id = match tree.node_for_dom_node(&dom_child) {
                    Some(child_node) => child_node.borrow().id,
                    None => {
                        let new_child = tree.get_or_create_node(&dom_child, update);
                        let child_node_id = new_child.borrow().id;
                        newly_created.insert(child_node_id);
                        child_node_id
                    },
                };

                // TODO: We actually need to propagate damage within the accessibility tree, rather than
                // assuming it matches the DOM tree, but this will do for now.
                any_descendant_updated |= tree.update_node_and_descendants(&dom_child, update);

                child_node_id
            })
            .collect();

        if new_children != self.children() {
            let old_children = self.children();
            for old_child_id in old_children {
                if !new_children.contains(old_child_id) {
                    let removed_child = tree.assert_node_for_id(*old_child_id);
                    removed_child.borrow().set_subtree_state_change(
                        TreeChange::Removed,
                        tree,
                        update,
                    );
                }
            }
            for new_child_id in new_children.iter() {
                if !newly_created.contains(new_child_id) && !old_children.contains(new_child_id) {
                    let moved_child = tree.assert_node_for_id(*new_child_id);
                    moved_child.borrow().set_subtree_state_change(
                        TreeChange::PendingMove,
                        tree,
                        update,
                    );
                }
            }
            self.set_children(new_children);
        }

        any_descendant_updated
    }

    /// Recursively mark this subtree as having the given `TreeChange `.
    ///
    /// This is used when a node is `Moved` or `Removed`, since its entire subtree will also need to
    /// be marked accordingly. When a node is `New`, it's marked as such when it is created. We
    /// shouldn't call this method in that case, since it may have descendants which are not being
    /// created in this update and shouldn't have a `New` state. Any descendants which are new will
    /// already have their `New` state set when they are created.
    ///
    /// Note: if a node is moved, the requested `change` must always be `Moved(Pending)`: the logic
    /// in this method will determine whether the move is `Complete` and set the stored value
    /// accordingly.
    fn set_subtree_state_change(
        &self,
        change: TreeChange,
        tree: &mut AccessibilityTree,
        update: &mut AccessibilityUpdate,
    ) {
        let old_change = update.tree_changes.get(&self.id);

        assert!(
            change != TreeChange::New,
            "New shouldn't be set recursively"
        );
        assert!(
            change != TreeChange::Moved,
            "Incoming change must never be Moved"
        );

        let new_change = old_change
            .map(|old_change| match (old_change, change) {
                (TreeChange::PendingMove, TreeChange::Removed) => TreeChange::Moved,
                (TreeChange::Removed, TreeChange::PendingMove) => TreeChange::Moved,
                _ => {
                    unreachable!("Logically impossible state change: {old_change:?} → {change:?}")
                },
            })
            .unwrap_or(change);

        update.tree_changes.insert(self.id, new_change);

        for child_id in self.children().to_vec() {
            let child = tree.assert_node_for_id(child_id);
            // `new_change` might be different per node, if only some nodes were moved elsewhere.
            child
                .borrow()
                .set_subtree_state_change(change, tree, update);
        }
    }

    fn update_node(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
        tree: &mut AccessibilityTree,
        any_descendant_updated: bool,
    ) {
        self.set_role(role_from_dom_node(dom_node));
        if dom_node.is_element() {
            if any_descendant_updated && let Some(text) = self.label_from_descendants(tree) {
                self.set_label(text.as_str());
            }
        } else if dom_node.type_id() == Some(LayoutNodeType::Text) {
            let text_content = dom_node.text_content();
            trace!("node text content = {text_content:?}");
            // FIXME: this should take into account editing selection units (grapheme clusters?)
            self.set_value(&text_content);
        }
    }

    fn label_from_descendants(&self, tree: &AccessibilityTree) -> Option<String> {
        if !NAME_FROM_CONTENTS_ROLES.contains(&self.role()) {
            return None;
        }
        let mut children = VecDeque::from_iter(self.children().iter().copied());
        let mut text = String::new();
        while let Some(child_id) = children.pop_front() {
            let child = tree.assert_node_for_id(child_id);
            let child = child.borrow();
            match child.role() {
                Role::TextRun => {
                    if let Some(child_text) = child.value() {
                        text.push_str(child_text);
                    }
                },
                _ => {
                    for id in child.children().iter().rev() {
                        children.push_front(*id);
                    }
                },
            }
        }
        Some(text.trim().to_owned())
    }

    fn print(&self, tree: &AccessibilityTree, print_tree: &mut PrintTree) {
        if self.children().is_empty() {
            print_tree.add_item(format!("{self:?}"));
            return;
        }

        print_tree.new_level(format!("{self:?}"));

        for child_id in self.children() {
            let child = tree.assert_node_for_id(*child_id);
            child.borrow().print(tree, print_tree);
        }
        print_tree.end_level();
    }

    // TODO: use macros to generate getter/setter methods.

    fn children(&self) -> &[NodeId] {
        self.accesskit_node.children()
    }

    fn set_children(&mut self, children: Vec<NodeId>) {
        self.accesskit_node.set_children(children);
        self.updated = true;
    }

    fn role(&self) -> Role {
        self.accesskit_node.role()
    }

    fn set_role(&mut self, role: Role) {
        if role == self.accesskit_node.role() {
            return;
        }
        self.accesskit_node.set_role(role);
        self.updated = true;
    }

    fn label(&self) -> Option<&str> {
        self.accesskit_node.label()
    }

    fn set_label(&mut self, label: &str) {
        if Some(label) == self.accesskit_node.label() {
            return;
        }
        self.accesskit_node.set_label(label);
        self.updated = true;
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

    fn set_value(&mut self, value: &str) {
        if Some(value) == self.accesskit_node.value() {
            return;
        }
        self.accesskit_node.set_value(value);
        self.updated = true;
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
        if !self.children().is_empty() {
            write!(f, "\nchildren: {:?}", self.children())?;
        }
        Ok(())
    }
}

impl AccessibilityUpdate {
    fn new() -> Self {
        Self {
            changed_nodes: FxHashSet::default(),
            tree_changes: FxHashMap::default(),
        }
    }

    fn add(&mut self, node: &mut AccessibilityNode) {
        self.changed_nodes.insert(node.id);

        node.updated = false;
    }

    fn finalize(mut self, tree: &mut AccessibilityTree) -> Option<accesskit::TreeUpdate> {
        let root_node_id = tree
            .root_node_id
            .expect("AccessibilityUpdate::finalize() called but no root_node_id set in tree");

        if self.changed_nodes.is_empty() {
            assert!(self.tree_changes.is_empty());
            return None;
        }

        for (id, change) in self.tree_changes.drain() {
            match change {
                TreeChange::New => {
                    let _ = tree.assert_node_for_id(id);
                },
                TreeChange::Removed => {
                    let Some(_) = tree.remove_node(id) else {
                        panic!("Removed node with id {id:?} not found in tree");
                    };
                },
                TreeChange::PendingMove => {
                    unreachable!(
                        "Pending move found for node id {id:?} when draining tree state changes"
                    );
                },
                TreeChange::Moved => {
                    // We only track moved nodes to avoid incorrectly marking them as removed, so
                    // we just silently drop them here.
                },
            }
        }

        let accesskit_tree = accesskit::Tree::new(root_node_id);
        let tree_update = accesskit::TreeUpdate {
            nodes: self
                .changed_nodes
                .into_iter()
                .map(|id| {
                    (
                        id,
                        tree.assert_node_for_id(id).borrow().accesskit_node.clone(),
                    )
                })
                .collect(),
            tree: Some(accesskit_tree),
            focus: accesskit::NodeId(1),
            tree_id: tree.tree_id,
        };

        if tree
            .debug
            .is_enabled(DiagnosticsLoggingOption::AccessibilityTree)
        {
            tree.print();
        }

        if pref!(expensive_accessibility_test_assertions_enabled) {
            tree.assert_integrity();
        }

        Some(tree_update)
    }
}

#[cfg(test)]
#[test]
fn test_accessibility_update_add_some_nodes_twice() {
    let mut tree = AccessibilityTree::new(accesskit::TreeId::ROOT, Epoch::default());
    tree.root_node_id = Some(accesskit::NodeId(2));

    for (id, role) in [
        (3, Role::GenericContainer),
        (4, Role::Heading),
        (5, Role::Paragraph),
    ] {
        let id = accesskit::NodeId(id);
        tree.nodes.insert(
            id,
            ArcRefCell::new(AccessibilityNode::new_with_role(id, role)),
        );
    }

    let mut update = AccessibilityUpdate::new();

    {
        let node_3 = tree.assert_node_for_id(accesskit::NodeId(3));
        let mut node_3 = node_3.borrow_mut();
        let node_4 = tree.assert_node_for_id(accesskit::NodeId(4));
        let mut node_4 = node_4.borrow_mut();
        let node_5 = tree.assert_node_for_id(accesskit::NodeId(5));
        let mut node_5 = node_5.borrow_mut();

        update.add(&mut node_5);
        update.add(&mut node_3);
        update.add(&mut node_4);
        update.add(&mut node_4);

        node_3.set_role(Role::ScrollView);
        update.add(&mut node_3);
    }

    let mut update_result = update
        .finalize(&mut tree)
        .expect("finalize should produce an update result");
    let mut tree_update = update_result.tree_update;
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
