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

impl of tree::rd_tree_ops<node> for scope<node_data, box> {
    fn each_child(node: node, f: fn(node) -> bool) {
        tree::each_child(self, node, f)
    }

    fn with_tree_fields<R>(node: node, f: fn(tree::fields<node>) -> R) -> R {
        f(self.rd(node) { |f| f.tree })
    }
}

impl of tree::wr_tree_ops<node> for scope<node_data, box> {
    fn add_child(node: node, child: node) {
        tree::add_child(self, node, child)
    }

    fn with_tree_fields<R>(node: node, f: fn(tree::fields<node>) -> R) -> R {
        f(self.wr(node) { |f| f.tree })
    }
}
