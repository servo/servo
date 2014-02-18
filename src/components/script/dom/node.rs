/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::utils::{ErrorResult, Fallible, NotFound, HierarchyRequest};
use dom::bindings::utils;
use dom::characterdata::CharacterData;
use dom::document::{AbstractDocument, DocumentTypeId};
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementTypeId, HTMLImageElementTypeId, HTMLIframeElementTypeId};
use dom::element::{HTMLAnchorElementTypeId, HTMLStyleElementTypeId};
use dom::eventtarget::{AbstractEventTarget, EventTarget, NodeTypeId};
use dom::htmliframeelement::HTMLIFrameElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::nodelist::{NodeList};
use dom::text::Text;
use dom::processinginstruction::ProcessingInstruction;
use layout_interface::{LayoutChan, ReapLayoutDataMsg, UntrustedNodeAddress};
use servo_util::str::{DOMString, null_str_as_empty};

use js::jsapi::{JSContext, JSObject, JSRuntime};
use js::jsfriendapi;
use std::cast::transmute;
use std::cast;
use std::cell::{RefCell, Ref, RefMut};
use std::iter::Filter;
use std::libc::uintptr_t;
use std::util;
use std::unstable::raw::Box;

//
// The basic Node structure
//

/// This is what a Node looks like if you do not know what kind of node it is. To unpack it, use
/// downcast().
///
/// FIXME: This should be replaced with a trait once they can inherit from structs.
#[deriving(Eq)]
pub struct AbstractNode {
    priv obj: *mut (),
}

/// An HTML node.
pub struct Node {
    /// The JavaScript reflector for this node.
    eventtarget: EventTarget,

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
    priv owner_doc: Option<AbstractDocument>,

    /// The live list of children return by .childNodes.
    child_list: Option<@mut NodeList>,

    /// A bitfield of flags for node items.
    priv flags: NodeFlags,

    /// Layout information. Only the layout task may touch this data.
    ///
    /// FIXME(pcwalton): We need to send these back to the layout task to be destroyed when this
    /// node is finalized.
    layout_data: LayoutDataRef,
}

/// Flags for node items.
pub struct NodeFlags(u8);

impl NodeFlags {
    pub fn new(type_id: NodeTypeId) -> NodeFlags {
        let mut flags = NodeFlags(0);
        match type_id {
            DocumentNodeTypeId(_) => { flags.set_is_in_doc(true); }
            _ => {}
        }
        flags
    }
}

/// Specifies whether this node is in a document.
bitfield!(NodeFlags, is_in_doc, set_is_in_doc, 0x01)
/// Specifies whether this node is hover state for this node
bitfield!(NodeFlags, get_in_hover_state, set_is_in_hover_state, 0x02)

#[unsafe_destructor]
impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            let this: &mut Node = cast::transmute(self);
            this.reap_layout_data()
        }
    }
}

/// suppress observers flag
/// http://dom.spec.whatwg.org/#concept-node-insert
/// http://dom.spec.whatwg.org/#concept-node-remove
enum SuppressObserver {
    Suppressed,
    Unsuppressed
}

/// Encapsulates the abstract layout data.
pub struct LayoutData {
    priv chan: Option<LayoutChan>,
    priv data: *(),
}

pub struct LayoutDataRef {
    data_cell: RefCell<Option<LayoutData>>,
}

impl LayoutDataRef {
    pub fn new() -> LayoutDataRef {
        LayoutDataRef {
            data_cell: RefCell::new(None),
        }
    }

    pub unsafe fn from_data<T>(data: ~T) -> LayoutDataRef {
        LayoutDataRef {
            data_cell: RefCell::new(Some(cast::transmute(data))),
        }
    }

    /// Returns true if there is layout data present.
    #[inline]
    pub fn is_present(&self) -> bool {
        let data_ref = self.data_cell.borrow();
        data_ref.get().is_some()
    }

    /// Take the chan out of the layout data if it is present.
    pub fn take_chan(&self) -> Option<LayoutChan> {
        let mut data_ref = self.data_cell.borrow_mut();
        let layout_data = data_ref.get();
        match *layout_data {
            None => None,
            Some(..) => Some(layout_data.get_mut_ref().chan.take_unwrap()),
        }
    }

    /// Borrows the layout data immutably, *asserting that there are no mutators*. Bad things will
    /// happen if you try to mutate the layout data while this is held. This is the only thread-
    /// safe layout data accessor.
    #[inline]
    pub unsafe fn borrow_unchecked(&self) -> *Option<LayoutData> {
        cast::transmute(&self.data_cell)
    }

    /// Borrows the layout data immutably. This function is *not* thread-safe.
    #[inline]
    pub fn borrow<'a>(&'a self) -> Ref<'a,Option<LayoutData>> {
        self.data_cell.borrow()
    }

    /// Borrows the layout data mutably. This function is *not* thread-safe.
    ///
    /// FIXME(pcwalton): We should really put this behind a `MutLayoutView` phantom type, to
    /// prevent CSS selector matching from mutably accessing nodes it's not supposed to and racing
    /// on it. This has already resulted in one bug!
    #[inline]
    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a,Option<LayoutData>> {
        self.data_cell.borrow_mut()
    }
}

/// A trait that represents abstract layout data.
/// 
/// FIXME(pcwalton): Very very unsafe!!! We need to send these back to the layout task to be
/// destroyed when this node is finalized.
pub trait TLayoutData {}

/// The different types of nodes.
#[deriving(Eq)]
pub enum NodeTypeId {
    DoctypeNodeTypeId,
    DocumentFragmentNodeTypeId,
    CommentNodeTypeId,
    DocumentNodeTypeId(DocumentTypeId),
    ElementNodeTypeId(ElementTypeId),
    TextNodeTypeId,
    ProcessingInstructionNodeTypeId,
}

impl Clone for AbstractNode {
    fn clone(&self) -> AbstractNode {
        *self
    }
}

impl AbstractNode {
    #[inline]
    pub fn node<'a>(&'a self) -> &'a Node {
        unsafe {
            let box_: *mut Box<Node> = cast::transmute(self.obj);
            &(*box_).data
        }
    }

    #[inline]
    pub fn mut_node<'a>(&'a self) -> &'a mut Node {
        unsafe {
            let box_: *mut Box<Node> = cast::transmute(self.obj);
            &mut (*box_).data
        }
    }

    #[inline]
    pub fn parent_node(&self) -> Option<AbstractNode> {
        self.node().parent_node
    }

    #[inline]
    pub fn first_child(&self) -> Option<AbstractNode> {
        self.node().first_child
    }

    #[inline]
    pub fn last_child(&self) -> Option<AbstractNode> {
        self.node().last_child
    }

    pub fn is_element(&self) -> bool {
        match self.type_id() {
            ElementNodeTypeId(..) => true,
            _ => false
        }
    }

    pub fn is_document(&self) -> bool {
        match self.type_id() {
            DocumentNodeTypeId(..) => true,
            _ => false
        }
    }

    /// If the given untrusted node address represents a valid DOM node in the given runtime,
    /// returns it.
    pub fn from_untrusted_node_address(runtime: *JSRuntime, candidate: UntrustedNodeAddress)
                                       -> AbstractNode {
        unsafe {
            let candidate: uintptr_t = cast::transmute(candidate);
            let object: *JSObject = jsfriendapi::bindgen::JS_GetAddressableObject(runtime,
                                                                                  candidate);
            if object.is_null() {
                fail!("Attempted to create an `AbstractNode` from an invalid pointer!")
            }
            let boxed_node: *mut Box<Node> = utils::unwrap(object);
            AbstractNode::from_box(boxed_node)
        }
    }

    pub fn is_inclusive_ancestor_of(&self, parent: AbstractNode) -> bool {
        *self == parent || parent.ancestors().any(|ancestor| ancestor == *self)
    }

    pub fn is_parent_of(&self, child: AbstractNode) -> bool {
        child.parent_node() == Some(*self)
    }
}

impl<'a> AbstractNode {
    // Unsafe accessors

    pub unsafe fn as_cacheable_wrapper(&self) -> @mut Reflectable {
        match self.type_id() {
            TextNodeTypeId => {
                let node: @mut Text = cast::transmute(self.obj);
                node as @mut Reflectable
            }
            _ => {
                fail!("unsupported node type")
            }
        }
    }

    /// Allow consumers to recreate an AbstractNode from the raw boxed type.
    /// Must only be used in situations where the boxed type is in the inheritance
    /// chain for nodes.
    ///
    /// FIXME(pcwalton): Mark unsafe?
    pub fn from_box<T>(ptr: *mut Box<T>) -> AbstractNode {
        AbstractNode {
            obj: ptr as *mut ()
        }
    }

    /// Allow consumers to upcast from derived classes.
    pub fn from_document(doc: AbstractDocument) -> AbstractNode {
        unsafe {
            cast::transmute(doc)
        }
    }

    pub fn from_eventtarget(target: AbstractEventTarget) -> AbstractNode {
        assert!(target.is_node());
        unsafe {
            cast::transmute(target)
        }
    }

    // Convenience accessors

    /// Returns the type ID of this node. Fails if this node is borrowed mutably.
    #[inline]
    pub fn type_id(self) -> NodeTypeId {
        self.node().type_id
    }

    /// Returns the previous sibling of this node. Fails if this node is borrowed mutably.
    pub fn prev_sibling(self) -> Option<AbstractNode> {
        self.node().prev_sibling
    }

    /// Returns the next sibling of this node. Fails if this node is borrowed mutably.
    pub fn next_sibling(self) -> Option<AbstractNode> {
        self.node().next_sibling
    }

    //
    // Downcasting borrows
    //

    pub fn transmute<'a, T, R>(self, f: |&'a T| -> R) -> R {
        unsafe {
            let node_box: *mut Box<Node> = transmute(self.obj);
            let node = &mut (*node_box).data;
            let old = node.abstract;
            node.abstract = Some(self);
            let box_: *Box<T> = transmute(self.obj);
            let rv = f(&(*box_).data);
            node.abstract = old;
            rv
        }
    }

    pub fn transmute_mut<T, R>(self, f: |&mut T| -> R) -> R {
        unsafe {
            let node_box: *mut Box<Node> = transmute(self.obj);
            let node = &mut (*node_box).data;
            let old = node.abstract;
            node.abstract = Some(self);
            let box_: *Box<T> = transmute(self.obj);
            let rv = f(cast::transmute(&(*box_).data));
            node.abstract = old;
            rv
        }
    }

    // FIXME: This should be doing dynamic borrow checking for safety.
    pub fn is_characterdata(self) -> bool {
        self.is_text() || self.is_comment() || self.is_processing_instruction()
    }

    pub fn with_imm_characterdata<R>(self, f: |&CharacterData| -> R) -> R {
        if !self.is_characterdata() {
            fail!(~"node is not characterdata");
        }
        self.transmute(f)
    }

    pub fn with_mut_characterdata<R>(self, f: |&mut CharacterData| -> R) -> R {
        if !self.is_characterdata() {
            fail!(~"node is not characterdata");
        }
        self.transmute_mut(f)
    }

    pub fn is_doctype(self) -> bool {
        self.type_id() == DoctypeNodeTypeId
    }

    pub fn with_imm_doctype<R>(self, f: |&DocumentType| -> R) -> R {
        if !self.is_doctype() {
            fail!(~"node is not doctype");
        }
        self.transmute(f)
    }

    pub fn with_mut_doctype<R>(self, f: |&mut DocumentType| -> R) -> R {
        if !self.is_doctype() {
            fail!(~"node is not doctype");
        }
        self.transmute_mut(f)
    }

    #[inline]
    pub fn is_comment(self) -> bool {
        // FIXME(pcwalton): Temporary workaround for the lack of inlining of autogenerated `Eq`
        // implementations in Rust.
        match self.type_id() {
            CommentNodeTypeId => true,
            _ => false,
        }
    }

    #[inline]
    pub fn is_text(self) -> bool {
        // FIXME(pcwalton): Temporary workaround for the lack of inlining of autogenerated `Eq`
        // implementations in Rust.
        match self.type_id() {
            TextNodeTypeId => true,
            _ => false,
        }
    }

    pub fn with_imm_text<R>(self, f: |&Text| -> R) -> R {
        if !self.is_text() {
            fail!(~"node is not text");
        }
        self.transmute(f)
    }

    pub fn with_mut_text<R>(self, f: |&mut Text| -> R) -> R {
        if !self.is_text() {
            fail!(~"node is not text");
        }
        self.transmute_mut(f)
    }

    #[inline]
    pub fn is_processing_instruction(self) -> bool {
        self.type_id() == ProcessingInstructionNodeTypeId
    }

    pub fn with_imm_processing_instruction<R>(self, f: |&ProcessingInstruction| -> R)  -> R {
        if !self.is_processing_instruction() {
            fail!("node is not processing instruction");
        }
        self.transmute(f)
    }

    // FIXME: This should be doing dynamic borrow checking for safety.
    pub fn with_imm_element<R>(self, f: |&Element| -> R) -> R {
        if !self.is_element() {
            fail!(~"node is not an element");
        }
        self.transmute(f)
    }

    // FIXME: This should be doing dynamic borrow checking for safety.
    pub fn as_mut_element<R>(self, f: |&mut Element| -> R) -> R {
        if !self.is_element() {
            fail!(~"node is not an element");
        }
        self.transmute_mut(f)
    }

    #[inline]
    pub fn is_image_element(self) -> bool {
        match self.type_id() {
            ElementNodeTypeId(HTMLImageElementTypeId) => true,
            _ => false,
        }
    }

    pub fn with_mut_image_element<R>(self, f: |&mut HTMLImageElement| -> R) -> R {
        if !self.is_image_element() {
            fail!(~"node is not an image element");
        }
        self.transmute_mut(f)
    }

    pub fn is_iframe_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLIframeElementTypeId)
    }

    pub fn with_mut_iframe_element<R>(self, f: |&mut HTMLIFrameElement| -> R) -> R {
        if !self.is_iframe_element() {
            fail!(~"node is not an iframe element");
        }
        self.transmute_mut(f)
    }

    pub fn is_style_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLStyleElementTypeId)
    }

    pub fn is_anchor_element(self) -> bool {
        self.type_id() == ElementNodeTypeId(HTMLAnchorElementTypeId)
    }

    pub unsafe fn raw_object(self) -> *mut Box<Node> {
        cast::transmute(self.obj)
    }

    pub fn from_raw(raw: *mut Box<Node>) -> AbstractNode {
        AbstractNode {
            obj: raw as *mut ()
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
        debug!("{:s}", s);

        // FIXME: this should have a pure version?
        for kid in self.children() {
            kid.dump_indent(indent + 1u)
        }
    }

    /// Returns a string that describes this node.
    pub fn debug_str(&self) -> ~str {
        format!("{:?}", self.type_id())
    }

    //
    // Convenience accessors
    //

    pub fn children(&self) -> AbstractNodeChildrenIterator {
        self.node().children()
    }

    pub fn child_elements(&self) -> Filter<AbstractNode, AbstractNodeChildrenIterator> {
        self.node().child_elements()
    }

    pub fn is_in_doc(&self) -> bool {
        self.node().flags.is_in_doc()
    }

    pub fn get_hover_state(&self) -> bool {
        self.node().flags.get_in_hover_state()
    }

    pub fn set_hover_state(&self, state: bool) {
        self.mut_node().flags.set_is_in_hover_state(state);
    }
}

impl AbstractNode {
    // http://dom.spec.whatwg.org/#dom-node-appendchild
    pub fn AppendChild(self, node: AbstractNode) -> Fallible<AbstractNode> {
        self.node().AppendChild(self, node)
    }

    // http://dom.spec.whatwg.org/#dom-node-replacechild
    pub fn ReplaceChild(self, node: AbstractNode, child: AbstractNode) -> Fallible<AbstractNode> {
        self.node().ReplaceChild(self, node, child)
    }

    // http://dom.spec.whatwg.org/#dom-node-removechild
    pub fn RemoveChild(self, node: AbstractNode) -> Fallible<AbstractNode> {
        self.node().RemoveChild(self, node)
    }

    // http://dom.spec.whatwg.org/#node-is-inserted
    fn node_inserted(self) {
        assert!(self.parent_node().is_some());
        let document = self.node().owner_doc();

        // Register elements having "id" attribute to the owner doc.
        document.mut_document().register_nodes_with_id(&self);

        document.document().content_changed();
    }

    // http://dom.spec.whatwg.org/#node-is-removed
    fn node_removed(self) {
        assert!(self.parent_node().is_none());
        let document = self.node().owner_doc();

        // Unregister elements having "id".
        document.mut_document().unregister_nodes_with_id(&self);

        document.document().content_changed();
    }

    //
    // Pointer stitching
    //

    /// Adds a new child to the end of this node's list of children.
    ///
    /// Fails unless `new_child` is disconnected from the tree.
    fn add_child(&self, new_child: AbstractNode, before: Option<AbstractNode>) {
        let this_node = self.mut_node();
        let new_child_node = new_child.mut_node();
        assert!(new_child_node.parent_node.is_none());
        assert!(new_child_node.prev_sibling.is_none());
        assert!(new_child_node.next_sibling.is_none());
        match before {
            Some(before) => {
                let before_node = before.mut_node();
                // XXX Should assert that parent is self.
                assert!(before_node.parent_node.is_some());
                match before_node.prev_sibling {
                    None => {
                        // XXX Should assert that before is the first child of
                        //     self.
                        this_node.set_first_child(Some(new_child.clone()));
                    },
                    Some(prev_sibling) => {
                        let prev_sibling_node = prev_sibling.mut_node();
                        prev_sibling_node.set_next_sibling(Some(new_child.clone()));
                        new_child_node.set_prev_sibling(Some(prev_sibling.clone()));
                    },
                }
                before_node.set_prev_sibling(Some(new_child.clone()));
                new_child_node.set_next_sibling(Some(before.clone()));
            },
            None => {
                match this_node.last_child {
                    None => this_node.set_first_child(Some(new_child.clone())),
                    Some(last_child) => {
                        let last_child_node = last_child.mut_node();
                        assert!(last_child_node.next_sibling.is_none());
                        last_child_node.set_next_sibling(Some(new_child.clone()));
                        new_child_node.set_prev_sibling(Some(last_child.clone()));
                    }
                }

                this_node.set_last_child(Some(new_child.clone()));
            },
        }

        new_child_node.set_parent_node(Some((*self).clone()));
    }

    /// Removes the given child from this node's list of children.
    ///
    /// Fails unless `child` is a child of this node. (FIXME: This is not yet checked.)
    fn remove_child(&self, child: AbstractNode) {
        let this_node = self.mut_node();
        let child_node = child.mut_node();
        assert!(child_node.parent_node.is_some());

        match child_node.prev_sibling {
            None => this_node.set_first_child(child_node.next_sibling),
            Some(prev_sibling) => {
                let prev_sibling_node = prev_sibling.mut_node();
                prev_sibling_node.set_next_sibling(child_node.next_sibling);
            }
        }

        match child_node.next_sibling {
            None => this_node.set_last_child(child_node.prev_sibling),
            Some(next_sibling) => {
                let next_sibling_node = next_sibling.mut_node();
                next_sibling_node.set_prev_sibling(child_node.prev_sibling);
            }
        }

        child_node.set_prev_sibling(None);
        child_node.set_next_sibling(None);
        child_node.set_parent_node(None);
    }
}

//
// Iteration and traversal
//

pub struct AbstractNodeChildrenIterator {
    priv current_node: Option<AbstractNode>,
}

impl Iterator<AbstractNode> for AbstractNodeChildrenIterator {
    fn next(&mut self) -> Option<AbstractNode> {
        let node = self.current_node;
        self.current_node = self.current_node.and_then(|node| {
            node.next_sibling()
        });
        node
    }
}

pub struct AncestorIterator {
    priv current: Option<AbstractNode>,
}

impl Iterator<AbstractNode> for AncestorIterator {
    fn next(&mut self) -> Option<AbstractNode> {
        if self.current.is_none() {
            return None;
        }

        // FIXME: Do we need two clones here?
        let x = self.current.get_ref().clone();
        self.current = x.parent_node();
        Some(x.clone())
    }
}

// FIXME: Do this without precomputing a vector of refs.
// Easy for preorder; harder for postorder.
pub struct TreeIterator {
    priv nodes: ~[AbstractNode],
    priv index: uint,
}

impl TreeIterator {
    fn new(nodes: ~[AbstractNode]) -> TreeIterator {
        TreeIterator {
            nodes: nodes,
            index: 0,
        }
    }
}

impl Iterator<AbstractNode> for TreeIterator {
    fn next(&mut self) -> Option<AbstractNode> {
        if self.index >= self.nodes.len() {
            None
        } else {
            let v = self.nodes[self.index].clone();
            self.index += 1;
            Some(v)
        }
    }
}

pub struct NodeIterator {
    start_node: AbstractNode,
    current_node: Option<AbstractNode>,
    depth: uint,
    priv include_start: bool,
    priv include_descendants_of_void: bool
}

impl NodeIterator {
    pub fn new(start_node: AbstractNode, include_start: bool, include_descendants_of_void: bool) -> NodeIterator {
        NodeIterator {
            start_node: start_node,
            current_node: None,
            depth: 0,
            include_start: include_start,
            include_descendants_of_void: include_descendants_of_void
        }
    }

    fn next_child(&self, node: AbstractNode) -> Option<AbstractNode> {
        if !self.include_descendants_of_void &&
           node.is_element() {
            node.with_imm_element(|elem| {
                if elem.is_void() {
                    None
                } else {
                    node.first_child()
                }
            })
        } else {
            node.first_child()
        }
    }
}

impl Iterator<AbstractNode> for NodeIterator {
    fn next(&mut self) -> Option<AbstractNode> {
         self.current_node = match self.current_node {
            None => {
                if self.include_start {
                    Some(self.start_node)
                } else {
                    self.next_child(self.start_node)
                }
            },
            Some(node) => {
                match self.next_child(node) {
                    Some(child) => {
                        self.depth += 1;
                        Some(child)
                    },
                    None if node == self.start_node => None,
                    None => {
                        match node.next_sibling() {
                            Some(sibling) => Some(sibling),
                            None => {
                                let mut candidate = node;
                                while candidate.next_sibling().is_none() {
                                    candidate = candidate.parent_node().expect("Got to root without reaching start node");
                                    self.depth -= 1;
                                    if candidate == self.start_node {
                                        break;
                                    }
                                }
                                if candidate != self.start_node {
                                    candidate.next_sibling()
                                } else {
                                    None
                                }
                            }
                        }
                    }
                }
            }
        };
        self.current_node
    }
}

fn gather_abstract_nodes(cur: &AbstractNode, refs: &mut ~[AbstractNode], postorder: bool) {
    if !postorder {
        refs.push(cur.clone());
    }
    for kid in cur.children() {
        gather_abstract_nodes(&kid, refs, postorder)
    }
    if postorder {
        refs.push(cur.clone());
    }
}

impl AbstractNode {
    /// Iterates over all ancestors of this node.
    pub fn ancestors(&self) -> AncestorIterator {
        AncestorIterator {
            current: self.parent_node(),
        }
    }

    pub fn inclusively_following_siblings(&self) -> AbstractNodeChildrenIterator {
        AbstractNodeChildrenIterator {
            current_node: Some(*self),
        }
    }

    pub fn following_siblings(&self) -> AbstractNodeChildrenIterator {
        AbstractNodeChildrenIterator {
            current_node: self.next_sibling(),
        }
    }

    /// Iterates over this node and all its descendants, in preorder.
    pub fn traverse_preorder(&self) -> TreeIterator {
        let mut nodes = ~[];
        gather_abstract_nodes(self, &mut nodes, false);
        TreeIterator::new(nodes)
    }

    /// Iterates over this node and all its descendants, in postorder.
    pub fn sequential_traverse_postorder(&self) -> TreeIterator {
        let mut nodes = ~[];
        gather_abstract_nodes(self, &mut nodes, true);
        TreeIterator::new(nodes)
    }
}

impl Node {
    pub fn owner_doc(&self) -> AbstractDocument {
        self.owner_doc.unwrap()
    }

    pub fn set_owner_doc(&mut self, document: AbstractDocument) {
        self.owner_doc = Some(document);
    }

    pub fn children(&self) -> AbstractNodeChildrenIterator {
        AbstractNodeChildrenIterator {
            current_node: self.first_child,
        }
    }

    pub fn child_elements(&self) -> Filter<AbstractNode, AbstractNodeChildrenIterator> {
        self.children().filter(|node| node.is_element())
    }

    pub fn reflect_node<N: Reflectable>
            (node:      @mut N,
             document:  AbstractDocument,
             wrap_fn:   extern "Rust" fn(*JSContext, *JSObject, @mut N) -> *JSObject)
             -> AbstractNode {
        assert!(node.reflector().get_jsobject().is_null());
        let node = reflect_dom_object(node, document.document().window, wrap_fn);
        assert!(node.reflector().get_jsobject().is_not_null());
        // JS owns the node now, so transmute_copy to not increase the refcount
        AbstractNode {
            obj: unsafe { cast::transmute_copy(&node) },
        }
    }

    pub fn new_inherited(type_id: NodeTypeId, doc: AbstractDocument) -> Node {
        Node::new_(type_id, Some(doc))
    }

    pub fn new_without_doc(type_id: NodeTypeId) -> Node {
        Node::new_(type_id, None)
    }

    fn new_(type_id: NodeTypeId, doc: Option<AbstractDocument>) -> Node {
        Node {
            eventtarget: EventTarget::new_inherited(NodeTypeId),
            type_id: type_id,

            abstract: None,

            parent_node: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,

            owner_doc: doc,
            child_list: None,

            flags: NodeFlags::new(type_id),

            layout_data: LayoutDataRef::new(),
        }
    }

    /// Sends layout data, if any, back to the script task to be destroyed.
    pub unsafe fn reap_layout_data(&mut self) {
        if self.layout_data.is_present() {
            let layout_data = util::replace(&mut self.layout_data, LayoutDataRef::new());
            let layout_chan = layout_data.take_chan();
            match layout_chan {
                None => {}
                Some(chan) => chan.send(ReapLayoutDataMsg(layout_data)),
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-nodetype
    pub fn NodeType(&self) -> u16 {
        match self.type_id {
            ElementNodeTypeId(_)            => 1,
            TextNodeTypeId                  => 3,
            ProcessingInstructionNodeTypeId => 7,
            CommentNodeTypeId               => 8,
            DocumentNodeTypeId(_)           => 9,
            DoctypeNodeTypeId               => 10,
            DocumentFragmentNodeTypeId      => 11,
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-nodename
    pub fn NodeName(&self, abstract_self: AbstractNode) -> DOMString {
        match self.type_id {
            ElementNodeTypeId(..) => {
                abstract_self.with_imm_element(|element| {
                    element.TagName()
                })
            }
            TextNodeTypeId => ~"#text",
            ProcessingInstructionNodeTypeId => {
                abstract_self.with_imm_processing_instruction(|processing_instruction| {
                    processing_instruction.Target()
                })
            }
            CommentNodeTypeId => ~"#comment",
            DoctypeNodeTypeId => {
                abstract_self.with_imm_doctype(|doctype| {
                    doctype.name.clone()
                })
            },
            DocumentFragmentNodeTypeId => ~"#document-fragment",
            DocumentNodeTypeId(_) => ~"#document"
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-baseuri
    pub fn GetBaseURI(&self) -> Option<DOMString> {
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-ownerdocument
    pub fn GetOwnerDocument(&self) -> Option<AbstractDocument> {
        match self.type_id {
            ElementNodeTypeId(..) |
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId |
            DoctypeNodeTypeId |
            DocumentFragmentNodeTypeId => Some(self.owner_doc()),
            DocumentNodeTypeId(_) => None
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-parentnode
    pub fn GetParentNode(&self) -> Option<AbstractNode> {
        self.parent_node
    }

    // http://dom.spec.whatwg.org/#dom-node-parentelement
    pub fn GetParentElement(&self) -> Option<AbstractNode> {
        self.parent_node.filtered(|parent| parent.is_element())
    }

    // http://dom.spec.whatwg.org/#dom-node-haschildnodes
    pub fn HasChildNodes(&self) -> bool {
        self.first_child.is_some()
    }

    // http://dom.spec.whatwg.org/#dom-node-firstchild
    pub fn GetFirstChild(&self) -> Option<AbstractNode> {
        self.first_child
    }

    // http://dom.spec.whatwg.org/#dom-node-lastchild
    pub fn GetLastChild(&self) -> Option<AbstractNode> {
        self.last_child
    }

    // http://dom.spec.whatwg.org/#dom-node-previoussibling
    pub fn GetPreviousSibling(&self) -> Option<AbstractNode> {
        self.prev_sibling
    }

    // http://dom.spec.whatwg.org/#dom-node-nextsibling
    pub fn GetNextSibling(&self) -> Option<AbstractNode> {
        self.next_sibling
    }

    // http://dom.spec.whatwg.org/#dom-node-nodevalue
    pub fn GetNodeValue(&self, abstract_self: AbstractNode) -> Option<DOMString> {
        match self.type_id {
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                abstract_self.with_imm_characterdata(|characterdata| {
                    Some(characterdata.Data())
                })
            }
            _ => {
                None
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-nodevalue
    pub fn SetNodeValue(&mut self, _abstract_self: AbstractNode, _val: Option<DOMString>)
                        -> ErrorResult {
        // FIXME: Stub - https://github.com/mozilla/servo/issues/1655
        Ok(())
    }

    // http://dom.spec.whatwg.org/#dom-node-textcontent
    pub fn GetTextContent(&self, abstract_self: AbstractNode) -> Option<DOMString> {
        match self.type_id {
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => {
                let mut content = ~"";
                for node in abstract_self.traverse_preorder() {
                    if node.is_text() {
                        node.with_imm_text(|text| {
                            content.push_str(text.characterdata.data.as_slice());
                        })
                    }
                }
                Some(content)
            }
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                abstract_self.with_imm_characterdata(|characterdata| {
                    Some(characterdata.Data())
                })
            }
            DoctypeNodeTypeId |
            DocumentNodeTypeId(_) => {
                None
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-childnodes
    pub fn ChildNodes(&mut self, abstract_self: AbstractNode) -> @mut NodeList {
        match self.child_list {
            None => {
                let window = self.owner_doc().document().window;
                let list = NodeList::new_child_list(window, abstract_self);
                self.child_list = Some(list);
                list
            }
            Some(list) => list
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-adopt
    fn adopt(node: AbstractNode, document: AbstractDocument) {
        // Step 1.
        match node.parent_node() {
            Some(parent) => Node::remove(node, parent, Unsuppressed),
            None => (),
        }

        // Step 2.
        if node.node().owner_doc() != document {
            for descendant in node.traverse_preorder() {
                descendant.mut_node().set_owner_doc(document);
            }
        }

        // Step 3.
        // If node is an element, it is _affected by a base URL change_.
    }

    // http://dom.spec.whatwg.org/#concept-node-pre-insert
    fn pre_insert(node: AbstractNode, parent: AbstractNode, child: Option<AbstractNode>)
                  -> Fallible<AbstractNode> {
        // Step 1.
        match parent.type_id() {
            DocumentNodeTypeId(..) |
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => (),
            _ => return Err(HierarchyRequest)
        }

        // Step 2.
        if node.is_inclusive_ancestor_of(parent) {
            return Err(HierarchyRequest);
        }

        // Step 3.
        match child {
            Some(child) if !parent.is_parent_of(child) => return Err(NotFound),
            _ => ()
        }

        // Step 4-5.
        match node.type_id() {
            TextNodeTypeId => {
                match node.parent_node() {
                    Some(parent) if parent.is_document() => return Err(HierarchyRequest),
                    _ => ()
                }
            }
            DoctypeNodeTypeId => {
                match node.parent_node() {
                    Some(parent) if !parent.is_document() => return Err(HierarchyRequest),
                    _ => ()
                }
            }
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(_) |
            ProcessingInstructionNodeTypeId |
            CommentNodeTypeId => (),
            DocumentNodeTypeId(..) => return Err(HierarchyRequest)
        }

        // Step 6.
        match parent.type_id() {
            DocumentNodeTypeId(_) => {
                match node.type_id() {
                    // Step 6.1
                    DocumentFragmentNodeTypeId => {
                        // Step 6.1.1(b)
                        if node.children().any(|c| c.is_text()) {
                            return Err(HierarchyRequest);
                        }
                        match node.child_elements().len() {
                            0 => (),
                            // Step 6.1.2
                            1 => {
                                // FIXME: change to empty() when https://github.com/mozilla/rust/issues/11218
                                // will be fixed
                                if parent.child_elements().len() > 0 {
                                    return Err(HierarchyRequest);
                                }
                                match child {
                                    Some(child) if child.inclusively_following_siblings()
                                                        .any(|child| child.is_doctype()) => {
                                        return Err(HierarchyRequest);
                                    }
                                    _ => (),
                                }
                            },
                            // Step 6.1.1(a)
                            _ => return Err(HierarchyRequest),
                        }
                    },
                    // Step 6.2
                    ElementNodeTypeId(_) => {
                        // FIXME: change to empty() when https://github.com/mozilla/rust/issues/11218
                        // will be fixed
                        if parent.child_elements().len() > 0 {
                            return Err(HierarchyRequest);
                        }
                        match child {
                            Some(child) if child.inclusively_following_siblings()
                                                .any(|child| child.is_doctype()) => {
                                return Err(HierarchyRequest);
                            }
                            _ => (),
                        }
                    },
                    // Step 6.3
                    DoctypeNodeTypeId => {
                        if parent.children().any(|c| c.is_doctype()) {
                            return Err(HierarchyRequest);
                        }
                        match child {
                            Some(child) => {
                                if parent.children()
                                    .take_while(|&c| c != child)
                                    .any(|c| c.is_element()) {
                                    return Err(HierarchyRequest);
                                }
                            },
                            None => {
                                // FIXME: change to empty() when https://github.com/mozilla/rust/issues/11218
                                // will be fixed
                                if parent.child_elements().len() > 0 {
                                    return Err(HierarchyRequest);
                                }
                            },
                        }
                    },
                    TextNodeTypeId |
                    ProcessingInstructionNodeTypeId |
                    CommentNodeTypeId => (),
                    DocumentNodeTypeId(_) => unreachable!(),
                }
            },
            _ => (),
        }

        // Step 7-8.
        let referenceChild = if child != Some(node) {
            child
        } else {
            node.next_sibling()
        };

        // Step 9.
        Node::adopt(node, parent.node().owner_doc());

        // Step 10.
        Node::insert(node, parent, referenceChild, Unsuppressed);

        // Step 11.
        return Ok(node)
    }

    // http://dom.spec.whatwg.org/#concept-node-insert
    fn insert(node: AbstractNode,
              parent: AbstractNode,
              child: Option<AbstractNode>,
              suppress_observers: SuppressObserver) {
        // XXX assert owner_doc
        // Step 1-3: ranges.
        // Step 4.
        let nodes = match node.type_id() {
            DocumentFragmentNodeTypeId => node.children().collect(),
            _ => ~[node],
        };

        // Step 5: DocumentFragment, mutation records.
        // Step 6: DocumentFragment.
        match node.type_id() {
            DocumentFragmentNodeTypeId => {
                for c in node.children() {
                    Node::remove(c, node, Suppressed);
                }
            },
            _ => (),
        }

        // Step 7: mutation records.
        // Step 8.
        for node in nodes.iter() {
            parent.add_child(*node, child);
            node.mut_node().flags.set_is_in_doc(parent.is_in_doc());
        }

        // Step 9.
        match suppress_observers {
            Unsuppressed => {
                for node in nodes.iter() {
                    node.node_inserted();
                }
            }
            Suppressed => ()
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-replace-all
    pub fn replace_all(node: Option<AbstractNode>, parent: AbstractNode) {
        // Step 1.
        match node {
            Some(node) => Node::adopt(node, parent.node().owner_doc()),
            None => (),
        }

        // Step 2.
        let removedNodes: ~[AbstractNode] = parent.children().collect();

        // Step 3.
        let addedNodes = match node {
            None => ~[],
            Some(node) => match node.type_id() {
                DocumentFragmentNodeTypeId => node.children().collect(),
                _ => ~[node],
            },
        };

        // Step 4.
        for child in parent.children() {
            Node::remove(child, parent, Suppressed);
        }

        // Step 5.
        match node {
            Some(node) => Node::insert(node, parent, None, Suppressed),
            None => (),
        }

        // Step 6: mutation records.

        // Step 7.
        for removedNode in removedNodes.iter() {
            removedNode.node_removed();
        }
        for addedNode in addedNodes.iter() {
            addedNode.node_inserted();
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-pre-remove
    fn pre_remove(child: AbstractNode, parent: AbstractNode) -> Fallible<AbstractNode> {
        // Step 1.
        if child.parent_node() != Some(parent) {
            return Err(NotFound);
        }

        // Step 2.
        Node::remove(child, parent, Unsuppressed);

        // Step 3.
        Ok(child)
    }

    // http://dom.spec.whatwg.org/#concept-node-remove
    fn remove(node: AbstractNode, parent: AbstractNode, suppress_observers: SuppressObserver) {
        assert!(node.parent_node() == Some(parent));

        // Step 1-5: ranges.
        // Step 6-7: mutation observers.
        // Step 8.
        parent.remove_child(node);
        node.mut_node().flags.set_is_in_doc(false);

        // Step 9.
        match suppress_observers {
            Suppressed => (),
            Unsuppressed => node.node_removed(),
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-textcontent
    pub fn SetTextContent(&mut self, abstract_self: AbstractNode, value: Option<DOMString>)
                          -> ErrorResult {
        let value = null_str_as_empty(&value);
        match self.type_id {
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => {
                // Step 1-2.
                let node = if value.len() == 0 {
                    None
                } else {
                    let document = self.owner_doc();
                    Some(document.document().CreateTextNode(document, value))
                };
                // Step 3.
                Node::replace_all(node, abstract_self);
            }
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                self.wait_until_safe_to_modify_dom();

                abstract_self.with_mut_characterdata(|characterdata| {
                    characterdata.data = value.clone();

                    // Notify the document that the content of this node is different
                    let document = self.owner_doc();
                    document.document().content_changed();
                })
            }
            DoctypeNodeTypeId |
            DocumentNodeTypeId(_) => {}
        }
        Ok(())
    }

    // http://dom.spec.whatwg.org/#dom-node-insertbefore
    pub fn InsertBefore(&self, abstract_self: AbstractNode, node: AbstractNode, child: Option<AbstractNode>)
                        -> Fallible<AbstractNode> {
        Node::pre_insert(node, abstract_self, child)
    }

    pub fn wait_until_safe_to_modify_dom(&self) {
        let document = self.owner_doc();
        document.document().wait_until_safe_to_modify_dom();
    }

    // http://dom.spec.whatwg.org/#dom-node-appendchild
    pub fn AppendChild(&self, abstract_self: AbstractNode, node: AbstractNode)
                       -> Fallible<AbstractNode> {
        Node::pre_insert(node, abstract_self, None)
    }

    // http://dom.spec.whatwg.org/#concept-node-replace
    pub fn ReplaceChild(&self, parent: AbstractNode, node: AbstractNode, child: AbstractNode)
                        -> Fallible<AbstractNode> {
        // Step 1.
        match parent.type_id() {
            DocumentNodeTypeId(..) |
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => (),
            _ => return Err(HierarchyRequest)
        }

        // Step 2.
        if node.is_inclusive_ancestor_of(parent) {
            return Err(HierarchyRequest);
        }

        // Step 3.
        if !parent.is_parent_of(child) {
            return Err(NotFound);
        }

        // Step 4-5.
        match node.type_id() {
            TextNodeTypeId if parent.is_document() => return Err(HierarchyRequest),
            DoctypeNodeTypeId if !parent.is_document() => return Err(HierarchyRequest),
            DocumentFragmentNodeTypeId |
            DoctypeNodeTypeId |
            ElementNodeTypeId(..) |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId |
            CommentNodeTypeId => (),
            DocumentNodeTypeId(..) => return Err(HierarchyRequest)
        }

        // Step 6.
        match parent.type_id() {
            DocumentNodeTypeId(..) => {
                match node.type_id() {
                    // Step 6.1
                    DocumentFragmentNodeTypeId => {
                        // Step 6.1.1(b)
                        if node.children().any(|c| c.is_text()) {
                            return Err(HierarchyRequest);
                        }
                        match node.child_elements().len() {
                            0 => (),
                            // Step 6.1.2
                            1 => {
                                if parent.child_elements().any(|c| c != child) {
                                    return Err(HierarchyRequest);
                                }
                                if child.following_siblings()
                                        .any(|child| child.is_doctype()) {
                                    return Err(HierarchyRequest);
                                }
                            },
                            // Step 6.1.1(a)
                            _ => return Err(HierarchyRequest)
                        }
                    },
                    // Step 6.2
                    ElementNodeTypeId(..) => {
                        if parent.child_elements().any(|c| c != child) {
                            return Err(HierarchyRequest);
                        }
                        if child.following_siblings()
                                .any(|child| child.is_doctype()) {
                            return Err(HierarchyRequest);
                        }
                    },
                    // Step 6.3
                    DoctypeNodeTypeId => {
                        if parent.children().any(|c| c.is_doctype() && c != child) {
                            return Err(HierarchyRequest);
                        }
                        if parent.children()
                            .take_while(|&c| c != child)
                            .any(|c| c.is_element()) {
                            return Err(HierarchyRequest);
                        }
                    },
                    TextNodeTypeId |
                    ProcessingInstructionNodeTypeId |
                    CommentNodeTypeId => (),
                    DocumentNodeTypeId(..) => unreachable!()
                }
            },
            _ => ()
        }

        // Ok if not caught by previous error checks.
        if node == child {
            return Ok(child);
        }

        // Step 7-8.
        let reference_child = if child.next_sibling() != Some(node) {
            child.next_sibling()
        } else {
            node.next_sibling()
        };

        // Step 9.
        Node::adopt(node, parent.node().owner_doc());

        {
            // Step 10.
            Node::remove(child, parent, Suppressed);

            // Step 11.
            Node::insert(node, parent, reference_child, Suppressed);
        }

        // Step 12-14.
        // Step 13: mutation records.
        child.node_removed();
        if node.type_id() == DocumentFragmentNodeTypeId {
            for child_node in node.children() {
                child_node.node_inserted();
            }
        } else {
            node.node_inserted();
        }

        // Step 15.
        Ok(child)
    }

    // http://dom.spec.whatwg.org/#dom-node-removechild
    pub fn RemoveChild(&self, abstract_self: AbstractNode, node: AbstractNode)
                       -> Fallible<AbstractNode> {
        Node::pre_remove(node, abstract_self)
    }

    // http://dom.spec.whatwg.org/#dom-node-normalize
    pub fn Normalize(&mut self) {
        // FIXME: stub - https://github.com/mozilla/servo/issues/1655
    }

    // http://dom.spec.whatwg.org/#dom-node-clonenode
    pub fn CloneNode(&self, _deep: bool) -> Fallible<AbstractNode> {
        // FIXME: stub - https://github.com/mozilla/servo/issues/1240
        fail!("stub")
    }

    // http://dom.spec.whatwg.org/#dom-node-isequalnode
    pub fn IsEqualNode(&self, abstract_self: AbstractNode, maybe_node: Option<AbstractNode>) -> bool {
        fn is_equal_doctype(node: AbstractNode, other: AbstractNode) -> bool {
            node.with_imm_doctype(|doctype| {
                other.with_imm_doctype(|other_doctype| {
                    (doctype.name == other_doctype.name) &&
                    (doctype.public_id == other_doctype.public_id) &&
                    (doctype.system_id == other_doctype.system_id)
                })
            })
        }
        fn is_equal_element(node: AbstractNode, other: AbstractNode) -> bool {
            node.with_imm_element(|element| {
                other.with_imm_element(|other_element| {
                    // FIXME: namespace prefix
                    (element.namespace == other_element.namespace) &&
                    (element.tag_name == other_element.tag_name) &&
                    (element.attrs.len() == other_element.attrs.len())
                })
            })
        }
        fn is_equal_processinginstruction(node: AbstractNode, other: AbstractNode) -> bool {
            node.with_imm_processing_instruction(|pi| {
                other.with_imm_processing_instruction(|other_pi| {
                    (pi.target == other_pi.target) &&
                    (pi.characterdata.data == other_pi.characterdata.data)
                })
            })
        }
        fn is_equal_characterdata(node: AbstractNode, other: AbstractNode) -> bool {
            node.with_imm_characterdata(|characterdata| {
                other.with_imm_characterdata(|other_characterdata| {
                    characterdata.data == other_characterdata.data
                })
            })
        }
        fn is_equal_element_attrs(node: AbstractNode, other: AbstractNode) -> bool {
            node.with_imm_element(|element| {
                other.with_imm_element(|other_element| {
                    assert!(element.attrs.len() == other_element.attrs.len());
                    element.attrs.iter().all(|attr| {
                        other_element.attrs.iter().any(|other_attr| {
                            (attr.namespace == other_attr.namespace) &&
                            (attr.local_name == other_attr.local_name) &&
                            (attr.value == other_attr.value)
                        })
                    })
                })
            })
        }
        fn is_equal_node(this: AbstractNode, node: AbstractNode) -> bool {
            // Step 2.
            if this.type_id() != node.type_id() {
                return false;
            }

            match node.type_id() {
                // Step 3.
                DoctypeNodeTypeId if !is_equal_doctype(this, node) => return false,
                ElementNodeTypeId(..) if !is_equal_element(this, node) => return false,
                ProcessingInstructionNodeTypeId if !is_equal_processinginstruction(this, node) => return false,
                TextNodeTypeId |
                CommentNodeTypeId if !is_equal_characterdata(this, node) => return false,
                // Step 4.
                ElementNodeTypeId(..) if !is_equal_element_attrs(this, node) => return false,
                _ => ()
            }

            // Step 5.
            if this.children().len() != node.children().len() {
                return false;
            }

            // Step 6.
            this.children().zip(node.children()).all(|(child, other_child)| is_equal_node(child, other_child))
        }
        match maybe_node {
            // Step 1.
            None => false,
            // Step 2-6.
            Some(node) => is_equal_node(abstract_self, node)
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-comparedocumentposition
    pub fn CompareDocumentPosition(&self, _other: AbstractNode) -> u16 {
        // FIXME: stub - https://github.com/mozilla/servo/issues/1655
        0
    }

    // http://dom.spec.whatwg.org/#dom-node-contains
    pub fn Contains(&self, abstract_self: AbstractNode, maybe_other: Option<AbstractNode>) -> bool {
        match maybe_other {
            None => false,
            Some(other) => abstract_self.is_inclusive_ancestor_of(other)
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-lookupprefix
    pub fn LookupPrefix(&self, _prefix: Option<DOMString>) -> Option<DOMString> {
        // FIXME: stub - https://github.com/mozilla/servo/issues/1655
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-lookupnamespaceuri
    pub fn LookupNamespaceURI(&self, _namespace: Option<DOMString>) -> Option<DOMString> {
        // FIXME: stub - https://github.com/mozilla/servo/issues/1655
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-isdefaultnamespace
    pub fn IsDefaultNamespace(&self, _namespace: Option<DOMString>) -> bool {
        // FIXME: stub - https://github.com/mozilla/servo/issues/1655
        false
    }

    // http://dom.spec.whatwg.org/#dom-node-namespaceuri
    pub fn GetNamespaceURI(&self) -> Option<DOMString> {
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-prefix
    pub fn GetPrefix(&self) -> Option<DOMString> {
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-localname
    pub fn GetLocalName(&self) -> Option<DOMString> {
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-hasattributes
    pub fn HasAttributes(&self) -> bool {
        false
    }

    //
    // Low-level pointer stitching
    //

    pub fn set_parent_node(&mut self, new_parent_node: Option<AbstractNode>) {
        let doc = self.owner_doc();
        doc.document().wait_until_safe_to_modify_dom();
        self.parent_node = new_parent_node
    }

    pub fn set_first_child(&mut self, new_first_child: Option<AbstractNode>) {
        let doc = self.owner_doc();
        doc.document().wait_until_safe_to_modify_dom();
        self.first_child = new_first_child
    }

    pub fn set_last_child(&mut self, new_last_child: Option<AbstractNode>) {
        let doc = self.owner_doc();
        doc.document().wait_until_safe_to_modify_dom();
        self.last_child = new_last_child
    }

    pub fn set_prev_sibling(&mut self, new_prev_sibling: Option<AbstractNode>) {
        let doc = self.owner_doc();
        doc.document().wait_until_safe_to_modify_dom();
        self.prev_sibling = new_prev_sibling
    }

    pub fn set_next_sibling(&mut self, new_next_sibling: Option<AbstractNode>) {
        let doc = self.owner_doc();
        doc.document().wait_until_safe_to_modify_dom();
        self.next_sibling = new_next_sibling
    }

    pub fn get_hover_state(&self) -> bool {
        self.flags.get_in_hover_state()
    }

    pub fn set_hover_state(&mut self, state: bool) {
        self.flags.set_is_in_hover_state(state);
    }
}

impl Reflectable for Node {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        self.eventtarget.mut_reflector()
    }
}

