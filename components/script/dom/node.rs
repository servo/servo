/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use dom::attr::{Attr, AttrHelpers};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding::NamedNodeMapMethods;
use dom::bindings::codegen::Bindings::NodeBinding::{NodeConstants, NodeMethods};
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::codegen::Bindings::ProcessingInstructionBinding::ProcessingInstructionMethods;
use dom::bindings::codegen::InheritTypes::{CommentCast, DocumentCast, DocumentTypeCast};
use dom::bindings::codegen::InheritTypes::{ElementCast, TextCast, NodeCast, ElementDerived};
use dom::bindings::codegen::InheritTypes::{CharacterDataCast, NodeBase, NodeDerived};
use dom::bindings::codegen::InheritTypes::{ProcessingInstructionCast, EventTargetCast};
use dom::bindings::codegen::InheritTypes::{HTMLLegendElementDerived, HTMLFieldSetElementDerived};
use dom::bindings::codegen::InheritTypes::HTMLOptGroupElementDerived;
use dom::bindings::conversions;
use dom::bindings::error::Fallible;
use dom::bindings::error::Error::{NotFound, HierarchyRequest, Syntax};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, JSRef, LayoutJS, RootedReference, Temporary, Root, Unrooted};
use dom::bindings::js::{TemporaryPushable, OptionalRootedRootable};
use dom::bindings::js::{ResultRootable, OptionalRootable, MutNullableJS};
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{Reflectable, reflect_dom_object};
use dom::characterdata::CharacterData;
use dom::comment::Comment;
use dom::document::{Document, DocumentHelpers, IsHTMLDocument, DocumentSource};
use dom::documentfragment::DocumentFragment;
use dom::documenttype::DocumentType;
use dom::element::{AttributeHandlers, Element, ElementCreator, ElementTypeId};
use dom::element::ElementHelpers;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::HTMLElementTypeId;
use dom::nodelist::NodeList;
use dom::processinginstruction::ProcessingInstruction;
use dom::text::Text;
use dom::virtualmethods::{VirtualMethods, vtable_for};
use dom::window::Window;
use geom::rect::Rect;
use layout_interface::{LayoutChan, Msg};
use devtools_traits::NodeInfo;
use script_traits::UntrustedNodeAddress;
use util::geometry::Au;
use util::str::{DOMString, null_str_as_empty};
use style::selectors::{Selector, AttrSelector, NamespaceConstraint};
use style::selectors::parse_author_origin_selector_list_from_str;
use style::selector_matching::matches;
use style::properties::ComputedValues;
use style;

use js::jsapi::{JSContext, JSObject, JSTracer, JSRuntime};
use js::jsfriendapi;
use core::nonzero::NonZero;
use libc;
use libc::{uintptr_t, c_void};
use std::borrow::ToOwned;
use std::cell::{Cell, RefCell, Ref, RefMut};
use std::default::Default;
use std::iter::{FilterMap, Peekable};
use std::mem;
use std::sync::Arc;
use uuid;
use string_cache::QualName;

//
// The basic Node structure
//

/// An HTML node.
#[dom_struct]
pub struct Node {
    /// The JavaScript reflector for this node.
    eventtarget: EventTarget,

    /// The type of node that this is.
    type_id: NodeTypeId,

    /// The parent of this node.
    parent_node: MutNullableJS<Node>,

    /// The first child of this node.
    first_child: MutNullableJS<Node>,

    /// The last child of this node.
    last_child: MutNullableJS<Node>,

    /// The next sibling of this node.
    next_sibling: MutNullableJS<Node>,

    /// The previous sibling of this node.
    prev_sibling: MutNullableJS<Node>,

    /// The document that this node belongs to.
    owner_doc: MutNullableJS<Document>,

    /// The live list of children return by .childNodes.
    child_list: MutNullableJS<NodeList>,

    /// A bitfield of flags for node items.
    flags: Cell<NodeFlags>,

    /// Layout information. Only the layout task may touch this data.
    ///
    /// Must be sent back to the layout task to be destroyed when this
    /// node is finalized.
    layout_data: LayoutDataRef,

    unique_id: DOMRefCell<String>,
}

impl NodeDerived for EventTarget {
    fn is_node(&self) -> bool {
        match *self.type_id() {
            EventTargetTypeId::Node(_) => true,
            _ => false
        }
    }
}

bitflags! {
    #[doc = "Flags for node items."]
    #[jstraceable]
    flags NodeFlags: u16 {
        #[doc = "Specifies whether this node is in a document."]
        const IS_IN_DOC = 0x01,
        #[doc = "Specifies whether this node is in hover state."]
        const IN_HOVER_STATE = 0x02,
        #[doc = "Specifies whether this node is in disabled state."]
        const IN_DISABLED_STATE = 0x04,
        #[doc = "Specifies whether this node is in enabled state."]
        const IN_ENABLED_STATE = 0x08,
        #[doc = "Specifies whether this node _must_ be reflowed regardless of style differences."]
        const HAS_CHANGED = 0x10,
        #[doc = "Specifies whether this node needs style recalc on next reflow."]
        const IS_DIRTY = 0x20,
        #[doc = "Specifies whether this node has siblings (inclusive of itself) which \
                  changed since the last reflow."]
        const HAS_DIRTY_SIBLINGS = 0x40,
        #[doc = "Specifies whether this node has descendants (inclusive of itself) which \
                 have changed since the last reflow."]
        const HAS_DIRTY_DESCENDANTS = 0x80,
        // TODO: find a better place to keep this (#4105)
        // https://critic.hoppipolla.co.uk/showcomment?chain=8873
        // Perhaps using a Set in Document?
        #[doc = "Specifies whether or not there is an authentic click in progress on \
                 this element."]
        const CLICK_IN_PROGRESS = 0x100,
    }
}

impl NodeFlags {
    pub fn new(type_id: NodeTypeId) -> NodeFlags {
        let dirty = HAS_CHANGED | IS_DIRTY | HAS_DIRTY_SIBLINGS | HAS_DIRTY_DESCENDANTS;
        match type_id {
            NodeTypeId::Document => IS_IN_DOC | dirty,
            // The following elements are enabled by default.
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptGroupElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement)) |
            //NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMenuItemElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFieldSetElement)) => IN_ENABLED_STATE | dirty,
            _ => dirty,
        }
    }
}

#[unsafe_destructor]
impl Drop for Node {
    #[allow(unsafe_blocks)]
    fn drop(&mut self) {
        unsafe {
            self.reap_layout_data();
        }
    }
}

/// suppress observers flag
/// http://dom.spec.whatwg.org/#concept-node-insert
/// http://dom.spec.whatwg.org/#concept-node-remove
#[derive(Copy)]
enum SuppressObserver {
    Suppressed,
    Unsuppressed
}

/// Layout data that is shared between the script and layout tasks.
pub struct SharedLayoutData {
    /// The results of CSS styling for this node.
    pub style: Option<Arc<ComputedValues>>,
}

/// Encapsulates the abstract layout data.
pub struct LayoutData {
    chan: Option<LayoutChan>,
    _shared_data: SharedLayoutData,
    _data: NonZero<*const ()>,
}

pub struct LayoutDataRef {
    pub data_cell: RefCell<Option<LayoutData>>,
}

no_jsmanaged_fields!(LayoutDataRef);

impl LayoutDataRef {
    pub fn new() -> LayoutDataRef {
        LayoutDataRef {
            data_cell: RefCell::new(None),
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
        match &mut *layout_data {
            &mut None => None,
            &mut Some(ref mut layout_data) => Some(layout_data.chan.take().unwrap()),
        }
    }

    /// Borrows the layout data immutably, *asserting that there are no mutators*. Bad things will
    /// happen if you try to mutate the layout data while this is held. This is the only thread-
    /// safe layout data accessor.
    #[inline]
    pub unsafe fn borrow_unchecked(&self) -> *const Option<LayoutData> {
        mem::transmute(&self.data_cell)
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

unsafe impl Send for LayoutDataRef {}

/// The different types of nodes.
#[derive(Copy, PartialEq, Debug)]
#[jstraceable]
pub enum NodeTypeId {
    DocumentType,
    DocumentFragment,
    Comment,
    Document,
    Element(ElementTypeId),
    Text,
    ProcessingInstruction,
}

trait PrivateNodeHelpers {
    fn node_inserted(self);
    fn node_removed(self, parent_in_doc: bool);
    fn add_child(self, new_child: JSRef<Node>, before: Option<JSRef<Node>>);
    fn remove_child(self, child: JSRef<Node>);
}

impl<'a> PrivateNodeHelpers for JSRef<'a, Node> {
    // http://dom.spec.whatwg.org/#node-is-inserted
    fn node_inserted(self) {
        assert!(self.parent_node().is_some());
        let document = document_from_node(self).root();
        let is_in_doc = self.is_in_doc();

        for node in self.traverse_preorder() {
            vtable_for(&node).bind_to_tree(is_in_doc);
        }

        let parent = self.parent_node().root();
        parent.map(|parent| vtable_for(&parent.r()).child_inserted(self));
        document.r().content_and_heritage_changed(self, NodeDamage::OtherNodeDamage);
    }

    // http://dom.spec.whatwg.org/#node-is-removed
    fn node_removed(self, parent_in_doc: bool) {
        assert!(self.parent_node().is_none());
        for node in self.traverse_preorder() {
            vtable_for(&node).unbind_from_tree(parent_in_doc);
        }
    }

    //
    // Pointer stitching
    //

    /// Adds a new child to the end of this node's list of children.
    ///
    /// Fails unless `new_child` is disconnected from the tree.
    fn add_child(self, new_child: JSRef<Node>, before: Option<JSRef<Node>>) {
        assert!(new_child.parent_node().is_none());
        assert!(new_child.prev_sibling().is_none());
        assert!(new_child.next_sibling().is_none());
        match before {
            Some(ref before) => {
                assert!(before.parent_node().root().r() == Some(self));
                match before.prev_sibling().root() {
                    None => {
                        assert!(Some(*before) == self.first_child().root().r());
                        self.first_child.assign(Some(new_child));
                    },
                    Some(prev_sibling) => {
                        prev_sibling.r().next_sibling.assign(Some(new_child));
                        new_child.prev_sibling.assign(Some(prev_sibling.r()));
                    },
                }
                before.prev_sibling.assign(Some(new_child));
                new_child.next_sibling.assign(Some(*before));
            },
            None => {
                match self.last_child().root() {
                    None => self.first_child.assign(Some(new_child)),
                    Some(last_child) => {
                        assert!(last_child.r().next_sibling().is_none());
                        last_child.r().next_sibling.assign(Some(new_child));
                        new_child.prev_sibling.assign(Some(last_child.r()));
                    }
                }

                self.last_child.assign(Some(new_child));
            },
        }

        new_child.parent_node.assign(Some(self));
    }

    /// Removes the given child from this node's list of children.
    ///
    /// Fails unless `child` is a child of this node.
    fn remove_child(self, child: JSRef<Node>) {
        assert!(child.parent_node().root().r() == Some(self));

        match child.prev_sibling.get().root() {
            None => {
                self.first_child.assign(child.next_sibling.get());
            }
            Some(prev_sibling) => {
                prev_sibling.r().next_sibling.assign(child.next_sibling.get());
            }
        }

        match child.next_sibling.get().root() {
            None => {
                self.last_child.assign(child.prev_sibling.get());
            }
            Some(next_sibling) => {
                next_sibling.r().prev_sibling.assign(child.prev_sibling.get());
            }
        }

        child.prev_sibling.clear();
        child.next_sibling.clear();
        child.parent_node.clear();
    }
}

pub struct QuerySelectorIterator<'a> {
    selectors: Vec<Selector>,
    iterator: TreeIterator<'a>,
}

impl<'a> QuerySelectorIterator<'a> {
    unsafe fn new(iter: TreeIterator<'a>, selectors: Vec<Selector>) -> QuerySelectorIterator<'a> {
        QuerySelectorIterator {
            selectors: selectors,
            iterator: iter,
        }
    }
}

impl<'a> Iterator for QuerySelectorIterator<'a> {
    type Item = JSRef<'a, Node>;

    fn next(&mut self) -> Option<JSRef<'a, Node>> {
        let selectors = &self.selectors;
        // TODO(cgaebel): Is it worth it to build a bloom filter here
        // (instead of passing `None`)? Probably.
        self.iterator.find(|node| node.is_element() && matches(selectors, node, &mut None))
    }
}

pub trait NodeHelpers<'a> {
    fn ancestors(self) -> AncestorIterator<'a>;
    fn inclusive_ancestors(self) -> AncestorIterator<'a>;
    fn children(self) -> NodeChildrenIterator<'a>;
    fn rev_children(self) -> ReverseChildrenIterator;
    fn child_elements(self) -> ChildElementIterator<'a>;
    fn following_siblings(self) -> NodeChildrenIterator<'a>;
    fn is_in_doc(self) -> bool;
    fn is_inclusive_ancestor_of(self, parent: JSRef<'a, Node>) -> bool;    // FIXME: See #3960
    fn is_parent_of(self, child: JSRef<Node>) -> bool;

    fn type_id(self) -> NodeTypeId;

    fn parent_node(self) -> Option<Temporary<Node>>;
    fn first_child(self) -> Option<Temporary<Node>>;
    fn last_child(self) -> Option<Temporary<Node>>;
    fn prev_sibling(self) -> Option<Temporary<Node>>;
    fn next_sibling(self) -> Option<Temporary<Node>>;

    fn owner_doc(self) -> Temporary<Document>;
    fn set_owner_doc(self, document: JSRef<Document>);
    fn is_in_html_doc(self) -> bool;

    fn is_element(self) -> bool;
    fn is_document(self) -> bool;
    fn is_doctype(self) -> bool;
    fn is_text(self) -> bool;
    fn is_anchor_element(self) -> bool;

    fn get_flag(self, flag: NodeFlags) -> bool;
    fn set_flag(self, flag: NodeFlags, value: bool);

    fn get_hover_state(self) -> bool;
    fn set_hover_state(self, state: bool);

    fn get_disabled_state(self) -> bool;
    fn set_disabled_state(self, state: bool);

    fn get_enabled_state(self) -> bool;
    fn set_enabled_state(self, state: bool);

    fn get_has_changed(self) -> bool;
    fn set_has_changed(self, state: bool);

    fn get_is_dirty(self) -> bool;
    fn set_is_dirty(self, state: bool);

    fn get_has_dirty_siblings(self) -> bool;
    fn set_has_dirty_siblings(self, state: bool);

    fn get_has_dirty_descendants(self) -> bool;
    fn set_has_dirty_descendants(self, state: bool);

    /// Marks the given node as `IS_DIRTY`, its siblings as `HAS_DIRTY_SIBLINGS` (to deal with
    /// sibling selectors), its ancestors as `HAS_DIRTY_DESCENDANTS`, and its descendants as
    /// `IS_DIRTY`. If anything more than the node's style was damaged, this method also sets the
    /// `HAS_CHANGED` flag.
    fn dirty(self, damage: NodeDamage);

    /// Similar to `dirty`, but will always walk the ancestors to mark them dirty,
    /// too. This is useful when a node is reparented. The node will frequently
    /// already be marked as `changed` to skip double-dirties, but the ancestors
    /// still need to be marked as `HAS_DIRTY_DESCENDANTS`.
    ///
    /// See #4170
    fn force_dirty_ancestors(self, damage: NodeDamage);

    fn dirty_impl(self, damage: NodeDamage, force_ancestors: bool);

    fn dump(self);
    fn dump_indent(self, indent: uint);
    fn debug_str(self) -> String;

    fn traverse_preorder(self) -> TreeIterator<'a>;
    fn inclusively_following_siblings(self) -> NodeChildrenIterator<'a>;

    fn to_trusted_node_address(self) -> TrustedNodeAddress;

    fn get_bounding_content_box(self) -> Rect<Au>;
    fn get_content_boxes(self) -> Vec<Rect<Au>>;

    fn query_selector(self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>>;
    unsafe fn query_selector_iter(self, selectors: DOMString) -> Fallible<QuerySelectorIterator<'a>>;
    fn query_selector_all(self, selectors: DOMString) -> Fallible<Temporary<NodeList>>;

    fn remove_self(self);

    fn get_unique_id(self) -> String;
    fn summarize(self) -> NodeInfo;
}

impl<'a> NodeHelpers<'a> for JSRef<'a, Node> {
    /// Dumps the subtree rooted at this node, for debugging.
    fn dump(self) {
        self.dump_indent(0);
    }

    /// Dumps the node tree, for debugging, with indentation.
    fn dump_indent(self, indent: uint) {
        let mut s = String::new();
        for _ in range(0, indent) {
            s.push_str("    ");
        }

        s.push_str(self.debug_str().as_slice());
        debug!("{:?}", s);

        // FIXME: this should have a pure version?
        for kid in self.children() {
            kid.dump_indent(indent + 1u)
        }
    }

    /// Returns a string that describes this node.
    fn debug_str(self) -> String {
        format!("{:?}", self.type_id)
    }

    fn is_in_doc(self) -> bool {
        self.flags.get().contains(IS_IN_DOC)
    }

    /// Returns the type ID of this node. Fails if this node is borrowed mutably.
    fn type_id(self) -> NodeTypeId {
        self.type_id
    }

    fn parent_node(self) -> Option<Temporary<Node>> {
        self.parent_node.get()
    }

    fn first_child(self) -> Option<Temporary<Node>> {
        self.first_child.get()
    }

    fn last_child(self) -> Option<Temporary<Node>> {
        self.last_child.get()
    }

    /// Returns the previous sibling of this node. Fails if this node is borrowed mutably.
    fn prev_sibling(self) -> Option<Temporary<Node>> {
        self.prev_sibling.get()
    }

    /// Returns the next sibling of this node. Fails if this node is borrowed mutably.
    fn next_sibling(self) -> Option<Temporary<Node>> {
        self.next_sibling.get()
    }

    #[inline]
    fn is_element(self) -> bool {
        match self.type_id {
            NodeTypeId::Element(..) => true,
            _ => false
        }
    }

    #[inline]
    fn is_document(self) -> bool {
        self.type_id == NodeTypeId::Document
    }

    #[inline]
    fn is_anchor_element(self) -> bool {
        self.type_id == NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement))
    }

    #[inline]
    fn is_doctype(self) -> bool {
        self.type_id == NodeTypeId::DocumentType
    }

    #[inline]
    fn is_text(self) -> bool {
        self.type_id == NodeTypeId::Text
    }

    fn get_flag(self, flag: NodeFlags) -> bool {
        self.flags.get().contains(flag)
    }

    fn set_flag(self, flag: NodeFlags, value: bool) {
        let mut flags = self.flags.get();

        if value {
            flags.insert(flag);
        } else {
            flags.remove(flag);
        }

        self.flags.set(flags);
    }

    fn get_hover_state(self) -> bool {
        self.get_flag(IN_HOVER_STATE)
    }

    fn set_hover_state(self, state: bool) {
        self.set_flag(IN_HOVER_STATE, state)
    }

    fn get_disabled_state(self) -> bool {
        self.get_flag(IN_DISABLED_STATE)
    }

    fn set_disabled_state(self, state: bool) {
        self.set_flag(IN_DISABLED_STATE, state)
    }

    fn get_enabled_state(self) -> bool {
        self.get_flag(IN_ENABLED_STATE)
    }

    fn set_enabled_state(self, state: bool) {
        self.set_flag(IN_ENABLED_STATE, state)
    }

    fn get_has_changed(self) -> bool {
        self.get_flag(HAS_CHANGED)
    }

    fn set_has_changed(self, state: bool) {
        self.set_flag(HAS_CHANGED, state)
    }

    fn get_is_dirty(self) -> bool {
        self.get_flag(IS_DIRTY)
    }

    fn set_is_dirty(self, state: bool) {
        self.set_flag(IS_DIRTY, state)
    }

    fn get_has_dirty_siblings(self) -> bool {
        self.get_flag(HAS_DIRTY_SIBLINGS)
    }

    fn set_has_dirty_siblings(self, state: bool) {
        self.set_flag(HAS_DIRTY_SIBLINGS, state)
    }

    fn get_has_dirty_descendants(self) -> bool {
        self.get_flag(HAS_DIRTY_DESCENDANTS)
    }

    fn set_has_dirty_descendants(self, state: bool) {
        self.set_flag(HAS_DIRTY_DESCENDANTS, state)
    }

    fn force_dirty_ancestors(self, damage: NodeDamage) {
        self.dirty_impl(damage, true)
    }

    fn dirty(self, damage: NodeDamage) {
        self.dirty_impl(damage, false)
    }

    fn dirty_impl(self, damage: NodeDamage, force_ancestors: bool) {
        // 1. Dirty self.
        match damage {
            NodeDamage::NodeStyleDamaged => {}
            NodeDamage::OtherNodeDamage => self.set_has_changed(true),
        }

        if self.get_is_dirty() && !force_ancestors {
            return
        }

        // 2. Dirty descendants.
        fn dirty_subtree(node: JSRef<Node>) {
            // Stop if this subtree is already dirty.
            if node.get_is_dirty() { return }

            node.set_flag(IS_DIRTY | HAS_DIRTY_SIBLINGS | HAS_DIRTY_DESCENDANTS, true);

            for kid in node.children() {
                dirty_subtree(kid);
            }
        }

        dirty_subtree(self);

        // 3. Dirty siblings.
        //
        // TODO(cgaebel): This is a very conservative way to account for sibling
        // selectors. Maybe we can do something smarter in the future.
        let parent =
            match self.parent_node() {
                None         => return,
                Some(parent) => parent,
            };

        for sibling in parent.root().r().children() {
            sibling.set_has_dirty_siblings(true);
        }

        // 4. Dirty ancestors.
        for ancestor in self.ancestors() {
            if !force_ancestors && ancestor.get_has_dirty_descendants() { break }
            ancestor.set_has_dirty_descendants(true);
        }
    }

    /// Iterates over this node and all its descendants, in preorder.
    fn traverse_preorder(self) -> TreeIterator<'a> {
        TreeIterator::new(self)
    }

    fn inclusively_following_siblings(self) -> NodeChildrenIterator<'a> {
        NodeChildrenIterator {
            current: Some(self.clone()),
        }
    }

    fn is_inclusive_ancestor_of(self, parent: JSRef<'a, Node>) -> bool {
        self == parent || parent.ancestors().any(|ancestor| ancestor == self)
    }

    fn following_siblings(self) -> NodeChildrenIterator<'a> {
        NodeChildrenIterator {
            current: self.next_sibling().root().map(|next| next.get_unsound_ref_forever()),
        }
    }

    fn is_parent_of(self, child: JSRef<Node>) -> bool {
        match child.parent_node() {
            Some(ref parent) if parent == &Temporary::from_rooted(self) => true,
            _ => false
        }
    }

    fn to_trusted_node_address(self) -> TrustedNodeAddress {
        TrustedNodeAddress(&*self as *const Node as *const libc::c_void)
    }

    fn get_bounding_content_box(self) -> Rect<Au> {
        window_from_node(self).root().r().page().content_box_query(self.to_trusted_node_address())
    }

    fn get_content_boxes(self) -> Vec<Rect<Au>> {
        window_from_node(self).root().r().page().content_boxes_query(self.to_trusted_node_address())
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn query_selector(self, selectors: DOMString) -> Fallible<Option<Temporary<Element>>> {
        // Step 1.
        match parse_author_origin_selector_list_from_str(selectors.as_slice()) {
            // Step 2.
            Err(()) => return Err(Syntax),
            // Step 3.
            Ok(ref selectors) => {
                let root = self.ancestors().last().unwrap_or(self.clone());
                Ok(root.traverse_preorder()
                       .filter_map(ElementCast::to_ref)
                       .find(|element| matches(selectors, &NodeCast::from_ref(*element), &mut None))
                       .map(Temporary::from_rooted))
            }
        }
    }

    /// Get an iterator over all nodes which match a set of selectors
    /// Be careful not to do anything which may manipulate the DOM tree whilst iterating, otherwise
    /// the iterator may be invalidated
    unsafe fn query_selector_iter(self, selectors: DOMString)
                                  -> Fallible<QuerySelectorIterator<'a>> {
        // Step 1.
        let nodes;
        let root = self.ancestors().last().unwrap_or(self.clone());
        match parse_author_origin_selector_list_from_str(selectors.as_slice()) {
            // Step 2.
            Err(()) => return Err(Syntax),
            // Step 3.
            Ok(selectors) => {
                nodes = QuerySelectorIterator::new(root.traverse_preorder(), selectors);
            }
        };
        Ok(nodes)
    }

    // http://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    #[allow(unsafe_blocks)]
    fn query_selector_all(self, selectors: DOMString) -> Fallible<Temporary<NodeList>> {
        // Step 1.
        unsafe {
            self.query_selector_iter(selectors).map(|iter| {
                let window = window_from_node(self).root();
                NodeList::new_simple_list(window.r(), iter.collect())
            })
        }
    }


    fn ancestors(self) -> AncestorIterator<'a> {
        AncestorIterator {
            current: self.parent_node.get().map(|node| node.root().get_unsound_ref_forever()),
        }
    }

    fn inclusive_ancestors(self) -> AncestorIterator<'a> {
        AncestorIterator {
            current: Some(self.clone())
        }
    }

    fn owner_doc(self) -> Temporary<Document> {
        self.owner_doc.get().unwrap()
    }

    fn set_owner_doc(self, document: JSRef<Document>) {
        self.owner_doc.assign(Some(document.clone()));
    }

    fn is_in_html_doc(self) -> bool {
        self.owner_doc().root().r().is_html_document()
    }

    fn children(self) -> NodeChildrenIterator<'a> {
        NodeChildrenIterator {
            current: self.first_child.get().map(|node| node.root().get_unsound_ref_forever()),
        }
    }

    fn rev_children(self) -> ReverseChildrenIterator {
        ReverseChildrenIterator {
            current: self.last_child.get().root(),
        }
    }

    fn child_elements(self) -> ChildElementIterator<'a> {
        fn cast(n: JSRef<Node>) -> Option<JSRef<Element>> {
            ElementCast::to_ref(n)
        }

        self.children()
            .filter_map(cast as fn(JSRef<Node>) -> Option<JSRef<Element>>)
            .peekable()
    }

    fn remove_self(self) {
        match self.parent_node().root() {
            Some(parent) => parent.r().remove_child(self),
            None => ()
        }
    }

    fn get_unique_id(self) -> String {
        self.unique_id.borrow().clone()
    }

    fn summarize(self) -> NodeInfo {
        if self.unique_id.borrow().is_empty() {
            let mut unique_id = self.unique_id.borrow_mut();
            *unique_id = uuid::Uuid::new_v4().to_simple_string();
        }

        NodeInfo {
            uniqueId: self.unique_id.borrow().clone(),
            baseURI: self.GetBaseURI().unwrap_or("".to_owned()),
            parent: self.GetParentNode().root().map(|node| node.r().get_unique_id()).unwrap_or("".to_owned()),
            nodeType: self.NodeType() as uint,
            namespaceURI: "".to_owned(), //FIXME
            nodeName: self.NodeName(),
            numChildren: self.ChildNodes().root().r().Length() as uint,

            //FIXME doctype nodes only
            name: "".to_owned(),
            publicId: "".to_owned(),
            systemId: "".to_owned(),

            attrs: {
                let e: Option<JSRef<Element>> = ElementCast::to_ref(self);
                match e {
                    Some(element) => element.summarize(),
                    None => vec!(),
                }
            },

            isDocumentElement:
                self.owner_doc().root()
                    .r()
                    .GetDocumentElement()
                    .map(|elem| NodeCast::from_ref(elem.root().r()) == self)
                    .unwrap_or(false),

            shortValue: self.GetNodeValue().unwrap_or("".to_owned()), //FIXME: truncate
            incompleteValue: false, //FIXME: reflect truncation
        }
    }

}

/// If the given untrusted node address represents a valid DOM node in the given runtime,
/// returns it.
#[allow(unsafe_blocks)]
pub fn from_untrusted_node_address(runtime: *mut JSRuntime, candidate: UntrustedNodeAddress)
    -> Temporary<Node> {
    unsafe {
        let candidate: uintptr_t = mem::transmute(candidate.0);
        let object: *mut JSObject = jsfriendapi::bindgen::JS_GetAddressableObject(runtime,
                                                                                  candidate);
        if object.is_null() {
            panic!("Attempted to create a `JS<Node>` from an invalid pointer!")
        }
        let boxed_node: *const Node = conversions::unwrap(object);
        Temporary::from_unrooted(Unrooted::from_raw(boxed_node))
    }
}

pub trait LayoutNodeHelpers {
    unsafe fn type_id_for_layout(&self) -> NodeTypeId;

    unsafe fn parent_node_ref(&self) -> Option<LayoutJS<Node>>;
    unsafe fn first_child_ref(&self) -> Option<LayoutJS<Node>>;
    unsafe fn last_child_ref(&self) -> Option<LayoutJS<Node>>;
    unsafe fn prev_sibling_ref(&self) -> Option<LayoutJS<Node>>;
    unsafe fn next_sibling_ref(&self) -> Option<LayoutJS<Node>>;

    unsafe fn owner_doc_for_layout(&self) -> LayoutJS<Document>;

    unsafe fn is_element_for_layout(&self) -> bool;
    unsafe fn get_flag(self, flag: NodeFlags) -> bool;
    unsafe fn set_flag(self, flag: NodeFlags, value: bool);
}

impl LayoutNodeHelpers for LayoutJS<Node> {
    #[inline]
    unsafe fn type_id_for_layout(&self) -> NodeTypeId {
        (*self.unsafe_get()).type_id
    }

    #[inline]
    unsafe fn is_element_for_layout(&self) -> bool {
        (*self.unsafe_get()).is_element()
    }

    #[inline]
    unsafe fn parent_node_ref(&self) -> Option<LayoutJS<Node>> {
        (*self.unsafe_get()).parent_node.get_inner_as_layout()
    }

    #[inline]
    unsafe fn first_child_ref(&self) -> Option<LayoutJS<Node>> {
        (*self.unsafe_get()).first_child.get_inner_as_layout()
    }

    #[inline]
    unsafe fn last_child_ref(&self) -> Option<LayoutJS<Node>> {
        (*self.unsafe_get()).last_child.get_inner_as_layout()
    }

    #[inline]
    unsafe fn prev_sibling_ref(&self) -> Option<LayoutJS<Node>> {
        (*self.unsafe_get()).prev_sibling.get_inner_as_layout()
    }

    #[inline]
    unsafe fn next_sibling_ref(&self) -> Option<LayoutJS<Node>> {
        (*self.unsafe_get()).next_sibling.get_inner_as_layout()
    }

    #[inline]
    unsafe fn owner_doc_for_layout(&self) -> LayoutJS<Document> {
        (*self.unsafe_get()).owner_doc.get_inner_as_layout().unwrap()
    }

    #[inline]
    unsafe fn get_flag(self, flag: NodeFlags) -> bool {
        (*self.unsafe_get()).flags.get().contains(flag)
    }

    #[inline]
    unsafe fn set_flag(self, flag: NodeFlags, value: bool) {
        let this = self.unsafe_get();
        let mut flags = (*this).flags.get();

        if value {
            flags.insert(flag);
        } else {
            flags.remove(flag);
        }

        (*this).flags.set(flags);
    }
}

pub trait RawLayoutNodeHelpers {
    unsafe fn get_hover_state_for_layout(&self) -> bool;
    unsafe fn get_disabled_state_for_layout(&self) -> bool;
    unsafe fn get_enabled_state_for_layout(&self) -> bool;
    fn type_id_for_layout(&self) -> NodeTypeId;
}

impl RawLayoutNodeHelpers for Node {
    #[inline]
    unsafe fn get_hover_state_for_layout(&self) -> bool {
        self.flags.get().contains(IN_HOVER_STATE)
    }
    #[inline]
    unsafe fn get_disabled_state_for_layout(&self) -> bool {
        self.flags.get().contains(IN_DISABLED_STATE)
    }
    #[inline]
    unsafe fn get_enabled_state_for_layout(&self) -> bool {
        self.flags.get().contains(IN_ENABLED_STATE)
    }
    #[inline]
    fn type_id_for_layout(&self) -> NodeTypeId {
        self.type_id
    }
}


//
// Iteration and traversal
//

pub type ChildElementIterator<'a> =
    Peekable<FilterMap<NodeChildrenIterator<'a>,
                       fn(JSRef<Node>) -> Option<JSRef<Element>>>>;

pub struct NodeChildrenIterator<'a> {
    current: Option<JSRef<'a, Node>>,
}

impl<'a> Iterator for NodeChildrenIterator<'a> {
    type Item = JSRef<'a, Node>;

    fn next(&mut self) -> Option<JSRef<'a, Node>> {
        let node = self.current;
        self.current = node.and_then(|node| node.next_sibling().map(|node| node.root().get_unsound_ref_forever()));
        node
    }
}

pub struct ReverseChildrenIterator {
    current: Option<Root<Node>>,
}

impl Iterator for ReverseChildrenIterator {
    type Item = Temporary<Node>;

    fn next(&mut self) -> Option<Temporary<Node>> {
        let node = self.current.r().map(Temporary::from_rooted);
        self.current = self.current.take().and_then(|node| node.r().prev_sibling()).root();
        node
    }
}

pub struct AncestorIterator<'a> {
    current: Option<JSRef<'a, Node>>,
}

impl<'a> Iterator for AncestorIterator<'a> {
    type Item = JSRef<'a, Node>;

    fn next(&mut self) -> Option<JSRef<'a, Node>> {
        let node = self.current;
        self.current = node.and_then(|node| node.parent_node().map(|node| node.root().get_unsound_ref_forever()));
        node
    }
}

pub struct TreeIterator<'a> {
    stack: Vec<JSRef<'a, Node>>,
}

impl<'a> TreeIterator<'a> {
    fn new(root: JSRef<'a, Node>) -> TreeIterator<'a> {
        let mut stack = vec!();
        stack.push(root);

        TreeIterator {
            stack: stack,
        }
    }
}

impl<'a> Iterator for TreeIterator<'a> {
    type Item = JSRef<'a, Node>;

    fn next(&mut self) -> Option<JSRef<'a, Node>> {
        let ret = self.stack.pop();
        ret.map(|node| {
            self.stack.extend(node.rev_children().map(|c| c.root().get_unsound_ref_forever()))
        });
        ret
    }
}

#[must_root]
pub struct NodeIterator {
    pub start_node: JS<Node>,
    pub current_node: Option<JS<Node>>,
    pub depth: uint,
    include_start: bool,
    include_descendants_of_void: bool
}

impl NodeIterator {
    pub fn new<'a>(start_node: JSRef<'a, Node>,
                   include_start: bool,
                   include_descendants_of_void: bool) -> NodeIterator {
        NodeIterator {
            start_node: JS::from_rooted(start_node),
            current_node: None,
            depth: 0,
            include_start: include_start,
            include_descendants_of_void: include_descendants_of_void
        }
    }

    fn next_child<'b>(&self, node: JSRef<'b, Node>) -> Option<JSRef<'b, Node>> {
        let skip = |&:element: JSRef<Element>| {
            !self.include_descendants_of_void && element.is_void()
        };

        match ElementCast::to_ref(node) {
            Some(element) if skip(element) => None,
            _ => node.first_child().map(|child| child.root().get_unsound_ref_forever()),
        }
    }
}

impl<'a> Iterator for NodeIterator {
    type Item = JSRef<'a, Node>;

    fn next(&mut self) -> Option<JSRef<'a, Node>> {
        self.current_node = match self.current_node.as_ref().map(|node| node.root()) {
            None => {
                if self.include_start {
                    Some(self.start_node)
                } else {
                    self.next_child(self.start_node.root().r())
                        .map(|child| JS::from_rooted(child))
                }
            },
            Some(node) => {
                match self.next_child(node.r()) {
                    Some(child) => {
                        self.depth += 1;
                        Some(JS::from_rooted(child))
                    },
                    None if JS::from_rooted(node.r()) == self.start_node => None,
                    None => {
                        match node.r().next_sibling().root() {
                            Some(sibling) => Some(JS::from_rooted(sibling.r())),
                            None => {
                                let mut candidate = node.get_unsound_ref_forever();
                                while candidate.next_sibling().is_none() {
                                    candidate = candidate.parent_node()
                                                         .expect("Got to root without reaching start node")
                                                         .root()
                                                         .get_unsound_ref_forever();
                                    self.depth -= 1;
                                    if JS::from_rooted(candidate) == self.start_node {
                                        break;
                                    }
                                }
                                if JS::from_rooted(candidate) != self.start_node {
                                    candidate.next_sibling().map(|node| JS::from_rooted(node.root().r()))
                                } else {
                                    None
                                }
                            }
                        }
                    }
                }
            }
        };
        self.current_node.map(|node| node.root().get_unsound_ref_forever())
    }
}

/// Specifies whether children must be recursively cloned or not.
#[derive(Copy, PartialEq)]
pub enum CloneChildrenFlag {
    CloneChildren,
    DoNotCloneChildren
}

fn as_uintptr<T>(t: &T) -> uintptr_t { t as *const T as uintptr_t }

impl Node {
    pub fn reflect_node<N: Reflectable+NodeBase>
            (node:      Box<N>,
             document:  JSRef<Document>,
             wrap_fn:   extern "Rust" fn(*mut JSContext, GlobalRef, Box<N>) -> Temporary<N>)
             -> Temporary<N> {
        let window = document.window().root();
        reflect_dom_object(node, GlobalRef::Window(window.r()), wrap_fn)
    }

    pub fn new_inherited(type_id: NodeTypeId, doc: JSRef<Document>) -> Node {
        Node::new_(type_id, Some(doc.clone()))
    }

    pub fn new_without_doc(type_id: NodeTypeId) -> Node {
        Node::new_(type_id, None)
    }

    fn new_(type_id: NodeTypeId, doc: Option<JSRef<Document>>) -> Node {
        Node {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::Node(type_id)),
            type_id: type_id,

            parent_node: Default::default(),
            first_child: Default::default(),
            last_child: Default::default(),
            next_sibling: Default::default(),
            prev_sibling: Default::default(),
            owner_doc: MutNullableJS::new(doc),
            child_list: Default::default(),
            flags: Cell::new(NodeFlags::new(type_id)),

            layout_data: LayoutDataRef::new(),

            unique_id: DOMRefCell::new(String::new()),
        }
    }

    #[inline]
    pub fn layout_data(&self) -> Ref<Option<LayoutData>> {
        self.layout_data.borrow()
    }

    #[inline]
    pub fn layout_data_mut(&self) -> RefMut<Option<LayoutData>> {
        self.layout_data.borrow_mut()
    }

    #[inline]
    pub unsafe fn layout_data_unchecked(&self) -> *const Option<LayoutData> {
        self.layout_data.borrow_unchecked()
    }

    // http://dom.spec.whatwg.org/#concept-node-adopt
    pub fn adopt(node: JSRef<Node>, document: JSRef<Document>) {
        // Step 1.
        match node.parent_node().root() {
            Some(parent) => {
                Node::remove(node, parent.r(), SuppressObserver::Unsuppressed);
            }
            None => (),
        }

        // Step 2.
        let node_doc = document_from_node(node).root();
        if node_doc.r() != document {
            for descendant in node.traverse_preorder() {
                descendant.set_owner_doc(document);
            }
        }

        // Step 3.
        // If node is an element, it is _affected by a base URL change_.
    }

    // http://dom.spec.whatwg.org/#concept-node-pre-insert
    fn pre_insert(node: JSRef<Node>, parent: JSRef<Node>, child: Option<JSRef<Node>>)
                  -> Fallible<Temporary<Node>> {
        // Step 1.
        match parent.type_id() {
            NodeTypeId::Document |
            NodeTypeId::DocumentFragment |
            NodeTypeId::Element(..) => (),
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
            NodeTypeId::Text => {
                if parent.is_document() {
                    return Err(HierarchyRequest);
                }
            },
            NodeTypeId::DocumentType => {
                if !parent.is_document() {
                    return Err(HierarchyRequest);
                }
            },
            NodeTypeId::DocumentFragment |
            NodeTypeId::Element(_) |
            NodeTypeId::ProcessingInstruction |
            NodeTypeId::Comment => (),
            NodeTypeId::Document => return Err(HierarchyRequest)
        }

        // Step 6.
        match parent.type_id() {
            NodeTypeId::Document => {
                match node.type_id() {
                    // Step 6.1
                    NodeTypeId::DocumentFragment => {
                        // Step 6.1.1(b)
                        if node.children().any(|c| c.is_text()) {
                            return Err(HierarchyRequest);
                        }
                        match node.child_elements().count() {
                            0 => (),
                            // Step 6.1.2
                            1 => {
                                if !parent.child_elements().is_empty() {
                                    return Err(HierarchyRequest);
                                }
                                match child {
                                    Some(child) => {
                                        if child.inclusively_following_siblings()
                                            .any(|child| child.is_doctype()) {
                                                return Err(HierarchyRequest)
                                            }
                                    }
                                    _ => (),
                                }
                            },
                            // Step 6.1.1(a)
                            _ => return Err(HierarchyRequest),
                        }
                    },
                    // Step 6.2
                    NodeTypeId::Element(_) => {
                        if !parent.child_elements().is_empty() {
                            return Err(HierarchyRequest);
                        }
                        match child {
                            Some(ref child) => {
                                if child.inclusively_following_siblings()
                                    .any(|child| child.is_doctype()) {
                                        return Err(HierarchyRequest)
                                    }
                            }
                            _ => (),
                        }
                    },
                    // Step 6.3
                    NodeTypeId::DocumentType => {
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
                                if !parent.child_elements().is_empty() {
                                    return Err(HierarchyRequest);
                                }
                            },
                        }
                    },
                    NodeTypeId::Text |
                    NodeTypeId::ProcessingInstruction |
                    NodeTypeId::Comment => (),
                    NodeTypeId::Document => unreachable!(),
                }
            },
            _ => (),
        }

        // Step 7-8.
        let reference_child = match child {
            Some(child) if child.clone() == node => node.next_sibling().map(|node| node.root().get_unsound_ref_forever()),
            _ => child
        };

        // Step 9.
        let document = document_from_node(parent).root();
        Node::adopt(node, document.r());

        // Step 10.
        Node::insert(node, parent, reference_child, SuppressObserver::Unsuppressed);

        // Step 11.
        return Ok(Temporary::from_rooted(node))
    }

    // http://dom.spec.whatwg.org/#concept-node-insert
    fn insert(node: JSRef<Node>,
              parent: JSRef<Node>,
              child: Option<JSRef<Node>>,
              suppress_observers: SuppressObserver) {
        fn do_insert(node: JSRef<Node>, parent: JSRef<Node>, child: Option<JSRef<Node>>) {
            parent.add_child(node, child);
            let is_in_doc = parent.is_in_doc();
            for kid in node.traverse_preorder() {
                let mut flags = kid.flags.get();
                if is_in_doc {
                    flags.insert(IS_IN_DOC);
                } else {
                    flags.remove(IS_IN_DOC);
                }
                kid.flags.set(flags);
            }
        }

        fn fire_observer_if_necessary(node: JSRef<Node>, suppress_observers: SuppressObserver) {
            match suppress_observers {
                SuppressObserver::Unsuppressed => node.node_inserted(),
                SuppressObserver::Suppressed => ()
            }
        }

        // XXX assert owner_doc
        // Step 1-3: ranges.

        match node.type_id() {
            NodeTypeId::DocumentFragment => {
                // Step 4.
                // Step 5: DocumentFragment, mutation records.
                // Step 6: DocumentFragment.
                let mut kids = Vec::new();
                for kid in node.children() {
                    kids.push(kid.clone());
                    Node::remove(kid, node, SuppressObserver::Suppressed);
                }

                // Step 7: mutation records.
                // Step 8.
                for kid in kids.iter() {
                    do_insert((*kid).clone(), parent, child);
                }

                for kid in kids.into_iter() {
                    fire_observer_if_necessary(kid, suppress_observers);
                }
            }
            _ => {
                // Step 4.
                // Step 5: DocumentFragment, mutation records.
                // Step 6: DocumentFragment.
                // Step 7: mutation records.
                // Step 8.
                do_insert(node, parent, child);
                // Step 9.
                fire_observer_if_necessary(node, suppress_observers);
            }
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-replace-all
    fn replace_all(node: Option<JSRef<Node>>, parent: JSRef<Node>) {
        // Step 1.
        match node {
            Some(node) => {
                let document = document_from_node(parent).root();
                Node::adopt(node, document.r());
            }
            None => (),
        }

        // Step 2.
        let removed_nodes: Vec<JSRef<Node>> = parent.children().collect();

        // Step 3.
        let added_nodes = match node {
            None => vec!(),
            Some(node) => match node.type_id() {
                NodeTypeId::DocumentFragment => node.children().collect(),
                _ => vec!(node.clone()),
            },
        };

        // Step 4.
        for child in parent.children() {
            Node::remove(child, parent, SuppressObserver::Suppressed);
        }

        // Step 5.
        match node {
            Some(node) => Node::insert(node, parent, None, SuppressObserver::Suppressed),
            None => (),
        }

        // Step 6: mutation records.

        // Step 7.
        let parent_in_doc = parent.is_in_doc();
        for removed_node in removed_nodes.iter() {
            removed_node.node_removed(parent_in_doc);
        }
        for added_node in added_nodes.iter() {
            added_node.node_inserted();
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-pre-remove
    fn pre_remove(child: JSRef<Node>, parent: JSRef<Node>) -> Fallible<Temporary<Node>> {
        // Step 1.
        match child.parent_node() {
            Some(ref node) if node != &Temporary::from_rooted(parent) => return Err(NotFound),
            None => return Err(NotFound),
            _ => ()
        }

        // Step 2.
        Node::remove(child, parent, SuppressObserver::Unsuppressed);

        // Step 3.
        Ok(Temporary::from_rooted(child))
    }

    // http://dom.spec.whatwg.org/#concept-node-remove
    fn remove(node: JSRef<Node>, parent: JSRef<Node>, suppress_observers: SuppressObserver) {
        assert!(node.parent_node().map_or(false, |node_parent| node_parent == Temporary::from_rooted(parent)));

        // Step 1-5: ranges.
        // Step 6-7: mutation observers.
        // Step 8.
        parent.remove_child(node);

        node.set_flag(IS_IN_DOC, false);

        // Step 9.
        match suppress_observers {
            SuppressObserver::Suppressed => (),
            SuppressObserver::Unsuppressed => node.node_removed(parent.is_in_doc()),
        }
    }

    // http://dom.spec.whatwg.org/#concept-node-clone
    pub fn clone(node: JSRef<Node>, maybe_doc: Option<JSRef<Document>>,
                 clone_children: CloneChildrenFlag) -> Temporary<Node> {

        // Step 1.
        let document = match maybe_doc {
            Some(doc) => JS::from_rooted(doc).root(),
            None => node.owner_doc().root()
        };

        // Step 2.
        // XXXabinader: clone() for each node as trait?
        let copy: Root<Node> = match node.type_id() {
            NodeTypeId::DocumentType => {
                let doctype: JSRef<DocumentType> = DocumentTypeCast::to_ref(node).unwrap();
                let doctype = DocumentType::new(doctype.name().clone(),
                                                Some(doctype.public_id().clone()),
                                                Some(doctype.system_id().clone()), document.r());
                NodeCast::from_temporary(doctype)
            },
            NodeTypeId::DocumentFragment => {
                let doc_fragment = DocumentFragment::new(document.r());
                NodeCast::from_temporary(doc_fragment)
            },
            NodeTypeId::Comment => {
                let comment: JSRef<Comment> = CommentCast::to_ref(node).unwrap();
                let comment = Comment::new(comment.characterdata().data().clone(), document.r());
                NodeCast::from_temporary(comment)
            },
            NodeTypeId::Document => {
                let document: JSRef<Document> = DocumentCast::to_ref(node).unwrap();
                let is_html_doc = match document.is_html_document() {
                    true => IsHTMLDocument::HTMLDocument,
                    false => IsHTMLDocument::NonHTMLDocument,
                };
                let window = document.window().root();
                let document = Document::new(window.r(), Some(document.url()),
                                             is_html_doc, None,
                                             DocumentSource::NotFromParser);
                NodeCast::from_temporary(document)
            },
            NodeTypeId::Element(..) => {
                let element: JSRef<Element> = ElementCast::to_ref(node).unwrap();
                let name = QualName {
                    ns: element.namespace().clone(),
                    local: element.local_name().clone()
                };
                let element = Element::create(name,
                    element.prefix().as_ref().map(|p| p.as_slice().to_owned()),
                    document.r(), ElementCreator::ScriptCreated);
                NodeCast::from_temporary(element)
            },
            NodeTypeId::Text => {
                let text: JSRef<Text> = TextCast::to_ref(node).unwrap();
                let text = Text::new(text.characterdata().data().clone(), document.r());
                NodeCast::from_temporary(text)
            },
            NodeTypeId::ProcessingInstruction => {
                let pi: JSRef<ProcessingInstruction> = ProcessingInstructionCast::to_ref(node).unwrap();
                let pi = ProcessingInstruction::new(pi.target().clone(),
                                                    pi.characterdata().data().clone(), document.r());
                NodeCast::from_temporary(pi)
            },
        }.root();

        // Step 3.
        let document = match DocumentCast::to_ref(copy.r()) {
            Some(doc) => doc,
            None => document.r(),
        };
        assert!(copy.r().owner_doc().root().r() == document);

        // Step 4 (some data already copied in step 2).
        match node.type_id() {
            NodeTypeId::Document => {
                let node_doc: JSRef<Document> = DocumentCast::to_ref(node).unwrap();
                let copy_doc: JSRef<Document> = DocumentCast::to_ref(copy.r()).unwrap();
                copy_doc.set_encoding_name(node_doc.encoding_name().clone());
                copy_doc.set_quirks_mode(node_doc.quirks_mode());
            },
            NodeTypeId::Element(..) => {
                let node_elem: JSRef<Element> = ElementCast::to_ref(node).unwrap();
                let copy_elem: JSRef<Element> = ElementCast::to_ref(copy.r()).unwrap();

                // FIXME: https://github.com/mozilla/servo/issues/1737
                let window = document.window().root();
                for attr in node_elem.attrs().iter().map(|attr| attr.root()) {
                    copy_elem.attrs_mut().push_unrooted(
                        &Attr::new(window.r(),
                                   attr.r().local_name().clone(), attr.r().value().clone(),
                                   attr.r().name().clone(), attr.r().namespace().clone(),
                                   attr.r().prefix().clone(), Some(copy_elem)));
                }
            },
            _ => ()
        }

        // Step 5: cloning steps.
        vtable_for(&node).cloning_steps(copy.r(), maybe_doc, clone_children);

        // Step 6.
        if clone_children == CloneChildrenFlag::CloneChildren {
            for child in node.children() {
                let child_copy = Node::clone(child, Some(document), clone_children).root();
                let _inserted_node = Node::pre_insert(child_copy.r(), copy.r(), None);
            }
        }

        // Step 7.
        Temporary::from_rooted(copy.r())
    }

    /// Sends layout data, if any, back to the layout task to be destroyed.
    unsafe fn reap_layout_data(&mut self) {
        if self.layout_data.is_present() {
            let layout_data = mem::replace(&mut self.layout_data, LayoutDataRef::new());
            let layout_chan = layout_data.take_chan();
            match layout_chan {
                None => {}
                Some(chan) => {
                    let LayoutChan(chan) = chan;
                    chan.send(Msg::ReapLayoutData(layout_data)).unwrap()
                },
            }
        }
    }

    pub fn collect_text_contents<'a, T: Iterator<Item=JSRef<'a, Node>>>(iterator: T) -> String {
        let mut content = String::new();
        for node in iterator {
            let text: Option<JSRef<Text>> = TextCast::to_ref(node);
            match text {
                Some(text) => content.push_str(text.characterdata().data().as_slice()),
                None => (),
            }
        }
        content
    }
}

impl<'a> NodeMethods for JSRef<'a, Node> {
    // http://dom.spec.whatwg.org/#dom-node-nodetype
    fn NodeType(self) -> u16 {
        match self.type_id {
            NodeTypeId::Element(_)            => NodeConstants::ELEMENT_NODE,
            NodeTypeId::Text                  => NodeConstants::TEXT_NODE,
            NodeTypeId::ProcessingInstruction => NodeConstants::PROCESSING_INSTRUCTION_NODE,
            NodeTypeId::Comment               => NodeConstants::COMMENT_NODE,
            NodeTypeId::Document              => NodeConstants::DOCUMENT_NODE,
            NodeTypeId::DocumentType          => NodeConstants::DOCUMENT_TYPE_NODE,
            NodeTypeId::DocumentFragment      => NodeConstants::DOCUMENT_FRAGMENT_NODE,
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-nodename
    fn NodeName(self) -> DOMString {
        match self.type_id {
            NodeTypeId::Element(..) => {
                let elem: JSRef<Element> = ElementCast::to_ref(self).unwrap();
                elem.TagName()
            }
            NodeTypeId::Text => "#text".to_owned(),
            NodeTypeId::ProcessingInstruction => {
                let processing_instruction: JSRef<ProcessingInstruction> =
                    ProcessingInstructionCast::to_ref(self).unwrap();
                processing_instruction.Target()
            }
            NodeTypeId::Comment => "#comment".to_owned(),
            NodeTypeId::DocumentType => {
                let doctype: JSRef<DocumentType> = DocumentTypeCast::to_ref(self).unwrap();
                doctype.name().clone()
            },
            NodeTypeId::DocumentFragment => "#document-fragment".to_owned(),
            NodeTypeId::Document => "#document".to_owned()
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-baseuri
    fn GetBaseURI(self) -> Option<DOMString> {
        // FIXME (#1824) implement.
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-ownerdocument
    fn GetOwnerDocument(self) -> Option<Temporary<Document>> {
        match self.type_id {
            NodeTypeId::Element(..) |
            NodeTypeId::Comment |
            NodeTypeId::Text |
            NodeTypeId::ProcessingInstruction |
            NodeTypeId::DocumentType |
            NodeTypeId::DocumentFragment => Some(self.owner_doc()),
            NodeTypeId::Document => None
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-parentnode
    fn GetParentNode(self) -> Option<Temporary<Node>> {
        self.parent_node.get()
    }

    // http://dom.spec.whatwg.org/#dom-node-parentelement
    fn GetParentElement(self) -> Option<Temporary<Element>> {
        self.parent_node.get()
                        .and_then(|parent| {
                            let parent = parent.root();
                            ElementCast::to_ref(parent.r()).map(|elem| {
                                Temporary::from_rooted(elem)
                            })
                        })
    }

    // http://dom.spec.whatwg.org/#dom-node-haschildnodes
    fn HasChildNodes(self) -> bool {
        self.first_child.get().is_some()
    }

    // http://dom.spec.whatwg.org/#dom-node-childnodes
    fn ChildNodes(self) -> Temporary<NodeList> {
        self.child_list.or_init(|| {
            let doc = self.owner_doc().root();
            let window = doc.r().window().root();
            NodeList::new_child_list(window.r(), self)
        })
    }

    // http://dom.spec.whatwg.org/#dom-node-firstchild
    fn GetFirstChild(self) -> Option<Temporary<Node>> {
        self.first_child.get()
    }

    // http://dom.spec.whatwg.org/#dom-node-lastchild
    fn GetLastChild(self) -> Option<Temporary<Node>> {
        self.last_child.get()
    }

    // http://dom.spec.whatwg.org/#dom-node-previoussibling
    fn GetPreviousSibling(self) -> Option<Temporary<Node>> {
        self.prev_sibling.get()
    }

    // http://dom.spec.whatwg.org/#dom-node-nextsibling
    fn GetNextSibling(self) -> Option<Temporary<Node>> {
        self.next_sibling.get()
    }

    // http://dom.spec.whatwg.org/#dom-node-nodevalue
    fn GetNodeValue(self) -> Option<DOMString> {
        match self.type_id {
            NodeTypeId::Comment |
            NodeTypeId::Text |
            NodeTypeId::ProcessingInstruction => {
                let chardata: JSRef<CharacterData> = CharacterDataCast::to_ref(self).unwrap();
                Some(chardata.Data())
            }
            _ => {
                None
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-nodevalue
    fn SetNodeValue(self, val: Option<DOMString>) {
        match self.type_id {
            NodeTypeId::Comment |
            NodeTypeId::Text |
            NodeTypeId::ProcessingInstruction => {
                self.SetTextContent(val)
            }
            _ => {}
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-textcontent
    fn GetTextContent(self) -> Option<DOMString> {
        match self.type_id {
            NodeTypeId::DocumentFragment |
            NodeTypeId::Element(..) => {
                let content = Node::collect_text_contents(self.traverse_preorder());
                Some(content)
            }
            NodeTypeId::Comment |
            NodeTypeId::Text |
            NodeTypeId::ProcessingInstruction => {
                let characterdata: JSRef<CharacterData> = CharacterDataCast::to_ref(self).unwrap();
                Some(characterdata.Data())
            }
            NodeTypeId::DocumentType |
            NodeTypeId::Document => {
                None
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-textcontent
    fn SetTextContent(self, value: Option<DOMString>) {
        let value = null_str_as_empty(&value);
        match self.type_id {
            NodeTypeId::DocumentFragment |
            NodeTypeId::Element(..) => {
                // Step 1-2.
                let node = if value.len() == 0 {
                    None
                } else {
                    let document = self.owner_doc().root();
                    Some(NodeCast::from_temporary(document.r().CreateTextNode(value)))
                }.root();

                // Step 3.
                Node::replace_all(node.r(), self);
            }
            NodeTypeId::Comment |
            NodeTypeId::Text |
            NodeTypeId::ProcessingInstruction => {
                let characterdata: JSRef<CharacterData> = CharacterDataCast::to_ref(self).unwrap();
                characterdata.set_data(value);

                // Notify the document that the content of this node is different
                let document = self.owner_doc().root();
                document.r().content_changed(self, NodeDamage::OtherNodeDamage);
            }
            NodeTypeId::DocumentType |
            NodeTypeId::Document => {}
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-insertbefore
    fn InsertBefore(self, node: JSRef<Node>, child: Option<JSRef<Node>>) -> Fallible<Temporary<Node>> {
        Node::pre_insert(node, self, child)
    }

    // http://dom.spec.whatwg.org/#dom-node-appendchild
    fn AppendChild(self, node: JSRef<Node>) -> Fallible<Temporary<Node>> {
        Node::pre_insert(node, self, None)
    }

    // http://dom.spec.whatwg.org/#concept-node-replace
    fn ReplaceChild(self, node: JSRef<Node>, child: JSRef<Node>) -> Fallible<Temporary<Node>> {

        // Step 1.
        match self.type_id {
            NodeTypeId::Document |
            NodeTypeId::DocumentFragment |
            NodeTypeId::Element(..) => (),
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
            NodeTypeId::Text if self.is_document() => return Err(HierarchyRequest),
            NodeTypeId::DocumentType if !self.is_document() => return Err(HierarchyRequest),
            NodeTypeId::DocumentFragment |
            NodeTypeId::DocumentType |
            NodeTypeId::Element(..) |
            NodeTypeId::Text |
            NodeTypeId::ProcessingInstruction |
            NodeTypeId::Comment => (),
            NodeTypeId::Document => return Err(HierarchyRequest)
        }

        // Step 6.
        match self.type_id {
            NodeTypeId::Document => {
                match node.type_id() {
                    // Step 6.1
                    NodeTypeId::DocumentFragment => {
                        // Step 6.1.1(b)
                        if node.children().any(|c| c.is_text()) {
                            return Err(HierarchyRequest);
                        }
                        match node.child_elements().count() {
                            0 => (),
                            // Step 6.1.2
                            1 => {
                                if self.child_elements().any(|c| NodeCast::from_ref(c) != child) {
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
                    NodeTypeId::Element(..) => {
                        if self.child_elements().any(|c| NodeCast::from_ref(c) != child) {
                            return Err(HierarchyRequest);
                        }
                        if child.following_siblings()
                                .any(|child| child.is_doctype()) {
                            return Err(HierarchyRequest);
                        }
                    },
                    // Step 6.3
                    NodeTypeId::DocumentType => {
                        if self.children().any(|c| c.is_doctype() && c != child) {
                            return Err(HierarchyRequest);
                        }
                        if self.children()
                            .take_while(|c| *c != child)
                            .any(|c| c.is_element()) {
                            return Err(HierarchyRequest);
                        }
                    },
                    NodeTypeId::Text |
                    NodeTypeId::ProcessingInstruction |
                    NodeTypeId::Comment => (),
                    NodeTypeId::Document => unreachable!()
                }
            },
            _ => ()
        }

        // Ok if not caught by previous error checks.
        if node.clone() == child {
            return Ok(Temporary::from_rooted(child));
        }

        // Step 7-8.
        let next_sibling = child.next_sibling().map(|node| node.root().get_unsound_ref_forever());
        let reference_child = match next_sibling {
            Some(sibling) if sibling == node => node.next_sibling().map(|node| node.root().get_unsound_ref_forever()),
            _ => next_sibling
        };

        // Step 9.
        let document = document_from_node(self).root();
        Node::adopt(node, document.r());

        {
            // Step 10.
            Node::remove(child, self, SuppressObserver::Suppressed);

            // Step 11.
            Node::insert(node, self, reference_child, SuppressObserver::Suppressed);
        }

        // Step 12-14.
        // Step 13: mutation records.
        child.node_removed(self.is_in_doc());
        if node.type_id() == NodeTypeId::DocumentFragment {
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
    fn RemoveChild(self, node: JSRef<Node>)
                       -> Fallible<Temporary<Node>> {
        Node::pre_remove(node, self)
    }

    // http://dom.spec.whatwg.org/#dom-node-normalize
    fn Normalize(self) {
        let mut prev_text = None;
        for child in self.children() {
            let t: Option<JSRef<Text>> = TextCast::to_ref(child);
            match t {
                Some(text) => {
                    let characterdata: JSRef<CharacterData> = CharacterDataCast::from_ref(text);
                    if characterdata.Length() == 0 {
                        self.remove_child(child);
                    } else {
                        match prev_text {
                            Some(text_node) => {
                                let prev_characterdata: JSRef<CharacterData> = CharacterDataCast::from_ref(text_node);
                                let _ = prev_characterdata.AppendData(characterdata.Data());
                                self.remove_child(child);
                            },
                            None => prev_text = Some(text)
                        }
                    }
                },
                None => {
                    child.Normalize();
                    prev_text = None;
                }
            }
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-clonenode
    fn CloneNode(self, deep: bool) -> Temporary<Node> {
        Node::clone(self, None, if deep {
            CloneChildrenFlag::CloneChildren
        } else {
            CloneChildrenFlag::DoNotCloneChildren
        })
    }

    // http://dom.spec.whatwg.org/#dom-node-isequalnode
    fn IsEqualNode(self, maybe_node: Option<JSRef<Node>>) -> bool {
        fn is_equal_doctype(node: JSRef<Node>, other: JSRef<Node>) -> bool {
            let doctype: JSRef<DocumentType> = DocumentTypeCast::to_ref(node).unwrap();
            let other_doctype: JSRef<DocumentType> = DocumentTypeCast::to_ref(other).unwrap();
            (*doctype.name() == *other_doctype.name()) &&
            (*doctype.public_id() == *other_doctype.public_id()) &&
            (*doctype.system_id() == *other_doctype.system_id())
        }
        fn is_equal_element(node: JSRef<Node>, other: JSRef<Node>) -> bool {
            let element: JSRef<Element> = ElementCast::to_ref(node).unwrap();
            let other_element: JSRef<Element> = ElementCast::to_ref(other).unwrap();
            // FIXME: namespace prefix
            (*element.namespace() == *other_element.namespace()) &&
            (*element.local_name() == *other_element.local_name()) &&
            (element.attrs().len() == other_element.attrs().len())
        }
        fn is_equal_processinginstruction(node: JSRef<Node>, other: JSRef<Node>) -> bool {
            let pi: JSRef<ProcessingInstruction> = ProcessingInstructionCast::to_ref(node).unwrap();
            let other_pi: JSRef<ProcessingInstruction> = ProcessingInstructionCast::to_ref(other).unwrap();
            (*pi.target() == *other_pi.target()) &&
            (*pi.characterdata().data() == *other_pi.characterdata().data())
        }
        fn is_equal_characterdata(node: JSRef<Node>, other: JSRef<Node>) -> bool {
            let characterdata: JSRef<CharacterData> = CharacterDataCast::to_ref(node).unwrap();
            let other_characterdata: JSRef<CharacterData> = CharacterDataCast::to_ref(other).unwrap();
            *characterdata.data() == *other_characterdata.data()
        }
        fn is_equal_element_attrs(node: JSRef<Node>, other: JSRef<Node>) -> bool {
            let element: JSRef<Element> = ElementCast::to_ref(node).unwrap();
            let other_element: JSRef<Element> = ElementCast::to_ref(other).unwrap();
            assert!(element.attrs().len() == other_element.attrs().len());
            element.attrs().iter().map(|attr| attr.root()).all(|attr| {
                other_element.attrs().iter().map(|attr| attr.root()).any(|other_attr| {
                    (*attr.r().namespace() == *other_attr.r().namespace()) &&
                    (attr.r().local_name() == other_attr.r().local_name()) &&
                    (attr.r().value().as_slice() == other_attr.r().value().as_slice())
                })
            })
        }
        fn is_equal_node(this: JSRef<Node>, node: JSRef<Node>) -> bool {
            // Step 2.
            if this.type_id() != node.type_id() {
                return false;
            }

            match node.type_id() {
                // Step 3.
                NodeTypeId::DocumentType if !is_equal_doctype(this, node) => return false,
                NodeTypeId::Element(..) if !is_equal_element(this, node) => return false,
                NodeTypeId::ProcessingInstruction if !is_equal_processinginstruction(this, node) => return false,
                NodeTypeId::Text |
                NodeTypeId::Comment if !is_equal_characterdata(this, node) => return false,
                // Step 4.
                NodeTypeId::Element(..) if !is_equal_element_attrs(this, node) => return false,
                _ => ()
            }

            // Step 5.
            if this.children().count() != node.children().count() {
                return false;
            }

            // Step 6.
            this.children().zip(node.children()).all(|(ref child, ref other_child)| {
                is_equal_node(*child, *other_child)
            })
        }
        match maybe_node {
            // Step 1.
            None => false,
            // Step 2-6.
            Some(node) => is_equal_node(self, node)
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-comparedocumentposition
    fn CompareDocumentPosition(self, other: JSRef<Node>) -> u16 {
        if self.clone() == other {    // FIXME: See issue #3960
            // step 2.
            0
        } else {
            let mut lastself = self.clone();
            let mut lastother = other.clone();
            for ancestor in self.ancestors() {
                if ancestor == other {
                    // step 4.
                    return NodeConstants::DOCUMENT_POSITION_CONTAINS +
                           NodeConstants::DOCUMENT_POSITION_PRECEDING;
                }
                lastself = ancestor.clone();
            }
            for ancestor in other.ancestors() {
                if ancestor == self {
                    // step 5.
                    return NodeConstants::DOCUMENT_POSITION_CONTAINED_BY +
                           NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                }
                lastother = ancestor.clone();
            }

            if lastself != lastother {
                let abstract_uint: uintptr_t = as_uintptr(&self);
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
                if child == other {
                    // step 6.
                    return NodeConstants::DOCUMENT_POSITION_PRECEDING;
                }
                if child == self {
                    // step 7.
                    return NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                }
            }
            unreachable!()
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-contains
    fn Contains(self, maybe_other: Option<JSRef<Node>>) -> bool {
        match maybe_other {
            None => false,
            Some(other) => self.is_inclusive_ancestor_of(other)
        }
    }

    // http://dom.spec.whatwg.org/#dom-node-lookupprefix
    fn LookupPrefix(self, _prefix: Option<DOMString>) -> Option<DOMString> {
        // FIXME (#1826) implement.
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-lookupnamespaceuri
    fn LookupNamespaceURI(self, _namespace: Option<DOMString>) -> Option<DOMString> {
        // FIXME (#1826) implement.
        None
    }

    // http://dom.spec.whatwg.org/#dom-node-isdefaultnamespace
    fn IsDefaultNamespace(self, _namespace: Option<DOMString>) -> bool {
        // FIXME (#1826) implement.
        false
    }
}



/// The address of a node known to be valid. These are sent from script to layout,
/// and are also used in the HTML parser interface.

#[allow(raw_pointer_derive)]
#[derive(Clone, PartialEq, Eq, Copy)]
pub struct TrustedNodeAddress(pub *const c_void);

unsafe impl Send for TrustedNodeAddress {}

pub fn document_from_node<T: NodeBase+Reflectable>(derived: JSRef<T>) -> Temporary<Document> {
    let node: JSRef<Node> = NodeCast::from_ref(derived);
    node.owner_doc()
}

pub fn window_from_node<T: NodeBase+Reflectable>(derived: JSRef<T>) -> Temporary<Window> {
    let document = document_from_node(derived).root();
    document.r().window()
}

impl<'a> VirtualMethods for JSRef<'a, Node> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let eventtarget: &JSRef<EventTarget> = EventTargetCast::from_borrowed_ref(self);
        Some(eventtarget as &VirtualMethods)
    }
}

impl<'a> style::node::TNode<'a, JSRef<'a, Element>> for JSRef<'a, Node> {
    fn parent_node(self) -> Option<JSRef<'a, Node>> {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn parent_node<'a, T: NodeHelpers<'a>>(this: T) -> Option<Temporary<Node>> {
            this.parent_node()
        }

        parent_node(self).map(|node| node.root().get_unsound_ref_forever())
    }

    fn first_child(self) -> Option<JSRef<'a, Node>> {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn first_child<'a, T: NodeHelpers<'a>>(this: T) -> Option<Temporary<Node>> {
            this.first_child()
        }

        first_child(self).map(|node| node.root().get_unsound_ref_forever())
    }

    fn last_child(self) -> Option<JSRef<'a, Node>> {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn last_child<'a, T: NodeHelpers<'a>>(this: T) -> Option<Temporary<Node>> {
            this.last_child()
        }

        last_child(self).map(|node| node.root().get_unsound_ref_forever())
    }

    fn prev_sibling(self) -> Option<JSRef<'a, Node>> {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn prev_sibling<'a, T: NodeHelpers<'a>>(this: T) -> Option<Temporary<Node>> {
            this.prev_sibling()
        }

        prev_sibling(self).map(|node| node.root().get_unsound_ref_forever())
    }

    fn next_sibling(self) -> Option<JSRef<'a, Node>> {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn next_sibling<'a, T: NodeHelpers<'a>>(this: T) -> Option<Temporary<Node>> {
            this.next_sibling()
        }

        next_sibling(self).map(|node| node.root().get_unsound_ref_forever())
    }

    fn is_document(self) -> bool {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn is_document<'a, T: NodeHelpers<'a>>(this: T) -> bool {
            this.is_document()
        }

        is_document(self)
    }

    fn is_element(self) -> bool {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn is_element<'a, T: NodeHelpers<'a>>(this: T) -> bool {
            this.is_element()
        }

        is_element(self)
    }

    fn as_element(self) -> JSRef<'a, Element> {
        ElementCast::to_ref(self).unwrap()
    }

    fn match_attr<F>(self, attr: &AttrSelector, test: F) -> bool
        where F: Fn(&str) -> bool
    {
        let name = {
            if self.is_html_element_in_html_document() {
                &attr.lower_name
            } else {
                &attr.name
            }
        };
        match attr.namespace {
            NamespaceConstraint::Specific(ref ns) => {
                self.as_element().get_attribute(ns.clone(), name).root()
                    .map_or(false, |attr| test(attr.r().value().as_slice()))
            },
            NamespaceConstraint::Any => {
                self.as_element().get_attributes(name).into_iter()
                    .map(|attr| attr.root())
                    .any(|attr| test(attr.r().value().as_slice()))
            }
        }
    }

    fn is_html_element_in_html_document(self) -> bool {
        self.as_element().html_element_in_html_document()
    }

    fn has_changed(self) -> bool { self.get_has_changed() }
    unsafe fn set_changed(self, value: bool) { self.set_has_changed(value) }

    fn is_dirty(self) -> bool { self.get_is_dirty() }
    unsafe fn set_dirty(self, value: bool) { self.set_is_dirty(value) }

    fn has_dirty_siblings(self) -> bool { self.get_has_dirty_siblings() }
    unsafe fn set_dirty_siblings(self, value: bool) { self.set_has_dirty_siblings(value) }

    fn has_dirty_descendants(self) -> bool { self.get_has_dirty_descendants() }
    unsafe fn set_dirty_descendants(self, value: bool) { self.set_has_dirty_descendants(value) }
}

pub trait DisabledStateHelpers {
    fn check_ancestors_disabled_state_for_form_control(self);
    fn check_parent_disabled_state_for_option(self);
    fn check_disabled_attribute(self);
}

impl<'a> DisabledStateHelpers for JSRef<'a, Node> {
    fn check_ancestors_disabled_state_for_form_control(self) {
        if self.get_disabled_state() { return; }
        for ancestor in self.ancestors().filter(|ancestor| ancestor.is_htmlfieldsetelement()) {
            if !ancestor.get_disabled_state() { continue; }
            if ancestor.is_parent_of(self) {
                self.set_disabled_state(true);
                self.set_enabled_state(false);
                return;
            }
            match ancestor.children().find(|child| child.is_htmllegendelement()) {
                Some(legend) => {
                    // XXXabinader: should we save previous ancestor to avoid this iteration?
                    if self.ancestors().any(|ancestor| ancestor == legend) { continue; }
                },
                None => ()
            }
            self.set_disabled_state(true);
            self.set_enabled_state(false);
            return;
        }
    }

    fn check_parent_disabled_state_for_option(self) {
        if self.get_disabled_state() { return; }
        match self.parent_node().root() {
            Some(ref parent) if parent.r().is_htmloptgroupelement() && parent.r().get_disabled_state() => {
                self.set_disabled_state(true);
                self.set_enabled_state(false);
            },
            _ => ()
        }
    }

    fn check_disabled_attribute(self) {
        let elem: JSRef<'a, Element> = ElementCast::to_ref(self).unwrap();
        let has_disabled_attrib = elem.has_attribute(&atom!("disabled"));
        self.set_disabled_state(has_disabled_attrib);
        self.set_enabled_state(!has_disabled_attrib);
    }
}

/// A summary of the changes that happened to a node.
#[derive(Copy, Clone, PartialEq)]
pub enum NodeDamage {
    /// The node's `style` attribute changed.
    NodeStyleDamaged,
    /// Other parts of a node changed; attributes, text content, etc.
    OtherNodeDamage,
}

