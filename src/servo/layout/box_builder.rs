#[doc="Creates CSS boxes from a DOM."]

import dom::base::{nk_div, nk_img, node};
import dom::rcu::reader_methods;
import gfx::geom;
import /*layout::*/base::{bk_block, bk_intrinsic, box, box_kind, btree, ntree};
import /*layout::*/base::{rd_tree_ops, wr_tree_ops};
import /*layout::*/style::style::{di_block, di_inline};
import util::tree;

export box_builder_methods;

fn new_box(n: node, kind: box_kind) -> @box {
    @box({tree: tree::empty(),
          node: n,
          mut bounds: geom::zero_rect_au(),
          kind: kind })
}

impl box_builder_priv_methods for node {
    fn construct_boxes() -> @box {
        let b = new_box(self, self.determine_box_kind());
        self.aux::<()>({ |a| a.box = some(b); });
        ret b;
    }

    #[doc="
        Determines the kind of box that this node needs. Also, for images,
        computes the intrinsic size.
    "]
    fn determine_box_kind() -> box_kind {
        alt self.rd({ |n| n.kind }) {
            nk_img(size) { bk_intrinsic(@size) }
            nk_div       { bk_block            }
        }
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

