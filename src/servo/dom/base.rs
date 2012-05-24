import dom::rcu::{writer_methods};
import gfx::geom::{au, size};
import layout::base::layout_data;
import util::tree;

enum node_data = {
    tree: tree::fields<node>,
    kind: ~node_kind,
};

enum node_kind {
    nk_element(element),
    nk_text(str)
}

class element {
    let tag_name: str;
    let subclass: ~element_subclass;

    new(tag_name: str, -subclass: ~element_subclass) {
        self.tag_name = tag_name;
        self.subclass = subclass;
    }
}

enum element_subclass {
    es_unknown,
    es_div,
    es_img(size<au>)
}

#[doc="The rd_aux data is a (weak) pointer to the layout data, which contains
       the CSS info as well as the primary box.  Note that there may be multiple
       boxes per DOM node."]
type node = rcu::handle<node_data, layout_data>;

type node_scope = rcu::scope<node_data, layout_data>;

fn node_scope() -> node_scope { rcu::scope() }

impl methods for node_scope {
    fn new_node(-k: node_kind) -> node {
        self.handle(node_data({tree: tree::empty(),
                               kind: ~k}))
    }
}

impl of tree::rd_tree_ops<node> for node_scope {
    fn each_child(node: node, f: fn(node) -> bool) {
        tree::each_child(self, node, f)
    }

    fn get_parent(node: node) -> option<node> {
        tree::get_parent(self, node)
    }

    fn with_tree_fields<R>(node: node, f: fn(tree::fields<node>) -> R) -> R {
        self.rd(node) { |n| f(n.tree) }
    }
}

impl of tree::wr_tree_ops<node> for node_scope {
    fn add_child(node: node, child: node) {
        tree::add_child(self, node, child)
    }

    fn with_tree_fields<R>(node: node, f: fn(tree::fields<node>) -> R) -> R {
        self.wr(node) { |n| f(n.tree) }
    }
}

