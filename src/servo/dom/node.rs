/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use content::content_task::global_content;
use dom::bindings::codegen;
use dom::bindings::node;
use dom::bindings::utils::WrapperCache;
use dom::bindings;
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::element::{Element, ElementTypeId, HTMLImageElement, HTMLImageElementTypeId};
use dom::element::{HTMLStyleElementTypeId};
use js::rust::Compartment;
use layout::debug::DebugMethods;
use layout::flow::FlowContext;
use newcss::complete::CompleteSelectResults;

use core::cast::transmute;
use servo_util::tree::{TreeNode, TreeNodeRef, TreeUtils};

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
    fn eq(&self, other: &AbstractNode) -> bool {
        self.obj == other.obj
    }
    fn ne(&self, other: &AbstractNode) -> bool {
        self.obj != other.obj
    }
}

/// An HTML node.
pub struct Node {
    /// The JavaScript wrapper for this node.
    wrapper: WrapperCache,

    /// The type of node that this is.
    type_id: NodeTypeId,

    abstract: Option<AbstractNode>,

    /// The parent of this node.
    parent_node: Option<AbstractNode>,

    /// The first child of this node.
    first_child: Option<AbstractNode>,

    /// The last child of this node.
    last_child: Option<AbstractNode>,

    /// The next sibling of this node.
    next_sibling: Option<AbstractNode>,

    /// The previous sibling of this node.
    prev_sibling: Option<AbstractNode>,

    /// The document that this node belongs to.
    owner_doc: Option<@mut Document>,

    /// Layout information. You must not touch this if you are not layout.
    priv layout_data: Option<@mut LayoutData>
}

/// The different types of nodes.
#[deriving(Eq)]
pub enum NodeTypeId {
    DoctypeNodeTypeId,
    CommentNodeTypeId,
    ElementNodeTypeId(ElementTypeId),
    TextNodeTypeId,
}

//
// Auxiliary layout data
//

/// Data that layout associates with a node.
pub struct LayoutData {
    /// The results of CSS styling for this node.
    style: Option<CompleteSelectResults>,

    /// The CSS flow that this node is associated with.
    flow: Option<FlowContext>,
}

impl LayoutData {
    /// Creates new layout data.
    pub fn new() -> LayoutData {
        LayoutData {
            style: None,
            flow: None,
        }
    }
}

//
// Basic node types
//

/// The `DOCTYPE` tag.
pub struct Doctype {
    parent: Node,
    name: ~str,
    public_id: Option<~str>,
    system_id: Option<~str>,
    force_quirks: bool
}

impl Doctype {
    /// Creates a new `DOCTYPE` tag.
    pub fn new(name: ~str,
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

/// An HTML comment.
pub struct Comment {
    parent: CharacterData,
}

impl Comment {
    /// Creates a new HTML comment.
    pub fn new(text: ~str) -> Comment {
        Comment {
            parent: CharacterData::new(CommentNodeTypeId, text)
        }
    }
}

/// An HTML text node.
pub struct Text {
    parent: CharacterData,
}

impl Text {
    /// Creates a new HTML text node.
    pub fn new(text: ~str) -> Text {
        Text {
            parent: CharacterData::new(TextNodeTypeId, text)
        }
    }
}

impl Clone for AbstractNode {
    fn clone(&self) -> AbstractNode {
        *self
    }
}

impl TreeNode<AbstractNode> for Node {
    fn parent_node(&self) -> Option<AbstractNode> {
        self.parent_node
    }
    fn first_child(&self) -> Option<AbstractNode> {
        self.first_child
    }
    fn last_child(&self) -> Option<AbstractNode> {
        self.last_child
    }
    fn prev_sibling(&self) -> Option<AbstractNode> {
        self.prev_sibling
    }
    fn next_sibling(&self) -> Option<AbstractNode> {
        self.next_sibling
    }

    fn set_parent_node(&mut self, new_parent_node: Option<AbstractNode>) {
        self.parent_node = new_parent_node
    }
    fn set_first_child(&mut self, new_first_child: Option<AbstractNode>) {
        self.first_child = new_first_child
    }
    fn set_last_child(&mut self, new_last_child: Option<AbstractNode>) {
        self.last_child = new_last_child
    }
    fn set_prev_sibling(&mut self, new_prev_sibling: Option<AbstractNode>) {
        self.prev_sibling = new_prev_sibling
    }
    fn set_next_sibling(&mut self, new_next_sibling: Option<AbstractNode>) {
        self.next_sibling = new_next_sibling
    }
}

impl TreeNodeRef<Node> for AbstractNode {
    // FIXME: The duplication between `with_base` and `with_mut_base` is ugly.
    fn with_base<R>(&self, callback: &fn(&Node) -> R) -> R {
        self.transmute(callback)
    }

    fn with_mut_base<R>(&self, callback: &fn(&mut Node) -> R) -> R {
        self.transmute_mut(callback)
    }
}

impl AbstractNode {
    // Convenience accessors

    /// Returns the type ID of this node. Fails if this node is borrowed mutably.
    pub fn type_id(self) -> NodeTypeId {
        self.with_base(|b| b.type_id)
    }

    /// Returns the parent node of this node. Fails if this node is borrowed mutably.
    pub fn parent_node(self) -> Option<AbstractNode> {
        self.with_base(|b| b.parent_node)
    }

    /// Returns the first child of this node. Fails if this node is borrowed mutably.
    pub fn first_child(self) -> Option<AbstractNode> {
        self.with_base(|b| b.first_child)
    }

    /// Returns the last child of this node. Fails if this node is borrowed mutably.
    pub fn last_child(self) -> Option<AbstractNode> {
        self.with_base(|b| b.last_child)
    }

    /// Returns the previous sibling of this node. Fails if this node is borrowed mutably.
    pub fn prev_sibling(self) -> Option<AbstractNode> {
        self.with_base(|b| b.prev_sibling)
    }

    /// Returns the next sibling of this node. Fails if this node is borrowed mutably.
    pub fn next_sibling(self) -> Option<AbstractNode> {
        self.with_base(|b| b.next_sibling)
    }

    // NB: You must not call these if you are not layout. We should do something with scoping to
    // ensure this.
    pub fn layout_data(self) -> @mut LayoutData {
        self.with_base(|b| b.layout_data.get())
    }
    pub fn has_layout_data(self) -> bool {
        self.with_base(|b| b.layout_data.is_some())
    }
    pub fn set_layout_data(self, data: @mut LayoutData) {
        self.with_mut_base(|b| b.layout_data = Some(data))
    }

    //
    // Downcasting borrows
    //

    pub fn transmute<T, R>(self, f: &fn(&T) -> R) -> R {
        unsafe {
            let node_box: *mut bindings::utils::rust_box<Node> = transmute(self.obj);
            let node = &mut (*node_box).payload;
            let old = node.abstract;
            node.abstract = Some(self);
            let box: *bindings::utils::rust_box<T> = transmute(self.obj);
            let rv = f(&(*box).payload);
            node.abstract = old;
            rv
        }
    }

    pub fn transmute_mut<T, R>(self, f: &fn(&mut T) -> R) -> R {
        unsafe {
            let node_box: *mut bindings::utils::rust_box<Node> = transmute(self.obj);
            let node = &mut (*node_box).payload;
            let old = node.abstract;
            node.abstract = Some(self);
            let box: *bindings::utils::rust_box<T> = transmute(self.obj);
            let rv = f(cast::transmute(&(*box).payload));
            node.abstract = old;
            rv
        }
    }

    pub fn is_text(self) -> bool {
        self.type_id() == TextNodeTypeId
    }

    // FIXME: This should be doing dynamic borrow checking for safety.
    pub fn with_imm_text<R>(self, f: &fn(&Text) -> R) -> R {
        if !self.is_text() {
            fail!(~"node is not text");
        }
        self.transmute(f)
    }

    pub fn is_element(self) -> bool {
        match self.type_id() {
            ElementNodeTypeId(*) => true,
            _ => false
        }
    }

    // FIXME: This should be doing dynamic borrow checking for safety.
    pub fn with_imm_element<R>(self, f: &fn(&Element) -> R) -> R {
        if !self.is_element() {
            fail!(~"node is not an element");
        }
        self.transmute(f)
    }

    // FIXME: This should be doing dynamic borrow checking for safety.
    pub fn as_mut_element<R>(self, f: &fn(&mut Element) -> R) -> R {
        if !self.is_element() {
            fail!(~"node is not an element");
        }
        self.transmute_mut(f)
    }

    pub fn is_image_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLImageElementTypeId)
    }

    pub fn with_imm_image_element<R>(self, f: &fn(&HTMLImageElement) -> R) -> R {
        if !self.is_image_element() {
            fail!(~"node is not an image element");
        }
        self.transmute(f)
    }

    pub fn with_mut_image_element<R>(self, f: &fn(&mut HTMLImageElement) -> R) -> R {
        if !self.is_image_element() {
            fail!(~"node is not an image element");
        }
        self.transmute_mut(f)
    }

    pub fn is_style_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLStyleElementTypeId)
    }

    pub unsafe fn raw_object(self) -> *mut Node {
        self.obj
    }

    pub fn from_raw(raw: *mut Node) -> AbstractNode {
        AbstractNode {
            obj: raw
        }
    }
}

impl DebugMethods for AbstractNode {
    // Dumps the subtree rooted at this node, for debugging.
    fn dump(&self) {
        self.dump_indent(0);
    }

    // Dumps the node tree, for debugging, with indentation.
    fn dump_indent(&self, indent: uint) {
        let mut s = ~"";
        for uint::range(0u, indent) |_i| {
            s += ~"    ";
        }

        s += self.debug_str();
        debug!("%s", s);

        // FIXME: this should have a pure version?
        for self.each_child() |kid| {
            kid.dump_indent(indent + 1u)
        }
    }

    fn debug_str(&self) -> ~str {
        fmt!("%?", self.type_id())
    }
}

impl Node {
    pub unsafe fn as_abstract_node<N>(node: ~N) -> AbstractNode {
        // This surrenders memory management of the node!
        let mut node = AbstractNode {
            obj: transmute(node),
        };
        let cx = global_content().compartment.get().cx.ptr;
        node::create(cx, &mut node);
        node
    }

    pub fn add_to_doc(&mut self, doc: @mut Document) {
        self.owner_doc = Some(doc);
        let mut node = self.first_child;
        while node.is_some() {
            for node.get().traverse_preorder |node| {
                do node.with_mut_base |node_base| {
                    node_base.owner_doc = Some(doc);
                }
            };
            node = node.get().next_sibling();
        }
    }

    pub fn new(type_id: NodeTypeId) -> Node {
        Node {
            wrapper: WrapperCache::new(),
            type_id: type_id,

            abstract: None,

            parent_node: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,

            owner_doc: None,

            layout_data: None,
        }
    }
}

pub fn define_bindings(compartment: @mut Compartment) {
    bindings::window::init(compartment);
    bindings::document::init(compartment);
    bindings::node::init(compartment);
    bindings::element::init(compartment);
    bindings::text::init(compartment);
    bindings::utils::initialize_global(compartment.global_obj.ptr);
    let mut unused = false;
    assert!(codegen::ClientRectBinding::DefineDOMInterface(compartment.cx.ptr,
                                                           compartment.global_obj.ptr,
                                                           &mut unused));
    assert!(codegen::ClientRectListBinding::DefineDOMInterface(compartment.cx.ptr,
                                                               compartment.global_obj.ptr,
                                                               &mut unused));
    assert!(codegen::HTMLCollectionBinding::DefineDOMInterface(compartment.cx.ptr,
                                                               compartment.global_obj.ptr,
                                                               &mut unused));
    assert!(codegen::DOMParserBinding::DefineDOMInterface(compartment.cx.ptr,
                                                          compartment.global_obj.ptr,
                                                          &mut unused));
    assert!(codegen::EventBinding::DefineDOMInterface(compartment.cx.ptr,
                                                      compartment.global_obj.ptr,
                                                      &mut unused));
    assert!(codegen::EventTargetBinding::DefineDOMInterface(compartment.cx.ptr,
                                                            compartment.global_obj.ptr,
                                                            &mut unused));
}
