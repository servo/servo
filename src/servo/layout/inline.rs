#[doc="Inline layout."]

use base::{Box, InlineBox, BTree};
use css::values::{BoxAuto, BoxLength, Px};
use dom::rcu;
use geom::point::Point2D;
use geom::size::Size2D;
use gfx::geometry::{au, px_to_au};
use num::Num;
use util::tree;

trait InlineLayout {
    fn reflow_inline();
}

#[doc="The main reflow routine for inline layout."]
impl @Box : InlineLayout {
    fn reflow_inline() {
        assert self.kind == InlineBox;

        #debug["starting reflow inline"];

        // FIXME: This is clownshoes inline layout and is not even close to
        // correct.
        let y = 0;
        let mut x = 0;
        let mut current_height = 0;

        // loop over children and set them at the proper horizontal offset
        for tree::each_child(BTree, self) |kid| {
            kid.bounds.origin = Point2D(au(x), au(y));
            x += *kid.bounds.size.width;
            current_height = i32::max(current_height, *kid.bounds.size.height);
        }

        let height = match self.appearance.height { 
            BoxLength(Px(p)) => px_to_au(p.to_int()),
            BoxAuto => au(current_height),
            _ => fail ~"inhereit_height failed, height is neither a Px or auto"
        };

        let width = match self.appearance.width { 
            BoxLength(Px(p)) => px_to_au(p.to_int()),
            BoxAuto => au(i32::max(x, *self.bounds.size.width)),
            _ => fail ~"inhereit_width failed, width is neither a Px or auto"
        };

        // The maximum available width should have been set in the top-down pass
        self.bounds.size = Size2D(width, height);

        #debug["reflow_inline size=%?", copy self.bounds];
    }
}

