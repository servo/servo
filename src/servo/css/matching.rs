/**
 * High-level interface to CSS selector matching.
 */
use std::arc::{ARC, get, clone};
use dom::node::{Node, NodeTree};
use newcss::select::{SelectCtx, SelectResults};
use layout::context::LayoutContext;
use select_handler::NodeSelectHandler;

trait MatchMethods {
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
        let mut i = 0u;
        
        for NodeTree.each_child(&self) |kid| {
            i = i + 1u;
            kid.restyle_subtree(select_ctx); 
        }

        let select_handler = NodeSelectHandler {
            node: self
        };
        let style = select_ctx.select_style(&self, &select_handler);
        self.set_style(move style);
    }
}
