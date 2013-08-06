/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use dom::bindings::codegen::TextBinding;
use dom::bindings::node;
use dom::bindings::utils::{WrapperCache, DOMString, null_string, ErrorResult};
use dom::bindings::utils::{BindingObject, CacheableWrapper};
use dom::bindings;
use dom::characterdata::CharacterData;
use dom::document::AbstractDocument;
use dom::element::{Element, ElementTypeId, HTMLImageElement, HTMLImageElementTypeId, HTMLIframeElementTypeId, HTMLIframeElement};
use dom::element::{HTMLStyleElementTypeId};
use dom::window::Window;

use std::cast;
use std::cast::transmute;
use std::libc::c_void;
use std::uint;
use js::jsapi::{JSObject, JSContext};
use js::rust::Compartment;
use netsurfcss::util::VoidPtrLike;
use servo_util::tree::{TreeNode, TreeNodeRef, TreeUtils};

//
// The basic Node structure
//

/// A phantom type representing the script task's view of this node. Script is able to mutate
/// nodes but may not access layout data.
#[deriving(Eq)]
pub struct ScriptView;

/// A phantom type representing the layout task's view of the node. Layout is not allowed to mutate
/// nodes but may access layout data.
#[deriving(Eq)]
pub struct LayoutView;

// We shouldn't need Eq for ScriptView and LayoutView; see Rust #7671.

/// This is what a Node looks like if you do not know what kind of node it is. To unpack it, use
/// downcast().
///
/// FIXME: This should be replaced with a trait once they can inherit from structs.
#[deriving(Eq)]
pub struct AbstractNode<View> {
    priv obj: *mut Node<View>,
}

pub struct AbstractNodeChildrenIterator<View> {
    priv current_node: Option<AbstractNode<View>>,
}

/// An HTML node.
///
/// `View` describes extra data associated with this node that this task has access to. For
/// the script task, this is the unit type `()`. For the layout task, this is
/// `layout::aux::LayoutData`.
pub struct Node<View> {
    /// The JavaScript wrapper for this node.
    wrapper: WrapperCache,

    /// The type of node that this is.
    type_id: NodeTypeId,

    abstract: Option<AbstractNode<View>>,

    /// The parent of this node.
    parent_node: Option<AbstractNode<View>>,

    /// The first child of this node.
    first_child: Option<AbstractNode<View>>,

    /// The last child of this node.
    last_child: Option<AbstractNode<View>>,

    /// The next sibling of this node.
    next_sibling: Option<AbstractNode<View>>,

    /// The previous sibling of this node.
    prev_sibling: Option<AbstractNode<View>>,

    /// The document that this node belongs to.
    owner_doc: Option<AbstractDocument>,

    /// Layout information. Only the layout task may touch this data.
    priv layout_data: Option<@mut ()>
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
// Basic node types
//

/// The `DOCTYPE` tag.
pub struct Doctype<View> {
    parent: Node<View>,
    name: ~str,
    public_id: Option<~str>,
    system_id: Option<~str>,
    force_quirks: bool
}

impl Doctype<ScriptView> {
    /// Creates a new `DOCTYPE` tag.
    pub fn new(name: ~str,
               public_id: Option<~str>,
               system_id: Option<~str>,
               force_quirks: bool)
            -> Doctype<ScriptView> {
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

    pub fn Constructor(owner: @mut Window, text: &DOMString, _rv: &mut ErrorResult) -> AbstractNode<ScriptView> {
        let cx = unsafe {(*owner.page).js_info.get_ref().js_compartment.cx.ptr};
        unsafe { Node::as_abstract_node(cx, @Text::new(text.to_str())) }
    }

    pub fn SplitText(&self, _offset: u32, _rv: &mut ErrorResult) -> AbstractNode<ScriptView> {
        fail!("unimplemented")
    }

    pub fn GetWholeText(&self, _rv: &mut ErrorResult) -> DOMString {
        null_string
    }
}

impl<View> Clone for AbstractNode<View> {
    fn clone(&self) -> AbstractNode<View> {
        *self
    }
}

impl<View> TreeNode<AbstractNode<View>> for Node<View> {
    fn parent_node(&self) -> Option<AbstractNode<View>> {
        self.parent_node
    }
    fn first_child(&self) -> Option<AbstractNode<View>> {
        self.first_child
    }
    fn last_child(&self) -> Option<AbstractNode<View>> {
        self.last_child
    }
    fn prev_sibling(&self) -> Option<AbstractNode<View>> {
        self.prev_sibling
    }
    fn next_sibling(&self) -> Option<AbstractNode<View>> {
        self.next_sibling
    }

    fn set_parent_node(&mut self, new_parent_node: Option<AbstractNode<View>>) {
        self.parent_node = new_parent_node
    }
    fn set_first_child(&mut self, new_first_child: Option<AbstractNode<View>>) {
        self.first_child = new_first_child
    }
    fn set_last_child(&mut self, new_last_child: Option<AbstractNode<View>>) {
        self.last_child = new_last_child
    }
    fn set_prev_sibling(&mut self, new_prev_sibling: Option<AbstractNode<View>>) {
        self.prev_sibling = new_prev_sibling
    }
    fn set_next_sibling(&mut self, new_next_sibling: Option<AbstractNode<View>>) {
        self.next_sibling = new_next_sibling
    }
}

impl<View> TreeNodeRef<Node<View>> for AbstractNode<View> {
    // FIXME: The duplication between `with_base` and `with_mut_base` is ugly.
    fn with_base<R>(&self, callback: &fn(&Node<View>) -> R) -> R {
        self.transmute(callback)
    }

    fn with_mut_base<R>(&self, callback: &fn(&mut Node<View>) -> R) -> R {
        self.transmute_mut(callback)
    }
}

impl<'self, View> AbstractNode<View> {
    // Unsafe accessors

    pub unsafe fn as_cacheable_wrapper(&self) -> @mut CacheableWrapper {
        match self.type_id() {
            TextNodeTypeId => {
                let node: @mut Text = cast::transmute(self.obj);
                node as @mut CacheableWrapper
            }
            _ => {
                fail!("unsupported node type")
            }
        }
    }

    /// Returns the layout data, unsafely cast to whatever type layout wishes. Only layout is
    /// allowed to call this. This is wildly unsafe and is therefore marked as such.
    pub unsafe fn unsafe_layout_data<T>(self) -> @mut T {
        do self.with_base |base| {
            transmute(base.layout_data.get())
        }
    }
    /// Returns true if this node has layout data and false otherwise.
    pub unsafe fn unsafe_has_layout_data(self) -> bool {
        do self.with_base |base| {
            base.layout_data.is_some()
        }
    }
    /// Sets the layout data, unsafely casting the type as layout wishes. Only layout is allowed
    /// to call this. This is wildly unsafe and is therefore marked as such.
    pub unsafe fn unsafe_set_layout_data<T>(self, data: @mut T) {
        // Don't decrement the refcount on data, since we're giving it to the
        // base structure.
        cast::forget(data);

        do self.with_mut_base |base| {
            base.layout_data = Some(transmute(data))
        }
    }

    // Convenience accessors

    /// Returns the type ID of this node. Fails if this node is borrowed mutably.
    pub fn type_id(self) -> NodeTypeId {
        self.with_base(|b| b.type_id)
    }

    /// Returns the parent node of this node. Fails if this node is borrowed mutably.
    pub fn parent_node(self) -> Option<AbstractNode<View>> {
        self.with_base(|b| b.parent_node)
    }

    /// Returns the first child of this node. Fails if this node is borrowed mutably.
    pub fn first_child(self) -> Option<AbstractNode<View>> {
        self.with_base(|b| b.first_child)
    }

    /// Returns the last child of this node. Fails if this node is borrowed mutably.
    pub fn last_child(self) -> Option<AbstractNode<View>> {
        self.with_base(|b| b.last_child)
    }

    /// Returns the previous sibling of this node. Fails if this node is borrowed mutably.
    pub fn prev_sibling(self) -> Option<AbstractNode<View>> {
        self.with_base(|b| b.prev_sibling)
    }

    /// Returns the next sibling of this node. Fails if this node is borrowed mutably.
    pub fn next_sibling(self) -> Option<AbstractNode<View>> {
        self.with_base(|b| b.next_sibling)
    }

    /// Is this node a root?
    pub fn is_root(self) -> bool {
        self.parent_node().is_none()
    }

    //
    // Downcasting borrows
    //

    pub fn transmute<T, R>(self, f: &fn(&T) -> R) -> R {
        unsafe {
            let node_box: *mut bindings::utils::rust_box<Node<View>> = transmute(self.obj);
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
            let node_box: *mut bindings::utils::rust_box<Node<View>> = transmute(self.obj);
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

    pub fn is_iframe_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLIframeElementTypeId)
    }

    pub fn with_imm_iframe_element<R>(self, f: &fn(&HTMLIframeElement) -> R) -> R {
        if !self.is_iframe_element() {
            fail!(~"node is not an iframe element");
        }
        self.transmute(f)
    }

    pub fn with_mut_iframe_element<R>(self, f: &fn(&mut HTMLIframeElement) -> R) -> R {
        if !self.is_iframe_element() {
            fail!(~"node is not an iframe element");
        }
        self.transmute_mut(f)
    }

    pub fn is_style_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLStyleElementTypeId)
    }

    pub unsafe fn raw_object(self) -> *mut Node<View> {
        self.obj
    }

    pub fn from_raw(raw: *mut Node<View>) -> AbstractNode<View> {
        AbstractNode {
            obj: raw
        }
    }

    /// Dumps the subtree rooted at this node, for debugging.
    pub fn dump(&self) {
        self.dump_indent(0);
    }

    /// Dumps the node tree, for debugging, with indentation.
    pub fn dump_indent(&self, indent: uint) {
        let mut s = ~"";
        for uint::range(0u, indent) |_i| {
            s.push_str("    ");
        }

        s.push_str(self.debug_str());
        debug!("%s", s);

        // FIXME: this should have a pure version?
        for self.each_child() |kid| {
            kid.dump_indent(indent + 1u)
        }
    }

    /// Returns a string that describes this node.
    pub fn debug_str(&self) -> ~str {
        fmt!("%?", self.type_id())
    }

    pub fn children(&self) -> AbstractNodeChildrenIterator<View> {
        AbstractNodeChildrenIterator {
            current_node: self.first_child(),
        }
    }
}

impl<View> Iterator<AbstractNode<View>> for AbstractNodeChildrenIterator<View> {
    pub fn next(&mut self) -> Option<AbstractNode<View>> {
        let node = self.current_node;
        self.current_node = self.current_node.chain(|node| node.next_sibling());
        node
    }
}

impl Node<ScriptView> {
    pub unsafe fn as_abstract_node<N>(cx: *JSContext, node: @N) -> AbstractNode<ScriptView> {
        // This surrenders memory management of the node!
        let mut node = AbstractNode {
            obj: transmute(node),
        };
        node::create(cx, &mut node);
        node
    }

    pub fn add_to_doc(&mut self, doc: AbstractDocument) {
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

    pub fn new(type_id: NodeTypeId) -> Node<ScriptView> {
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

    pub fn getNodeType(&self) -> i32 {
        match self.type_id {
            ElementNodeTypeId(_) => 1,
            TextNodeTypeId       => 3,
            CommentNodeTypeId    => 8,
            DoctypeNodeTypeId    => 10
        }
    }

    pub fn getNextSibling(&mut self) -> Option<&mut AbstractNode<ScriptView>> {
        match self.next_sibling {
            // transmute because the compiler can't deduce that the reference
            // is safe outside of with_mut_base blocks.
            Some(ref mut n) => Some(unsafe { cast::transmute(n) }),
            None => None
        }
    }

    pub fn getFirstChild(&mut self) -> Option<&mut AbstractNode<ScriptView>> {
        match self.first_child {
            // transmute because the compiler can't deduce that the reference
            // is safe outside of with_mut_base blocks.
            Some(ref mut n) => Some(unsafe { cast::transmute(n) }),
            None => None
        }
    }
 }

impl Node<ScriptView> {
    pub fn NodeType(&self) -> u16 {
        0
    }

    pub fn NodeName(&self) -> DOMString {
        null_string
    }

    pub fn GetBaseURI(&self) -> DOMString {
        null_string
    }

    pub fn GetOwnerDocument(&self) -> Option<AbstractDocument> {
        None
    }

    pub fn GetParentNode(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn GetParentElement(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn HasChildNodes(&self) -> bool {
        false
    }

    pub fn GetFirstChild(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn GetLastChild(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn GetPreviousSibling(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn GetNextSibling(&self) -> Option<AbstractNode<ScriptView>> {
        None
    }

    pub fn GetNodeValue(&self) -> DOMString {
        null_string
    }

    pub fn SetNodeValue(&mut self, _val: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn GetTextContent(&self) -> DOMString {
        null_string
    }

    pub fn SetTextContent(&mut self, _val: &DOMString, _rv: &mut ErrorResult) {
    }

    pub fn InsertBefore(&mut self, _node: AbstractNode<ScriptView>, _child: Option<AbstractNode<ScriptView>>, _rv: &mut ErrorResult) -> AbstractNode<ScriptView> {
        fail!("stub")
    }

    pub fn AppendChild(&mut self, _node: AbstractNode<ScriptView>, _rv: &mut ErrorResult) -> AbstractNode<ScriptView> {
        fail!("stub")
    }

    pub fn ReplaceChild(&mut self, _node: AbstractNode<ScriptView>, _child: AbstractNode<ScriptView>, _rv: &mut ErrorResult) -> AbstractNode<ScriptView> {
        fail!("stub")
    }

    pub fn RemoveChild(&mut self, _node: AbstractNode<ScriptView>, _rv: &mut ErrorResult) -> AbstractNode<ScriptView> {
        fail!("stub")
    }

    pub fn Normalize(&mut self) {
    }

    pub fn CloneNode(&self, _deep: bool, _rv: &mut ErrorResult) -> AbstractNode<ScriptView> {
        fail!("stub")
    }

    pub fn IsEqualNode(&self, _node: Option<AbstractNode<ScriptView>>) -> bool {
        false
    }

    pub fn CompareDocumentPosition(&self, _other: AbstractNode<ScriptView>) -> u16 {
        0
    }

    pub fn Contains(&self, _other: Option<AbstractNode<ScriptView>>) -> bool {
        false
    }

    pub fn LookupPrefix(&self, _prefix: &DOMString) -> DOMString {
        null_string
    }

    pub fn LookupNamespaceURI(&self, _namespace: &DOMString) -> DOMString {
        null_string
    }

    pub fn IsDefaultNamespace(&self, _namespace: &DOMString) -> bool {
        false
    }

    pub fn GetNamespaceURI(&self) -> DOMString {
        null_string
    }

    pub fn GetPrefix(&self) -> DOMString {
        null_string
    }

    pub fn GetLocalName(&self) -> DOMString {
        null_string
    }

    pub fn HasAttributes(&self) -> bool {
        false
    }
}

/// The CSS library requires that DOM nodes be convertible to `*c_void` via the `VoidPtrLike`
/// trait.
impl VoidPtrLike for AbstractNode<LayoutView> {
    fn from_void_ptr(node: *c_void) -> AbstractNode<LayoutView> {
        assert!(node.is_not_null());
        unsafe {
            cast::transmute(node)
        }
    }

    fn to_void_ptr(&self) -> *c_void {
        unsafe {
            cast::transmute(*self)
        }
    }
}

pub fn define_bindings(compartment: @mut Compartment) {
    bindings::node::init(compartment);
    bindings::element::init(compartment);
    bindings::text::init(compartment);
    bindings::utils::initialize_global(compartment.global_obj.ptr);
    bindings::codegen::RegisterBindings::Register(compartment);
}

impl CacheableWrapper for Node<ScriptView> {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        unsafe { cast::transmute(&mut self.wrapper) }
    }

    fn wrap_object_shared(@mut self, _cx: *JSContext, _scope: *JSObject) -> *JSObject {
        fail!(~"need to implement wrapping");
    }
}

impl BindingObject for Node<ScriptView> {
    fn GetParentObject(&self, _cx: *JSContext) -> Option<@mut CacheableWrapper> {
        match self.parent_node {
            Some(node) => Some(unsafe {node.as_cacheable_wrapper()}),
            None => None
        }
    }
}

impl CacheableWrapper for Text {
    fn get_wrappercache(&mut self) -> &mut WrapperCache {
        self.parent.get_wrappercache()
    }

    fn wrap_object_shared(@mut self, cx: *JSContext, scope: *JSObject) -> *JSObject {
        let mut unused = false;
        TextBinding::Wrap(cx, scope, self, &mut unused)
    }
}

impl BindingObject for Text {
    fn GetParentObject(&self, cx: *JSContext) -> Option<@mut CacheableWrapper> {
        self.parent.GetParentObject(cx)
    }
}
