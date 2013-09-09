/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use dom::bindings::node;
use dom::bindings::utils::{WrapperCache, DOMString, null_string, str, ErrorResult};
use dom::bindings::utils::{BindingObject, CacheableWrapper, rust_box};
use dom::bindings;
use dom::characterdata::CharacterData;
use dom::document::AbstractDocument;
use dom::element::{Element, ElementTypeId, HTMLImageElementTypeId, HTMLIframeElementTypeId};
use dom::element::{HTMLStyleElementTypeId};
use dom::htmlimageelement::HTMLImageElement;
use dom::htmliframeelement::HTMLIFrameElement;
use dom::text::Text;

use std::cast;
use std::cast::transmute;
use std::libc::c_void;
use extra::arc::Arc;
use js::jsapi::{JSObject, JSContext};
use js::rust::Compartment;
use netsurfcss::util::VoidPtrLike;
use newcss::complete::CompleteSelectResults;
use servo_util::tree::{TreeNode, TreeNodeRef};
use servo_util::range::Range;
use gfx::display_list::DisplayList;

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
/// `LayoutData`.
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
    priv layout_data: LayoutData,
}

/// The different types of nodes.
#[deriving(Eq)]
pub enum NodeTypeId {
    DoctypeNodeTypeId,
    CommentNodeTypeId,
    ElementNodeTypeId(ElementTypeId),
    TextNodeTypeId,
}

impl<View> Clone for AbstractNode<View> {
    fn clone(&self) -> AbstractNode<View> {
        *self
    }
}

impl<View> TreeNodeRef<Node<View>> for AbstractNode<View> {
    fn parent_node(node: &Node<View>) -> Option<AbstractNode<View>> {
        node.parent_node
    }
    fn first_child(node: &Node<View>) -> Option<AbstractNode<View>> {
        node.first_child
    }
    fn last_child(node: &Node<View>) -> Option<AbstractNode<View>> {
        node.last_child
    }
    fn prev_sibling(node: &Node<View>) -> Option<AbstractNode<View>> {
        node.prev_sibling
    }
    fn next_sibling(node: &Node<View>) -> Option<AbstractNode<View>> {
        node.next_sibling
    }

    fn set_parent_node(node: &mut Node<View>, new_parent_node: Option<AbstractNode<View>>) {
        node.parent_node = new_parent_node
    }
    fn set_first_child(node: &mut Node<View>, new_first_child: Option<AbstractNode<View>>) {
        node.first_child = new_first_child
    }
    fn set_last_child(node: &mut Node<View>, new_last_child: Option<AbstractNode<View>>) {
        node.last_child = new_last_child
    }
    fn set_prev_sibling(node: &mut Node<View>, new_prev_sibling: Option<AbstractNode<View>>) {
        node.prev_sibling = new_prev_sibling
    }
    fn set_next_sibling(node: &mut Node<View>, new_next_sibling: Option<AbstractNode<View>>) {
        node.next_sibling = new_next_sibling
    }

    // FIXME: The duplication between `with_base` and `with_mut_base` is ugly.
    fn with_base<R>(&self, callback: &fn(&Node<View>) -> R) -> R {
        self.transmute(callback)
    }

    fn with_mut_base<R>(&self, callback: &fn(&mut Node<View>) -> R) -> R {
        self.transmute_mut(callback)
    }
}

impl<View> TreeNode<AbstractNode<View>> for Node<View> { }

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

    /// Allow consumers to recreate an AbstractNode from the raw boxed type.
    /// Must only be used in situations where the boxed type is in the inheritance
    /// chain for nodes.
    pub fn from_box<T>(ptr: *mut rust_box<T>) -> AbstractNode<View> {
        AbstractNode {
            obj: ptr as *mut Node<View>
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

    // FIXME: This should be doing dynamic borrow checking for safety.
    pub fn is_characterdata(self) -> bool {
        // FIXME: ProcessingInstruction
        self.is_text() || self.is_comment()
    }

    pub fn with_imm_characterdata<R>(self, f: &fn(&CharacterData) -> R) -> R {
        if !self.is_characterdata() {
            fail!(~"node is not characterdata");
        }
        self.transmute(f)
    }

    pub fn with_mut_characterdata<R>(self, f: &fn(&mut CharacterData) -> R) -> R {
        if !self.is_characterdata() {
            fail!(~"node is not characterdata");
        }
        self.transmute_mut(f)
    }

    pub fn is_comment(self) -> bool {
        self.type_id() == CommentNodeTypeId
    }

    pub fn is_text(self) -> bool {
        self.type_id() == TextNodeTypeId
    }

    pub fn with_imm_text<R>(self, f: &fn(&Text) -> R) -> R {
        if !self.is_text() {
            fail!(~"node is not text");
        }
        self.transmute(f)
    }

    pub fn with_mut_text<R>(self, f: &fn(&mut Text) -> R) -> R {
        if !self.is_text() {
            fail!(~"node is not text");
        }
        self.transmute_mut(f)
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

    pub fn with_imm_iframe_element<R>(self, f: &fn(&HTMLIFrameElement) -> R) -> R {
        if !self.is_iframe_element() {
            fail!(~"node is not an iframe element");
        }
        self.transmute(f)
    }

    pub fn with_mut_iframe_element<R>(self, f: &fn(&mut HTMLIFrameElement) -> R) -> R {
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
        for _ in range(0, indent) {
            s.push_str("    ");
        }

        s.push_str(self.debug_str());
        debug!("%s", s);

        // FIXME: this should have a pure version?
        for kid in self.children() {
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
    fn next(&mut self) -> Option<AbstractNode<View>> {
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
        let mut cur_node = self.first_child;
        while cur_node.is_some() {
            for node in cur_node.unwrap().traverse_preorder() {
                do node.with_mut_base |node_base| {
                    node_base.owner_doc = Some(doc);
                }
            };
            cur_node = cur_node.unwrap().next_sibling();
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

            layout_data: LayoutData::new(),
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
        match self.type_id {
          ElementNodeTypeId(*) => {
            let mut content = ~"";
            for node in self.abstract.unwrap().traverse_preorder() {
                if node.is_text() {
                    do node.with_imm_text() |text| {
                        let s = text.parent.Data();
                        content = content + s.to_str();
                    }
                }
            }
            str(content)
          }
          CommentNodeTypeId | TextNodeTypeId => {
            do self.abstract.unwrap().with_imm_characterdata() |characterdata| {
                characterdata.Data()
            }
          }
          DoctypeNodeTypeId => {
            null_string
          }
        }
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

// This stuff is notionally private to layout, but we put it here because it needs
// to be stored in a Node, and we can't have cross-crate cyclic dependencies.

pub struct DisplayBoxes {
    display_list: Option<Arc<DisplayList<AbstractNode<()>>>>,
    range: Option<Range>,
}

/// Data that layout associates with a node.
pub struct LayoutData {
    /// The results of CSS styling for this node.
    style: Option<CompleteSelectResults>,

    /// Description of how to account for recent style changes.
    restyle_damage: Option<int>,

    /// The boxes assosiated with this flow.
    /// Used for getBoundingClientRect and friends.
    boxes: DisplayBoxes,
}

impl LayoutData {
    /// Creates new layout data.
    pub fn new() -> LayoutData {
        LayoutData {
            style: None,
            restyle_damage: None,
            boxes: DisplayBoxes { display_list: None, range: None },
        }
    }
}

impl AbstractNode<LayoutView> {
    // These accessors take a continuation rather than returning a reference, because
    // an AbstractNode doesn't have a lifetime parameter relating to the underlying
    // Node.  Also this makes it easier to switch to RWArc if we decide that is
    // necessary.
    pub fn read_layout_data<R>(self, blk: &fn(data: &LayoutData) -> R) -> R {
        do self.with_base |b| {
            blk(&b.layout_data)
        }
    }

    pub fn write_layout_data<R>(self, blk: &fn(data: &mut LayoutData) -> R) -> R {
        do self.with_mut_base |b| {
            blk(&mut b.layout_data)
        }
    }
}
