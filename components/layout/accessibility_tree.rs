/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::LazyLock;

use accesskit::Role;
use app_units::Au;
use html5ever::{LocalName, local_name};
use layout_api::BoxAreaType;
use layout_api::wrapper_traits::{ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use log::trace;
use rustc_hash::{FxHashMap, FxHashSet};
use script::layout_dom::ServoThreadSafeLayoutNode;
use style::dom::{NodeInfo, OpaqueNode};

use crate::display_list::StackingContextTree;
use crate::query::process_box_area_request;

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
        stacking_context_tree: &StackingContextTree,
    ) -> Option<accesskit::TreeUpdate> {
        let mut tree_update = AccessibilityUpdate::new(self.accesskit_tree.clone(), self.tree_id);
        self.update_root_dom_node(root_dom_node, &mut tree_update, stacking_context_tree);

        Some(tree_update.accesskit_update)
    }

    fn update_root_dom_node(
        &mut self,
        root_dom_node: ServoThreadSafeLayoutNode<'_>,
        tree_update: &mut AccessibilityUpdate,
        stacking_context_tree: &StackingContextTree,
    ) {
        let root_dom_node_id = Self::to_accesskit_id(&root_dom_node.opaque());
        let root_accessibility_node_id = accesskit::NodeId(0);
        let root_accessibility_node = self.nodes.get_mut(&root_accessibility_node_id).unwrap();
        root_accessibility_node
            .accesskit_node
            .set_children(vec![root_dom_node_id]);

        // Update bounds for root web area
        let viewport_size = stacking_context_tree.paint_info.viewport_details.size;
        let origin: accesskit::Point = Default::default();
        let size = accesskit::Size {
            width: viewport_size.width.into(),
            height: viewport_size.height.into(),
        };
        root_accessibility_node
            .accesskit_node
            .set_bounds(accesskit::Rect::from_origin_size(origin, size));

        tree_update.add(&root_accessibility_node);

        self.update_node(root_dom_node, tree_update, stacking_context_tree);
    }

    fn update_node(
        &mut self,
        dom_node: ServoThreadSafeLayoutNode<'_>,
        tree_update: &mut AccessibilityUpdate,
        stacking_context_tree: &StackingContextTree,
    ) {
        // TODO: actually compute whether updated or not :)
        let updated = true;

        // TODO: read accessibility damage from dom_node (right now, assume damage is complete)

        let mut new_children: Vec<accesskit::NodeId> = vec![];
        for dom_child in dom_node.children() {
            let child_node = self.get_or_create_node(dom_child);
            // FIXME: don't want to call get_or_create twice!
            new_children.push(child_node.id);
            self.update_node(dom_child, tree_update, stacking_context_tree);
        }

        let accessibility_node = self.get_or_create_node(dom_node);
        if let Some(bounds) = get_bounds(dom_node, stacking_context_tree) {
            let origin = accesskit::Point {
                x: bounds.origin.x.to_f64_px(),
                y: bounds.origin.y.to_f64_px(),
            };
            let size = accesskit::Size {
                width: bounds.size.width.to_f64_px(),
                height: bounds.size.height.to_f64_px(),
            };
            //info!("origin: {:?}, size: {:?}", origin, size);
            accessibility_node
                .accesskit_node
                .set_bounds(accesskit::Rect::from_origin_size(origin, size));
            //info!(
            //    "accesskit_node bounds: {:?}",
            //    accessibility_node.accesskit_node.bounds()
            //);
        }

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
            // accesskit_node.set_character_lengths(
            //     text_content
            //         .chars()
            //         .map(|c| c.len_utf8() as u8)
            //         .collect::<Box<[u8]>>(),
            // );
        } else if let Some(dom_element) = dom_node.as_element() {
            if dom_element.is_html_element() {
                if let Some(role) = HTML_ELEMENT_ROLE_MAPPINGS.get(dom_element.get_local_name()) {
                    accesskit_node.set_role(*role);
                }
                if NAME_FROM_TEXT_CONTENT.contains(dom_element.get_local_name()) {
                    if let Some(text_content) = dom_node.dangerous_get_dom_text_content() {
                        let text_content = text_content.trim();
                        accesskit_node.set_label(text_content);
                    }
                }
                if accesskit_node.label().is_none() {
                    let local_name = format!("<{}>", dom_element.get_local_name());
                    accesskit_node.set_label(local_name);
                }
                // TODO: if role is UNKNOWN or GROUP, and text content is empty, set label to html tag name
            }
        }

        //// bounds!

        node
    }

    fn to_accesskit_id(opaque: &OpaqueNode) -> accesskit::NodeId {
        accesskit::NodeId(opaque.0 as u64)
    }
}

fn get_bounds(
    dom_node: ServoThreadSafeLayoutNode<'_>,
    stacking_context_tree: &StackingContextTree,
) -> Option<euclid::default::Rect<Au>> {
    // TODO: figure out units
    let css_box =
        process_box_area_request(stacking_context_tree, dom_node, BoxAreaType::Border, false)?;
    Some(css_box.to_untyped())
}

impl AccessibilityNode {
    fn new(id: accesskit::NodeId) -> Self {
        Self {
            id,
            accesskit_node: accesskit::Node::new(Role::Unknown),
        }
    }
}

static NAME_FROM_TEXT_CONTENT: LazyLock<FxHashSet<LocalName>> = LazyLock::new(|| {
    [
        local_name!("a"),
        local_name!("address"),
        local_name!("b"),
        local_name!("button"),
        local_name!("caption"),
        local_name!("del"),
        local_name!("dfn"),
        local_name!("em"),
        local_name!("figcaption"),
        local_name!("h1"),
        local_name!("h2"),
        local_name!("h3"),
        local_name!("h4"),
        local_name!("h5"),
        local_name!("h6"),
        local_name!("i"),
        local_name!("ins"),
        local_name!("li"),
        local_name!("option"),
        local_name!("small"),
        local_name!("span"),
        local_name!("strong"),
        local_name!("td"),
        local_name!("tfoot"),
        local_name!("th"),
        local_name!("time"),
        local_name!("u"),
    ]
    .into_iter()
    .collect()
});

/// <https://www.w3.org/TR/html-aam-1.0/#html-element-role-mappings>
///
/// FIXME: converted mechanically for now, so this will have many errors
static HTML_ELEMENT_ROLE_MAPPINGS: LazyLock<FxHashMap<LocalName, Role>> = LazyLock::new(|| {
    [
        (local_name!("a"), Role::Link),
        (local_name!("address"), Role::Group),
        (local_name!("area"), Role::Link),
        (local_name!("area"), Role::GenericContainer),
        (local_name!("article"), Role::Article),
        (local_name!("aside"), Role::Complementary),
        (local_name!("b"), Role::GenericContainer),
        (local_name!("bdi"), Role::GenericContainer),
        (local_name!("bdo"), Role::GenericContainer),
        (local_name!("blockquote"), Role::Blockquote),
        (local_name!("body"), Role::GenericContainer),
        (local_name!("button"), Role::Button),
        (local_name!("caption"), Role::Caption),
        (local_name!("code"), Role::Code),
        (local_name!("data"), Role::GenericContainer),
        (local_name!("datalist"), Role::ListBox),
        (local_name!("dd"), Role::Definition),
        (local_name!("del"), Role::ContentDeletion),
        (local_name!("details"), Role::Group),
        (local_name!("dfn"), Role::Term),
        (local_name!("dialog"), Role::Dialog),
        (local_name!("dir"), Role::List),
        (local_name!("div"), Role::GenericContainer),
        (local_name!("dl"), Role::List),
        (local_name!("dt"), Role::Term),
        (local_name!("em"), Role::Emphasis),
        (local_name!("fieldset"), Role::Group),
        (local_name!("figcaption"), Role::Caption),
        (local_name!("figure"), Role::Figure),
        (local_name!("footer"), Role::ContentInfo),
        (local_name!("form"), Role::Form),
        (local_name!("h1"), Role::Heading),
        (local_name!("h2"), Role::Heading),
        (local_name!("h3"), Role::Heading),
        (local_name!("h4"), Role::Heading),
        (local_name!("h5"), Role::Heading),
        (local_name!("h6"), Role::Heading),
        (local_name!("header"), Role::Banner),
        (local_name!("hgroup"), Role::Group),
        (local_name!("hr"), Role::Splitter),
        (local_name!("html"), Role::GenericContainer),
        (local_name!("i"), Role::GenericContainer),
        (local_name!("img"), Role::Image),
        (local_name!("ins"), Role::ContentInsertion),
        (local_name!("li"), Role::ListItem),
        (local_name!("main"), Role::Main),
        (local_name!("mark"), Role::Mark),
        (local_name!("menu"), Role::List),
        (local_name!("meter"), Role::Meter),
        (local_name!("nav"), Role::Navigation),
        (local_name!("ol"), Role::List),
        (local_name!("optgroup"), Role::Group),
        (local_name!("option"), Role::ListBoxOption),
        (local_name!("output"), Role::Status),
        (local_name!("p"), Role::Paragraph),
        (local_name!("pre"), Role::GenericContainer),
        (local_name!("progress"), Role::ProgressIndicator),
        (local_name!("q"), Role::GenericContainer),
        (local_name!("s"), Role::ContentDeletion),
        (local_name!("samp"), Role::GenericContainer),
        (local_name!("search"), Role::Search),
        (local_name!("section"), Role::Region),
        (local_name!("select"), Role::ListBox),
        (local_name!("select"), Role::ComboBox),
        (local_name!("small"), Role::GenericContainer),
        (local_name!("span"), Role::GenericContainer),
        (local_name!("strong"), Role::Strong),
        (local_name!("table"), Role::Table),
        (local_name!("tbody"), Role::RowGroup),
        (local_name!("td"), Role::Cell),
        (local_name!("textarea"), Role::TextInput),
        (local_name!("tfoot"), Role::RowGroup),
        (local_name!("th"), Role::Cell),
        (local_name!("th"), Role::ColumnHeader),
        (local_name!("th"), Role::RowHeader),
        (local_name!("thead"), Role::RowGroup),
        (local_name!("time"), Role::Time),
        (local_name!("tr"), Role::Row),
        (local_name!("u"), Role::GenericContainer),
        (local_name!("ul"), Role::List),
    ]
    .into_iter()
    .collect()
});

/*
Accessibility damage: needs to be in LayoutDamage since RestyleDamage is already fully subscribed
This seems to be available from node.owner_doc().ensure_pending_restyle(self). Ok.

*/

/*
Accessibility tree update:

- Traverse (flat) DOM tree
- for each node in traversal:
    - check layout damage to see whether there is accessiblity damage
    - if yes, retrieve accessibility node:
        - compute ax node id from node.opaque()
        - fetch from map
        - (it should already exist?)
    - run update based on value of accessibility damage
    - if damage requires computing children, traverse DOM node children and
      add stub accessibility nodes for each, so that the next iteration can
      find them

*/

/*
But what about actions :(

Martin says we can go the other way
Delan found node::from_untrusted_node_address, we can convert an OpaqueNode to an untrusted
address (using OpaqueNode.into()))
If the accessibility tree is _clean_, then we won't have any nodes in the accessibility tree
which don't correlate to live DOM nodes, i.e. if we lookup the accessibility node id we'll
get a miss if the DOM node is gone.

So: when we get an action request, before trying to perform the action we need to first do
a reflow and ensure the accessibility tree is clean, then lookup the accesskit node ID to
get the live accessibility node.
We don't want any script to run during this process, however this should be fine because the
action request will come in _on the script thread_ and will block any other script from
running.
*/
