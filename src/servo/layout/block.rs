#[doc="Block layout."]

import dom::style::{Px, Mm, Pt, Auto, Percent, Unit};
import geom::point::Point2D;
import geom::size::Size2D;
import gfx::geometry::{px_to_au, au};
import util::tree;
import base::{Box, BlockBox, BTree};

trait BlockLayoutMethods {
    fn reflow_block();
}

#[doc="The public block layout methods."]
impl @Box : BlockLayoutMethods {
    #[doc="The main reflow routine for block layout."]
    fn reflow_block() {
        assert self.kind == BlockBox;

        #debug["starting reflow block"];

        // Root here is the root of the reflow, not necessarily the doc as
        // a whole.
        //
        // This routine:
        // - generates root.bounds.size
        // - generates root.bounds.origin for each child

        let mut current_height = 0;

        // Find the combined height of all the children and mark the
        // relative heights of the children in the box
        for tree::each_child(BTree, self) |c| {
            // FIXME subtract borders, margins, etc
            c.bounds.origin = Point2D(au(0), au(current_height));
            current_height += *c.bounds.size.height;
        }

        let height = match self.appearance.height { 
            Px(p) => px_to_au(p.to_int()),
            Auto => au(current_height),
            _ => fail ~"inhereit_height failed, height is neither a Px or auto"
        };

        // FIXME: Width is wrong in the calculation below.
        let width = match self.appearance.width { 
            Px(p) => px_to_au(p.to_int()),
            Auto => self.bounds.size.width, // Do nothing here, width was set by top-down pass
            _ => fail ~"inhereit_height failed, width is neither a Px or auto"
        };

        self.bounds.size = Size2D(width, height);

        #debug["reflow_block size=%?", copy self.bounds];
    }
}

