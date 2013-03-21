//
// The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.
//

use dom::bindings;
use dom::bindings::codegen;
use dom::bindings::utils::WrapperCache;
use dom::document::Document;
use dom::element::{Element, ElementTypeId, HTMLImageElement, HTMLImageElementTypeId};
use dom::element::{HTMLStyleElementTypeId};
use dom::window::Window;
use layout::debug::DebugMethods;
use layout::flow::FlowContext;
use newcss::complete::CompleteSelectResults;

use core::cast::transmute;
use core::ptr::null;
use geom::size::Size2D;
use js::crust::*;
use js::glue::bindgen::RUST_OBJECT_TO_JSVAL;
use js::jsapi::{JSClass, JSObject, JSPropertySpec, JSContext, jsid, JSVal, JSBool};
use js::jsapi::bindgen::JS_SetReservedSlot;
use js::rust::Compartment;
use std::arc::ARC;

//
// The basic Node structure
//

/// This is what a Node looks like if you do not know what kind of node it is. To unpack it, use
/// downcast().
///
/// FIXME: This should be replaced with a trait once they can inherit from structs.
pub struct AbstractNode {
    priv obj: *mut Node,
}

impl Eq for AbstractNode {
    pure fn eq(&self, other: &AbstractNode) -> bool { self.obj == other.obj }
    pure fn ne(&self, other: &AbstractNode) -> bool { self.obj != other.obj }
}

pub struct Node {
    wrapper: WrapperCache,
    type_id: NodeTypeId,

    parent_node: Option<AbstractNode>,
    first_child: Option<AbstractNode>,
    last_child: Option<AbstractNode>,
    next_sibling: Option<AbstractNode>,
    prev_sibling: Option<AbstractNode>,

    // You must not touch this if you are not layout.
    priv layout_data: Option<@mut LayoutData>
}

#[deriving_eq]
pub enum NodeTypeId {
    DoctypeNodeTypeId,
    CommentNodeTypeId,
    ElementNodeTypeId(ElementTypeId),
    TextNodeTypeId,
}

//
// Auxiliary layout data
//

pub struct LayoutData {
    style: Option<CompleteSelectResults>,
    flow: Option<@mut FlowContext>,
}

impl LayoutData {
    static pub fn new() -> LayoutData {
        LayoutData {
            style: None,
            flow: None,
        }
    }
}

//
// Basic node types
//

pub struct Doctype {
    parent: Node,
    name: ~str,
    public_id: Option<~str>,
    system_id: Option<~str>,
    force_quirks: bool
}

impl Doctype {
    static pub fn new(name: ~str,
                      public_id: Option<~str>,
                      system_id: Option<~str>,
                      force_quirks: bool)
                   -> Doctype {
        Doctype {
            parent: Node::new(DoctypeNodeTypeId),
            name: name,
            public_id: public_id,
            system_id: system_id,
            force_quirks: force_quirks,
        }
    }
}

pub struct Comment {
    parent: Node,
    text: ~str,
}

impl Comment {
    static pub fn new(text: ~str) -> Comment {
        Comment {
            parent: Node::new(CommentNodeTypeId),
            text: text
        }
    }
}

pub struct Text {
    parent: Node,
    text: ~str,
}

impl Text {
    static pub fn new(text: ~str) -> Text {
        Text {
            parent: Node::new(TextNodeTypeId),
            text: text
        }
    }
}

pub impl AbstractNode {
    //
    // Convenience accessors
    //
    // FIXME: Fold these into util::tree.

    fn type_id(self)         -> NodeTypeId           { self.with_imm_node(|n| n.type_id)      }
    fn parent_node(self)     -> Option<AbstractNode> { self.with_imm_node(|n| n.parent_node)  }
    fn first_child(self)     -> Option<AbstractNode> { self.with_imm_node(|n| n.first_child)  }
    fn last_child(self)      -> Option<AbstractNode> { self.with_imm_node(|n| n.last_child)   }
    fn prev_sibling(self)    -> Option<AbstractNode> { self.with_imm_node(|n| n.prev_sibling) }
    fn next_sibling(self)    -> Option<AbstractNode> { self.with_imm_node(|n| n.next_sibling) }

    // NB: You must not call these if you are not layout. We should do something with scoping to
    // ensure this.
    fn layout_data(self)     -> @mut LayoutData {
        self.with_imm_node(|n| n.layout_data.get())
    }
    fn has_layout_data(self) -> bool {
        self.with_imm_node(|n| n.layout_data.is_some())
    }
    fn set_layout_data(self, data: @mut LayoutData) {
        self.with_mut_node(|n| n.layout_data = Some(data))
    }

    //
    // Tree operations
    //
    // FIXME: Fold this into util::tree.
    //

    fn is_leaf(self) -> bool { self.first_child().is_none() }

    // Invariant: `child` is disconnected from the document.
    fn append_child(self, child: AbstractNode) {
        fail_unless!(self != child);

        do self.with_mut_node |parent_n| {
            do child.with_mut_node |child_n| {
                fail_unless!(child_n.parent_node.is_none());
                fail_unless!(child_n.prev_sibling.is_none());
                fail_unless!(child_n.next_sibling.is_none());

                child_n.parent_node = Some(self);

                match parent_n.last_child {
                    None => parent_n.first_child = Some(child),
                    Some(last_child) => {
                        do last_child.with_mut_node |last_child_n| {
                            fail_unless!(last_child_n.next_sibling.is_none());
                            last_child_n.next_sibling = Some(child);
                        }
                    }
                }

                child_n.prev_sibling = parent_n.last_child;
                parent_n.last_child = Some(child);
            }
        }
    }

    //
    // Tree traversal
    //
    // FIXME: Fold this into util::tree.
    //

    fn each_child(self, f: &fn(AbstractNode) -> bool) {
        let mut current_opt = self.first_child();
        while !current_opt.is_none() {
            let current = current_opt.get();
            if !f(current) {
                break;
            }
            current_opt = current.next_sibling();
        }
    }

    fn traverse_preorder(self, f: &fn(AbstractNode) -> bool) -> bool {
        if !f(self) {
            return false;
        }
        for self.each_child |kid| {
            if !kid.traverse_preorder(f) {
                return false;
            }
        }
        true
    }

    fn traverse_postorder(self, f: &fn(AbstractNode) -> bool) -> bool {
        for self.each_child |kid| {
            if !kid.traverse_postorder(f) {
                return false;
            }
        }
        f(self)
    }

    //
    // Downcasting borrows
    //

    fn transmute<T, R>(self, f: &fn(&T) -> R) -> R {
        unsafe {
            let box: *bindings::utils::rust_box<T> = transmute(self.obj);
            f(&(*box).payload)
        }
    }

    fn transmute_mut<T, R>(self, f: &fn(&mut T) -> R) -> R {
        unsafe {
            let box: *bindings::utils::rust_box<T> = transmute(self.obj);
            f(cast::transmute(&(*box).payload))
        }
    }

    fn with_imm_node<R>(self, f: &fn(&Node) -> R) -> R {
        self.transmute(f)
    }

    fn with_mut_node<R>(self, f: &fn(&mut Node) -> R) -> R {
        self.transmute_mut(f)
    }

    fn is_text(self) -> bool { self.type_id() == TextNodeTypeId }

    // FIXME: This should be doing dynamic borrow checking for safety.
    fn with_imm_text<R>(self, f: &fn(&Text) -> R) -> R {
        if !self.is_text() {
            fail!(~"node is not text");
        }
        self.transmute(f)
    }

    fn is_element(self) -> bool {
        match self.type_id() {
            ElementNodeTypeId(*) => true,
            _ => false
        }
    }

    // FIXME: This should be doing dynamic borrow checking for safety.
    fn with_imm_element<R>(self, f: &fn(&Element) -> R) -> R {
        if !self.is_element() {
            fail!(~"node is not an element");
        }
        self.transmute(f)
    }

    // FIXME: This should be doing dynamic borrow checking for safety.
    fn as_mut_element<R>(self, f: &fn(&mut Element) -> R) -> R {
        if !self.is_element() {
            fail!(~"node is not an element");
        }
        self.transmute_mut(f)
    }

    fn is_image_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLImageElementTypeId)
    }

    fn with_imm_image_element<R>(self, f: &fn(&HTMLImageElement) -> R) -> R {
        if !self.is_image_element() {
            fail!(~"node is not an image element");
        }
        self.transmute(f)
    }

    fn with_mut_image_element<R>(self, f: &fn(&mut HTMLImageElement) -> R) -> R {
        if !self.is_image_element() {
            fail!(~"node is not an image element");
        }
        self.transmute_mut(f)
    }

    fn is_style_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLStyleElementTypeId)
    }

    unsafe fn raw_object(self) -> *mut Node {
        self.obj
    }
}

impl DebugMethods for AbstractNode {
    // Dumps the subtree rooted at this node, for debugging.
    pure fn dump(&self) {
        self.dump_indent(0);
    }

    // Dumps the node tree, for debugging, with indentation.
    pure fn dump_indent(&self, indent: uint) {
        let mut s = ~"";
        for uint::range(0u, indent) |_i| {
            s += ~"    ";
        }

        s += self.debug_str();
        debug!("%s", s);

        // FIXME: this should have a pure version?
        unsafe {
            for self.each_child() |kid| {
                kid.dump_indent(indent + 1u) 
            }
        }
    }

    pure fn debug_str(&self) -> ~str {
        // Unsafe due to the call to type_id().
        unsafe {
            fmt!("%?", self.type_id())
        }
    }
}

impl Node {
    static pub unsafe fn as_abstract_node<N>(node: ~N) -> AbstractNode {
        // This surrenders memory management of the node!
        AbstractNode {
            obj: transmute(node),
        }
    }

    static pub fn new(type_id: NodeTypeId) -> Node {
        Node {
            wrapper: WrapperCache::new(),
            type_id: type_id,

            parent_node: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,

            layout_data: None,
        }
    }
}

pub fn define_bindings(compartment: @mut Compartment, doc: @mut Document, win: @mut Window) {
    bindings::window::init(compartment, win);
    bindings::document::init(compartment, doc);
    bindings::node::init(compartment);
    bindings::element::init(compartment);
    bindings::utils::initialize_global(compartment.global_obj.ptr);
    let mut unused = false;
    fail_unless!(codegen::ClientRectBinding::DefineDOMInterface(compartment.cx.ptr,
                                                                compartment.global_obj.ptr,
                                                                &mut unused));
    fail_unless!(codegen::ClientRectListBinding::DefineDOMInterface(compartment.cx.ptr,
                                                                    compartment.global_obj.ptr,
                                                                    &mut unused));
    fail_unless!(codegen::HTMLCollectionBinding::DefineDOMInterface(compartment.cx.ptr,
                                                                    compartment.global_obj.ptr,
                                                                    &mut unused));
}
