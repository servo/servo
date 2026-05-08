/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use std::collections::VecDeque;
use std::sync::atomic::AtomicU64;
use std::sync::{LazyLock, atomic};

use accesskit::Role;
use layout_api::{LayoutElement, LayoutNode, LayoutNodeType};
use log::trace;
use rustc_hash::{FxHashMap, FxHashSet};
use script::layout_dom::ServoLayoutNode;
use servo_base::Epoch;
use style::dom::{NodeInfo, OpaqueNode};
use web_atoms::{LocalName, local_name};

use crate::ArcRefCell;

/// An in-progress [`accesskit::TreeUpdate`] that automatically avoids storing any node twice.
struct AccessibilityUpdate {
    accesskit_update: accesskit::TreeUpdate,
    nodes: FxHashMap<accesskit::NodeId, accesskit::Node>,
}

#[derive(Debug)]
struct AccessibilityNode {
    id: accesskit::NodeId,
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
    nodes: FxHashMap<accesskit::NodeId, ArcRefCell<AccessibilityNode>>,
    opaque_node_to_id: FxHashMap<OpaqueNode, accesskit::NodeId>,
    tree_id: accesskit::TreeId,
    epoch: Epoch,
}

impl AccessibilityTree {
    pub(super) fn new(tree_id: accesskit::TreeId, epoch: Epoch) -> Self {
        Self {
            nodes: FxHashMap::default(),
            opaque_node_to_id: FxHashMap::default(),
            tree_id,
            epoch,
        }
    }

    /// Update this tree based on the current state of the given DOM tree, and if anything changed,
    /// return an [`accesskit::TreeUpdate`] representing what changed.
    pub(super) fn update_tree(
        &mut self,
        root_dom_node: &ServoLayoutNode<'_>,
    ) -> Option<accesskit::TreeUpdate> {
        let root_node = self.get_or_create_node(root_dom_node);
        let mut tree_update =
            AccessibilityUpdate::new(accesskit::Tree::new(root_node.borrow().id), self.tree_id);
        let any_node_updated = self.update_node_and_descendants(root_dom_node, &mut tree_update);

        if !any_node_updated {
            return None;
        }

        Some(tree_update.finalize())
    }

    /// Update this tree starting at the given DOM node, adding any changed nodes to the given
    /// [`AccessibilityUpdate`].
    fn update_node_and_descendants(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
        tree_update: &mut AccessibilityUpdate,
    ) -> bool {
        let node = self.assert_node_by_dom_node(dom_node);
        let mut node = node.borrow_mut();

        // TODO: read accessibility damage (right now, assume damage is complete)
        let any_descendant_updated = node.update_descendants(dom_node, self, tree_update);

        node.update_node(dom_node, self, any_descendant_updated);

        if node.updated {
            tree_update.add(&mut node);
            return true;
        }

        any_descendant_updated
    }

    fn get_or_create_node(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
    ) -> ArcRefCell<AccessibilityNode> {
        let id = self.id_for_opaque(dom_node.opaque());

        let node = self
            .nodes
            .entry(id)
            .or_insert_with(|| ArcRefCell::new(AccessibilityNode::new(id)));
        {
            let mut new_node = node.borrow_mut();

            new_node.opaque_node = Some(dom_node.opaque());
            if let Some(dom_element) = dom_node.as_element() {
                let local_name = dom_element.local_name().to_ascii_lowercase();
                new_node.set_html_tag(&local_name);
            }
        }

        node.clone()
    }

    fn assert_node_by_dom_node(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
    ) -> ArcRefCell<AccessibilityNode> {
        let id = self.id_for_opaque(dom_node.opaque());
        let node = self.assert_node_by_id(id);
        assert!(node.borrow().opaque_node == Some(dom_node.opaque()));
        node
    }

    fn assert_node_by_id(&self, id: accesskit::NodeId) -> ArcRefCell<AccessibilityNode> {
        let Some(node) = self.nodes.get(&id) else {
            panic!("Stale node ID found: {id:?}");
        };
        node.clone()
    }

    fn id_for_opaque(&mut self, opaque: OpaqueNode) -> accesskit::NodeId {
        let id = self.opaque_node_to_id.entry(opaque).or_insert_with(|| {
            static LAST_ID: AtomicU64 = AtomicU64::new(0);
            LAST_ID.fetch_add(1, atomic::Ordering::SeqCst).into()
        });
        *id
    }

    pub(crate) fn epoch(&self) -> Epoch {
        self.epoch
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
    fn new(id: accesskit::NodeId) -> Self {
        Self::new_with_role(id, Role::Unknown)
    }

    fn new_with_role(id: accesskit::NodeId, role: accesskit::Role) -> Self {
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
        tree_update: &mut AccessibilityUpdate,
    ) -> bool {
        let mut any_descendant_updated = false;
        let new_children = dom_node
            .flat_tree_children()
            .map(|dom_child| {
                let child_node = tree.get_or_create_node(&dom_child);
                let child_node_id = child_node.borrow().id;

                // TODO: We actually need to propagate damage within the accessibility tree, rather than
                // assuming it matches the DOM tree, but this will do for now.
                any_descendant_updated |= tree.update_node_and_descendants(&dom_child, tree_update);

                child_node_id
            })
            .collect();
        if new_children != self.children() {
            self.set_children(new_children);
        }
        any_descendant_updated
    }

    fn update_node(
        &mut self,
        dom_node: &ServoLayoutNode<'_>,
        tree: &mut AccessibilityTree,
        any_descendant_updated: bool,
    ) {
        self.set_role(role_from_dom_node(dom_node));
        if dom_node.is_element() {
            if any_descendant_updated {
                if let Some(text) = self.label_from_descendants(tree) {
                    self.set_label(text.as_str());
                }
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
            let child = tree.assert_node_by_id(child_id);
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
