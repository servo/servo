/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use dom::attr::Attr;
use dom::bindings::codegen::InheritTypes::{CommentCast, DocumentCast, DocumentTypeCast};
use dom::bindings::codegen::InheritTypes::{ElementCast, TextCast, NodeCast, ElementDerived};
use dom::bindings::codegen::InheritTypes::{CharacterDataCast, NodeBase, NodeDerived};
use dom::bindings::codegen::InheritTypes::{ProcessingInstructionCast, EventTargetCast};
use dom::bindings::codegen::BindingDeclarations::NodeBinding::NodeConstants;
use dom::bindings::js::{JS, JSRef, RootedReference, Temporary, Root, OptionalUnrootable};
use dom::bindings::js::{OptionalSettable, TemporaryPushable, OptionalRootedRootable};
use dom::bindings::js::{ResultRootable, OptionalRootable};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::bindings::error::{ErrorResult, Fallible, NotFound, HierarchyRequest};
use dom::bindings::utils;
use dom::characterdata::{CharacterData, CharacterDataMethods};
use dom::comment::Comment;
use dom::document::{Document, DocumentMethods, DocumentHelpers, HTMLDocument, NonHTMLDocument};
use dom::documentfragment::DocumentFragment;
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementMethods, ElementTypeId, HTMLAnchorElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::nodelist::{NodeList};
use dom::processinginstruction::{ProcessingInstruction, ProcessingInstructionMethods};
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

trait PrivateNodeHelpers {
    fn set_parent_node(&mut self, new_parent_node: Option<JSRef<Node>>);
    fn set_first_child(&mut self, new_first_child: Option<JSRef<Node>>);
    fn set_last_child(&mut self, new_last_child: Option<JSRef<Node>>);
    fn set_prev_sibling(&mut self, new_prev_sibling: Option<JSRef<Node>>);
    fn set_next_sibling(&mut self, new_next_sibling: Option<JSRef<Node>>);

    fn node_inserted(&self);
    fn node_removed(&self);
    fn add_child(&mut self, new_child: &mut JSRef<Node>, before: Option<JSRef<Node>>);
    fn remove_child(&mut self, child: &mut JSRef<Node>);
}

impl<'a> PrivateNodeHelpers for JSRef<'a, Node> {
    // http://dom.spec.whatwg.org/#node-is-inserted
    fn node_inserted(&self) {
        assert!(self.parent_node().is_some());
        let document = document_from_node(self).root();

        if self.is_in_doc() {
            for mut node in self.traverse_preorder() {
                vtable_for(&mut node).bind_to_tree();
            }
        }

        let mut parent = self.parent_node().root();
        parent.as_mut().map(|parent| vtable_for(&mut **parent).child_inserted(self));

        document.deref().content_changed();
    }

    // http://dom.spec.whatwg.org/#node-is-removed
    fn node_removed(&self) {
        assert!(self.parent_node().is_none());
        let document = document_from_node(self).root();

        for mut node in self.traverse_preorder() {
            // XXX how about if the node wasn't in the tree in the first place?
            vtable_for(&mut node).unbind_from_tree();
        }

        document.deref().content_changed();
    }

    //
    // Pointer stitching
    //

    /// Adds a new child to the end of this node's list of children.
    ///
    /// Fails unless `new_child` is disconnected from the tree.
    fn add_child(&mut self, new_child: &mut JSRef<Node>, mut before: Option<JSRef<Node>>) {
        assert!(new_child.parent_node().is_none());
        assert!(new_child.prev_sibling().is_none());
        assert!(new_child.next_sibling().is_none());
        match before {
            Some(ref mut before) => {
                // XXX Should assert that parent is self.
                assert!(before.parent_node().is_some());
                match before.prev_sibling().root() {
                    None => {
                        // XXX Should assert that before is the first child of
                        //     self.
                        self.set_first_child(Some(new_child.clone()));
                    },
                    Some(mut prev_sibling) => {
                        prev_sibling.set_next_sibling(Some(new_child.clone()));
                        new_child.set_prev_sibling(Some((*prev_sibling).clone()));
                    },
                }
                before.set_prev_sibling(Some(new_child.clone()));
                new_child.set_next_sibling(Some(before.clone()));
            },
            None => {
                match self.last_child().root() {
                    None => self.set_first_child(Some(new_child.clone())),
                    Some(mut last_child) => {
                        assert!(last_child.next_sibling().is_none());
                        last_child.set_next_sibling(Some(new_child.clone()));
                        new_child.set_prev_sibling(Some((*last_child).clone()));
                    }
                }

                self.set_last_child(Some(new_child.clone()));
            },
        }

        new_child.set_parent_node(Some(self.clone()));
    }

    /// Removes the given child from this node's list of children.
    ///
    /// Fails unless `child` is a child of this node. (FIXME: This is not yet checked.)
    fn remove_child(&mut self, child: &mut JSRef<Node>) {
        assert!(child.parent_node.is_some());

        match child.prev_sibling.root() {
            None => {
                let next_sibling = child.next_sibling.root();
                self.set_first_child(next_sibling.root_ref());
            }
            Some(ref mut prev_sibling) => {
                let next_sibling = child.next_sibling.root();
                prev_sibling.set_next_sibling(next_sibling.root_ref());
            }
        }

        match child.next_sibling.root() {
            None => {
                let prev_sibling = child.prev_sibling.root();
                self.set_last_child(prev_sibling.root_ref());
            }
            Some(ref mut next_sibling) => {
                let prev_sibling = child.prev_sibling.root();
                next_sibling.set_prev_sibling(prev_sibling.root_ref());
            }
        }

        child.set_prev_sibling(None);
        child.set_next_sibling(None);
        child.set_parent_node(None);
    }

    //
    // Low-level pointer stitching
    //

    fn set_parent_node(&mut self, new_parent_node: Option<JSRef<Node>>) {
        let doc = self.owner_doc().root();
        doc.deref().wait_until_safe_to_modify_dom();
        self.parent_node.assign(new_parent_node);
    }

    fn set_first_child(&mut self, new_first_child: Option<JSRef<Node>>) {
        let doc = self.owner_doc().root();
        doc.deref().wait_until_safe_to_modify_dom();
        self.first_child.assign(new_first_child);
    }

    fn set_last_child(&mut self, new_last_child: Option<JSRef<Node>>) {
        let doc = self.owner_doc().root();
        doc.deref().wait_until_safe_to_modify_dom();
        self.last_child.assign(new_last_child);
    }

    fn set_prev_sibling(&mut self, new_prev_sibling: Option<JSRef<Node>>) {
        let doc = self.owner_doc().root();
        doc.deref().wait_until_safe_to_modify_dom();
        self.prev_sibling.assign(new_prev_sibling);
    }

    fn set_next_sibling(&mut self, new_next_sibling: Option<JSRef<Node>>) {
        let doc = self.owner_doc().root();
        doc.deref().wait_until_safe_to_modify_dom();
        self.next_sibling.assign(new_next_sibling);
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

    fn parent_node(&self) -> Option<Temporary<Node>>;
    fn first_child(&self) -> Option<Temporary<Node>>;
    fn last_child(&self) -> Option<Temporary<Node>>;
    fn prev_sibling(&self) -> Option<Temporary<Node>>;
    fn next_sibling(&self) -> Option<Temporary<Node>>;

    fn owner_doc(&self) -> Temporary<Document>;
    fn set_owner_doc(&mut self, document: &JSRef<Document>);

    fn wait_until_safe_to_modify_dom(&self);

    fn is_element(&self) -> bool;
    fn is_document(&self) -> bool;
    fn is_doctype(&self) -> bool;
    fn is_text(&self) -> bool;
    fn is_anchor_element(&self) -> bool;

    fn get_hover_state(&self) -> bool;
    fn set_hover_state(&mut self, state: bool);

    fn dump(&self);
    fn dump_indent(&self, indent: uint);
    fn debug_str(&self) -> ~str;

    fn traverse_preorder<'a>(&'a self) -> TreeIterator<'a>;
    fn sequential_traverse_postorder<'a>(&'a self) -> TreeIterator<'a>;
    fn inclusively_following_siblings<'a>(&'a self) -> AbstractNodeChildrenIterator<'a>;

    fn to_trusted_node_address(&self) -> TrustedNodeAddress;

    fn get_bounding_content_box(&self) -> Rect<Au>;
    fn get_content_boxes(&self) -> Vec<Rect<Au>>;

    fn remove_self(&mut self);
}

impl<'a> NodeHelpers for JSRef<'a, Node> {
    /// Dumps the subtree rooted at this node, for debugging.
    fn dump(&self) {
        self.dump_indent(0);
    }

    /// Dumps the node tree, for debugging, with indentation.
    fn dump_indent(&self, indent: uint) {
        let mut s = "".to_owned();
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

    fn is_in_doc(&self) -> bool {
        self.deref().flags.is_in_doc()
    }

    /// Returns the type ID of this node. Fails if this node is borrowed mutably.
    fn type_id(&self) -> NodeTypeId {
        self.deref().type_id
    }

    fn parent_node(&self) -> Option<Temporary<Node>> {
        self.deref().parent_node.clone().map(|node| Temporary::new(node))
    }

    fn first_child(&self) -> Option<Temporary<Node>> {
        self.deref().first_child.clone().map(|node| Temporary::new(node))
    }

    fn last_child(&self) -> Option<Temporary<Node>> {
        self.deref().last_child.clone().map(|node| Temporary::new(node))
    }

    /// Returns the previous sibling of this node. Fails if this node is borrowed mutably.
    fn prev_sibling(&self) -> Option<Temporary<Node>> {
        self.deref().prev_sibling.clone().map(|node| Temporary::new(node))
    }

    /// Returns the next sibling of this node. Fails if this node is borrowed mutably.
    fn next_sibling(&self) -> Option<Temporary<Node>> {
        self.deref().next_sibling.clone().map(|node| Temporary::new(node))
    }

    #[inline]
    fn is_element(&self) -> bool {
        match self.type_id {
            ElementNodeTypeId(..) => true,
            _ => false
        }
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
        match self.type_id {
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

    fn get_hover_state(&self) -> bool {
        self.flags.get_in_hover_state()
    }

    fn set_hover_state(&mut self, state: bool) {
        self.flags.set_is_in_hover_state(state);
    }

    /// Iterates over this node and all its descendants, in preorder.
    fn traverse_preorder<'a>(&'a self) -> TreeIterator<'a> {
        let mut nodes = vec!();
        gather_abstract_nodes(self, &mut nodes, false);
        TreeIterator::new(nodes)
    }

    /// Iterates over this node and all its descendants, in postorder.
    fn sequential_traverse_postorder<'a>(&'a self) -> TreeIterator<'a> {
        let mut nodes = vec!();
        gather_abstract_nodes(self, &mut nodes, true);
        TreeIterator::new(nodes)
    }

    fn inclusively_following_siblings<'a>(&'a self) -> AbstractNodeChildrenIterator<'a> {
        AbstractNodeChildrenIterator {
            current_node: Some(self.clone()),
        }
    }

    fn is_inclusive_ancestor_of(&self, parent: &JSRef<Node>) -> bool {
        self == parent || parent.ancestors().any(|ancestor| &ancestor == self)
    }

    fn following_siblings(&self) -> AbstractNodeChildrenIterator {
        AbstractNodeChildrenIterator {
            current_node: self.next_sibling().root().map(|next| next.deref().clone()),
        }
    }

    fn is_parent_of(&self, child: &JSRef<Node>) -> bool {
        match child.parent_node() {
            Some(ref parent) if *parent == Temporary::from_rooted(self) => true,
            _ => false
        }
    }

    fn to_trusted_node_address(&self) -> TrustedNodeAddress {
        TrustedNodeAddress(self.deref() as *Node as *libc::c_void)
    }

    fn get_bounding_content_box(&self) -> Rect<Au> {
        let window = window_from_node(self).root();
        let page = window.deref().page();
        let (chan, port) = channel();
        let addr = self.to_trusted_node_address();
        let ContentBoxResponse(rect) = page.query_layout(ContentBoxQuery(addr, chan), port);
        rect
    }

    fn get_content_boxes(&self) -> Vec<Rect<Au>> {
        let window = window_from_node(self).root();
        let page = window.deref().page();
        let (chan, port) = channel();
        let addr = self.to_trusted_node_address();
        let ContentBoxesResponse(rects) = page.query_layout(ContentBoxesQuery(addr, chan), port);
        rects
    }

    fn ancestors(&self) -> AncestorIterator {
        AncestorIterator {
            current: self.parent_node.clone().map(|node| (*node.root()).clone()),
        }
    }

    fn owner_doc(&self) -> Temporary<Document> {
        Temporary::new(self.owner_doc.get_ref().clone())
    }

    fn set_owner_doc(&mut self, document: &JSRef<Document>) {
        self.owner_doc.assign(Some(document.clone()));
    }

    fn children(&self) -> AbstractNodeChildrenIterator {
        AbstractNodeChildrenIterator {
            current_node: self.first_child.clone().map(|node| (*node.root()).clone()),
        }
    }

    fn child_elements(&self) -> ChildElementIterator {
        self.children()
            .filter(|node| {
                node.is_element()
            })
            .map(|node| {
                let elem: &JSRef<Element> = ElementCast::to_ref(&node).unwrap();
                elem.clone()
            })
    }

    fn wait_until_safe_to_modify_dom(&self) {
        let document = self.owner_doc().root();
        document.deref().wait_until_safe_to_modify_dom();
    }

    fn remove_self(&mut self) {
        match self.parent_node().root() {
            Some(ref mut parent) => parent.remove_child(self),
            None => ()
        }
    }
}

/// If the given untrusted node address represents a valid DOM node in the given runtime,
/// returns it.
pub fn from_untrusted_node_address(runtime: *JSRuntime, candidate: UntrustedNodeAddress)
    -> Temporary<Node> {
    unsafe {
        let candidate: uintptr_t = cast::transmute(candidate);
        let object: *JSObject = jsfriendapi::bindgen::JS_GetAddressableObject(runtime,
                                                                              candidate);
        if object.is_null() {
            fail!("Attempted to create a `JS<Node>` from an invalid pointer!")
        }
        let boxed_node: *mut Node = utils::unwrap(object);
        Temporary::new(JS::from_raw(boxed_node))
    }
}

pub trait LayoutNodeHelpers {
    unsafe fn type_id_for_layout(&self) -> NodeTypeId;

    unsafe fn parent_node_ref<'a>(&'a self) -> Option<&'a JS<Node>>;
    unsafe fn first_child_ref<'a>(&'a self) -> Option<&'a JS<Node>>;
    unsafe fn last_child_ref<'a>(&'a self) -> Option<&'a JS<Node>>;
    unsafe fn prev_sibling_ref<'a>(&'a self) -> Option<&'a JS<Node>>;
    unsafe fn next_sibling_ref<'a>(&'a self) -> Option<&'a JS<Node>>;

    unsafe fn owner_doc_for_layout<'a>(&'a self) -> &'a JS<Document>;

    unsafe fn is_element_for_layout(&self) -> bool;
}

impl LayoutNodeHelpers for JS<Node> {
    unsafe fn type_id_for_layout(&self) -> NodeTypeId {
        (*self.unsafe_get()).type_id
    }

    unsafe fn is_element_for_layout(&self) -> bool {
        (*self.unsafe_get()).is_element()
    }

    #[inline]
    unsafe fn parent_node_ref<'a>(&'a self) -> Option<&'a JS<Node>> {
        (*self.unsafe_get()).parent_node.as_ref()
    }

    #[inline]
    unsafe fn first_child_ref<'a>(&'a self) -> Option<&'a JS<Node>> {
        (*self.unsafe_get()).first_child.as_ref()
    }

    #[inline]
    unsafe fn last_child_ref<'a>(&'a self) -> Option<&'a JS<Node>> {
        (*self.unsafe_get()).last_child.as_ref()
    }

    #[inline]
    unsafe fn prev_sibling_ref<'a>(&'a self) -> Option<&'a JS<Node>> {
        (*self.unsafe_get()).prev_sibling.as_ref()
    }

    #[inline]
    unsafe fn next_sibling_ref<'a>(&'a self) -> Option<&'a JS<Node>> {
        (*self.unsafe_get()).next_sibling.as_ref()
    }

    unsafe fn owner_doc_for_layout<'a>(&'a self) -> &'a JS<Document> {
        (*self.unsafe_get()).owner_doc.get_ref()
    }
}

pub trait RawLayoutNodeHelpers {
    unsafe fn get_hover_state_for_layout(&self) -> bool;
}

impl RawLayoutNodeHelpers for Node {
    unsafe fn get_hover_state_for_layout(&self) -> bool {
        self.flags.get_in_hover_state()
    }
}


//
// Iteration and traversal
//

pub type ChildElementIterator<'a, 'b> = Map<'a, JSRef<'b, Node>,
                                            JSRef<'b, Element>,
                                            Filter<'a, JSRef<'b, Node>, AbstractNodeChildrenIterator<'b>>>;

pub struct AbstractNodeChildrenIterator<'a> {
    current_node: Option<JSRef<'a, Node>>,
}

impl<'a> Iterator<JSRef<'a, Node>> for AbstractNodeChildrenIterator<'a> {
    fn next(&mut self) -> Option<JSRef<'a, Node>> {
        let node = self.current_node.clone();
        self.current_node = node.clone().and_then(|node| {
            node.next_sibling().map(|node| (*node.root()).clone())
        });
        node
    }
}

pub struct AncestorIterator<'a> {
    current: Option<JSRef<'a, Node>>,
}

impl<'a> Iterator<JSRef<'a, Node>> for AncestorIterator<'a> {
    fn next(&mut self) -> Option<JSRef<'a, Node>> {
        if self.current.is_none() {
            return None;
        }

        // FIXME: Do we need two clones here?
        let x = self.current.get_ref().clone();
        self.current = x.parent_node().map(|node| (*node.root()).clone());
        Some(x)
    }
}

// FIXME: Do this without precomputing a vector of refs.
// Easy for preorder; harder for postorder.
pub struct TreeIterator<'a> {
    nodes: Vec<JSRef<'a, Node>>,
    index: uint,
}

impl<'a> TreeIterator<'a> {
    fn new(nodes: Vec<JSRef<'a, Node>>) -> TreeIterator<'a> {
        TreeIterator {
            nodes: nodes,
            index: 0,
        }
    }
}

impl<'a> Iterator<JSRef<'a, Node>> for TreeIterator<'a> {
    fn next(&mut self) -> Option<JSRef<'a, Node>> {
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
    pub fn new<'a>(start_node: &JSRef<'a, Node>,
                   include_start: bool,
                   include_descendants_of_void: bool) -> NodeIterator {
        NodeIterator {
            start_node: start_node.unrooted(),
            current_node: None,
            depth: 0,
            include_start: include_start,
            include_descendants_of_void: include_descendants_of_void
        }
    }

    fn next_child<'b>(&self, node: &JSRef<'b, Node>) -> Option<JSRef<Node>> {
        if !self.include_descendants_of_void && node.is_element() {
            let elem: &JSRef<Element> = ElementCast::to_ref(node).unwrap();
            if elem.deref().is_void() {
                None
            } else {
                node.first_child().map(|child| (*child.root()).clone())
            }
        } else {
            node.first_child().map(|child| (*child.root()).clone())
        }
    }
}

impl<'a> Iterator<JSRef<'a, Node>> for NodeIterator {
    fn next(&mut self) -> Option<JSRef<Node>> {
        self.current_node = match self.current_node.as_ref().map(|node| node.root()) {
            None => {
                if self.include_start {
                    Some(self.start_node.clone())
                } else {
                    self.next_child(&*self.start_node.root())
                        .map(|child| child.unrooted())
                }
            },
            Some(node) => {
                match self.next_child(&*node) {
                    Some(child) => {
                        self.depth += 1;
                        Some(child.unrooted())
                    },
                    None if node.deref().unrooted() == self.start_node => None,
                    None => {
                        match node.deref().next_sibling().root() {
                            Some(sibling) => Some(sibling.deref().unrooted()),
                            None => {
                                let mut candidate = node.deref().clone();
                                while candidate.next_sibling().is_none() {
                                    candidate = (*candidate.parent_node()
                                                          .expect("Got to root without reaching start node")
                                                          .root()).clone();
                                    self.depth -= 1;
                                    if candidate.unrooted() == self.start_node {
                                        break;
                                    }
                                }
                                if candidate.unrooted() != self.start_node {
                                    candidate.next_sibling().map(|node| node.root().unrooted())
                                } else {
                                    None
                                }
                            }
                        }
                    }
                }
            }
        };
        self.current_node.clone().map(|node| (*node.root()).clone())
    }
}

fn gather_abstract_nodes<'a>(cur: &JSRef<'a, Node>, refs: &mut Vec<JSRef<'a, Node>>, postorder: bool) {
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

/// Specifies whether children must be recursively cloned or not.
#[deriving(Eq)]
pub enum CloneChildrenFlag {
    CloneChildren,
    DoNotCloneChildren
}

fn as_uintptr<T>(t: &T) -> uintptr_t { t as *T as uintptr_t }

impl Node {
    pub fn reflect_node<N: Reflectable+NodeBase>
            (node:      ~N,
             document:  &JSRef<Document>,
             wrap_fn:   extern "Rust" fn(*JSContext, &JSRef<Window>, ~N) -> JS<N>)
             -> Temporary<N> {
        assert!(node.reflector().get_jsobject().is_null());
        let window = document.deref().window.root();
        let node = reflect_dom_object(node, &window.root_ref(), wrap_fn).root();
        assert!(node.deref().reflector().get_jsobject().is_not_null());
        Temporary::from_rooted(&*node)
    }

    pub fn new_inherited(type_id: NodeTypeId, doc: &JSRef<Document>) -> Node {
        Node::new_(type_id, Some(doc.clone()))
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

            owner_doc: doc.unrooted(),
            child_list: None,

            flags: NodeFlags::new(type_id),

            layout_data: LayoutDataRef::new(),
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-adopt
    pub fn adopt(node: &mut JSRef<Node>, document: &JSRef<Document>) {
        // Step 1.
        match node.parent_node().root() {
            Some(mut parent) => {
                Node::remove(node, &mut *parent, Unsuppressed);
            }
            None => (),
        }

        // Step 2.
        let node_doc = document_from_node(node).root();
        if &*node_doc != document {
            for mut descendant in node.traverse_preorder() {
                descendant.set_owner_doc(document);
            }
        }

        // Step 3.
        // If node is an element, it is _affected by a base URL change_.
    }

    // http://dom.spec.whatwg.org/#concept-node-pre-insert
    fn pre_insert(node: &mut JSRef<Node>, parent: &mut JSRef<Node>, child: Option<JSRef<Node>>)
                  -> Fallible<Temporary<Node>> {
        // Step 1.
        match parent.type_id() {
            DocumentNodeTypeId |
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
            Some(ref child) if !parent.is_parent_of(child) => return Err(NotFound),
            _ => ()
        }

        // Step 4-5.
        match node.type_id() {
            TextNodeTypeId => {
                match node.parent_node().root() {
                    Some(ref parent) if parent.is_document() => return Err(HierarchyRequest),
                    _ => ()
                }
            }
            DoctypeNodeTypeId => {
                match node.parent_node().root() {
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
            Some(ref child) if child == node => node.next_sibling().map(|node| (*node.root()).clone()),
            _ => child
        };

        // Step 9.
        let document = document_from_node(parent).root();
        Node::adopt(node, &*document);

        // Step 10.
        Node::insert(node, parent, referenceChild, Unsuppressed);

        // Step 11.
        return Ok(Temporary::from_rooted(node))
    }

    // http://dom.spec.whatwg.org/#concept-node-insert
    fn insert(node: &mut JSRef<Node>,
              parent: &mut JSRef<Node>,
              child: Option<JSRef<Node>>,
              suppress_observers: SuppressObserver) {
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
                for mut c in node.children() {
                    Node::remove(&mut c, node, Suppressed);
                }
            },
            _ => (),
        }

        // Step 7: mutation records.
        // Step 8.
        for node in nodes.mut_iter() {
            parent.add_child(node, child.clone());
            node.deref_mut().flags.set_is_in_doc(parent.is_in_doc());
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
    fn replace_all(mut node: Option<JSRef<Node>>, parent: &mut JSRef<Node>) {

        // Step 1.
        match node {
            Some(ref mut node) => {
                let document = document_from_node(parent).root();
                Node::adopt(node, &*document);
            }
            None => (),
        }

        // Step 2.
        let removedNodes: Vec<JSRef<Node>> = parent.children().collect();

        // Step 3.
        let addedNodes = match node {
            None => vec!(),
            Some(ref node) => match node.type_id() {
                DocumentFragmentNodeTypeId => node.children().collect(),
                _ => vec!(node.clone()),
            },
        };

        // Step 4.
        for mut child in parent.children() {
            Node::remove(&mut child, parent, Suppressed);
        }

        // Step 5.
        match node {
            Some(ref mut node) => Node::insert(node, parent, None, Suppressed),
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
    fn pre_remove(child: &mut JSRef<Node>, parent: &mut JSRef<Node>) -> Fallible<Temporary<Node>> {
        // Step 1.
        match child.parent_node() {
            Some(ref node) if *node != Temporary::from_rooted(parent) => return Err(NotFound),
            _ => ()
        }

        // Step 2.
        Node::remove(child, parent, Unsuppressed);

        // Step 3.
        Ok(Temporary::from_rooted(child))
    }

    // http://dom.spec.whatwg.org/#concept-node-remove
    fn remove(node: &mut JSRef<Node>, parent: &mut JSRef<Node>, suppress_observers: SuppressObserver) {
        assert!(node.parent_node().map_or(false, |node_parent| node_parent == Temporary::from_rooted(parent)));

        // Step 1-5: ranges.
        // Step 6-7: mutation observers.
        // Step 8.
        parent.remove_child(node);
        node.deref_mut().flags.set_is_in_doc(false);

        // Step 9.
        match suppress_observers {
            Suppressed => (),
            Unsuppressed => node.node_removed(),
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-clone
    pub fn clone(node: &JSRef<Node>, maybe_doc: Option<&JSRef<Document>>,
                 clone_children: CloneChildrenFlag) -> Temporary<Node> {

        // Step 1.
        let mut document = match maybe_doc {
            Some(doc) => doc.unrooted().root(),
            None => node.owner_doc().root()
        };

        // Step 2.
        // XXXabinader: clone() for each node as trait?
        let mut copy: Root<Node> = match node.type_id() {
            DoctypeNodeTypeId => {
                let doctype: &JSRef<DocumentType> = DocumentTypeCast::to_ref(node).unwrap();
                let doctype = doctype.deref();
                let doctype = DocumentType::new(doctype.name.clone(),
                                                Some(doctype.public_id.clone()),
                                                Some(doctype.system_id.clone()), &*document);
                NodeCast::from_unrooted(doctype)
            },
            DocumentFragmentNodeTypeId => {
                let doc_fragment = DocumentFragment::new(&*document);
                NodeCast::from_unrooted(doc_fragment)
            },
            CommentNodeTypeId => {
                let comment: &JSRef<Comment> = CommentCast::to_ref(node).unwrap();
                let comment = comment.deref();
                let comment = Comment::new(comment.characterdata.data.clone(), &*document);
                NodeCast::from_unrooted(comment)
            },
            DocumentNodeTypeId => {
                let document: &JSRef<Document> = DocumentCast::to_ref(node).unwrap();
                let is_html_doc = match document.is_html_document {
                    true => HTMLDocument,
                    false => NonHTMLDocument
                };
                let window = document.window.root();
                let document = Document::new(&*window, Some(document.url().clone()),
                                             is_html_doc, None);
                NodeCast::from_unrooted(document)
            },
            ElementNodeTypeId(..) => {
                let element: &JSRef<Element> = ElementCast::to_ref(node).unwrap();
                let element = element.deref();
                let element = build_element_from_tag(element.local_name.clone(), &*document);
                NodeCast::from_unrooted(element)
            },
            TextNodeTypeId => {
                let text: &JSRef<Text> = TextCast::to_ref(node).unwrap();
                let text = text.deref();
                let text = Text::new(text.characterdata.data.clone(), &*document);
                NodeCast::from_unrooted(text)
            },
            ProcessingInstructionNodeTypeId => {
                let pi: &JSRef<ProcessingInstruction> = ProcessingInstructionCast::to_ref(node).unwrap();
                let pi = pi.deref();
                let pi = ProcessingInstruction::new(pi.target.clone(),
                                                    pi.characterdata.data.clone(), &*document);
                NodeCast::from_unrooted(pi)
            },
        }.root();

        // Step 3.
        let document = if copy.is_document() {
            let doc: &JSRef<Document> = DocumentCast::to_ref(&*copy).unwrap();
            doc.unrooted().root()
        } else {
            document.unrooted().root()
        };
        assert!(&*copy.owner_doc().root() == &*document);

        // Step 4 (some data already copied in step 2).
        match node.type_id() {
            DocumentNodeTypeId => {
                let node_doc: &JSRef<Document> = DocumentCast::to_ref(node).unwrap();
                let copy_doc: &mut JSRef<Document> = DocumentCast::to_mut_ref(&mut *copy).unwrap();
                copy_doc.set_encoding_name(node_doc.encoding_name.clone());
                copy_doc.set_quirks_mode(node_doc.quirks_mode());
            },
            ElementNodeTypeId(..) => {
                let node_elem: &JSRef<Element> = ElementCast::to_ref(node).unwrap();
                let node_elem = node_elem.deref();
                let copy_elem: &mut JSRef<Element> = ElementCast::to_mut_ref(&mut *copy).unwrap();

                // XXX: to avoid double borrowing compile error. we might be able to fix this after #1854
                let copy_elem_alias = copy_elem.clone();

                let copy_elem = copy_elem.deref_mut();
                // FIXME: https://github.com/mozilla/servo/issues/1737
                copy_elem.namespace = node_elem.namespace.clone();
                let window = document.deref().window.root();
                for attr in node_elem.attrs.iter().map(|attr| attr.root()) {
                    copy_elem.attrs.push_unrooted(
                        &Attr::new(&*window,
                                   attr.deref().local_name.clone(), attr.deref().value.clone(),
                                   attr.deref().name.clone(), attr.deref().namespace.clone(),
                                   attr.deref().prefix.clone(), &copy_elem_alias));
                }
            },
            _ => ()
        }

        // Step 5: cloning steps.

        // Step 6.
        if clone_children == CloneChildren {
            for ref child in node.children() {
                let mut child_copy = Node::clone(&*child, Some(&*document), clone_children).root();
                let _inserted_node = Node::pre_insert(&mut *child_copy, &mut *copy, None);
            }
        }

        // Step 7.
        Temporary::from_rooted(&*copy)
    }

    /// Sends layout data, if any, back to the script task to be destroyed.
    unsafe fn reap_layout_data(&mut self) {
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
}

pub trait NodeMethods {
    fn NodeType(&self) -> u16;
    fn NodeName(&self) -> DOMString;
    fn GetBaseURI(&self) -> Option<DOMString>;
    fn GetOwnerDocument(&self) -> Option<Temporary<Document>>;
    fn GetParentNode(&self) -> Option<Temporary<Node>>;
    fn GetParentElement(&self) -> Option<Temporary<Element>>;
    fn HasChildNodes(&self) -> bool;
    fn ChildNodes(&mut self) -> Temporary<NodeList>;
    fn GetFirstChild(&self) -> Option<Temporary<Node>>;
    fn GetLastChild(&self) -> Option<Temporary<Node>>;
    fn GetPreviousSibling(&self) -> Option<Temporary<Node>>;
    fn GetNextSibling(&self) -> Option<Temporary<Node>>;
    fn GetNodeValue(&self) -> Option<DOMString>;
    fn SetNodeValue(&mut self, val: Option<DOMString>) -> ErrorResult;
    fn GetTextContent(&self) -> Option<DOMString>;
    fn SetTextContent(&mut self, value: Option<DOMString>) -> ErrorResult;
    fn InsertBefore(&mut self, node: &mut JSRef<Node>, child: Option<JSRef<Node>>) -> Fallible<Temporary<Node>>;
    fn AppendChild(&mut self, node: &mut JSRef<Node>) -> Fallible<Temporary<Node>>;
    fn ReplaceChild(&mut self, node: &mut JSRef<Node>, child: &mut JSRef<Node>) -> Fallible<Temporary<Node>>;
    fn RemoveChild(&mut self, node: &mut JSRef<Node>) -> Fallible<Temporary<Node>>;
    fn Normalize(&mut self);
    fn CloneNode(&self, deep: bool) -> Temporary<Node>;
    fn IsEqualNode(&self, maybe_node: Option<JSRef<Node>>) -> bool;
    fn CompareDocumentPosition(&self, other: &JSRef<Node>) -> u16;
    fn Contains(&self, maybe_other: Option<JSRef<Node>>) -> bool;
    fn LookupPrefix(&self, _prefix: Option<DOMString>) -> Option<DOMString>;
    fn LookupNamespaceURI(&self, _namespace: Option<DOMString>) -> Option<DOMString>;
    fn IsDefaultNamespace(&self, _namespace: Option<DOMString>) -> bool;
}

impl<'a> NodeMethods for JSRef<'a, Node> {
    // http://dom.spec.whatwg.org/#dom-node-nodetype
    fn NodeType(&self) -> u16 {
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
    fn NodeName(&self) -> DOMString {
        match self.type_id {
            ElementNodeTypeId(..) => {
                let elem: &JSRef<Element> = ElementCast::to_ref(self).unwrap();
                elem.TagName()
            }
            TextNodeTypeId => "#text".to_owned(),
            ProcessingInstructionNodeTypeId => {
                let processing_instruction: &JSRef<ProcessingInstruction> =
                    ProcessingInstructionCast::to_ref(self).unwrap();
                processing_instruction.Target()
            }
            CommentNodeTypeId => "#comment".to_owned(),
            DoctypeNodeTypeId => {
                let doctype: &JSRef<DocumentType> = DocumentTypeCast::to_ref(self).unwrap();
                doctype.deref().name.clone()
            },
            DocumentFragmentNodeTypeId => "#document-fragment".to_owned(),
            DocumentNodeTypeId => "#document".to_owned()
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-baseuri
    fn GetBaseURI(&self) -> Option<DOMString> {
        // FIXME (#1824) implement.
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-ownerdocument
    fn GetOwnerDocument(&self) -> Option<Temporary<Document>> {
        match self.type_id {
            ElementNodeTypeId(..) |
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId |
            DoctypeNodeTypeId |
            DocumentFragmentNodeTypeId => Some(self.owner_doc()),
            DocumentNodeTypeId => None
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-parentnode
    fn GetParentNode(&self) -> Option<Temporary<Node>> {
        self.parent_node.clone().map(|node| Temporary::new(node))
    }

    // http://dom.spec.whatwg.org/#dom-node-parentelement
    fn GetParentElement(&self) -> Option<Temporary<Element>> {
        self.parent_node.clone()
                        .and_then(|parent| {
                            let parent = parent.root();
                            ElementCast::to_ref(&*parent).map(|elem| {
                                Temporary::from_rooted(elem)
                            })
                        })
    }

    // http://dom.spec.whatwg.org/#dom-node-haschildnodes
    fn HasChildNodes(&self) -> bool {
        self.first_child.is_some()
    }

    // http://dom.spec.whatwg.org/#dom-node-childnodes
    fn ChildNodes(&mut self) -> Temporary<NodeList> {
        match self.child_list {
            None => (),
            Some(ref list) => return Temporary::new(list.clone()),
        }

        let doc = self.owner_doc().root();
        let window = doc.deref().window.root();
        let child_list = NodeList::new_child_list(&*window, self);
        self.child_list.assign(Some(child_list));
        Temporary::new(self.child_list.get_ref().clone())
    }

    // http://dom.spec.whatwg.org/#dom-node-firstchild
    fn GetFirstChild(&self) -> Option<Temporary<Node>> {
        self.first_child.clone().map(|node| Temporary::new(node))
    }

    // http://dom.spec.whatwg.org/#dom-node-lastchild
    fn GetLastChild(&self) -> Option<Temporary<Node>> {
        self.last_child.clone().map(|node| Temporary::new(node))
    }

    // http://dom.spec.whatwg.org/#dom-node-previoussibling
    fn GetPreviousSibling(&self) -> Option<Temporary<Node>> {
        self.prev_sibling.clone().map(|node| Temporary::new(node))
    }

    // http://dom.spec.whatwg.org/#dom-node-nextsibling
    fn GetNextSibling(&self) -> Option<Temporary<Node>> {
        self.next_sibling.clone().map(|node| Temporary::new(node))
    }

    // http://dom.spec.whatwg.org/#dom-node-nodevalue
    fn GetNodeValue(&self) -> Option<DOMString> {
        match self.type_id {
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                let chardata: &JSRef<CharacterData> = CharacterDataCast::to_ref(self).unwrap();
                Some(chardata.Data())
            }
            _ => {
                None
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-nodevalue
    fn SetNodeValue(&mut self, val: Option<DOMString>)
                        -> ErrorResult {
        match self.type_id {
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                self.SetTextContent(val)
            }
            _ => Ok(())
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-textcontent
    fn GetTextContent(&self) -> Option<DOMString> {
        match self.type_id {
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => {
                let mut content = "".to_owned();
                for node in self.traverse_preorder() {
                    if node.is_text() {
                        let text: &JSRef<Text> = TextCast::to_ref(&node).unwrap();
                        content.push_str(text.deref().characterdata.data.as_slice());
                    }
                }
                Some(content)
            }
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                let characterdata: &JSRef<CharacterData> = CharacterDataCast::to_ref(self).unwrap();
                Some(characterdata.Data())
            }
            DoctypeNodeTypeId |
            DocumentNodeTypeId => {
                None
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-textcontent
    fn SetTextContent(&mut self, value: Option<DOMString>)
                          -> ErrorResult {
        let value = null_str_as_empty(&value);
        match self.type_id {
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => {
                // Step 1-2.
                let node = if value.len() == 0 {
                    None
                } else {
                    let document = self.owner_doc().root();
                    Some(NodeCast::from_unrooted(document.deref().CreateTextNode(value)))
                }.root();

                // Step 3.
                Node::replace_all(node.root_ref(), self);
            }
            CommentNodeTypeId |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId => {
                self.wait_until_safe_to_modify_dom();

                {
                    let characterdata: &mut JSRef<CharacterData> = CharacterDataCast::to_mut_ref(self).unwrap();
                    characterdata.deref_mut().data = value.clone();
                }

                // Notify the document that the content of this node is different
                let document = self.owner_doc().root();
                document.deref().content_changed();
            }
            DoctypeNodeTypeId |
            DocumentNodeTypeId => {}
        }
        Ok(())
    }

    // http://dom.spec.whatwg.org/#dom-node-insertbefore
    fn InsertBefore(&mut self, node: &mut JSRef<Node>, child: Option<JSRef<Node>>) -> Fallible<Temporary<Node>> {
        Node::pre_insert(node, self, child)
    }

    // http://dom.spec.whatwg.org/#dom-node-appendchild
    fn AppendChild(&mut self, node: &mut JSRef<Node>) -> Fallible<Temporary<Node>> {
        Node::pre_insert(node, self, None)
    }

    // http://dom.spec.whatwg.org/#concept-node-replace
    fn ReplaceChild(&mut self, node: &mut JSRef<Node>, child: &mut JSRef<Node>) -> Fallible<Temporary<Node>> {

        // Step 1.
        match self.type_id() {
            DocumentNodeTypeId |
            DocumentFragmentNodeTypeId |
            ElementNodeTypeId(..) => (),
            _ => return Err(HierarchyRequest)
        }

        // Step 2.
        if node.is_inclusive_ancestor_of(self) {
            return Err(HierarchyRequest);
        }

        // Step 3.
        if !self.is_parent_of(child) {
            return Err(NotFound);
        }

        // Step 4-5.
        match node.type_id() {
            TextNodeTypeId if self.is_document() => return Err(HierarchyRequest),
            DoctypeNodeTypeId if !self.is_document() => return Err(HierarchyRequest),
            DocumentFragmentNodeTypeId |
            DoctypeNodeTypeId |
            ElementNodeTypeId(..) |
            TextNodeTypeId |
            ProcessingInstructionNodeTypeId |
            CommentNodeTypeId => (),
            DocumentNodeTypeId => return Err(HierarchyRequest)
        }

        // Step 6.
        match self.type_id() {
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
                                if self.child_elements().any(|c| NodeCast::from_ref(&c) != child) {
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
                        if self.child_elements().any(|c| NodeCast::from_ref(&c) != child) {
                            return Err(HierarchyRequest);
                        }
                        if child.following_siblings()
                                .any(|child| child.is_doctype()) {
                            return Err(HierarchyRequest);
                        }
                    },
                    // Step 6.3
                    DoctypeNodeTypeId => {
                        if self.children().any(|c| c.is_doctype() && &c != child) {
                            return Err(HierarchyRequest);
                        }
                        if self.children()
                            .take_while(|c| c != child)
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
        if *node == *child {
            return Ok(Temporary::from_rooted(child));
        }

        // Step 7-8.
        let next_sibling = child.next_sibling().map(|node| (*node.root()).clone());
        let reference_child = match next_sibling {
            Some(ref sibling) if sibling == node => node.next_sibling().map(|node| (*node.root()).clone()),
            _ => next_sibling
        };

        // Step 9.
        let document = document_from_node(self).root();
        Node::adopt(node, &*document);

        {
            // Step 10.
            Node::remove(child, self, Suppressed);

            // Step 11.
            Node::insert(node, self, reference_child, Suppressed);
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
        Ok(Temporary::from_rooted(child))
    }

    // http://dom.spec.whatwg.org/#dom-node-removechild
    fn RemoveChild(&mut self, node: &mut JSRef<Node>)
                       -> Fallible<Temporary<Node>> {
        Node::pre_remove(node, self)
    }

    // http://dom.spec.whatwg.org/#dom-node-normalize
    fn Normalize(&mut self) {
        let mut prev_text = None;
        for mut child in self.children() {
            if child.is_text() {
                let mut child_alias = child.clone();
                let characterdata: &JSRef<CharacterData> = CharacterDataCast::to_ref(&child).unwrap();
                if characterdata.Length() == 0 {
                    self.remove_child(&mut child_alias);
                } else {
                    match prev_text {
                        Some(ref mut text_node) => {
                            let prev_characterdata: &mut JSRef<CharacterData> = CharacterDataCast::to_mut_ref(text_node).unwrap();
                            let _ = prev_characterdata.AppendData(characterdata.Data());
                            self.remove_child(&mut child_alias);
                        },
                        None => prev_text = Some(child_alias)
                    }
                }
            } else {
                child.Normalize();
                prev_text = None;
            }

        }
    }

    // http://dom.spec.whatwg.org/#dom-node-clonenode
    fn CloneNode(&self, deep: bool) -> Temporary<Node> {
        match deep {
            true => Node::clone(self, None, CloneChildren),
            false => Node::clone(self, None, DoNotCloneChildren)
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-isequalnode
    fn IsEqualNode(&self, maybe_node: Option<JSRef<Node>>) -> bool {
        fn is_equal_doctype(node: &JSRef<Node>, other: &JSRef<Node>) -> bool {
            let doctype: &JSRef<DocumentType> = DocumentTypeCast::to_ref(node).unwrap();
            let other_doctype: &JSRef<DocumentType> = DocumentTypeCast::to_ref(other).unwrap();
            (doctype.deref().name == other_doctype.deref().name) &&
            (doctype.deref().public_id == other_doctype.deref().public_id) &&
            (doctype.deref().system_id == other_doctype.deref().system_id)
        }
        fn is_equal_element(node: &JSRef<Node>, other: &JSRef<Node>) -> bool {
            let element: &JSRef<Element> = ElementCast::to_ref(node).unwrap();
            let other_element: &JSRef<Element> = ElementCast::to_ref(other).unwrap();
            // FIXME: namespace prefix
            (element.deref().namespace == other_element.deref().namespace) &&
            (element.deref().local_name == other_element.deref().local_name) &&
            (element.deref().attrs.len() == other_element.deref().attrs.len())
        }
        fn is_equal_processinginstruction(node: &JSRef<Node>, other: &JSRef<Node>) -> bool {
            let pi: &JSRef<ProcessingInstruction> = ProcessingInstructionCast::to_ref(node).unwrap();
            let other_pi: &JSRef<ProcessingInstruction> = ProcessingInstructionCast::to_ref(other).unwrap();
            (pi.deref().target == other_pi.deref().target) &&
            (pi.deref().characterdata.data == other_pi.deref().characterdata.data)
        }
        fn is_equal_characterdata(node: &JSRef<Node>, other: &JSRef<Node>) -> bool {
            let characterdata: &JSRef<CharacterData> = CharacterDataCast::to_ref(node).unwrap();
            let other_characterdata: &JSRef<CharacterData> = CharacterDataCast::to_ref(other).unwrap();
            characterdata.deref().data == other_characterdata.deref().data
        }
        fn is_equal_element_attrs(node: &JSRef<Node>, other: &JSRef<Node>) -> bool {
            let element: &JSRef<Element> = ElementCast::to_ref(node).unwrap();
            let other_element: &JSRef<Element> = ElementCast::to_ref(other).unwrap();
            assert!(element.deref().attrs.len() == other_element.deref().attrs.len());
            element.deref().attrs.iter().map(|attr| attr.root()).all(|attr| {
                other_element.deref().attrs.iter().map(|attr| attr.root()).any(|other_attr| {
                    (attr.namespace == other_attr.namespace) &&
                    (attr.local_name == other_attr.local_name) &&
                    (attr.value == other_attr.value)
                })
            })
        }
        fn is_equal_node(this: &JSRef<Node>, node: &JSRef<Node>) -> bool {
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
            this.children().zip(node.children()).all(|(ref child, ref other_child)| {
                is_equal_node(child, other_child)
            })
        }
        match maybe_node {
            // Step 1.
            None => false,
            // Step 2-6.
            Some(ref node) => is_equal_node(self, node)
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-comparedocumentposition
    fn CompareDocumentPosition(&self, other: &JSRef<Node>) -> u16 {
        if self == other {
            // step 2.
            0
        } else {
            let mut lastself = self.clone();
            let mut lastother = other.clone();
            for ancestor in self.ancestors() {
                if &ancestor == other {
                    // step 4.
                    return NodeConstants::DOCUMENT_POSITION_CONTAINS +
                           NodeConstants::DOCUMENT_POSITION_PRECEDING;
                }
                lastself = ancestor.clone();
            }
            for ancestor in other.ancestors() {
                if &ancestor == self {
                    // step 5.
                    return NodeConstants::DOCUMENT_POSITION_CONTAINED_BY +
                           NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                }
                lastother = ancestor.clone();
            }

            if lastself != lastother {
                let abstract_uint: uintptr_t = as_uintptr(&*self);
                let other_uint: uintptr_t = as_uintptr(&*other);

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
                if &child == other {
                    // step 6.
                    return NodeConstants::DOCUMENT_POSITION_PRECEDING;
                }
                if &child == self {
                    // step 7.
                    return NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                }
            }
            unreachable!()
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-contains
    fn Contains(&self, maybe_other: Option<JSRef<Node>>) -> bool {
        match maybe_other {
            None => false,
            Some(ref other) => self.is_inclusive_ancestor_of(other)
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-lookupprefix
    fn LookupPrefix(&self, _prefix: Option<DOMString>) -> Option<DOMString> {
        // FIXME (#1826) implement.
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-lookupnamespaceuri
    fn LookupNamespaceURI(&self, _namespace: Option<DOMString>) -> Option<DOMString> {
        // FIXME (#1826) implement.
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-isdefaultnamespace
    fn IsDefaultNamespace(&self, _namespace: Option<DOMString>) -> bool {
        // FIXME (#1826) implement.
        false
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

pub fn document_from_node<T: NodeBase>(derived: &JSRef<T>) -> Temporary<Document> {
    let node: &JSRef<Node> = NodeCast::from_ref(derived);
    node.owner_doc()
}

pub fn window_from_node<T: NodeBase>(derived: &JSRef<T>) -> Temporary<Window> {
    let document = document_from_node(derived).root();
    Temporary::new(document.deref().window.clone())
}

impl<'a> VirtualMethods for JSRef<'a, Node> {
    fn super_type<'a>(&'a mut self) -> Option<&'a mut VirtualMethods:> {
        let eventtarget: &mut JSRef<EventTarget> = EventTargetCast::from_mut_ref(self);
        Some(eventtarget as &mut VirtualMethods:)
    }
}
