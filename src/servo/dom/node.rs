/* The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements. */
use css::styles::SpecifiedStyle;
use css::values::Stylesheet;
use dom::bindings;
use dom::document::Document;
use dom::element::{Attr, ElementData};
use dom::window::Window;
use geom::size::Size2D;
use gfx::geometry::au;
use js::crust::*;
use js::glue::bindgen::RUST_OBJECT_TO_JSVAL;
use js::jsapi::{JSClass, JSObject, JSPropertySpec, JSContext, jsid, jsval, JSBool};
use js::rust::{bare_compartment, compartment, methods};
use js::{JSPROP_ENUMERATE, JSPROP_SHARED};
use layout::debug::DebugMethods;
use layout::flow::FlowContext;
use ptr::null;
use std::arc::ARC;
use util::tree;

enum NodeData = {
    tree: tree::Tree<Node>,
    kind: ~NodeKind,
};


/* The tree holding Nodes (read-only) */
enum NodeTree { NodeTree }

impl NodeTree : tree::ReadMethods<Node> {
    fn each_child(node: Node, f: fn(Node) -> bool) {
        tree::each_child(self, node, f)
    }

    fn with_tree_fields<R>(&&n: Node, f: fn(tree::Tree<Node>) -> R) -> R {
        n.read(|n| f(n.tree))
    }
}

impl Node {
    fn traverse_preorder(preorder_cb: &fn(Node)) {
        preorder_cb(self);
        do NodeTree.each_child(self) |child| { child.traverse_preorder(preorder_cb); true }
    }

    fn traverse_postorder(postorder_cb: &fn(Node)) {
        do NodeTree.each_child(self) |child| { child.traverse_postorder(postorder_cb); true }
        postorder_cb(self);
    }
}

/* TODO: LayoutData is just a pointer, but we must circumvent the type
   system to actually compare the pointers. This should be fixed in
   with a generic implementation of rcu::Handle */
impl Node : cmp::Eq {
    pure fn eq(other : &Node) -> bool unsafe {
        let my_data : @LayoutData = @self.aux(|a| a);
        let ot_data : @LayoutData = @other.aux(|a| a);
        core::box::ptr_eq(my_data, ot_data)
    }
    pure fn ne(other : &Node) -> bool unsafe {
        !self.eq(other)
    }
}

impl Node : DebugMethods {
    /* Dumps the subtree rooted at this node, for debugging. */
    fn dump(&self) {
        self.dump_indent(0u);
    }
    /* Dumps the node tree, for debugging, with indentation. */
    fn dump_indent(&self, indent: uint) {
        let mut s = ~"";
        for uint::range(0u, indent) |_i| {
            s += ~"    ";
        }

        s += self.debug_str();
        debug!("%s", s);

        for NodeTree.each_child(*self) |kid| {
            kid.dump_indent(indent + 1u) 
        }
    }

    fn debug_str(&self) -> ~str {
        do self.read |n| { fmt!("%?", n.kind) }
    }
}

enum NodeKind {
    Doctype(DoctypeData),
    Comment(~str),
    Element(ElementData),
    Text(~str)
}

struct DoctypeData {
    name: ~str,
    public_id: Option<~str>,
    system_id: Option<~str>,
    force_quirks: bool
}

fn DoctypeData(name: ~str, public_id: Option<~str>,
               system_id: Option<~str>, force_quirks: bool) -> DoctypeData {
    DoctypeData {
        name : name,
        public_id : public_id,
        system_id : system_id,
        force_quirks : force_quirks,
    }
}



fn define_bindings(compartment: bare_compartment, doc: @Document,
                   win: @Window) {
    bindings::window::init(compartment, win);
    bindings::document::init(compartment, doc);
    bindings::node::init(compartment);
    bindings::element::init(compartment);
}


/** The RCU rd_aux data is a (weak) pointer to the layout data,
   defined by this `LayoutData` enum. It contains the CSS style object
   as well as the primary `RenderBox`.

   Note that there may be multiple boxes per DOM node. */
enum LayoutData = {
    mut style: ~SpecifiedStyle,
    mut flow:  Option<@FlowContext>
};

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

    fn get_parent(node: Node) -> Option<Node> {
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