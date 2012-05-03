import dom::rcu::methods;
import gfx::geom::{au, size};
import layout::base::box;
import util::tree;

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

