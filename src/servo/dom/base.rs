#[doc="The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements."]

import gfx::geometry::au;
import geom::size::Size2D;
import layout::base::LayoutData;
import util::tree;
import js::rust::{bare_compartment, compartment, methods};
import js::jsapi::{JSClass, JSObject, JSPropertySpec, JSContext, jsid, jsval, JSBool};
import js::{JSPROP_ENUMERATE, JSPROP_SHARED};
import js::crust::*;
import js::glue::bindgen::RUST_OBJECT_TO_JSVAL;
import dvec::{DVec, dvec};
import ptr::null;
import bindings;
import std::arc::arc;
import style::Stylesheet;

struct Window {
    let unused: int;
    new() {
        self.unused = 0;
    }
}

struct Document {
    let root: Node;
    let css_rules: arc<Stylesheet>;

    new(root: Node, -css_rules: Stylesheet) {
        self.root = root;
        self.css_rules = arc(css_rules);
    }
}

enum NodeData = {
    tree: tree::Tree<Node>,
    kind: ~NodeKind,
};

enum NodeKind {
    Element(ElementData),
    Text(~str)
}

struct ElementData {
    let tag_name: ~str;
    let kind: ~ElementKind;
    let attrs: DVec<~Attr>;

    new(-tag_name: ~str, -kind: ~ElementKind) {
        self.tag_name = tag_name;
        self.kind = kind;
        self.attrs = dvec();
    }

    fn get_attr(attr_name: ~str) -> option<~str> {
        let mut i = 0u;
        while i < self.attrs.len() {
            if attr_name == self.attrs[i].name {
                return some(copy self.attrs[i].value);
            }
            i += 1u;
        }

        none
    }
}

struct Attr {
    let name: ~str;
    let value: ~str;

    new(-name: ~str, -value: ~str) {
        self.name = name;
        self.value = value;
    }
}

fn define_bindings(compartment: bare_compartment, doc: @Document,
                   win: @Window) {
    bindings::window::init(compartment, win);
    bindings::document::init(compartment, doc);
    bindings::node::init(compartment);
    bindings::element::init(compartment);
}

enum ElementKind {
    UnknownElement,
    HTMLDivElement,
    HTMLHeadElement,
    HTMLImageElement({mut size: Size2D<au>}),
    HTMLScriptElement
}

#[doc="
    The rd_aux data is a (weak) pointer to the layout data, which contains the CSS info as well as
    the primary box.  Note that there may be multiple boxes per DOM node.
"]

type Node = rcu::Handle<NodeData, LayoutData>;

type NodeScope = rcu::Scope<NodeData, LayoutData>;

fn NodeScope() -> NodeScope {
    rcu::Scope()
}

trait NodeScopeExtensions {
    fn new_node(-k: NodeKind) -> Node;
}

#[allow(non_implicitly_copyable_typarams)]
impl NodeScope : NodeScopeExtensions {
    fn new_node(-k: NodeKind) -> Node {
        self.handle(NodeData({tree: tree::empty(), kind: ~k}))
    }
}

#[allow(non_implicitly_copyable_typarams)]
impl NodeScope : tree::ReadMethods<Node> {
    fn each_child(node: Node, f: fn(Node) -> bool) {
        tree::each_child(self, node, f)
    }

    fn get_parent(node: Node) -> option<Node> {
        tree::get_parent(self, node)
    }

    fn with_tree_fields<R>(node: Node, f: fn(tree::Tree<Node>) -> R) -> R {
        self.read(node, |n| f(n.tree))
    }
}

#[allow(non_implicitly_copyable_typarams)]
impl NodeScope : tree::WriteMethods<Node> {
    fn add_child(node: Node, child: Node) {
        tree::add_child(self, node, child)
    }

    fn with_tree_fields<R>(node: Node, f: fn(tree::Tree<Node>) -> R) -> R {
        self.write(node, |n| f(n.tree))
    }
}

