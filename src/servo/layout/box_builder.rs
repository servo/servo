#[doc="Creates CSS boxes from a DOM."]

import dom::base::node;
import dom::rcu::reader_methods;
import gfx::geom;
import /*layout::*/base::{box, btree, di_block, di_inline, ntree, rd_tree_ops};
import /*layout::*/base::wr_tree_ops;
import util::tree;

export box_builder_methods;

fn new_box(n: node) -> @box {
    @box({tree: tree::empty(),
          node: n,
          mut display: di_block,
          mut bounds: geom::zero_rect_au()})
}

impl box_builder_priv_methods for node {
    fn construct_boxes() -> @box {
        let b = new_box(self);
        self.set_aux(b);
        ret b;
    }
}

impl box_builder_methods for node {
    #[doc="Creates boxes for a subtree. This is the entry point."]
    fn construct_boxes_for_subtree() -> @box {
        let p_box = self.construct_boxes();
        for ntree.each_child(self) {
            |c|
            let c_box = c.construct_boxes_for_subtree();
            btree.add_child(p_box, c_box);
        }
        ret p_box;
    }
}

