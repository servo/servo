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
    updated: bool,
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
        self.update_node(dom_node);

        let mut any_descendant_updated = false;
        for dom_child in dom_node.flat_tree_children() {
            // TODO: We actually need to propagate damage within the accessibility tree, rather than
            // assuming it matches the DOM tree, but this will do for now.
            any_descendant_updated |= self.update_node_and_children(&dom_child, tree_update);
        }

        let node = self.assert_node_by_dom_node(dom_node);
        let mut node = node.borrow_mut();

        if any_descendant_updated {
            if let Some(text) = self.label_from_descendants(&node) {
                node.set_label(&text);
                node.updated = true;
            }
        }

        if node.updated {
            tree_update.add(&mut node);
            return true;
        }

        any_descendant_updated
    }

    fn update_node(&mut self, dom_node: &ServoLayoutNode<'_>) {
        let node = self.get_or_create_node(dom_node);
        let mut node = node.borrow_mut();

        let mut new_children: Vec<accesskit::NodeId> = vec![];
        for dom_child in dom_node.flat_tree_children() {
            let child_id = Self::to_accesskit_id(&dom_child.opaque());
            new_children.push(child_id);
        }
        if new_children != node.children() {
            node.set_children(new_children);
        }

        if let Some(dom_element) = dom_node.as_element() {
            let local_name = dom_element.local_name().to_ascii_lowercase();
            node.set_html_tag(&local_name);
            let role = HTML_ELEMENT_ROLE_MAPPINGS
                .get(&local_name)
                .unwrap_or(&Role::GenericContainer);
            node.set_role(*role);
        } else if dom_node.type_id() == Some(LayoutNodeType::Text) {
            node.set_role(Role::TextRun);
            let text_content = dom_node.text_content();
            trace!("node text content = {text_content:?}");
            // FIXME: this should take into account editing selection units (grapheme clusters?)
            node.set_value(&text_content);
        }
    }

    fn label_from_descendants(&self, node: &AccessibilityNode) -> Option<String> {
        if !NAME_FROM_CONTENTS_ROLES.contains(&node.role()) {
            return None;
        }
        let mut children = VecDeque::from_iter(node.children().iter().copied());
        let mut text = String::new();
        while let Some(child_id) = children.pop_front() {
            let child = self.assert_node_by_id(child_id);
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

    fn assert_node_by_dom_node(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
    ) -> ArcRefCell<AccessibilityNode> {
        let id = Self::to_accesskit_id(&dom_node.opaque());
        self.assert_node_by_id(id)
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
            updated: true,
        }
    }

    #[cfg(test)]
    fn new_with_role(id: accesskit::NodeId, role: accesskit::Role) -> Self {
        Self {
            id,
            accesskit_node: accesskit::Node::new(role),
            updated: true,
        }
    }

    // TODO: use macros to generate getter/setter methods.

    fn children(&self) -> &[accesskit::NodeId] {
        self.accesskit_node.children()
    }

    fn set_children(&mut self, children: Vec<accesskit::NodeId>) {
        self.accesskit_node.set_children(children);
        self.updated = true;
    }

    fn role(&self) -> accesskit::Role {
        self.accesskit_node.role()
    }

    fn set_role(&mut self, role: accesskit::Role) {
        if role == self.accesskit_node.role() {
            return;
        }
        self.accesskit_node.set_role(role);
        self.updated = true;
    }

    #[expect(dead_code)]
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

    #[expect(dead_code)]
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

    fn add(&mut self, node: &mut AccessibilityNode) {
        self.nodes.insert(node.id, node.accesskit_node.clone());
        node.updated = false;
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
    update.add(&mut AccessibilityNode::new_with_role(
        accesskit::NodeId(5),
        Role::Paragraph,
    ));
    update.add(&mut AccessibilityNode::new_with_role(
        accesskit::NodeId(3),
        Role::GenericContainer,
    ));
    update.add(&mut AccessibilityNode::new_with_role(
        accesskit::NodeId(4),
        Role::Heading,
    ));
    update.add(&mut AccessibilityNode::new_with_role(
        accesskit::NodeId(4),
        Role::Heading,
    ));
    update.add(&mut AccessibilityNode::new_with_role(
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

/// <https://w3c.github.io/aria/#namefromcontent>
static NAME_FROM_CONTENTS_ROLES: LazyLock<FxHashSet<Role>> =
    LazyLock::new(|| [(Role::Heading)].into_iter().collect());
