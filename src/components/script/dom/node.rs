/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use dom::attr::Attr;
use dom::bindings::codegen::InheritTypes::{CommentCast, DocumentCast, DocumentTypeCast};
use dom::bindings::codegen::InheritTypes::{ElementCast, TextCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{CharacterDataCast, NodeBase, NodeDerived};
use dom::bindings::codegen::InheritTypes::{ProcessingInstructionCast, EventTargetCast};
use dom::bindings::codegen::BindingDeclarations::NodeBinding::NodeConstants;
use dom::bindings::js::{JS, JSRef, RootCollection, RootedReference};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::{ErrorResult, Fallible, NotFound, HierarchyRequest};
use dom::bindings::utils;
use dom::characterdata::CharacterData;
use dom::comment::Comment;
use dom::document::{Document, HTMLDocument, NonHTMLDocument};
use dom::documentfragment::DocumentFragment;
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementTypeId, HTMLAnchorElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::nodelist::{NodeList};
use dom::processinginstruction::ProcessingInstruction;
use dom::text::Text;
use dom::virtualmethods::{VirtualMethods, vtable_for};
use dom::window::Window;
use geom::rect::Rect;
use html::hubbub_html_parser::build_element_from_tag;
use layout_interface::{ContentBoxQuery, ContentBoxResponse, ContentBoxesQuery, ContentBoxesResponse,
                       LayoutChan, ReapLayoutDataMsg, TrustedNodeAddress, UntrustedNodeAddress};
use servo_util::geometry::Au;
use servo_util::str::{DOMString, null_str_as_empty};

use js::jsapi::{JSContext, JSObject, JSRuntime};
use js::jsfriendapi;
use libc;
use libc::uintptr_t;
use std::cast::transmute;
use std::cast;
use std::cell::{RefCell, Ref, RefMut};
use std::iter::{Map, Filter};
use std::mem;

use serialize::{Encoder, Encodable};

//
// The basic Node structure
//

/// An HTML node.
#[deriving(Encodable)]
pub struct Node {
    /// The JavaScript reflector for this node.
    pub eventtarget: EventTarget,

    /// The type of node that this is.
    pub type_id: NodeTypeId,

    /// The parent of this node.
    pub parent_node: Option<JS<Node>>,

    /// The first child of this node.
    pub first_child: Option<JS<Node>>,

    /// The last child of this node.
    pub last_child: Option<JS<Node>>,

    /// The next sibling of this node.
    pub next_sibling: Option<JS<Node>>,

    /// The previous sibling of this node.
    pub prev_sibling: Option<JS<Node>>,

    /// The document that this node belongs to.
    owner_doc: Option<JS<Document>>,

    /// The live list of children return by .childNodes.
    pub child_list: Option<JS<NodeList>>,

    /// A bitfield of flags for node items.
    flags: NodeFlags,

    /// Layout information. Only the layout task may touch this data.
    ///
    /// FIXME(pcwalton): We need to send these back to the layout task to be destroyed when this
    /// node is finalized.
    pub layout_data: LayoutDataRef,
}

impl<S: Encoder<E>, E> Encodable<S, E> for LayoutDataRef {
    fn encode(&self, _s: &mut S) -> Result<(), E> {
        Ok(())
    }
}

impl NodeDerived for EventTarget {
    fn is_node(&self) -> bool {
        match self.type_id {
            NodeTargetTypeId(_) => true,
            _ => false
        }
    }
}

/// Flags for node items.
#[deriving(Encodable)]
pub struct NodeFlags(pub u8);

impl NodeFlags {
    pub fn new(type_id: NodeTypeId) -> NodeFlags {
        let mut flags = NodeFlags(0);
        match type_id {
            DocumentNodeTypeId => { flags.set_is_in_doc(true); }
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
            self.reap_layout_data()
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
    chan: Option<LayoutChan>,
    data: *(),
}

pub struct LayoutDataRef {
    pub data_cell: RefCell<Option<LayoutData>>,
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
        self.data_cell.borrow().is_some()
    }

    /// Take the chan out of the layout data if it is present.
    pub fn take_chan(&self) -> Option<LayoutChan> {
        let mut layout_data = self.data_cell.borrow_mut();
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
#[deriving(Eq,Encodable)]
pub enum NodeTypeId {
    DoctypeNodeTypeId,
    DocumentFragmentNodeTypeId,
    CommentNodeTypeId,
    DocumentNodeTypeId,
    ElementNodeTypeId(ElementTypeId),
    TextNodeTypeId,
    ProcessingInstructionNodeTypeId,
}

pub trait INode {
    fn AppendChild(&mut self, node: &mut JSRef<Node>) -> Fallible<JS<Node>>;
    fn ReplaceChild(&mut self, node: &mut JSRef<Node>, child: &mut JSRef<Node>) -> Fallible<JS<Node>>;
    fn RemoveChild(&mut self, node: &mut JSRef<Node>) -> Fallible<JS<Node>>;
}

impl INode for JS<Node> {
    fn AppendChild(&mut self, node: &mut JSRef<Node>) -> Fallible<JS<Node>> {
        let roots = RootCollection::new();
        let self_node = self.root(&roots);
        self.get_mut().AppendChild(&mut self_node.root_ref(), node)
    }

    fn ReplaceChild(&mut self, node: &mut JSRef<Node>, child: &mut JSRef<Node>) -> Fallible<JS<Node>> {
        let roots = RootCollection::new();
        let self_node = self.root(&roots);
        self.get_mut().ReplaceChild(&mut self_node.root_ref(), node, child)
    }

    fn RemoveChild(&mut self, node: &mut JSRef<Node>) -> Fallible<JS<Node>> {
        let roots = RootCollection::new();
        let self_node = self.root(&roots);
        self.get_mut().RemoveChild(&mut self_node.root_ref(), node)
    }
}

pub trait NodeHelpers {
    fn ancestors(&self) -> AncestorIterator;
    fn children(&self) -> AbstractNodeChildrenIterator;
    fn child_elements(&self) -> ChildElementIterator;
    fn following_siblings(&self) -> AbstractNodeChildrenIterator;
    fn is_in_doc(&self) -> bool;
    fn is_inclusive_ancestor_of(&self, parent: &JSRef<Node>) -> bool;
    fn is_parent_of(&self, child: &JSRef<Node>) -> bool;

    fn type_id(&self) -> NodeTypeId;

    fn parent_node(&self) -> Option<JS<Node>>;
    fn first_child(&self) -> Option<JS<Node>>;
    fn last_child(&self) -> Option<JS<Node>>;
    fn prev_sibling(&self) -> Option<JS<Node>>;
    fn next_sibling(&self) -> Option<JS<Node>>;

    fn is_element(&self) -> bool;
    fn is_document(&self) -> bool;
    fn is_doctype(&self) -> bool;
    fn is_text(&self) -> bool;
    fn is_anchor_element(&self) -> bool;

    fn node_inserted(&self);
    fn node_removed(&self);
    fn add_child(&mut self, new_child: &mut JSRef<Node>, before: Option<JSRef<Node>>);
    fn remove_child(&mut self, child: &mut JSRef<Node>);

    fn get_hover_state(&self) -> bool;
    fn set_hover_state(&mut self, state: bool);

    fn dump(&self);
    fn dump_indent(&self, indent: uint);
    fn debug_str(&self) -> ~str;

    fn traverse_preorder(&self) -> TreeIterator;
    fn sequential_traverse_postorder(&self) -> TreeIterator;
    fn inclusively_following_siblings(&self) -> AbstractNodeChildrenIterator;

    fn from_untrusted_node_address(runtime: *JSRuntime, candidate: UntrustedNodeAddress) -> Self;
    fn to_trusted_node_address(&self) -> TrustedNodeAddress;

    fn get_bounding_content_box(&self) -> Rect<Au>;
    fn get_content_boxes(&self) -> Vec<Rect<Au>>;
}

impl NodeHelpers for JS<Node> {
    /// Dumps the subtree rooted at this node, for debugging.
    fn dump(&self) {
        self.dump_indent(0);
    }

    /// Dumps the node tree, for debugging, with indentation.
    fn dump_indent(&self, indent: uint) {
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
    fn debug_str(&self) -> ~str {
        format!("{:?}", self.type_id())
    }

    /// Iterates over all ancestors of this node.
    fn ancestors(&self) -> AncestorIterator {
        self.get().ancestors()
    }

    fn children(&self) -> AbstractNodeChildrenIterator {
        self.get().children()
    }

    fn child_elements(&self) -> ChildElementIterator {
        self.get().child_elements()
    }

    fn is_in_doc(&self) -> bool {
        self.get().flags.is_in_doc()
    }

    /// Returns the type ID of this node. Fails if this node is borrowed mutably.
    fn type_id(&self) -> NodeTypeId {
        self.get().type_id
    }

    fn parent_node(&self) -> Option<JS<Node>> {
        self.get().parent_node.clone()
    }

    fn first_child(&self) -> Option<JS<Node>> {
        self.get().first_child.clone()
    }

    fn last_child(&self) -> Option<JS<Node>> {
        self.get().last_child.clone()
    }

    /// Returns the previous sibling of this node. Fails if this node is borrowed mutably.
    fn prev_sibling(&self) -> Option<JS<Node>> {
        self.get().prev_sibling.clone()
    }

    /// Returns the next sibling of this node. Fails if this node is borrowed mutably.
    fn next_sibling(&self) -> Option<JS<Node>> {
        self.get().next_sibling.clone()
    }

    #[inline]
    fn is_element(&self) -> bool {
        self.get().is_element()
    }

    #[inline]
    fn is_document(&self) -> bool {
        match self.type_id() {
            DocumentNodeTypeId => true,
            _ => false
        }
    }

    #[inline]
    fn is_anchor_element(&self) -> bool {
        match self.type_id() {
            ElementNodeTypeId(HTMLAnchorElementTypeId) => true,
            _ => false
        }
    }

    #[inline]
    fn is_doctype(&self) -> bool {
        match self.type_id() {
            DoctypeNodeTypeId => true,
            _ => false
        }
    }

    #[inline]
    fn is_text(&self) -> bool {
        // FIXME(pcwalton): Temporary workaround for the lack of inlining of autogenerated `Eq`
        // implementations in Rust.
        match self.type_id() {
            TextNodeTypeId => true,
            _ => false
        }
    }

    // http://dom.spec.whatwg.org/#node-is-inserted
    fn node_inserted(&self) {
        assert!(self.parent_node().is_some());
        let document = document_from_node(self);

        if self.is_in_doc() {
            for node in self.traverse_preorder() {
                vtable_for(&node).bind_to_tree();
            }
        }

        self.parent_node().map(|parent| vtable_for(&parent).child_inserted(self));
        document.get().content_changed();
    }

    // http://dom.spec.whatwg.org/#node-is-removed
    fn node_removed(&self) {
        assert!(self.parent_node().is_none());
        let document = document_from_node(self);

        for node in self.traverse_preorder() {
            // XXX how about if the node wasn't in the tree in the first place?
            vtable_for(&node).unbind_from_tree();
        }

        document.get().content_changed();
    }

    //
    // Pointer stitching
    //

    /// Adds a new child to the end of this node's list of children.
    ///
    /// Fails unless `new_child` is disconnected from the tree.
    fn add_child(&mut self, new_child_root: &mut JSRef<Node>, before: Option<JSRef<Node>>) {
        let roots = RootCollection::new();
        let mut new_child = new_child_root.unrooted();
        assert!(new_child.parent_node().is_none());
        assert!(new_child.prev_sibling().is_none());
        assert!(new_child.next_sibling().is_none());
        match before {
            Some(before_root) => {
                let mut before = before_root.unrooted();
                // XXX Should assert that parent is self.
                assert!(before.parent_node().is_some());
                match before.prev_sibling() {
                    None => {
                        // XXX Should assert that before is the first child of
                        //     self.
                        self.get_mut().set_first_child(Some(new_child_root.clone()));
                    },
                    Some(mut prev_sibling) => {
                        let prev_sibling_root = prev_sibling.root(&roots);
                        prev_sibling.get_mut().set_next_sibling(Some(new_child_root.clone()));
                        new_child.get_mut().set_prev_sibling(Some(prev_sibling_root.root_ref()));
                    },
                }
                before.get_mut().set_prev_sibling(Some(new_child_root.clone()));
                new_child.get_mut().set_next_sibling(Some(before_root.clone()));
            },
            None => {
                match self.last_child() {
                    None => self.get_mut().set_first_child(Some(new_child_root.clone())),
                    Some(mut last_child) => {
                        let last_child_root = last_child.root(&roots);
                        assert!(last_child.next_sibling().is_none());
                        last_child.get_mut().set_next_sibling(Some(new_child_root.clone()));
                        new_child.get_mut().set_prev_sibling(Some(last_child_root.root_ref()));
                    }
                }

                self.get_mut().set_last_child(Some(new_child_root.clone()));
            },
        }

        let self_root = self.root(&roots);
        new_child.get_mut().set_parent_node(Some(self_root.root_ref()));
    }

    /// Removes the given child from this node's list of children.
    ///
    /// Fails unless `child` is a child of this node. (FIXME: This is not yet checked.)
    fn remove_child(&mut self, child: &mut JSRef<Node>) {
        let roots = RootCollection::new();
        let this_node = self.get_mut();
        let child_node = child.get_mut();
        assert!(child_node.parent_node.is_some());

        match child_node.prev_sibling {
            None => {
                let next_sibling = child_node.next_sibling.as_ref().map(|next| next.root(&roots));
                this_node.set_first_child(next_sibling.root_ref());
            }
            Some(ref mut prev_sibling) => {
                let prev_sibling_node = prev_sibling.get_mut();
                let next_sibling = child_node.next_sibling.as_ref().map(|next| next.root(&roots));
                prev_sibling_node.set_next_sibling(next_sibling.root_ref());
            }
        }

        match child_node.next_sibling {
            None => {
                let prev_sibling = child_node.prev_sibling.as_ref().map(|prev| prev.root(&roots));
                this_node.set_last_child(prev_sibling.root_ref());
            }
            Some(ref mut next_sibling) => {
                let next_sibling_node = next_sibling.get_mut();
                let prev_sibling = child_node.prev_sibling.as_ref().map(|prev| prev.root(&roots));
                next_sibling_node.set_prev_sibling(prev_sibling.root_ref());
            }
        }

        child_node.set_prev_sibling(None);
        child_node.set_next_sibling(None);
        child_node.set_parent_node(None);
    }

    fn get_hover_state(&self) -> bool {
        self.get().flags.get_in_hover_state()
    }

    fn set_hover_state(&mut self, state: bool) {
        self.get_mut().flags.set_is_in_hover_state(state);
    }

    /// Iterates over this node and all its descendants, in preorder.
    fn traverse_preorder(&self) -> TreeIterator {
        let roots = RootCollection::new();
        let mut nodes = vec!();
        let self_root = self.root(&roots);
        gather_abstract_nodes(&self_root.root_ref(), &mut nodes, false);
        TreeIterator::new(nodes)
    }

    /// Iterates over this node and all its descendants, in postorder.
    fn sequential_traverse_postorder(&self) -> TreeIterator {
        let roots = RootCollection::new();
        let mut nodes = vec!();
        let self_root = self.root(&roots);
        gather_abstract_nodes(&self_root.root_ref(), &mut nodes, true);
        TreeIterator::new(nodes)
    }

    fn inclusively_following_siblings(&self) -> AbstractNodeChildrenIterator {
        AbstractNodeChildrenIterator {
            current_node: Some(self.clone()),
        }
    }

    fn is_inclusive_ancestor_of(&self, parent: &JSRef<Node>) -> bool {
        let parent = &parent.unrooted();
        self == parent || parent.ancestors().any(|ancestor| ancestor == *self)
    }

    fn following_siblings(&self) -> AbstractNodeChildrenIterator {
        AbstractNodeChildrenIterator {
            current_node: self.next_sibling(),
        }
    }

    fn is_parent_of(&self, child: &JSRef<Node>) -> bool {
        match child.unrooted().parent_node() {
            Some(ref parent) if parent == self => true,
            _ => false
        }
    }

    /// If the given untrusted node address represents a valid DOM node in the given runtime,
    /// returns it.
    fn from_untrusted_node_address(runtime: *JSRuntime, candidate: UntrustedNodeAddress)
        -> JS<Node> {
        unsafe {
            let candidate: uintptr_t = cast::transmute(candidate);
            let object: *JSObject = jsfriendapi::bindgen::JS_GetAddressableObject(runtime,
                                                                                  candidate);
            if object.is_null() {
                fail!("Attempted to create a `JS<Node>` from an invalid pointer!")
            }
            let boxed_node: *mut Node = utils::unwrap(object);
            JS::from_raw(boxed_node)
        }
    }

    fn to_trusted_node_address(&self) -> TrustedNodeAddress {
        TrustedNodeAddress(self.get() as *Node as *libc::c_void)
    }

    fn get_bounding_content_box(&self) -> Rect<Au> {
        let window = window_from_node(self);
        let page = window.get().page();
        let (chan, port) = channel();
        let addr = self.to_trusted_node_address();
        let ContentBoxResponse(rect) = page.query_layout(ContentBoxQuery(addr, chan), port);
        rect
    }

    fn get_content_boxes(&self) -> Vec<Rect<Au>> {
        let window = window_from_node(self);
        let page = window.get().page();
        let (chan, port) = channel();
        let addr = self.to_trusted_node_address();
        let ContentBoxesResponse(rects) = page.query_layout(ContentBoxesQuery(addr, chan), port);
        rects
    }
}

//
// Iteration and traversal
//

pub type ChildElementIterator<'a> = Map<'a, JS<Node>,
                                    JS<Element>,
                                    Filter<'a, JS<Node>, AbstractNodeChildrenIterator>>;

pub struct AbstractNodeChildrenIterator {
    current_node: Option<JS<Node>>,
}

impl Iterator<JS<Node>> for AbstractNodeChildrenIterator {
    fn next(&mut self) -> Option<JS<Node>> {
        let node = self.current_node.clone();
        self.current_node = node.clone().and_then(|node| {
            node.next_sibling()
        });
        node
    }
}

pub struct AncestorIterator {
    current: Option<JS<Node>>,
}

impl Iterator<JS<Node>> for AncestorIterator {
    fn next(&mut self) -> Option<JS<Node>> {
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
    nodes: Vec<JS<Node>>,
    index: uint,
}

impl TreeIterator {
    fn new(nodes: Vec<JS<Node>>) -> TreeIterator {
        TreeIterator {
            nodes: nodes,
            index: 0,
        }
    }
}

impl Iterator<JS<Node>> for TreeIterator {
    fn next(&mut self) -> Option<JS<Node>> {
        if self.index >= self.nodes.len() {
            None
        } else {
            let v = self.nodes.get(self.index).clone();
            self.index += 1;
            Some(v)
        }
    }
}

pub struct NodeIterator {
    pub start_node: JS<Node>,
    pub current_node: Option<JS<Node>>,
    pub depth: uint,
    include_start: bool,
    include_descendants_of_void: bool
}

impl NodeIterator {
    pub fn new(start_node: JS<Node>, include_start: bool, include_descendants_of_void: bool) -> NodeIterator {
        NodeIterator {
            start_node: start_node,
            current_node: None,
            depth: 0,
            include_start: include_start,
            include_descendants_of_void: include_descendants_of_void
        }
    }

    fn next_child(&self, node: &JSRef<Node>) -> Option<JS<Node>> {
        if !self.include_descendants_of_void && node.get().is_element() {
            let elem: JS<Element> = ElementCast::to(&node.unrooted()).unwrap();
            if elem.get().is_void() {
                None
            } else {
                node.get().first_child.clone()
            }
        } else {
            node.get().first_child.clone()
        }
    }
}

impl Iterator<JS<Node>> for NodeIterator {
    fn next(&mut self) -> Option<JS<Node>> {
        let roots = RootCollection::new();
         self.current_node = match self.current_node {
            None => {
                if self.include_start {
                    Some(self.start_node.clone())
                } else {
                    let start_node = self.start_node.root(&roots);
                    self.next_child(&start_node.root_ref())
                }
            },
            Some(ref node) => {
                let node_root = node.root(&roots);
                match self.next_child(&node_root.root_ref()) {
                    Some(child) => {
                        self.depth += 1;
                        Some(child.clone())
                    },
                    None if node == &self.start_node => None,
                    None => {
                        match node.next_sibling() {
                            Some(sibling) => Some(sibling),
                            None => {
                                let mut candidate = node.clone();
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
        self.current_node.clone()
    }
}

fn gather_abstract_nodes(cur: &JSRef<Node>, refs: &mut Vec<JS<Node>>, postorder: bool) {
    let roots = RootCollection::new();
    if !postorder {
        refs.push(cur.unrooted());
    }
    for kid in cur.unrooted().children() {
        let kid = kid.root(&roots);
        gather_abstract_nodes(&kid.root_ref(), refs, postorder)
    }
    if postorder {
        refs.push(cur.unrooted());
    }
}

/// Specifies whether children must be recursively cloned or not.
#[deriving(Eq)]
pub enum CloneChildrenFlag {
    CloneChildren,
    DoNotCloneChildren
}

fn as_uintptr<T>(t: &T) -> uintptr_t { t as *T as uintptr_t }

impl Node {
    pub fn ancestors(&self) -> AncestorIterator {
        AncestorIterator {
            current: self.parent_node.clone(),
        }
    }

    pub fn is_element(&self) -> bool {
        match self.type_id {
            ElementNodeTypeId(..) => true,
            _ => false
        }
    }

    pub fn owner_doc<'a>(&'a self) -> &'a JS<Document> {
        self.owner_doc.get_ref()
    }

    pub fn set_owner_doc(&mut self, document: &JSRef<Document>) {
        self.owner_doc = Some(document.unrooted());
    }

    pub fn children(&self) -> AbstractNodeChildrenIterator {
        AbstractNodeChildrenIterator {
            current_node: self.first_child.clone(),
        }
    }

    pub fn child_elements(&self) -> ChildElementIterator {
        self.children()
            .filter(|node| node.is_element())
            .map(|node| {
                let elem: JS<Element> = ElementCast::to(&node).unwrap();
                elem
            })
    }

    pub fn reflect_node<N: Reflectable+NodeBase>
            (node:      ~N,
             document:  &JSRef<Document>,
             wrap_fn:   extern "Rust" fn(*JSContext, &JSRef<Window>, ~N) -> JS<N>)
             -> JS<N> {
        let roots = RootCollection::new();
        assert!(node.reflector().get_jsobject().is_null());
        let window = document.get().window.root(&roots);
        let node = reflect_dom_object(node, &window.root_ref(), wrap_fn);
        assert!(node.reflector().get_jsobject().is_not_null());
        node
    }

    pub fn new_inherited(type_id: NodeTypeId, doc: JS<Document>) -> Node {
        let roots = RootCollection::new();
        let doc = doc.root(&roots);
        Node::new_(type_id, Some(doc.root_ref()))
    }

    pub fn new_without_doc(type_id: NodeTypeId) -> Node {
        Node::new_(type_id, None)
    }

    fn new_(type_id: NodeTypeId, doc: Option<JSRef<Document>>) -> Node {
        Node {
            eventtarget: EventTarget::new_inherited(NodeTargetTypeId(type_id)),
            type_id: type_id,

            parent_node: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,

            owner_doc: doc.map(|doc| doc.unrooted()),
            child_list: None,

            flags: NodeFlags::new(type_id),

            layout_data: LayoutDataRef::new(),
        }
    }

    /// Sends layout data, if any, back to the script task to be destroyed.
    pub unsafe fn reap_layout_data(&mut self) {
        if self.layout_data.is_present() {
            let layout_data = mem::replace(&mut self.layout_data, LayoutDataRef::new());
            let layout_chan = layout_data.take_chan();
            match layout_chan {
                None => {}
                Some(chan) => {
                    let LayoutChan(chan) = chan;
                    chan.send(ReapLayoutDataMsg(layout_data))
                },
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-nodetype
    pub fn NodeType(&self) -> u16 {
        match self.type_id {
            ElementNodeTypeId(_)            => NodeConstants::ELEMENT_NODE,
            TextNodeTypeId                  => NodeConstants::TEXT_NODE,
            ProcessingInstructionNodeTypeId => NodeConstants::PROCESSING_INSTRUCTION_NODE,
            CommentNodeTypeId               => NodeConstants::COMMENT_NODE,
            DocumentNodeTypeId              => NodeConstants::DOCUMENT_NODE,
            DoctypeNodeTypeId               => NodeConstants::DOCUMENT_TYPE_NODE,
            DocumentFragmentNodeTypeId      => NodeConstants::DOCUMENT_FRAGMENT_NODE,
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-nodename
    pub fn NodeName(&self, abstract_self: &JSRef<Node>) -> DOMString {
        match self.type_id {
            ElementNodeTypeId(..) => {
                let elem: JS<Element> = ElementCast::to(&abstract_self.unrooted()).unwrap();
                elem.get().TagName()
            }
            TextNodeTypeId => ~"#text",
            ProcessingInstructionNodeTypeId => {
                let processing_instruction: JS<ProcessingInstruction> =
                    ProcessingInstructionCast::to(&abstract_self.unrooted()).unwrap();
                processing_instruction.get().Target()
            }
            CommentNodeTypeId => ~"#comment",
            DoctypeNodeTypeId => {
                let doctype: JS<DocumentType> = DocumentTypeCast::to(&abstract_self.unrooted()).unwrap();
                doctype.get().name.clone()
            },
            DocumentFragmentNodeTypeId => ~"#document-fragment",
            DocumentNodeTypeId => ~"#document"
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-baseuri
    pub fn GetBaseURI(&self) -> Option<DOMString> {
        // FIXME (#1824) implement.
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-ownerdocument
    pub fn GetOwnerDocument(&self) -> Option<JS<Document>> {
        match self.type_id {
            ElementNodeTypeId(..) |
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId |
            DoctypeNodeTypeId |
            DocumentFragmentNodeTypeId => Some(self.owner_doc().clone()),
            DocumentNodeTypeId => None
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-parentnode
    pub fn GetParentNode(&self) -> Option<JS<Node>> {
        self.parent_node.clone()
    }

    // http://dom.spec.whatwg.org/#dom-node-parentelement
    pub fn GetParentElement(&self) -> Option<JS<Element>> {
        self.parent_node.clone().and_then(|parent| ElementCast::to(&parent))
    }

    // http://dom.spec.whatwg.org/#dom-node-haschildnodes
    pub fn HasChildNodes(&self) -> bool {
        self.first_child.is_some()
    }

    // http://dom.spec.whatwg.org/#dom-node-childnodes
    pub fn ChildNodes(&mut self, abstract_self: &JSRef<Node>) -> JS<NodeList> {
        let roots = RootCollection::new();
        match self.child_list {
            None => {
                let doc = self.owner_doc().clone();
                let doc = doc.get();
                let window = doc.window.root(&roots);
                let list = NodeList::new_child_list(&window.root_ref(), abstract_self);
                self.child_list = Some(list.clone());
                list
            }
            Some(ref list) => list.clone()
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-firstchild
    pub fn GetFirstChild(&self) -> Option<JS<Node>> {
        self.first_child.clone()
    }

    // http://dom.spec.whatwg.org/#dom-node-lastchild
    pub fn GetLastChild(&self) -> Option<JS<Node>> {
        self.last_child.clone()
    }

    // http://dom.spec.whatwg.org/#dom-node-previoussibling
    pub fn GetPreviousSibling(&self) -> Option<JS<Node>> {
        self.prev_sibling.clone()
    }

    // http://dom.spec.whatwg.org/#dom-node-nextsibling
    pub fn GetNextSibling(&self) -> Option<JS<Node>> {
        self.next_sibling.clone()
    }

    // http://dom.spec.whatwg.org/#dom-node-nodevalue
    pub fn GetNodeValue(&self, abstract_self: &JSRef<Node>) -> Option<DOMString> {
        match self.type_id {
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                let chardata: JS<CharacterData> = CharacterDataCast::to(&abstract_self.unrooted()).unwrap();
                Some(chardata.get().Data())
            }
            _ => {
                None
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-nodevalue
    pub fn SetNodeValue(&mut self, abstract_self: &mut JSRef<Node>, val: Option<DOMString>)
                        -> ErrorResult {
        match self.type_id {
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                self.SetTextContent(abstract_self, val)
            }
            _ => Ok(())
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-textcontent
    pub fn GetTextContent(&self, abstract_self: &JSRef<Node>) -> Option<DOMString> {
        match self.type_id {
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => {
                let mut content = ~"";
                for node in abstract_self.unrooted().traverse_preorder() {
                    if node.is_text() {
                        let text: JS<Text> = TextCast::to(&node).unwrap();
                        content.push_str(text.get().characterdata.data.as_slice());
                    }
                }
                Some(content)
            }
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                let characterdata: JS<CharacterData> = CharacterDataCast::to(&abstract_self.unrooted()).unwrap();
                Some(characterdata.get().Data())
            }
            DoctypeNodeTypeId |
            DocumentNodeTypeId => {
                None
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-textcontent
    pub fn SetTextContent(&mut self, abstract_self: &mut JSRef<Node>, value: Option<DOMString>)
                          -> ErrorResult {
        let roots = RootCollection::new();
        let value = null_str_as_empty(&value);
        match self.type_id {
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => {
                // Step 1-2.
                let node = if value.len() == 0 {
                    None
                } else {
                    let document = self.owner_doc();
                    let document = document.root(&roots);
                    Some(NodeCast::from(&document.get().CreateTextNode(&document.root_ref(), value)))
                };
                let node = node.map(|node| node.root(&roots));
                
                // Step 3.
                Node::replace_all(node.root_ref(), abstract_self);
            }
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                self.wait_until_safe_to_modify_dom();

                let mut characterdata: JS<CharacterData> = CharacterDataCast::to(&abstract_self.unrooted()).unwrap();
                characterdata.get_mut().data = value.clone();

                // Notify the document that the content of this node is different
                let document = self.owner_doc();
                document.get().content_changed();
            }
            DoctypeNodeTypeId |
            DocumentNodeTypeId => {}
        }
        Ok(())
    }

    // http://dom.spec.whatwg.org/#concept-node-adopt
    pub fn adopt(node_root: &mut JSRef<Node>, document: &JSRef<Document>) {
        let roots = RootCollection::new();
        let node = node_root.unrooted();
        // Step 1.
        match node.parent_node() {
            Some(ref mut parent) => {
                let parent = parent.root(&roots);
                Node::remove(node_root, &mut parent.root_ref(), Unsuppressed);
            }
            None => (),
        }

        // Step 2.
        if document_from_node(&node) != document.unrooted() {
            for mut descendant in node.traverse_preorder() {
                descendant.get_mut().set_owner_doc(document);
            }
        }

        // Step 3.
        // If node is an element, it is _affected by a base URL change_.
    }

    // http://dom.spec.whatwg.org/#concept-node-pre-insert
    fn pre_insert(node_root: &mut JSRef<Node>, parent_root: &mut JSRef<Node>, child: Option<JSRef<Node>>)
                  -> Fallible<JS<Node>> {
        let roots = RootCollection::new();
        let node = node_root.unrooted();
        let parent = parent_root.unrooted();
        // Step 1.
        match parent.type_id() {
            DocumentNodeTypeId |
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => (),
            _ => return Err(HierarchyRequest)
        }

        // Step 2.
        if node.is_inclusive_ancestor_of(parent_root) {
            return Err(HierarchyRequest);
        }

        // Step 3.
        match child {
            Some(ref child) if !parent.is_parent_of(child) => return Err(NotFound),
            _ => ()
        }

        // Step 4-5.
        match node.type_id() {
            TextNodeTypeId => {
                match node.parent_node() {
                    Some(ref parent) if parent.is_document() => return Err(HierarchyRequest),
                    _ => ()
                }
            }
            DoctypeNodeTypeId => {
                match node.parent_node() {
                    Some(ref parent) if !parent.is_document() => return Err(HierarchyRequest),
                    _ => ()
                }
            }
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(_) |
            ProcessingInstructionNodeTypeId |
            CommentNodeTypeId => (),
            DocumentNodeTypeId => return Err(HierarchyRequest)
        }

        // Step 6.
        let child = child.map(|child| child.unrooted());
        match parent.type_id() {
            DocumentNodeTypeId => {
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
                                    Some(ref child) if child.inclusively_following_siblings()
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
                            Some(ref child) if child.inclusively_following_siblings()
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
                            Some(ref child) => {
                                if parent.children()
                                    .take_while(|c| c != child)
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
                    DocumentNodeTypeId => unreachable!(),
                }
            },
            _ => (),
        }

        // Step 7-8.
        let referenceChild = match child {
            Some(ref child) if child == &node => node.next_sibling(),
            _ => child
        };
        let referenceChild = referenceChild.map(|child| child.root(&roots));

        // Step 9.
        let document = document_from_node(&parent);
        let document = document.root(&roots);
        Node::adopt(node_root, &document.root_ref());

        // Step 10.
        Node::insert(node_root, parent_root, referenceChild.root_ref(), Unsuppressed);

        // Step 11.
        return Ok(node)
    }

    // http://dom.spec.whatwg.org/#concept-node-insert
    fn insert(node_root: &mut JSRef<Node>,
              parent_root: &mut JSRef<Node>,
              child: Option<JSRef<Node>>,
              suppress_observers: SuppressObserver) {
        let roots = RootCollection::new();
        let node = node_root.unrooted();
        let mut parent = parent_root.unrooted();
        // XXX assert owner_doc
        // Step 1-3: ranges.
        // Step 4.
        let mut nodes = match node.type_id() {
            DocumentFragmentNodeTypeId => node.children().collect(),
            _ => vec!(node.clone()),
        };

        // Step 5: DocumentFragment, mutation records.
        // Step 6: DocumentFragment.
        match node.type_id() {
            DocumentFragmentNodeTypeId => {
                for c in node.children() {
                    let c = c.root(&roots);
                    Node::remove(&mut c.root_ref(), node_root, Suppressed);
                }
            },
            _ => (),
        }

        // Step 7: mutation records.
        // Step 8.
        for node in nodes.mut_iter() {
            parent.add_child(node_root, child.clone());
            node.get_mut().flags.set_is_in_doc(parent.is_in_doc());
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
    pub fn replace_all(mut node_root: Option<JSRef<Node>>, parent_root: &mut JSRef<Node>) {
        let roots = RootCollection::new();
        let node = node_root.as_ref().map(|node| node.unrooted());
        let parent = parent_root.unrooted();

        // Step 1.
        match node_root {
            Some(ref mut node) => {
                let document = document_from_node(&parent);
                let document = document.root(&roots);
                Node::adopt(node, &document.root_ref());
            }
            None => (),
        }

        // Step 2.
        let removedNodes: Vec<JS<Node>> = parent.children().collect();

        // Step 3.
        let addedNodes = match node {
            None => vec!(),
            Some(ref node) => match node.type_id() {
                DocumentFragmentNodeTypeId => node.children().collect(),
                _ => vec!(node.clone()),
            },
        };

        // Step 4.
        for child in parent.children() {
            let child = child.root(&roots);
            Node::remove(&mut child.root_ref(), parent_root, Suppressed);
        }

        // Step 5.
        match node_root {
            Some(ref mut node) => Node::insert(node, parent_root, None, Suppressed),
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
    fn pre_remove(child: &mut JSRef<Node>, parent: &mut JSRef<Node>) -> Fallible<JS<Node>> {
        // Step 1.
        match child.unrooted().parent_node() {
            Some(ref node) if node != &parent.unrooted() => return Err(NotFound),
            _ => ()
        }

        // Step 2.
        Node::remove(child, parent, Unsuppressed);

        // Step 3.
        Ok(child.unrooted())
    }

    // http://dom.spec.whatwg.org/#concept-node-remove
    fn remove(node_root: &mut JSRef<Node>, parent: &mut JSRef<Node>, suppress_observers: SuppressObserver) {
        let mut parent = parent.unrooted();
        let mut node = node_root.unrooted();
        assert!(node.parent_node().map_or(false, |ref node_parent| node_parent == &parent));

        // Step 1-5: ranges.
        // Step 6-7: mutation observers.
        // Step 8.
        parent.remove_child(node_root);
        node.get_mut().flags.set_is_in_doc(false);

        // Step 9.
        match suppress_observers {
            Suppressed => (),
            Unsuppressed => node.node_removed(),
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-clone
    pub fn clone(node: &JSRef<Node>, maybe_doc: Option<&JSRef<Document>>,
                 clone_children: CloneChildrenFlag) -> JS<Node> {
        let roots = RootCollection::new();

        // Step 1.
        let mut document = match maybe_doc {
            Some(doc) => doc.unrooted(),
            None => node.get().owner_doc().clone()
        };
        let document_root = document.root(&roots);

        // Step 2.
        // XXXabinader: clone() for each node as trait?
        let copy: JS<Node> = match node.get().type_id {
            DoctypeNodeTypeId => {
                let doctype: JS<DocumentType> = DocumentTypeCast::to(&node.unrooted()).unwrap();
                let doctype = doctype.get();
                let doctype = DocumentType::new(doctype.name.clone(),
                                                Some(doctype.public_id.clone()),
                                                Some(doctype.system_id.clone()), &document_root.root_ref());
                NodeCast::from(&doctype)
            },
            DocumentFragmentNodeTypeId => {
                let doc_fragment = DocumentFragment::new(&document_root.root_ref());
                NodeCast::from(&doc_fragment)
            },
            CommentNodeTypeId => {
                let comment: JS<Comment> = CommentCast::to(&node.unrooted()).unwrap();
                let comment = comment.get();
                let comment = Comment::new(comment.characterdata.data.clone(), &document_root.root_ref());
                NodeCast::from(&comment)
            },
            DocumentNodeTypeId => {
                let document: JS<Document> = DocumentCast::to(&node.unrooted()).unwrap();
                let document = document.get();
                let is_html_doc = match document.is_html_document {
                    true => HTMLDocument,
                    false => NonHTMLDocument
                };
                let window = document.window.root(&roots);
                let document = Document::new(&window.root_ref(), Some(document.url().clone()),
                                             is_html_doc, None);
                NodeCast::from(&document)
            },
            ElementNodeTypeId(..) => {
                let element: JS<Element> = ElementCast::to(&node.unrooted()).unwrap();
                let element = element.get();
                let element = build_element_from_tag(element.local_name.clone(), &document_root.root_ref());
                NodeCast::from(&element)
            },
            TextNodeTypeId => {
                let text: JS<Text> = TextCast::to(&node.unrooted()).unwrap();
                let text = text.get();
                let text = Text::new(text.characterdata.data.clone(), &document_root.root_ref());
                NodeCast::from(&text)
            },
            ProcessingInstructionNodeTypeId => {
                let pi: JS<ProcessingInstruction> = ProcessingInstructionCast::to(&node.unrooted()).unwrap();
                let pi = pi.get();
                let pi = ProcessingInstruction::new(pi.target.clone(),
                                                    pi.characterdata.data.clone(), &document_root.root_ref());
                NodeCast::from(&pi)
            },
        };

        // Step 3.
        if copy.is_document() {
            document = DocumentCast::to(&copy).unwrap();
        }
        let document_root = document.root(&roots);
        assert!(copy.get().owner_doc() == &document);

        // Step 4 (some data already copied in step 2).
        match node.get().type_id {
            DocumentNodeTypeId => {
                let node_doc: JS<Document> = DocumentCast::to(&node.unrooted()).unwrap();
                let node_doc = node_doc.get();
                let mut copy_doc: JS<Document> = DocumentCast::to(&copy).unwrap();
                let copy_doc = copy_doc.get_mut();
                copy_doc.set_encoding_name(node_doc.encoding_name.clone());
                copy_doc.set_quirks_mode(node_doc.quirks_mode());
            },
            ElementNodeTypeId(..) => {
                let node_elem: JS<Element> = ElementCast::to(&node.unrooted()).unwrap();
                let node_elem = node_elem.get();
                let mut copy_elem: JS<Element> = ElementCast::to(&copy).unwrap();

                // XXX: to avoid double borrowing compile error. we might be able to fix this after #1854
                let copy_elem_alias: JS<Element> = copy_elem.clone();

                let copy_elem = copy_elem.get_mut();
                // FIXME: https://github.com/mozilla/servo/issues/1737
                copy_elem.namespace = node_elem.namespace.clone();
                let window = document.get().window.root(&roots);
                for attr in node_elem.attrs.iter() {
                    let attr = attr.get();
                    copy_elem.attrs.push(Attr::new(&window.root_ref(),
                                                   attr.local_name.clone(), attr.value.clone(),
                                                   attr.name.clone(), attr.namespace.clone(),
                                                   attr.prefix.clone(), copy_elem_alias.clone()));
                }
            },
            _ => ()
        }

        // Step 5: cloning steps.

        // Step 6.
        if clone_children == CloneChildren {
            for ref child in node.get().children() {
                let child = child.root(&roots);
                let child_copy = Node::clone(&child.root_ref(), Some(&document_root.root_ref()), clone_children).root(&roots);
                let copy = copy.root(&roots);
                let _inserted_node = Node::pre_insert(&mut child_copy.root_ref(), &mut copy.root_ref(), None);
            }
        }

        // Step 7.
        copy
    }

    // http://dom.spec.whatwg.org/#dom-node-insertbefore
    pub fn InsertBefore(&self, abstract_self: &mut JSRef<Node>, node: &mut JSRef<Node>, child: Option<JSRef<Node>>)
                        -> Fallible<JS<Node>> {
        Node::pre_insert(node, abstract_self, child)
    }

    pub fn wait_until_safe_to_modify_dom(&self) {
        let document = self.owner_doc();
        document.get().wait_until_safe_to_modify_dom();
    }

    // http://dom.spec.whatwg.org/#dom-node-appendchild
    pub fn AppendChild(&self, abstract_self: &mut JSRef<Node>, node: &mut JSRef<Node>)
                       -> Fallible<JS<Node>> {
        Node::pre_insert(node, abstract_self, None)
    }

    // http://dom.spec.whatwg.org/#concept-node-replace
    pub fn ReplaceChild(&self, parent_root: &mut JSRef<Node>, node_root: &mut JSRef<Node>, child_root: &mut JSRef<Node>)
                        -> Fallible<JS<Node>> {
        let roots = RootCollection::new();
        let parent = parent_root.unrooted();
        let node = node_root.unrooted();
        let child = child_root.unrooted();

        // Step 1.
        match parent.type_id() {
            DocumentNodeTypeId |
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => (),
            _ => return Err(HierarchyRequest)
        }

        // Step 2.
        if node.is_inclusive_ancestor_of(parent_root) {
            return Err(HierarchyRequest);
        }

        // Step 3.
        if !parent.is_parent_of(child_root) {
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
            DocumentNodeTypeId => return Err(HierarchyRequest)
        }

        // Step 6.
        match parent.type_id() {
            DocumentNodeTypeId => {
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
                                if parent.child_elements().any(|c| NodeCast::from(&c) != child) {
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
                        if parent.child_elements().any(|c| NodeCast::from(&c) != child) {
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
                            .take_while(|c| c != &child)
                            .any(|c| c.is_element()) {
                            return Err(HierarchyRequest);
                        }
                    },
                    TextNodeTypeId |
                    ProcessingInstructionNodeTypeId |
                    CommentNodeTypeId => (),
                    DocumentNodeTypeId => unreachable!()
                }
            },
            _ => ()
        }

        // Ok if not caught by previous error checks.
        if node == child {
            return Ok(child);
        }

        // Step 7-8.
        let next_sibling = child.next_sibling();
        let reference_child = match next_sibling {
            Some(ref sibling) if sibling == &node => node.next_sibling(),
            _ => next_sibling
        };
        let reference_child = reference_child.map(|child| child.root(&roots));

        // Step 9.
        let document = document_from_node(&parent);
        let document = document.root(&roots);
        Node::adopt(node_root, &document.root_ref());

        {
            // Step 10.
            Node::remove(child_root, parent_root, Suppressed);

            // Step 11.
            Node::insert(node_root, parent_root, reference_child.root_ref(), Suppressed);
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
    pub fn RemoveChild(&self, abstract_self: &mut JSRef<Node>, node: &mut JSRef<Node>)
                       -> Fallible<JS<Node>> {
        Node::pre_remove(node, abstract_self)
    }

    // http://dom.spec.whatwg.org/#dom-node-normalize
    pub fn Normalize(&mut self, abstract_self: &mut JSRef<Node>) {
        let roots = RootCollection::new();
        let mut abstract_self = abstract_self.unrooted();
        let mut prev_text = None;
        for mut child in self.children() {
            if child.is_text() {
                let characterdata: JS<CharacterData> = CharacterDataCast::to(&child).unwrap();
                if characterdata.get().Length() == 0 {
                    let child = child.root(&roots);
                    abstract_self.remove_child(&mut child.root_ref());
                } else {
                    match prev_text {
                        Some(ref text_node) => {
                            let mut prev_characterdata: JS<CharacterData> = CharacterDataCast::to(text_node).unwrap();
                            let _ = prev_characterdata.get_mut().AppendData(characterdata.get().Data());
                            let child = child.root(&roots);
                            abstract_self.remove_child(&mut child.root_ref());
                        },
                        None => prev_text = Some(child)
                    }
                }
            } else {
                let c = child.root(&roots);
                child.get_mut().Normalize(&mut c.root_ref());
                prev_text = None;
            }

        }
    }

    // http://dom.spec.whatwg.org/#dom-node-clonenode
    pub fn CloneNode(&self, abstract_self: &mut JSRef<Node>, deep: bool) -> JS<Node> {
        match deep {
            true => Node::clone(abstract_self, None, CloneChildren),
            false => Node::clone(abstract_self, None, DoNotCloneChildren)
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-isequalnode
    pub fn IsEqualNode(&self, abstract_self: &JSRef<Node>, maybe_node: Option<JSRef<Node>>) -> bool {
        fn is_equal_doctype(node: &JSRef<Node>, other: &JSRef<Node>) -> bool {
            let doctype: JS<DocumentType> = DocumentTypeCast::to(&node.unrooted()).unwrap();
            let other_doctype: JS<DocumentType> = DocumentTypeCast::to(&other.unrooted()).unwrap();
            (doctype.get().name == other_doctype.get().name) &&
            (doctype.get().public_id == other_doctype.get().public_id) &&
            (doctype.get().system_id == other_doctype.get().system_id)
        }
        fn is_equal_element(node: &JSRef<Node>, other: &JSRef<Node>) -> bool {
            let element: JS<Element> = ElementCast::to(&node.unrooted()).unwrap();
            let other_element: JS<Element> = ElementCast::to(&other.unrooted()).unwrap();
            // FIXME: namespace prefix
            (element.get().namespace == other_element.get().namespace) &&
            (element.get().local_name == other_element.get().local_name) &&
            (element.get().attrs.len() == other_element.get().attrs.len())
        }
        fn is_equal_processinginstruction(node: &JSRef<Node>, other: &JSRef<Node>) -> bool {
            let pi: JS<ProcessingInstruction> = ProcessingInstructionCast::to(&node.unrooted()).unwrap();
            let other_pi: JS<ProcessingInstruction> = ProcessingInstructionCast::to(&other.unrooted()).unwrap();
            (pi.get().target == other_pi.get().target) &&
            (pi.get().characterdata.data == other_pi.get().characterdata.data)
        }
        fn is_equal_characterdata(node: &JSRef<Node>, other: &JSRef<Node>) -> bool {
            let characterdata: JS<CharacterData> = CharacterDataCast::to(&node.unrooted()).unwrap();
            let other_characterdata: JS<CharacterData> = CharacterDataCast::to(&other.unrooted()).unwrap();
            characterdata.get().data == other_characterdata.get().data
        }
        fn is_equal_element_attrs(node: &JSRef<Node>, other: &JSRef<Node>) -> bool {
            let element: JS<Element> = ElementCast::to(&node.unrooted()).unwrap();
            let other_element: JS<Element> = ElementCast::to(&other.unrooted()).unwrap();
            assert!(element.get().attrs.len() == other_element.get().attrs.len());
            element.get().attrs.iter().all(|attr| {
                other_element.get().attrs.iter().any(|other_attr| {
                    (attr.get().namespace == other_attr.get().namespace) &&
                    (attr.get().local_name == other_attr.get().local_name) &&
                    (attr.get().value == other_attr.get().value)
                })
            })
        }
        fn is_equal_node(this: &JSRef<Node>, node: &JSRef<Node>) -> bool {
            let roots = RootCollection::new();
            let this_node = this.unrooted();
            let other = node.unrooted();

            // Step 2.
            if this_node.type_id() != other.type_id() {
                return false;
            }

            match other.type_id() {
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
            if this_node.children().len() != other.children().len() {
                return false;
            }

            // Step 6.
            this_node.children().zip(other.children()).all(|(ref child, ref other_child)| {
                let child = child.root(&roots);
                let other_child = other_child.root(&roots);
                is_equal_node(&child.root_ref(), &other_child.root_ref())
            })
        }
        match maybe_node {
            // Step 1.
            None => false,
            // Step 2-6.
            Some(ref node) => is_equal_node(abstract_self, node)
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-comparedocumentposition
    pub fn CompareDocumentPosition(&self, abstract_self_root: &JSRef<Node>, other_root: &JSRef<Node>) -> u16 {
        let other = other_root.unrooted();
        let abstract_self = abstract_self_root.unrooted();
        if abstract_self == other {
            // step 2.
            0
        } else {
            let mut lastself = abstract_self.clone();
            let mut lastother = other.clone();
            for ancestor in abstract_self.ancestors() {
                if ancestor == other {
                    // step 4.
                    return NodeConstants::DOCUMENT_POSITION_CONTAINS +
                           NodeConstants::DOCUMENT_POSITION_PRECEDING;
                }
                lastself = ancestor;
            }
            for ancestor in other.ancestors() {
                if ancestor == abstract_self {
                    // step 5.
                    return NodeConstants::DOCUMENT_POSITION_CONTAINED_BY +
                           NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                }
                lastother = ancestor;
            }

            if lastself != lastother {
                let abstract_uint: uintptr_t = as_uintptr(&abstract_self.get());
                let other_uint: uintptr_t = as_uintptr(&other.get());

                let random = if abstract_uint < other_uint {
                    NodeConstants::DOCUMENT_POSITION_FOLLOWING
                } else {
                    NodeConstants::DOCUMENT_POSITION_PRECEDING
                };
                // step 3.
                return random +
                    NodeConstants::DOCUMENT_POSITION_DISCONNECTED +
                    NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC;
            }

            for child in lastself.traverse_preorder() {
                if child == other {
                    // step 6.
                    return NodeConstants::DOCUMENT_POSITION_PRECEDING;
                }
                if child == abstract_self {
                    // step 7.
                    return NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                }
            }
            unreachable!()
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-contains
    pub fn Contains(&self, abstract_self: &JSRef<Node>, maybe_other: Option<JSRef<Node>>) -> bool {
        match maybe_other {
            None => false,
            Some(ref other) => abstract_self.unrooted().is_inclusive_ancestor_of(other)
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-lookupprefix
    pub fn LookupPrefix(&self, _prefix: Option<DOMString>) -> Option<DOMString> {
        // FIXME (#1826) implement.
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-lookupnamespaceuri
    pub fn LookupNamespaceURI(&self, _namespace: Option<DOMString>) -> Option<DOMString> {
        // FIXME (#1826) implement.
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-isdefaultnamespace
    pub fn IsDefaultNamespace(&self, _namespace: Option<DOMString>) -> bool {
        // FIXME (#1826) implement.
        false
    }

    //
    // Low-level pointer stitching
    //

    pub fn set_parent_node(&mut self, new_parent_node: Option<JSRef<Node>>) {
        let doc = self.owner_doc().clone();
        doc.get().wait_until_safe_to_modify_dom();
        self.parent_node = new_parent_node.map(|node| node.unrooted())
    }

    pub fn set_first_child(&mut self, new_first_child: Option<JSRef<Node>>) {
        let doc = self.owner_doc().clone();
        doc.get().wait_until_safe_to_modify_dom();
        self.first_child = new_first_child.map(|node| node.unrooted())
    }

    pub fn set_last_child(&mut self, new_last_child: Option<JSRef<Node>>) {
        let doc = self.owner_doc().clone();
        doc.get().wait_until_safe_to_modify_dom();
        self.last_child = new_last_child.map(|node| node.unrooted())
    }

    pub fn set_prev_sibling(&mut self, new_prev_sibling: Option<JSRef<Node>>) {
        let doc = self.owner_doc().clone();
        doc.get().wait_until_safe_to_modify_dom();
        self.prev_sibling = new_prev_sibling.map(|node| node.unrooted())
    }

    pub fn set_next_sibling(&mut self, new_next_sibling: Option<JSRef<Node>>) {
        let doc = self.owner_doc().clone();
        doc.get().wait_until_safe_to_modify_dom();
        self.next_sibling = new_next_sibling.map(|node| node.unrooted())
    }

    pub fn get_hover_state(&self) -> bool {
        self.flags.get_in_hover_state()
    }

    pub fn set_hover_state(&mut self, state: bool) {
        self.flags.set_is_in_hover_state(state);
    }

    #[inline]
    pub fn parent_node_ref<'a>(&'a self) -> Option<&'a JS<Node>> {
        self.parent_node.as_ref()
    }

    #[inline]
    pub fn first_child_ref<'a>(&'a self) -> Option<&'a JS<Node>> {
        self.first_child.as_ref()
    }

    #[inline]
    pub fn last_child_ref<'a>(&'a self) -> Option<&'a JS<Node>> {
        self.last_child.as_ref()
    }

    #[inline]
    pub fn prev_sibling_ref<'a>(&'a self) -> Option<&'a JS<Node>> {
        self.prev_sibling.as_ref()
    }

    #[inline]
    pub fn next_sibling_ref<'a>(&'a self) -> Option<&'a JS<Node>> {
        self.next_sibling.as_ref()
    }

    pub unsafe fn get_hover_state_for_layout(&self) -> bool {
        self.flags.get_in_hover_state()
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

pub fn document_from_node<T: NodeBase>(derived: &JS<T>) -> JS<Document> {
    let node: JS<Node> = NodeCast::from(derived);
    node.get().owner_doc().clone()
}

pub fn window_from_node<T: NodeBase>(derived: &JS<T>) -> JS<Window> {
    let document: JS<Document> = document_from_node(derived);
    document.get().window.clone()
}

impl VirtualMethods for JS<Node> {
    fn super_type(&self) -> Option<~VirtualMethods:> {
        let eventtarget: JS<EventTarget> = EventTargetCast::from(self);
        Some(~eventtarget as ~VirtualMethods:)
    }
}
