/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use accesskit::Role;
use layout_api::LayoutNode;
use log::trace;
use rustc_hash::FxHashMap;
use script::layout_dom::ServoLayoutNode;
use servo_base::Epoch;
use style::dom::{NodeInfo, OpaqueNode};

struct AccessibilityUpdate {
    accesskit_update: accesskit::TreeUpdate,
    nodes: FxHashMap<accesskit::NodeId, accesskit::Node>,
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
    epoch: Epoch,
}

impl AccessibilityUpdate {
    fn new(tree: accesskit::Tree, tree_id: accesskit::TreeId) -> Self {
        Self {
            accesskit_update: accesskit::TreeUpdate {
                nodes: vec![],
                tree: Some(tree),
                focus: accesskit::NodeId(1),
                tree_id,
            },
            nodes: FxHashMap::default(),
        }
    }

    fn add(&mut self, node: &AccessibilityNode) {
        self.nodes.insert(node.id, node.accesskit_node.clone());
    }

    fn finalize(mut self) -> accesskit::TreeUpdate {
        self.accesskit_update.nodes.extend(self.nodes.drain());
        self.accesskit_update
    }
}

#[cfg(test)]
#[test]
fn test_accessibility_update_add_some_nodes_twice() {
    let mut update = AccessibilityUpdate::new(
        accesskit::Tree {
            root: accesskit::NodeId(2),
            toolkit_name: None,
            toolkit_version: None,
        },
        accesskit::TreeId::ROOT,
    );
    update.add(&AccessibilityNode::new_with_role(
        accesskit::NodeId(5),
        Role::Paragraph,
    ));
    update.add(&AccessibilityNode::new_with_role(
        accesskit::NodeId(3),
        Role::GenericContainer,
    ));
    update.add(&AccessibilityNode::new_with_role(
        accesskit::NodeId(4),
        Role::Heading,
    ));
    update.add(&AccessibilityNode::new_with_role(
        accesskit::NodeId(4),
        Role::Heading,
    ));
    update.add(&AccessibilityNode::new_with_role(
        accesskit::NodeId(3),
        Role::ScrollView,
    ));
    let mut tree_update = update.finalize();
    tree_update.nodes.sort_by_key(|(node_id, _node)| *node_id);
    assert_eq!(
        tree_update,
        accesskit::TreeUpdate {
            nodes: vec![
                (accesskit::NodeId(3), accesskit::Node::new(Role::ScrollView)),
                (accesskit::NodeId(4), accesskit::Node::new(Role::Heading)),
                (accesskit::NodeId(5), accesskit::Node::new(Role::Paragraph)),
            ],
            tree: Some(accesskit::Tree {
                root: accesskit::NodeId(2),
                toolkit_name: None,
                toolkit_version: None
            }),
            tree_id: accesskit::TreeId::ROOT,
            focus: accesskit::NodeId(1),
        }
    );
}

impl AccessibilityTree {
    const ROOT_NODE_ID: accesskit::NodeId = accesskit::NodeId(0);

    pub(super) fn new(tree_id: accesskit::TreeId, epoch: Epoch) -> Self {
        // The root node doesn't correspond to a DOM node, but contains the root DOM node.
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
            epoch,
        };
        tree.nodes.insert(root_node.id, root_node);

        tree
    }

    pub(super) fn update_tree(
        &mut self,
        root_dom_node: &ServoLayoutNode<'_>,
    ) -> Option<accesskit::TreeUpdate> {
        let mut tree_update = AccessibilityUpdate::new(self.accesskit_tree.clone(), self.tree_id);

        let root_dom_node_id = Self::to_accesskit_id(&root_dom_node.opaque());
        let root_node = self
            .nodes
            .get_mut(&AccessibilityTree::ROOT_NODE_ID)
            .unwrap();
        root_node
            .accesskit_node
            .set_children(vec![root_dom_node_id]);

        tree_update.add(root_node);

        self.update_node_and_children(root_dom_node, &mut tree_update);
        Some(tree_update.finalize())
    }

    fn update_node_and_children(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
        tree_update: &mut AccessibilityUpdate,
    ) {
        // TODO: read accessibility damage from dom_node (right now, assume damage is complete)

        let node = self.get_or_create_node_mut(dom_node);
        let accesskit_node = &mut node.accesskit_node;

        let mut new_children: Vec<accesskit::NodeId> = vec![];
        for dom_child in dom_node.flat_tree_children() {
            let child_id = Self::to_accesskit_id(&dom_child.opaque());
            new_children.push(child_id);
        }
        if new_children != accesskit_node.children() {
            accesskit_node.set_children(new_children);
        }

        if dom_node.is_text_node() {
            accesskit_node.set_role(Role::TextRun);
            let text_content = dom_node.text_content();
            trace!("node text content = {text_content:?}");
            // FIXME: this should take into account editing selection units (grapheme clusters?)
            accesskit_node.set_value(&*text_content);
        } else if dom_node.as_element().is_some() {
            accesskit_node.set_role(Role::GenericContainer);
        }

        tree_update.add(node);

        for dom_child in dom_node.flat_tree_children() {
            self.update_node_and_children(&dom_child, tree_update);
        }
    }

    fn get_or_create_node_mut(&mut self, dom_node: &ServoLayoutNode<'_>) -> &mut AccessibilityNode {
        let id = Self::to_accesskit_id(&dom_node.opaque());

        self.nodes
            .entry(id)
            .or_insert_with(|| AccessibilityNode::new(id))
    }

    fn to_accesskit_id(opaque: &OpaqueNode) -> accesskit::NodeId {
        accesskit::NodeId(opaque.0 as u64)
    }

    pub(crate) fn epoch(&self) -> Epoch {
        self.epoch
    }
}

impl AccessibilityNode {
    fn new(id: accesskit::NodeId) -> Self {
        Self {
            id,
            accesskit_node: accesskit::Node::new(Role::Unknown),
        }
    }

    #[cfg(test)]
    fn new_with_role(id: accesskit::NodeId, role: accesskit::Role) -> Self {
        Self {
            id,
            accesskit_node: accesskit::Node::new(role),
        }
    }
}
