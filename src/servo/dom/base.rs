#[doc="The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements."]

import dom::rcu::WriterMethods;
import gfx::geometry::au;
import geom::size::Size2D;
import layout::base::LayoutData;
import util::tree;
import js::rust::{bare_compartment, compartment, methods};
import js::jsapi::{JSClass, JSObject, JSPropertySpec, JSContext, jsid, jsval, JSBool};
import js::{JSPROP_ENUMERATE, JSPROP_SHARED};
import js::crust::*;
import js::glue::bindgen::RUST_OBJECT_TO_JSVAL;
import ptr::null;
import content::Document;
import bindings;

import dvec::{dvec, extensions};

enum NodeData = {
    tree: tree::Tree<Node>,
    kind: ~NodeKind,
};

enum NodeKind {
    Element(ElementData),
    Text(~str)
}

class ElementData {
    let tag_name: ~str;
    let kind: ~ElementKind;
    let attrs: dvec<~Attr>;

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

class Attr {
    let name: ~str;
    let value: ~str;

    new(-name: ~str, -value: ~str) {
        self.name = name;
        self.value = value;
    }
}

fn define_bindings(compartment: bare_compartment, doc: @Document) {
    //bindings::window::init(compartment);
    bindings::document::init(compartment, doc);
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

trait node_scope {
    fn new_node(-k: NodeKind) -> Node;
}

#[warn(no_non_implicitly_copyable_typarams)]
impl NodeScope of node_scope for NodeScope {
    fn new_node(-k: NodeKind) -> Node {
        self.handle(NodeData({tree: tree::empty(), kind: ~k}))
    }
}

#[warn(no_non_implicitly_copyable_typarams)]
impl TreeReadMethods of tree::ReadMethods<Node> for NodeScope {
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

#[warn(no_non_implicitly_copyable_typarams)]
impl TreeWriteMethods of tree::WriteMethods<Node> for NodeScope {
    fn add_child(node: Node, child: Node) {
        tree::add_child(self, node, child)
    }

    fn with_tree_fields<R>(node: Node, f: fn(tree::Tree<Node>) -> R) -> R {
        self.write(node, |n| f(n.tree))
    }
}

