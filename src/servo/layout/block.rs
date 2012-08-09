#[doc="Block layout."]

import geom::point::Point2D;
import geom::size::Size2D;
import gfx::geometry::au;
import util::tree;
import base::{Box, BlockBox, BTree};

trait BlockLayoutMethods {
    fn reflow_block(available_widh: au);
}

#[doc="The public block layout methods."]
impl @Box : BlockLayoutMethods {
    #[doc="The main reflow routine for block layout."]
    fn reflow_block(available_width: au) {
        assert self.kind == BlockBox;

        #debug["starting reflow block"];

        // Root here is the root of the reflow, not necessarily the doc as
        // a whole.
        //
        // This routine:
        // - generates root.bounds.size
        // - generates root.bounds.origin for each child
        // - and recursively computes the bounds for each child

        let mut current_height = 0;
        for tree::each_child(BTree, self) |c| {
            let mut blk_available_width = available_width;
            // FIXME subtract borders, margins, etc
            c.bounds.origin = Point2D(au(0), au(current_height));
            c.reflow(blk_available_width);
            current_height += *c.bounds.size.height;
        }

        // FIXME: Width is wrong in the calculation below.
        self.bounds.size = Size2D(available_width, au(current_height));

        #debug["reflow_block size=%?", copy self.bounds];
    }
}

