/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use accesskit::Role;
use layout_api::wrapper_traits::ThreadSafeLayoutNode;
use log::trace;
use rustc_hash::FxHashMap;
use script::layout_dom::ServoThreadSafeLayoutNode;
use style::dom::{NodeInfo, OpaqueNode};

struct AccessibilityUpdate {
    accesskit_update: accesskit::TreeUpdate,
}

#[derive(Debug)]
struct AccessibilityNode {
    id: accesskit::NodeId,
    accesskit_node: accesskit::Node,
}

#[derive(Debug)]
pub struct AccessibilityTree {
    nodes: FxHashMap<accesskit::NodeId, AccessibilityNode>,
    accesskit_tree: accesskit::Tree,
    tree_id: accesskit::TreeId,
}

impl AccessibilityUpdate {
    fn new(tree: accesskit::Tree, tree_id: accesskit::TreeId) -> Self {
        Self {
            accesskit_update: accesskit::TreeUpdate {
                nodes: Default::default(),
                tree: Some(tree),
                focus: accesskit::NodeId(1),
                tree_id,
            },
        }
    }

    fn add(&mut self, node: &AccessibilityNode) {
        self.accesskit_update
            .nodes
            .push((node.id, node.accesskit_node.clone()));
    }
}

impl AccessibilityTree {
    const ROOT_NODE_ID: accesskit::NodeId = accesskit::NodeId(0);

    pub(super) fn new(tree_id: accesskit::TreeId) -> Self {
        let mut root_node = AccessibilityNode::new(AccessibilityTree::ROOT_NODE_ID);
        root_node
            .accesskit_node
            .set_role(accesskit::Role::RootWebArea);
        root_node
            .accesskit_node
            .add_action(accesskit::Action::Focus);

        let mut tree = Self {
            nodes: Default::default(),
            accesskit_tree: accesskit::Tree::new(root_node.id),
            tree_id,
        };
        tree.nodes.insert(root_node.id, root_node);

        tree
    }

    pub(super) fn update_tree(
        &mut self,
        root_dom_node: ServoThreadSafeLayoutNode<'_>,
    ) -> Option<accesskit::TreeUpdate> {
        let mut tree_update = AccessibilityUpdate::new(self.accesskit_tree.clone(), self.tree_id);
        self.update_root_dom_node(root_dom_node, &mut tree_update);

        Some(tree_update.accesskit_update)
    }

    fn update_root_dom_node(
        &mut self,
        root_dom_node: ServoThreadSafeLayoutNode<'_>,
        tree_update: &mut AccessibilityUpdate,
    ) {
        let root_dom_node_id = Self::to_accesskit_id(&root_dom_node.opaque());
        let root_accessibility_node_id = accesskit::NodeId(0);
        let root_accessibility_node = self.nodes.get_mut(&root_accessibility_node_id).unwrap();
        root_accessibility_node
            .accesskit_node
            .set_children(vec![root_dom_node_id]);

        // TODO: Update bounds for root web area

        tree_update.add(root_accessibility_node);

        self.update_node(root_dom_node, tree_update);
    }

    fn update_node(
        &mut self,
        dom_node: ServoThreadSafeLayoutNode<'_>,
        tree_update: &mut AccessibilityUpdate,
    ) {
        // TODO: actually compute whether updated or not :)
        let updated = true;

        // TODO: read accessibility damage from dom_node (right now, assume damage is complete)

        let mut new_children: Vec<accesskit::NodeId> = vec![];
        for dom_child in dom_node.children() {
            let child_node = self.get_or_create_node(dom_child);
            // FIXME: don't want to call get_or_create twice!
            new_children.push(child_node.id);
            self.update_node(dom_child, tree_update);
        }

        let accessibility_node = self.get_or_create_node(dom_node);
        // TODO: bounds!

        // TODO: initialise/update properties here rather than in get_or_create_node

        // TODO: compare new_children to existing children
        accessibility_node.accesskit_node.set_children(new_children);

        if updated {
            tree_update.add(accessibility_node);
        }
    }

    fn get_or_create_node(
        &mut self,
        dom_node: ServoThreadSafeLayoutNode<'_>,
    ) -> &mut AccessibilityNode {
        let id = Self::to_accesskit_id(&dom_node.opaque());

        let node = self
            .nodes
            .entry(id)
            .or_insert_with(|| AccessibilityNode::new(id));

        let accesskit_node = &mut node.accesskit_node;

        // TODO: Move these branches into separate methods and call from update_node
        if dom_node.is_text_node() {
            accesskit_node.set_role(Role::TextRun);
            let text_content = dom_node.text_content();
            trace!("node text content = {:?}", text_content);
            accesskit_node.set_value(&*text_content);
            // FIXME: this should take into account editing selection units (grapheme clusters?)
        } else if let Some(_dom_element) = dom_node.as_element() {
            accesskit_node.set_role(Role::GenericContainer);
        }

        node
    }

    fn to_accesskit_id(opaque: &OpaqueNode) -> accesskit::NodeId {
        accesskit::NodeId(opaque.0 as u64)
    }
}

impl AccessibilityNode {
    fn new(id: accesskit::NodeId) -> Self {
        Self {
            id,
            accesskit_node: accesskit::Node::new(Role::Unknown),
        }
    }
}
