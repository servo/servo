/**
 * High-level interface to CSS selector matching.
 */
use std::arc::{ARC, get, clone};
use dom::node::{Node, LayoutData, NodeTree};
use core::dvec::DVec;
use newcss::values::*;
use newcss::{SelectCtx, SelectResults};
use newcss::color::{Color, rgb};
use newcss::color::css_colors::{white, black};
use layout::context::LayoutContext;
use select_handler::NodeSelectHandler;

trait StyleMethods {
    fn initialize_layout_data() -> Option<@LayoutData>;
    fn initialize_style_for_subtree(ctx: &LayoutContext, refs: &DVec<@LayoutData>);
    fn recompute_style_for_subtree(ctx: &LayoutContext, select_ctx: &SelectCtx);
}

impl Node : StyleMethods {
    /** If none exists, creates empty layout data for the node (the reader-auxiliary
     * box in the COW model) and populates it with an empty style object.
     */
    fn initialize_layout_data() -> Option<@LayoutData> {
        match self.has_aux() {
            false => {
                let data = @LayoutData({
                    mut style : None,
                    mut flow  : None
                });
                self.set_aux(data); Some(data)
            },
            true => None
        }
    }

    /**
     * Initializes layout data and styles for a Node tree, if any nodes do not have
     * this data already. Append created layout data to the task's GC roots.
     */
    fn initialize_style_for_subtree(_ctx: &LayoutContext, refs: &DVec<@LayoutData>) {
        do self.traverse_preorder |n| {
            match n.initialize_layout_data() {
                Some(r) => refs.push(r),
                None => {}
            }
        }
    }

    /**
     * Performs CSS selector matching on a subtree.

     * This is, importantly, the function that updates the layout data for
     * the node (the reader-auxiliary box in the COW model) with the
     * computed style.
     */
    fn recompute_style_for_subtree(ctx: &LayoutContext, select_ctx: &SelectCtx) {
        let mut i = 0u;
        
        for NodeTree.each_child(&self) |kid| {
            i = i + 1u;
            kid.recompute_style_for_subtree(ctx, select_ctx); 
        }

        let select_handler = NodeSelectHandler {
            node: self
        };
        let style = select_ctx.select_style(&self, &select_handler);
        self.set_style(move style);
    }
}
