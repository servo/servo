import dom::rcu;
import dom::rcu::methods;
import gfx::geom;
import gfx::geom::{size, rect, point, au};
import util::{tree};

enum box = @{
    tree: tree::fields<box>,
    node: node,
    mut bounds: geom::rect<au>
};

enum node_data = {
    tree: tree::fields<node>,
    kind: node_kind,

    // Points to the primary box.  Note that there may be multiple
    // boxes per DOM node.
    mut box: option<box>,
};

enum node_kind {
    nk_div,
    nk_img(size<au>)
}

type node = rcu::handle<node_data>;

impl of tree::tree for box {
    fn tree_fields() -> tree::fields<box> {
        ret self.tree;
    }
}

impl of tree::tree for node {
    fn tree_fields() -> tree::fields<node> {
        ret self.get().tree;
    }
}

fn new_node(+k: node_kind) -> node {
    rcu::handle(node_data({tree: tree::empty(),
                           kind: k,
                           mut box: none}))
}

fn new_box(n: node) -> box {
    box(@{tree: tree::empty(),
          node: n,
          mut bounds: geom::zero_rect_au()})
}

fn linked_box(n: node) -> box {
    let b = new_box(n);
    n.box = some(b);
    ret b;
}

fn reflow_block(root: box, available_width: au) {
    // Root here is the root of the reflow, not necessarily the doc as
    // a whole.

    alt root.node.get().kind {
      nk_img(size) {
        root.bounds.size = size;
        ret;
      }

      nk_div { /* fallthrough */ }
    }

    let mut current_height = 0;
    for tree::each_child(root) {|c|
        let mut blk_available_width = available_width;
        // FIXME subtract borders, margins, etc
        c.bounds.origin = {x: au(0), y: au(current_height)};
        reflow_block(c, blk_available_width);
        current_height += *c.bounds.size.height;
    }

    root.bounds.size = {width: available_width, // FIXME
                        height: au(current_height)};
}

#[cfg(test)]
mod test {

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

    fn flat_bounds(root: box) -> [geom::rect<au>] {
        let mut r = [];
        for tree::each_child(root) {|c|
            r += flat_bounds(c);
        }
        ret r + [root.bounds];
    }

    #[test]
    fn do_layout() {
        let n0 = new_node(nk_img(size(au(10),au(10))));
        let n1 = new_node(nk_img(size(au(10),au(15))));
        let n2 = new_node(nk_img(size(au(10),au(20))));
        let n3 = new_node(nk_div);

        tree::add_child(n3, n0);
        tree::add_child(n3, n1);
        tree::add_child(n3, n2);

        let b0 = linked_box(n0);
        let b1 = linked_box(n1);
        let b2 = linked_box(n2);
        let b3 = linked_box(n3);

        tree::add_child(b3, b0);
        tree::add_child(b3, b1);
        tree::add_child(b3, b2);

        reflow_block(b3, au(100));
        let fb = flat_bounds(b3);
        #debug["fb=%?", fb];
        assert fb == [geom::box(au(0), au(0), au(10), au(10)),   // n0
                      geom::box(au(0), au(10), au(10), au(15)),  // n1
                      geom::box(au(0), au(25), au(10), au(20)),  // n2
                      geom::box(au(0), au(0), au(100), au(45))]; // n3
    }
}

