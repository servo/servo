#[doc="Block layout."]

import gfx::geom::au;
import /*layout::*/base::*; // FIXME: Can't get around import *; resolve bug.
import util::tree;

#[doc="The public block layout methods."]
impl block_layout_methods for @box {
    #[doc="The main reflow routine for block layout."]
    fn reflow_block(available_width: au) {
        assert self.kind == bk_block;

        // Root here is the root of the reflow, not necessarily the doc as
        // a whole.
        //
        // This routine:
        // - generates root.bounds.size
        // - generates root.bounds.origin for each child
        // - and recursively computes the bounds for each child

        let mut current_height = 0;
        for tree::each_child(btree, self) {|c|
            let mut blk_available_width = available_width;
            // FIXME subtract borders, margins, etc
            c.bounds.origin = {mut x: au(0), mut y: au(current_height)};
            c.reflow(blk_available_width);
            current_height += *c.bounds.size.height;
        }

        self.bounds.size = {mut width: available_width, // FIXME
                            mut height: au(current_height)};

        #debug["reflow_block size=%?", self.bounds];
    }
}

