/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;
use std::sync::LazyLock;

pub(crate) use accessibility_traits::AccessibilityTree;
use accesskit::{Action, Node as AxNode, NodeId as AxNodeId, Role, Tree as AxTree};
use html5ever::{LocalName, local_name};
use layout_api::wrapper_traits::{LayoutNode, ThreadSafeLayoutNode};
use log::trace;
use rustc_hash::FxHashMap;
use script::layout_dom::ServoLayoutDocument;
use style::dom::{NodeInfo, TDocument, TElement, TNode};

use crate::FragmentTree;

// #[derive(MallocSizeOf)]
pub(crate) struct AccessibilityTreeCalculator {}

impl AccessibilityTreeCalculator {
    pub(crate) fn construct(
        document: ServoLayoutDocument<'_>,
        fragment_tree: Rc<FragmentTree>,
    ) -> AccessibilityTree {
        let mut ax_nodes: FxHashMap<AxNodeId, AxNode> = FxHashMap::default();
        let ax_document_id = AxNodeId(document.as_node().opaque().0 as u64);
        let ax_document = AxNode::new(Role::Document);
        ax_nodes.insert(ax_document_id, ax_document);
        let dom_root = document.root_element().unwrap().as_node();
        let mut dom_queue = vec![dom_root];
        'outer: loop {
            // find the next node in the queue that is not ‘display: none’.
            let dom_next = loop {
                let Some(next) = dom_queue.pop() else {
                    break 'outer;
                };
                if next
                    .style_data()
                    .is_none_or(|style| !style.element_data.borrow().styles.is_display_none())
                {
                    break next;
                }
            };
            // push its children for later traversal.
            // reverse order ensures that the first child will get popped first.
            for child in RevDomChildren::of(dom_next) {
                dom_queue.push(child);
            }
            // now process the node.
            trace!("DOM node: {dom_next:?}");
            let dom_parent = dom_next
                .parent_node()
                .expect("Even the root element must have a parent");
            let ax_parent_id = AxNodeId(dom_parent.opaque().0 as u64);
            let ax_next_id = AxNodeId(dom_next.opaque().0 as u64);
            let ax_parent = ax_nodes.get_mut(&ax_parent_id).expect("Guaranteed by us");
            ax_parent.push_child(ax_next_id);
            let mut ax_next = AxNode::default();
            if dom_next.is_text_node() {
                let text_content = dom_next.to_threadsafe().text_content();
                trace!("node text content = {:?}", text_content);
                ax_next.set_value(&*text_content);
                // FIXME: this should take into account editing selection units (grapheme clusters?)
                ax_next.set_character_lengths(
                    text_content
                        .chars()
                        .map(|c| c.len_utf8() as u8)
                        .collect::<Box<[u8]>>(),
                );
                ax_next.set_role(Role::TextRun);
            } else if let Some(element) = dom_next.as_element() {
                if element.is_html_element() {
                    if let Some(role) = HTML_ELEMENT_ROLE_MAPPINGS.get(element.local_name()) {
                        ax_next.set_role(*role);
                    }
                }
            }
            ax_next.add_action(Action::Click);
            ax_nodes.insert(ax_next_id, ax_next);
        }
        AccessibilityTree {
            ax_nodes,
            ax_tree: AxTree {
                root: ax_document_id,
                toolkit_name: None,
                toolkit_version: None,
            },
        }
    }
}

/// Like [`style::dom::DomChildren`], but reversed.
///
/// Do not use the tuple constructor; use [`Self::of`].
pub(crate) struct RevDomChildren<N>(Option<N>);
impl<N: TNode> RevDomChildren<N> {
    fn of(node: N) -> Self {
        Self(node.last_child())
    }
}
impl<N: TNode> Iterator for RevDomChildren<N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        let n = self.0.take()?;
        self.0 = n.prev_sibling();
        Some(n)
    }
}

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
