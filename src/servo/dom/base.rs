import dom::rcu::{scope, writer_methods};
import gfx::geom::{au, size};
import layout::base::box;
import util::tree;

enum node_data = {
    tree: tree::fields<node>,
    kind: node_kind,
};

enum node_kind {
    nk_div,
    nk_img(size<au>)
}

// The rd_aux data is a (weak) pointer to the primary box.  Note that
// there may be multiple boxes per DOM node.
type node = rcu::handle<node_data, box>;

impl methods for scope<node_data, box> {
    fn new_node(+k: node_kind) -> node {
        self.handle(node_data({tree: tree::empty(),
                               kind: k}))
    }
}
