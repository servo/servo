#[doc="The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements."]

use gfx::geometry::au;
use geom::size::Size2D;
use layout::base::LayoutData;
use util::tree;
use js::rust::{bare_compartment, compartment, methods};
use js::jsapi::{JSClass, JSObject, JSPropertySpec, JSContext, jsid, jsval, JSBool};
use js::{JSPROP_ENUMERATE, JSPROP_SHARED};
use js::crust::*;
use js::glue::bindgen::RUST_OBJECT_TO_JSVAL;
use dvec::DVec;
use ptr::null;
use dom::bindings;
use std::arc::ARC;
use css::values::Stylesheet;
use comm::{Port, Chan};
use content::content_task::{ControlMsg, Timer};

enum TimerControlMsg {
    Fire(~dom::bindings::window::TimerData),
    Close
}

struct Window {
    timer_chan: Chan<TimerControlMsg>,

    drop {
        self.timer_chan.send(Close);
    }
}

fn Window(content_port: Port<ControlMsg>) -> Window {
    let content_chan = Chan(content_port);
        
    Window {
        timer_chan: do task::spawn_listener |timer_port: Port<TimerControlMsg>| {
            loop {
                match timer_port.recv() {
                    Close => break,
                    Fire(td) => {
                        content_chan.send(Timer(copy td));
                    }
                }
            }
        }
    }
}

struct Document {
    root: Node,
    scope: NodeScope,
    css_rules: ARC<Stylesheet>,
}

fn Document(root: Node, scope: NodeScope, -css_rules: Stylesheet) -> Document {
    Document {
        root : root,
        scope : scope,
        css_rules : ARC(css_rules),
    }
}

enum NodeData = {
    tree: tree::Tree<Node>,
    kind: ~NodeKind,
};

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

struct ElementData {
    tag_name: ~str,
    kind: ~ElementKind,
    attrs: DVec<~Attr>,
}

impl ElementData {
    fn get_attr(attr_name: ~str) -> Option<~str> {
        let mut i = 0u;
        while i < self.attrs.len() {
            if attr_name == self.attrs[i].name {
                return Some(copy self.attrs[i].value);
            }
            i += 1u;
        }

        None
    }
}


fn ElementData(tag_name: ~str, kind: ~ElementKind) -> ElementData {
    ElementData {
        tag_name : tag_name,
        kind : kind,
        attrs : DVec(),
    }
}


struct Attr {
    name: ~str,
    value: ~str,
}

fn Attr(name: ~str, value: ~str) -> Attr {
    Attr {
        name : name,
        value : value,
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

