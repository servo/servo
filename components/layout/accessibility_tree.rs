/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use accesskit::{Node as AxNode, NodeId as AxNodeId, Role, Tree as AxTree};
use layout_api::wrapper_traits::{LayoutNode, ThreadSafeLayoutNode};
use log::trace;
use rustc_hash::FxHashMap;
use script::layout_dom::ServoLayoutDocument;
use style::dom::{NodeInfo, TDocument, TElement, TNode};

use crate::FragmentTree;

// #[derive(MallocSizeOf)]
#[derive(Debug)]
pub(crate) struct AccessibilityTree {
    ax_nodes: FxHashMap<AxNodeId, AxNode>,
    ax_tree: AxTree,
}

impl AccessibilityTree {
    pub(crate) fn construct(
        document: ServoLayoutDocument<'_>,
        fragment_tree: Rc<FragmentTree>,
    ) -> Self {
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
                ax_next.set_value(text_content);
                ax_next.set_role(Role::Unknown);
            }
            ax_nodes.insert(ax_next_id, ax_next);
        }
        Self {
            ax_nodes,
            ax_tree: AxTree {
                root: ax_document_id,
                toolkit_name: None,
                toolkit_version: None,
            },
        }
    }
    pub(crate) fn descendants(&self) -> AxDescendants<'_> {
        AxDescendants(self, vec![self.ax_tree.root])
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

pub(crate) struct AxDescendants<'tree>(&'tree AccessibilityTree, Vec<AxNodeId>);
impl<'tree> Iterator for AxDescendants<'tree> {
    type Item = (AxNodeId, &'tree AxNode);
    fn next(&mut self) -> Option<Self::Item> {
        let Some(result_id) = self.1.pop() else {
            return None;
        };
        let result_node = self.0.ax_nodes.get(&result_id).unwrap();
        for child_id in result_node.children().iter().rev() {
            self.1.push(*child_id);
        }
        Some((result_id, result_node))
    }
}
