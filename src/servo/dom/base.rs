import dom::rcu::{writer_methods};
import gfx::geom::{au, size};
import layout::base::layout_data;
import util::tree;
import dvec::{dvec, extensions};

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
    let attrs: dvec<~attr>;

    new(tag_name: str, -subclass: ~element_subclass) {
        self.tag_name = tag_name;
        self.subclass = subclass;
        self.attrs = dvec();
    }

    fn get_attr(attr_name: str) -> option<str> {
        let mut i = 0u;
        while i < self.attrs.len() {
            if attr_name == self.attrs[i].name {
                ret some(self.attrs[i].value);
            }
            i += 1u;
        }
        ret none;
    }
}

class attr {
    let name: str;
    let value: str;

    new(name: str, value: str) {
        self.name = name;
        self.value = value;
    }
}

enum element_subclass {
    es_unknown,
    es_div,
    es_img({mut size: size<au>}),
    es_head
}

#[doc="The rd_aux data is a (weak) pointer to the layout data, which
       contains the CSS info as well as the primary box.  Note that
       there may be multiple boxes per DOM node."]

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

