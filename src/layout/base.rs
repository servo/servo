import dom::rcu;
import dom::rcu::methods;

// FIXME--mut should be inherited
type point<A> = { mut x: A, mut y: A };
type size<A> = { mut width: A, mut height: A };
type rect<A> = { mut origin: point<A>, mut size: size<A> };

enum au = int;

type tree_fields<T> = {
    parent: option<T>,
    first_child: option<T>,
    last_child: option<T>,
    prev_sibling: option<T>,
    next_sibling: option<T>
};

enum box = @{
    tree: tree_fields<box>,
    node: node,
    mut bounds: rect<au>
};

type node_data = {
    tree: tree_fields<node>,
    kind: node_kind,

    // Points to the primary box.  Note that there may be multiple
    // boxes per DOM node.
    mut linfo: option<box>,
};

enum node_kind {
    nk_div,
    nk_img(size<au>)
}

enum node = rcu::handle<node_data>;

iface tree {
    fn tree_fields() -> tree_fields<self>;
}

impl of tree for box {
    fn tree_fields() -> tree_fields<box> {
        ret self.tree;
    }
}

impl of tree for node {
    fn tree_fields() -> tree_fields<node> {
        ret (*self).get().tree;
    }
}

fn each_child<T:copy tree>(
    node: T, f: fn(T) -> bool) {

    let mut p = node.tree_fields().first_child;
    loop {
        alt p {
          none { ret; }
          some(c) {
            if !f(c) { ret; }
            p = c.tree_fields().next_sibling;
          }
        }
    }
}

fn reflow_block(root: box, available_width: au) {
    // Root here is the root of the reflow, not necessarily the doc as
    // a whole.

    alt (*root.node).get().kind {
      nk_img(size) {
        root.bounds.size = size;
        ret;
      }

      nk_div { /* fallthrough */ }
    }

    let mut current_height = 0;
    for each_child(root) {|c|
        let mut blk_available_width = available_width;
        // FIXME subtract borders, margins, etc
        c.bounds.origin = {x: au(0), y: au(current_height)};
        reflow_block(c, blk_available_width);
        current_height += *c.bounds.size.height;
    }

    root.bounds.size = {width: available_width, // FIXME
                        height: au(current_height)};
}
