import dom::rcu;
import dom::rcu::methods;
import util::{tree, geom};
import geom::{size, rect, point, au};

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
    mut linfo: option<box>,
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
                           mut linfo: none}))
}

fn new_box(n: node) -> box {
    box(@{tree: tree::empty(),
          node: n,
          mut bounds: geom::zero_rect_au()})
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

/*
#[cfg(test)]
mod test {
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
}

#[test]
fn do_layout() {
    test::with_screen {|s|
        let n0 = node(nk_img(size(au(22),au(44))));
        let n1 = node(nk_img(size(au(22),au(44))));
        let n2 = node(nk_img(size(au(22),au(44))));
        let n3 = node(nk_div);

        tree::add_child(n3, n0);
        tree::add_child(n3, n1);
        tree::add_child(n3, n2);

        let b0 = box(n0);
        let b1 = box(n1);
        let b2 = box(n2);
        let b3 = box(n3);

        tree::add_child(b3, b0);
        tree::add_child(b3, b1);
        tree::add_child(b3, b2);
   }
}
*/