/* The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements. */
use newcss::complete::CompleteSelectResults;
use dom::bindings;
use dom::document::Document;
use dom::element::{Attr, ElementData};
use dom::window::Window;
use geom::size::Size2D;
use js::crust::*;
use js::glue::bindgen::RUST_OBJECT_TO_JSVAL;
use js::jsapi::{JSClass, JSObject, JSPropertySpec, JSContext, jsid, JSVal, JSBool};
use js::rust::{bare_compartment, compartment, methods};
use js::{JSPROP_ENUMERATE, JSPROP_SHARED};
use layout::debug::DebugMethods;
use layout::flow::FlowContext;
use ptr::null;
use std::arc::ARC;
use util::tree;

pub enum NodeData = {
    tree: tree::Tree<Node>,
    kind: ~NodeKind,
};

/* The tree holding Nodes (read-only) */
pub enum NodeTree { NodeTree }

impl NodeTree {
    fn each_child(node: &Node, f: fn(&Node) -> bool) {
        tree::each_child(&self, node, f)
    }

    fn get_parent(node: &Node) -> Option<Node> {
        tree::get_parent(&self, node)
    }
}

impl NodeTree : tree::ReadMethods<Node> {
    fn with_tree_fields<R>(n: &Node, f: fn(&tree::Tree<Node>) -> R) -> R {
        n.read(|n| f(&n.tree))
    }
}

impl Node {
    fn traverse_preorder(preorder_cb: &fn(Node)) {
        preorder_cb(self);
        do NodeTree.each_child(&self) |child| { child.traverse_preorder(preorder_cb); true }
    }

    fn traverse_postorder(postorder_cb: &fn(Node)) {
        do NodeTree.each_child(&self) |child| { child.traverse_postorder(postorder_cb); true }
        postorder_cb(self);
    }
}

impl Node : DebugMethods {
    /* Dumps the subtree rooted at this node, for debugging. */
    pure fn dump(&self) {
        self.dump_indent(0u);
    }
    /* Dumps the node tree, for debugging, with indentation. */
    pure fn dump_indent(&self, indent: uint) {
        let mut s = ~"";
        for uint::range(0u, indent) |_i| {
            s += ~"    ";
        }

        s += self.debug_str();
        debug!("%s", s);

        // FIXME: this should have a pure version?
        unsafe {
            for NodeTree.each_child(self) |kid| {
                kid.dump_indent(indent + 1u) 
            }
        }
    }

    pure fn debug_str(&self) -> ~str unsafe {
        do self.read |n| { fmt!("%?", n.kind) }
    }
}

impl Node {
    fn is_element(&self) -> bool {
        self.read(|n| match *n.kind { Element(*) => true, _ => false } )
    }
}

pub enum NodeKind {
    Doctype(DoctypeData),
    Comment(~str),
    Element(ElementData),
    Text(~str)
}

pub struct DoctypeData {
    name: ~str,
    public_id: Option<~str>,
    system_id: Option<~str>,
    force_quirks: bool
}

pub fn DoctypeData(name: ~str, public_id: Option<~str>,
               system_id: Option<~str>, force_quirks: bool) -> DoctypeData {
    DoctypeData {
        name : move name,
        public_id : move public_id,
        system_id : move system_id,
        force_quirks : force_quirks,
    }
}



pub fn define_bindings(compartment: &bare_compartment, doc: @Document,
                   win: @Window) {
    bindings::window::init(compartment, win);
    bindings::document::init(compartment, doc);
    bindings::node::init(compartment);
    bindings::element::init(compartment);
}


/** The COW rd_aux data is a (weak) pointer to the layout data,
   defined by this `LayoutData` enum. It contains the CSS style object
   as well as the primary `RenderBox`.

   Note that there may be multiple boxes per DOM node. */
enum LayoutData = {
    mut style: Option<CompleteSelectResults>,
    mut flow:  Option<@FlowContext>
};

pub type Node = cow::Handle<NodeData, LayoutData>;

pub type NodeScope = cow::Scope<NodeData, LayoutData>;

pub fn NodeScope() -> NodeScope {
    cow::Scope()
}

trait NodeScopeExtensions {
    fn new_node(+k: NodeKind) -> Node;
}

#[allow(non_implicitly_copyable_typarams)]
impl NodeScope : NodeScopeExtensions {
    fn new_node(k: NodeKind) -> Node {
        self.handle(&NodeData({tree: tree::empty(), kind: ~move k}))
    }
}

impl NodeScope {
    fn each_child(node: &Node, f: fn(&Node) -> bool) {
        tree::each_child(&self, node, f)
    }

    fn get_parent(node: &Node) -> Option<Node> {
        tree::get_parent(&self, node)
    }
}

#[allow(non_implicitly_copyable_typarams)]
impl NodeScope : tree::ReadMethods<Node> {
    fn with_tree_fields<R>(node: &Node, f: fn(&tree::Tree<Node>) -> R) -> R {
        self.read(node, |n| f(&n.tree))
    }
}

impl NodeScope {
    fn add_child(node: Node, child: Node) {
        tree::add_child(&self, node, child)
    }
}

#[allow(non_implicitly_copyable_typarams)]
impl NodeScope : tree::WriteMethods<Node> {
    pure fn eq(a: &Node, b: &Node) -> bool { a == b }

    fn with_tree_fields<R>(node: &Node, f: fn(&tree::Tree<Node>) -> R) -> R {
        self.write(node, |n| f(&n.tree))
    }
}
