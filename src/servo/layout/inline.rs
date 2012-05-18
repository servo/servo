#[doc="Inline layout."]

import dom::rcu;
import dom::rcu::reader_methods;
import gfx::geom::au;
import /*layout::*/base::*; // FIXME: Can't get around import *; resolve bug.
import /*layout::*/style::style::*; // ditto
import util::tree;

#[doc="The main reflow routine for inline layout."]
impl inline_layout_methods for @box {
    fn reflow_inline(available_width: au) {
        assert self.kind == bk_inline;

        // FIXME: This is clownshoes inline layout and is not even close to
        // correct.
        let y = self.bounds.origin.y;
        let mut x = 0, inline_available_width = *available_width;
        let mut current_height = 0;
        for tree::each_child(btree, self) {
            |kid|
            kid.bounds.origin = { mut x: au(x), mut y: y };
            kid.reflow(au(inline_available_width));
            inline_available_width -= *kid.bounds.size.width;
            x += *kid.bounds.size.width;
            current_height = int::max(current_height, *kid.bounds.size.height);
        }

        self.bounds.size = { mut width: available_width,
                             mut height: au(current_height) };

        #debug["reflow_inline size=%?", self.bounds];
    }
}

