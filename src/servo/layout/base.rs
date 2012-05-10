import dom::base::{nk_div, nk_img, node_data, node_kind, node};
import dom::rcu;
import dom::rcu::reader_methods;
import inline::inline_layout_methods;
import gfx::geom;
import gfx::geom::{size, rect, point, au};
import util::{tree};

enum display {
    di_block,
    di_inline
}

enum box = {
    tree: tree::fields<@box>,
    node: node,
    mut display: display,
    mut bounds: geom::rect<au>
};

enum ntree { ntree }
impl of tree::rd_tree_ops<node> for ntree {
    fn each_child(node: node, f: fn(node) -> bool) {
        tree::each_child(self, node, f)
    }

    fn with_tree_fields<R>(n: node, f: fn(tree::fields<node>) -> R) -> R {
        n.rd { |n| f(n.tree) }
    }
}

enum btree { btree }
impl of tree::rd_tree_ops<@box> for btree {
    fn each_child(node: @box, f: fn(&&@box) -> bool) {
        tree::each_child(self, node, f)
    }

    fn with_tree_fields<R>(b: @box, f: fn(tree::fields<@box>) -> R) -> R {
        f(b.tree)
    }
}

impl of tree::wr_tree_ops<@box> for btree {
    fn add_child(node: @box, child: @box) {
        tree::add_child(self, node, child)
    }

    fn with_tree_fields<R>(b: @box, f: fn(tree::fields<@box>) -> R) -> R {
        f(b.tree)
    }
}

impl block_layout_methods for @box {
    #[doc="The main reflow routine."]
    fn reflow(available_width: au) {
        alt self.display {
            di_block { self.reflow_block(available_width) }
            di_inline { self.reflow_inline(available_width) }
        }
    }

    #[doc="The main reflow routine for block layout."]
    fn reflow_block(available_width: au) {
        assert self.display == di_block;

        // Root here is the root of the reflow, not necessarily the doc as
        // a whole.
        //
        // This routine:
        // - generates root.bounds.size
        // - generates root.bounds.origin for each child
        // - and recursively computes the bounds for each child

        let k = self.node.rd() { |r| r.kind };
        alt k {
          nk_img(size) {
            self.bounds.size = size;
            ret;
          }

          nk_div { /* fallthrough */ }
        }

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

        #debug["reflow_block root=%? size=%?", k, self.bounds];
    }
}

#[cfg(test)]
mod test {
    import dom::base::{nk_img, node_data, node_kind, node, methods,
                       wr_tree_ops};
    import dom::rcu::scope;

    /*
    use sdl;
    import sdl::video;

    fn with_screen(f: fn(*sdl::surface)) {
        let screen = video::set_video_mode(
            320, 200, 32,
            [video::hwsurface], [video::doublebuf]);
        assert screen != ptr::null();

        f(screen);

        video::free_surface(screen);
    }
    */

    fn flat_bounds(root: @box) -> [geom::rect<au>] {
        let mut r = [];
        for tree::each_child(btree, root) {|c|
            r += flat_bounds(c);
        }
        ret r + [root.bounds];
    }

    #[test]
    fn do_layout() {
        let s = scope();

        let n0 = s.new_node(nk_img(size(au(10),au(10))));
        let n1 = s.new_node(nk_img(size(au(10),au(15))));
        let n2 = s.new_node(nk_img(size(au(10),au(20))));
        let n3 = s.new_node(nk_div);

        tree::add_child(s, n3, n0);
        tree::add_child(s, n3, n1);
        tree::add_child(s, n3, n2);

        let b0 = linked_box(n0);
        let b1 = linked_box(n1);
        let b2 = linked_box(n2);
        let b3 = linked_box(n3);

        tree::add_child(btree, b3, b0);
        tree::add_child(btree, b3, b1);
        tree::add_child(btree, b3, b2);

        b3.reflow_block(au(100));
        let fb = flat_bounds(b3);
        #debug["fb=%?", fb];
        assert fb == [geom::box(au(0), au(0), au(10), au(10)),   // n0
                      geom::box(au(0), au(10), au(10), au(15)),  // n1
                      geom::box(au(0), au(25), au(10), au(20)),  // n2
                      geom::box(au(0), au(0), au(100), au(45))]; // n3
    }
}

