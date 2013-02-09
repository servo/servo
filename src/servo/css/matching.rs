// High-level interface to CSS selector matching.

use css::node_util::NodeUtil;
use css::select_handler::NodeSelectHandler;
use dom::node::{Node, NodeTree};
use layout::context::LayoutContext;
use newcss::complete::CompleteSelectResults;
use newcss::select::{SelectCtx, SelectResults};

use std::arc::{ARC, get, clone};

pub trait MatchMethods {
    fn restyle_subtree(select_ctx: &SelectCtx);
}

impl Node : MatchMethods {
    /**
     * Performs CSS selector matching on a subtree.

     * This is, importantly, the function that updates the layout data for
     * the node (the reader-auxiliary box in the COW model) with the
     * computed style.
     */
    fn restyle_subtree(select_ctx: &SelectCtx) {

        // Only elements have styles
        if self.is_element() {
            let select_handler = NodeSelectHandler {
                node: self
            };
            let incomplete_results = select_ctx.select_style(&self, &select_handler);
            // Combine this node's results with its parent's to resolve all inherited values
            let complete_results = compose_results(&self, move incomplete_results);
            self.set_css_select_results(move complete_results);
        }

        let mut i = 0u;
        
        for NodeTree.each_child(&self) |kid| {
            i = i + 1u;
            kid.restyle_subtree(select_ctx); 
        }
    }
}

fn compose_results(node: &Node, results: SelectResults) -> CompleteSelectResults {
    match find_parent_element_node(node) {
        None => CompleteSelectResults::new_root(move results),
        Some(parent_node) => {
            let parent_results = parent_node.get_css_select_results();
            CompleteSelectResults::new_from_parent(parent_results, move results)
        }
    }    
}

fn find_parent_element_node(node: &Node) -> Option<Node> {
    use util::tree::parent;

    match parent(&NodeTree, node) {
        Some(ref parent) => {
            if parent.is_element() {
                Some(*parent)
            } else {
                find_parent_element_node(parent)
            }
        }
        None => None
    }
}
