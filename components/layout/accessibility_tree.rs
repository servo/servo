/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::VecDeque;
use std::sync::LazyLock;

use accesskit::Role;
use layout_api::{LayoutElement, LayoutNode, LayoutNodeType};
use log::trace;
use rustc_hash::{FxHashMap, FxHashSet};
use script::layout_dom::ServoLayoutNode;
use servo_base::Epoch;
use style::dom::OpaqueNode;
use web_atoms::{LocalName, local_name};

use crate::ArcRefCell;

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
    nodes: FxHashMap<accesskit::NodeId, ArcRefCell<AccessibilityNode>>,
    accesskit_tree: accesskit::Tree,
    tree_id: accesskit::TreeId,
    epoch: Epoch,
}

impl AccessibilityTree {
    const ROOT_NODE_ID: accesskit::NodeId = accesskit::NodeId(0);

    pub(super) fn new(tree_id: accesskit::TreeId, epoch: Epoch) -> Self {
        Self {
            nodes: FxHashMap::default(),
            accesskit_tree: accesskit::Tree::new(AccessibilityTree::ROOT_NODE_ID),
            tree_id,
            epoch,
        }
    }

    pub(super) fn update_tree(
        &mut self,
        root_dom_node: &ServoLayoutNode<'_>,
    ) -> Option<accesskit::TreeUpdate> {
        let root_node = self.get_or_create_node(root_dom_node);
        self.accesskit_tree.root = root_node.borrow().id;

        let mut tree_update = AccessibilityUpdate::new(self.accesskit_tree.clone(), self.tree_id);
        let any_node_updated = self.update_node_and_children(root_dom_node, &mut tree_update);

        if !any_node_updated {
            return None;
        }

        Some(tree_update.finalize())
    }

    fn update_node_and_children(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
        tree_update: &mut AccessibilityUpdate,
    ) -> bool {
        // TODO: read accessibility damage (right now, assume damage is complete)
        let (node_id, mut updated) = self.update_node(dom_node);

        let mut any_descendant_updated = false;
        for dom_child in dom_node.flat_tree_children() {
            // TODO: We actually need to propagate damage within the accessibility tree, rather than
            // assuming it matches the DOM tree, but this will do for now.
            any_descendant_updated |= self.update_node_and_children(&dom_child, tree_update);
        }

        if any_descendant_updated {
            let node_ref = self.assert_node_by_id(node_id);
            let mut node = node_ref.borrow_mut();
            if let Some(text) = self.label_from_descendants(&node) {
                node.accesskit_node.set_label(text);
                updated = true;
            }
        }

        if updated {
            let node = self.assert_node_by_id(node_id);
            tree_update.add(&node.borrow());
        }

        updated || any_descendant_updated
    }

    fn update_node(&mut self, dom_node: &ServoLayoutNode<'_>) -> (accesskit::NodeId, bool) {
        let node = self.get_or_create_node(dom_node);
        let mut node = node.borrow_mut();
        let accesskit_node = &mut node.accesskit_node;

        let mut new_children: Vec<accesskit::NodeId> = vec![];
        for dom_child in dom_node.flat_tree_children() {
            let child_id = Self::to_accesskit_id(&dom_child.opaque());
            new_children.push(child_id);
        }
        if new_children != accesskit_node.children() {
            accesskit_node.set_children(new_children);
        }

        if let Some(dom_element) = dom_node.as_element() {
            accesskit_node.set_role(Role::GenericContainer);
            let local_name = dom_element.local_name().to_ascii_lowercase();
            accesskit_node.set_html_tag(&*local_name);
            let role = HTML_ELEMENT_ROLE_MAPPINGS
                .get(&local_name)
                .unwrap_or(&Role::GenericContainer);
            accesskit_node.set_role(*role);
        } else if dom_node.type_id() == Some(LayoutNodeType::Text) {
            accesskit_node.set_role(Role::TextRun);
            let text_content = dom_node.text_content();
            trace!("node text content = {text_content:?}");
            // FIXME: this should take into account editing selection units (grapheme clusters?)
            accesskit_node.set_value(&*text_content);
        } else {
            accesskit_node.set_role(Role::GenericContainer);
        }

        // TODO: only return true if any update actually happened
        (node.id, true)
    }

    fn label_from_descendants(&self, node: &AccessibilityNode) -> Option<String> {
        if !NAME_FROM_CONTENTS_ROLES.contains(&node.accesskit_node.role()) {
            return None;
        }
        let mut children: VecDeque<accesskit::NodeId> = VecDeque::default();
        children.extend(node.accesskit_node.children());
        let mut text = String::new();
        while let Some(child_id) = children.pop_front() {
            let child = self.assert_node_by_id(child_id);
            let accesskit_node = &child.borrow().accesskit_node;
            match accesskit_node.role() {
                Role::TextRun => {
                    if let Some(child_text) = accesskit_node.value() {
                        text.push_str(child_text);
                    }
                },
                _ => {
                    for id in accesskit_node.children().iter().rev() {
                        children.push_front(*id);
                    }
                },
            }
        }
        let text: String = text.trim().to_owned();
        Some(text)
    }

    fn get_or_create_node(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
    ) -> ArcRefCell<AccessibilityNode> {
        let id = Self::to_accesskit_id(&dom_node.opaque());

        let node = self
            .nodes
            .entry(id)
            .or_insert_with(|| ArcRefCell::new(AccessibilityNode::new(id)));
        node.clone()
    }

    fn assert_node_by_id(&self, id: accesskit::NodeId) -> ArcRefCell<AccessibilityNode> {
        let Some(node) = self.nodes.get(&id) else {
            panic!("Stale node ID found: {id:?}");
        };
        node.clone()
    }

    // TODO: Using the OpaqueNode as the identifier for the accessibility node will inevitably
    // create issues as OpaqueNodes will be reused when DOM nodes are destroyed. Instead, we should
    // make a monotonically increasing ID, and have some other way to retrieve it based on DOM node.
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

static NAME_FROM_CONTENTS_ROLES: LazyLock<FxHashSet<Role>> =
    LazyLock::new(|| [(Role::Heading)].into_iter().collect());
