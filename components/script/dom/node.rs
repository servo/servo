/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use document_loader::DocumentLoader;
use dom::attr::{Attr, AttrHelpers};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::NamedNodeMapBinding::NamedNodeMapMethods;
use dom::bindings::codegen::Bindings::NodeBinding::{NodeConstants, NodeMethods};
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::codegen::Bindings::ProcessingInstructionBinding::ProcessingInstructionMethods;
use dom::bindings::codegen::InheritTypes::{CharacterDataCast, DocumentCast, DocumentDerived, DocumentTypeCast};
use dom::bindings::codegen::InheritTypes::{ElementCast, NodeCast, ElementDerived, EventTargetCast};
use dom::bindings::codegen::InheritTypes::{HTMLLegendElementDerived, HTMLFieldSetElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLOptGroupElementDerived, NodeBase, NodeDerived};
use dom::bindings::codegen::InheritTypes::{ProcessingInstructionCast, TextCast, TextDerived};
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::conversions;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::error::Error::{NotFound, HierarchyRequest, Syntax};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, LayoutJS, MutNullableHeap};
use dom::bindings::js::Root;
use dom::bindings::js::RootedReference;
use dom::bindings::trace::JSTraceable;
use dom::bindings::trace::RootedVec;
use dom::bindings::utils::{namespace_from_domstring, Reflectable, reflect_dom_object};
use dom::characterdata::{CharacterData, CharacterDataHelpers, CharacterDataTypeId};
use dom::comment::Comment;
use dom::document::{Document, DocumentHelpers, IsHTMLDocument, DocumentSource};
use dom::documentfragment::DocumentFragment;
use dom::documenttype::DocumentType;
use dom::element::{AttributeHandlers, Element, ElementCreator, ElementTypeId};
use dom::element::ElementHelpers;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::HTMLElementTypeId;
use dom::nodelist::{NodeList, NodeListHelpers};
use dom::processinginstruction::{ProcessingInstruction, ProcessingInstructionHelpers};
use dom::text::Text;
use dom::virtualmethods::{VirtualMethods, vtable_for};
use dom::window::{Window, WindowHelpers};
use euclid::rect::Rect;
use layout_interface::{LayoutChan, Msg};
use devtools_traits::NodeInfo;
use parse::html::parse_html_fragment;
use script_traits::UntrustedNodeAddress;
use util::geometry::Au;
use util::str::DOMString;
use util::task_state;
use selectors::parser::Selector;
use selectors::parser::parse_author_origin_selector_list_from_str;
use selectors::matching::matches;
use style::properties::ComputedValues;

use js::jsapi::{JSContext, JSObject, JSRuntime};
use core::nonzero::NonZero;
use libc;
use libc::{uintptr_t, c_void};
use std::borrow::ToOwned;
use std::cell::{Cell, RefCell, Ref, RefMut};
use std::default::Default;
use std::iter::{FilterMap, Peekable};
use std::mem;
use std::slice::ref_slice;
use std::sync::Arc;
use uuid;
use string_cache::{Atom, Namespace, QualName};

//
// The basic Node structure
//

/// An HTML node.
#[dom_struct]
#[derive(HeapSizeOf)]
pub struct Node {
    /// The JavaScript reflector for this node.
    eventtarget: EventTarget,

    /// The type of node that this is.
    type_id: NodeTypeId,

    /// The parent of this node.
    parent_node: MutNullableHeap<JS<Node>>,

    /// The first child of this node.
    first_child: MutNullableHeap<JS<Node>>,

    /// The last child of this node.
    last_child: MutNullableHeap<JS<Node>>,

    /// The next sibling of this node.
    next_sibling: MutNullableHeap<JS<Node>>,

    /// The previous sibling of this node.
    prev_sibling: MutNullableHeap<JS<Node>>,

    /// The document that this node belongs to.
    owner_doc: MutNullableHeap<JS<Document>>,

    /// The live list of children return by .childNodes.
    child_list: MutNullableHeap<JS<NodeList>>,

    /// The live count of children of this node.
    children_count: Cell<u32>,

    /// A bitfield of flags for node items.
    flags: Cell<NodeFlags>,

    /// Layout information. Only the layout task may touch this data.
    ///
    /// Must be sent back to the layout task to be destroyed when this
    /// node is finalized.
    layout_data: LayoutDataRef,

    unique_id: DOMRefCell<String>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self as *const Node == &*other
    }
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
    #[derive(JSTraceable, HeapSizeOf)]
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
        #[doc = "Specifies whether this node has the focus."]
        const IN_FOCUS_STATE = 0x200,
        #[doc = "Specifies whether this node is focusable and whether it is supposed \
                 to be reachable with using sequential focus navigation."]
        const SEQUENTIALLY_FOCUSABLE = 0x400,
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
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFieldSetElement)) =>
                IN_ENABLED_STATE | dirty,
            _ => dirty,
        }
    }
}

impl Drop for Node {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        self.layout_data.dispose(self);
    }
}

/// suppress observers flag
/// https://dom.spec.whatwg.org/#concept-node-insert
/// https://dom.spec.whatwg.org/#concept-node-remove
#[derive(Copy, Clone, HeapSizeOf)]
enum SuppressObserver {
    Suppressed,
    Unsuppressed
}

/// Layout data that is shared between the script and layout tasks.
#[derive(HeapSizeOf)]
pub struct SharedLayoutData {
    /// The results of CSS styling for this node.
    pub style: Option<Arc<ComputedValues>>,
}

/// Encapsulates the abstract layout data.
#[allow(raw_pointer_derive)]
#[derive(HeapSizeOf)]
pub struct LayoutData {
    _shared_data: SharedLayoutData,
    #[ignore_heap_size_of = "TODO(#6910) Box value that should be counted but the type lives in layout"]
    _data: NonZero<*const ()>,
}

#[allow(unsafe_code)]
unsafe impl Send for LayoutData {}

#[derive(HeapSizeOf)]
pub struct LayoutDataRef {
    data_cell: RefCell<Option<LayoutData>>,
}

no_jsmanaged_fields!(LayoutDataRef);

impl LayoutDataRef {
    pub fn new() -> LayoutDataRef {
        LayoutDataRef {
            data_cell: RefCell::new(None),
        }
    }

    /// Sends layout data, if any, back to the layout task to be destroyed.
    pub fn dispose(&self, node: &Node) {
        debug_assert!(task_state::get().is_script());
        if let Some(layout_data) = mem::replace(&mut *self.data_cell.borrow_mut(), None) {
            let win = window_from_node(node);
            let LayoutChan(chan) = win.layout_chan();
            chan.send(Msg::ReapLayoutData(layout_data)).unwrap()
        }
    }

    /// Borrows the layout data immutably, *assuming that there are no mutators*. Bad things will
    /// happen if you try to mutate the layout data while this is held. This is the only thread-
    /// safe layout data accessor.
    #[inline]
    #[allow(unsafe_code)]
    pub unsafe fn borrow_unchecked(&self) -> *const Option<LayoutData> {
        debug_assert!(task_state::get().is_layout());
        self.data_cell.as_unsafe_cell().get() as *const _
    }

    /// Borrows the layout data immutably. This function is *not* thread-safe.
    #[inline]
    pub fn borrow<'a>(&'a self) -> Ref<'a,Option<LayoutData>> {
        debug_assert!(task_state::get().is_layout());
        self.data_cell.borrow()
    }

    /// Borrows the layout data mutably. This function is *not* thread-safe.
    ///
    /// FIXME(pcwalton): We should really put this behind a `MutLayoutView` phantom type, to
    /// prevent CSS selector matching from mutably accessing nodes it's not supposed to and racing
    /// on it. This has already resulted in one bug!
    #[inline]
    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a,Option<LayoutData>> {
        debug_assert!(task_state::get().is_layout());
        self.data_cell.borrow_mut()
    }
}

/// The different types of nodes.
#[derive(JSTraceable, Copy, Clone, PartialEq, Debug)]
#[derive(HeapSizeOf)]
pub enum NodeTypeId {
    CharacterData(CharacterDataTypeId),
    DocumentType,
    DocumentFragment,
    Document,
    Element(ElementTypeId),
}

trait PrivateNodeHelpers {
    fn add_child(self, new_child: &Node, before: Option<&Node>);
    fn remove_child(self, child: &Node);
}

impl<'a> PrivateNodeHelpers for &'a Node {
    /// Adds a new child to the end of this node's list of children.
    ///
    /// Fails unless `new_child` is disconnected from the tree.
    fn add_child(self, new_child: &Node, before: Option<&Node>) {
        assert!(new_child.parent_node.get().is_none());
        assert!(new_child.prev_sibling.get().is_none());
        assert!(new_child.next_sibling.get().is_none());
        match before {
            Some(ref before) => {
                assert!(before.parent_node.get().map(Root::from_rooted).r() == Some(self));
                let prev_sibling = before.GetPreviousSibling();
                match prev_sibling {
                    None => {
                        assert!(Some(*before) == self.first_child.get().map(Root::from_rooted).r());
                        self.first_child.set(Some(JS::from_ref(new_child)));
                    },
                    Some(ref prev_sibling) => {
                        prev_sibling.next_sibling.set(Some(JS::from_ref(new_child)));
                        new_child.prev_sibling.set(Some(JS::from_ref(prev_sibling.r())));
                    },
                }
                before.prev_sibling.set(Some(JS::from_ref(new_child)));
                new_child.next_sibling.set(Some(JS::from_ref(before)));
            },
            None => {
                let last_child = self.GetLastChild();
                match last_child {
                    None => self.first_child.set(Some(JS::from_ref(new_child))),
                    Some(ref last_child) => {
                        assert!(last_child.next_sibling.get().is_none());
                        last_child.r().next_sibling.set(Some(JS::from_ref(new_child)));
                        new_child.prev_sibling.set(Some(JS::from_rooted(&last_child)));
                    }
                }

                self.last_child.set(Some(JS::from_ref(new_child)));
            },
        }

        new_child.parent_node.set(Some(JS::from_ref(self)));

        let parent_in_doc = self.is_in_doc();
        for node in new_child.traverse_preorder() {
            node.set_flag(IS_IN_DOC, parent_in_doc);
            vtable_for(&&*node).bind_to_tree(parent_in_doc);
        }
        let document = new_child.owner_doc();
        document.content_and_heritage_changed(new_child, NodeDamage::OtherNodeDamage);
    }

    /// Removes the given child from this node's list of children.
    ///
    /// Fails unless `child` is a child of this node.
    fn remove_child(self, child: &Node) {
        assert!(child.parent_node.get().map(Root::from_rooted).r() == Some(self));
        let prev_sibling = child.GetPreviousSibling();
        match prev_sibling {
            None => {
                self.first_child.set(child.next_sibling.get());
            }
            Some(ref prev_sibling) => {
                prev_sibling.next_sibling.set(child.next_sibling.get());
            }
        }
        let next_sibling = child.GetNextSibling();
        match next_sibling {
            None => {
                self.last_child.set(child.prev_sibling.get());
            }
            Some(ref next_sibling) => {
                next_sibling.prev_sibling.set(child.prev_sibling.get());
            }
        }

        child.prev_sibling.set(None);
        child.next_sibling.set(None);
        child.parent_node.set(None);

        let parent_in_doc = self.is_in_doc();
        for node in child.traverse_preorder() {
            node.set_flag(IS_IN_DOC, false);
            vtable_for(&&*node).unbind_from_tree(parent_in_doc);
            node.layout_data.dispose(&node);
        }

        let document = child.owner_doc();
        document.content_and_heritage_changed(child, NodeDamage::OtherNodeDamage);
    }
}

pub struct QuerySelectorIterator {
    selectors: Vec<Selector>,
    iterator: TreeIterator,
}

impl<'a> QuerySelectorIterator {
    #[allow(unsafe_code)]
    unsafe fn new(iter: TreeIterator, selectors: Vec<Selector>)
                  -> QuerySelectorIterator {
        QuerySelectorIterator {
            selectors: selectors,
            iterator: iter,
        }
    }
}

impl<'a> Iterator for QuerySelectorIterator {
    type Item = Root<Node>;

    fn next(&mut self) -> Option<Root<Node>> {
        let selectors = &self.selectors;
        // TODO(cgaebel): Is it worth it to build a bloom filter here
        // (instead of passing `None`)? Probably.
        self.iterator.by_ref().filter_map(|node| {
            if let Some(element) = ElementCast::to_root(node) {
                if matches(selectors, &element, None) {
                    return Some(NodeCast::from_root(element))
                }
            }
            None
        }).next()
    }
}

pub trait NodeHelpers {
    fn ancestors(self) -> AncestorIterator;
    fn inclusive_ancestors(self) -> AncestorIterator;
    fn children(self) -> NodeSiblingIterator;
    fn rev_children(self) -> ReverseSiblingIterator;
    fn child_elements(self) -> ChildElementIterator;
    fn following_siblings(self) -> NodeSiblingIterator;
    fn preceding_siblings(self) -> ReverseSiblingIterator;
    fn following_nodes(self, root: &Node) -> FollowingNodeIterator;
    fn preceding_nodes(self, root: &Node) -> PrecedingNodeIterator;
    fn descending_last_children(self) -> LastChildIterator;
    fn is_in_doc(self) -> bool;
    fn is_inclusive_ancestor_of(self, parent: &Node) -> bool;
    fn is_parent_of(self, child: &Node) -> bool;

    fn type_id(self) -> NodeTypeId;
    fn len(self) -> u32;
    fn index(self) -> u32;
    fn children_count(self) -> u32;

    fn owner_doc(self) -> Root<Document>;
    fn set_owner_doc(self, document: &Document);
    fn is_in_html_doc(self) -> bool;

    fn is_doctype(self) -> bool;
    fn is_anchor_element(self) -> bool;

    fn get_flag(self, flag: NodeFlags) -> bool;
    fn set_flag(self, flag: NodeFlags, value: bool);

    fn get_hover_state(self) -> bool;
    fn set_hover_state(self, state: bool);

    fn get_focus_state(self) -> bool;
    fn set_focus_state(self, state: bool);

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
    fn dump_indent(self, indent: u32);
    fn debug_str(self) -> String;

    fn traverse_preorder(self) -> TreeIterator;
    fn inclusively_following_siblings(self) -> NodeSiblingIterator;
    fn inclusively_preceding_siblings(self) -> ReverseSiblingIterator;

    fn to_trusted_node_address(self) -> TrustedNodeAddress;

    fn get_bounding_content_box(self) -> Rect<Au>;
    fn get_content_boxes(self) -> Vec<Rect<Au>>;
    fn get_client_rect(self) -> Rect<i32>;

    fn before(self, nodes: Vec<NodeOrString>) -> ErrorResult;
    fn after(self, nodes: Vec<NodeOrString>) -> ErrorResult;
    fn replace_with(self, nodes: Vec<NodeOrString>) -> ErrorResult;
    fn prepend(self, nodes: Vec<NodeOrString>) -> ErrorResult;
    fn append(self, nodes: Vec<NodeOrString>) -> ErrorResult;

    fn query_selector(self, selectors: DOMString) -> Fallible<Option<Root<Element>>>;
    #[allow(unsafe_code)]
    unsafe fn query_selector_iter(self, selectors: DOMString) -> Fallible<QuerySelectorIterator>;
    fn query_selector_all(self, selectors: DOMString) -> Fallible<Root<NodeList>>;

    fn remove_self(self);

    fn get_unique_id(self) -> String;
    fn summarize(self) -> NodeInfo;

    fn teardown(self);

    fn parse_fragment(self, markup: DOMString) -> Fallible<Root<DocumentFragment>>;

}

impl<'a> NodeHelpers for &'a Node {
    fn teardown(self) {
        self.layout_data.dispose(self);
        for kid in self.children() {
            kid.r().teardown();
        }
    }

    /// Dumps the subtree rooted at this node, for debugging.
    fn dump(self) {
        self.dump_indent(0);
    }

    /// Dumps the node tree, for debugging, with indentation.
    fn dump_indent(self, indent: u32) {
        let mut s = String::new();
        for _ in 0..indent {
            s.push_str("    ");
        }

        s.push_str(&*self.debug_str());
        debug!("{:?}", s);

        // FIXME: this should have a pure version?
        for kid in self.children() {
            kid.r().dump_indent(indent + 1)
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

    // https://dom.spec.whatwg.org/#concept-node-length
    fn len(self) -> u32 {
        match self.type_id {
            NodeTypeId::DocumentType => 0,
            NodeTypeId::CharacterData(_) => {
                CharacterDataCast::to_ref(self).unwrap().Length()
            },
            _ => self.children_count(),
        }
    }

    // https://dom.spec.whatwg.org/#concept-tree-index
    fn index(self) -> u32 {
        self.preceding_siblings().count() as u32
    }

    fn children_count(self) -> u32 {
        self.children_count.get()
    }

    #[inline]
    fn is_anchor_element(self) -> bool {
        self.type_id == NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement))
    }

    #[inline]
    fn is_doctype(self) -> bool {
        self.type_id == NodeTypeId::DocumentType
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
        self.set_flag(IN_HOVER_STATE, state);
        self.dirty(NodeDamage::NodeStyleDamaged);
    }

    fn get_focus_state(self) -> bool {
        self.get_flag(IN_FOCUS_STATE)
    }

    fn set_focus_state(self, state: bool) {
        self.set_flag(IN_FOCUS_STATE, state);
        self.dirty(NodeDamage::NodeStyleDamaged);
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
        fn dirty_subtree(node: &Node) {
            // Stop if this subtree is already dirty.
            if node.get_is_dirty() { return }

            node.set_flag(IS_DIRTY | HAS_DIRTY_SIBLINGS | HAS_DIRTY_DESCENDANTS, true);

            for kid in node.children() {
                dirty_subtree(kid.r());
            }
        }

        dirty_subtree(self);

        // 3. Dirty siblings.
        //
        // TODO(cgaebel): This is a very conservative way to account for sibling
        // selectors. Maybe we can do something smarter in the future.
        if !self.get_has_dirty_siblings() {
            let parent =
                match self.parent_node.get() {
                    None         => return,
                    Some(parent) => parent,
                }.root();

            for sibling in parent.r().children() {
                sibling.r().set_has_dirty_siblings(true);
            }
        }

        // 4. Dirty ancestors.
        for ancestor in self.ancestors() {
            if !force_ancestors && ancestor.r().get_has_dirty_descendants() { break }
            ancestor.r().set_has_dirty_descendants(true);
        }
    }

    /// Iterates over this node and all its descendants, in preorder.
    fn traverse_preorder(self) -> TreeIterator {
        TreeIterator::new(self)
    }

    fn inclusively_following_siblings(self) -> NodeSiblingIterator {
        NodeSiblingIterator {
            current: Some(Root::from_ref(self)),
        }
    }

    fn inclusively_preceding_siblings(self) -> ReverseSiblingIterator {
        ReverseSiblingIterator {
            current: Some(Root::from_ref(self)),
        }
    }

    fn is_inclusive_ancestor_of(self, parent: &Node) -> bool {
        self == parent || parent.ancestors().any(|ancestor| ancestor.r() == self)
    }

    fn following_siblings(self) -> NodeSiblingIterator {
        NodeSiblingIterator {
            current: self.GetNextSibling(),
        }
    }

    fn preceding_siblings(self) -> ReverseSiblingIterator {
        ReverseSiblingIterator {
            current: self.GetPreviousSibling(),
        }
    }

    fn following_nodes(self, root: &Node) -> FollowingNodeIterator {
        FollowingNodeIterator {
            current: Some(Root::from_ref(self)),
            root: Root::from_ref(root),
        }
    }

    fn preceding_nodes(self, root: &Node) -> PrecedingNodeIterator {
        PrecedingNodeIterator {
            current: Some(Root::from_ref(self)),
            root: Root::from_ref(root),
        }
    }

    fn descending_last_children(self) -> LastChildIterator {
        LastChildIterator {
            current: self.GetLastChild(),
        }
    }

    fn is_parent_of(self, child: &Node) -> bool {
        match child.parent_node.get() {
            Some(ref parent) => parent.root().r() == self,
            None => false,
        }
    }

    fn to_trusted_node_address(self) -> TrustedNodeAddress {
        TrustedNodeAddress(&*self as *const Node as *const libc::c_void)
    }

    fn get_bounding_content_box(self) -> Rect<Au> {
        window_from_node(self).r().content_box_query(self.to_trusted_node_address())
    }

    fn get_content_boxes(self) -> Vec<Rect<Au>> {
        window_from_node(self).r().content_boxes_query(self.to_trusted_node_address())
    }

    fn get_client_rect(self) -> Rect<i32> {
        window_from_node(self).r().client_rect_query(self.to_trusted_node_address())
    }

    // https://dom.spec.whatwg.org/#dom-childnode-before
    fn before(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let parent = &self.parent_node;

        // Step 2.
        let parent = match parent.get() {
            None => return Ok(()),
            Some(ref parent) => parent.root(),
        };

        // Step 3.
        let viable_previous_sibling = first_node_not_in(self.preceding_siblings(), &nodes);

        // Step 4.
        let node = try!(self.owner_doc().node_from_nodes_and_strings(nodes));

        // Step 5.
        let viable_previous_sibling = match viable_previous_sibling {
            Some(ref viable_previous_sibling) => viable_previous_sibling.next_sibling.get(),
            None => parent.first_child.get(),
        }.map(|s| s.root());

        // Step 6.
        try!(Node::pre_insert(&node, &parent, viable_previous_sibling.r()));

        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-childnode-after
    fn after(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let parent = &self.parent_node;

        // Step 2.
        let parent = match parent.get() {
            None => return Ok(()),
            Some(ref parent) => parent.root(),
        };

        // Step 3.
        let viable_next_sibling = first_node_not_in(self.following_siblings(), &nodes);

        // Step 4.
        let node = try!(self.owner_doc().node_from_nodes_and_strings(nodes));

        // Step 5.
        try!(Node::pre_insert(&node, &parent, viable_next_sibling.r()));

        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-childnode-replacewith
    fn replace_with(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        match self.parent_node.get() {
            None => {
                // Step 1.
                Ok(())
            },
            Some(ref parent_node) => {
                // Step 2.
                let doc = self.owner_doc();
                let node = try!(doc.r().node_from_nodes_and_strings(nodes));
                // Step 3.
                parent_node.root().r().ReplaceChild(node.r(), self).map(|_| ())
            },
        }
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    fn prepend(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let doc = self.owner_doc();
        let node = try!(doc.r().node_from_nodes_and_strings(nodes));
        // Step 2.
        let first_child = self.first_child.get().map(Root::from_rooted);
        Node::pre_insert(node.r(), self, first_child.r()).map(|_| ())
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    fn append(self, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let doc = self.owner_doc();
        let node = try!(doc.r().node_from_nodes_and_strings(nodes));
        // Step 2.
        self.AppendChild(node.r()).map(|_| ())
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    fn query_selector(self, selectors: DOMString) -> Fallible<Option<Root<Element>>> {
        // Step 1.
        match parse_author_origin_selector_list_from_str(&selectors) {
            // Step 2.
            Err(()) => return Err(Syntax),
            // Step 3.
            Ok(ref selectors) => {
                let root = self.ancestors().last();
                let root = root.r().unwrap_or(self.clone());
                Ok(root.traverse_preorder().filter_map(ElementCast::to_root).find(|element| {
                    matches(selectors, element, None)
                }))
            }
        }
    }

    /// Get an iterator over all nodes which match a set of selectors
    /// Be careful not to do anything which may manipulate the DOM tree whilst iterating, otherwise
    /// the iterator may be invalidated
    #[allow(unsafe_code)]
    unsafe fn query_selector_iter(self, selectors: DOMString)
                                  -> Fallible<QuerySelectorIterator> {
        // Step 1.
        match parse_author_origin_selector_list_from_str(&selectors) {
            // Step 2.
            Err(()) => Err(Syntax),
            // Step 3.
            Ok(selectors) => {
                let root = self.ancestors().last();
                let root = root.r().unwrap_or(self);
                Ok(QuerySelectorIterator::new(root.traverse_preorder(), selectors))
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    #[allow(unsafe_code)]
    fn query_selector_all(self, selectors: DOMString) -> Fallible<Root<NodeList>> {
        let window = window_from_node(self);
        let iter = try!(unsafe { self.query_selector_iter(selectors) });
        Ok(NodeList::new_simple_list(window.r(), iter))
    }

    fn ancestors(self) -> AncestorIterator {
        AncestorIterator {
            current: self.GetParentNode()
        }
    }

    fn inclusive_ancestors(self) -> AncestorIterator {
        AncestorIterator {
            current: Some(Root::from_ref(self))
        }
    }

    fn owner_doc(self) -> Root<Document> {
        self.owner_doc.get().unwrap().root()
    }

    fn set_owner_doc(self, document: &Document) {
        self.owner_doc.set(Some(JS::from_ref(document)));
    }

    fn is_in_html_doc(self) -> bool {
        self.owner_doc().r().is_html_document()
    }

    fn children(self) -> NodeSiblingIterator {
        NodeSiblingIterator {
            current: self.GetFirstChild(),
        }
    }

    fn rev_children(self) -> ReverseSiblingIterator {
        ReverseSiblingIterator {
            current: self.GetLastChild(),
        }
    }

    fn child_elements(self) -> ChildElementIterator {
        fn to_temporary(node: Root<Node>) -> Option<Root<Element>> {
            ElementCast::to_root(node)
        }
        self.children()
            .filter_map(to_temporary as fn(_) -> _)
            .peekable()
    }

    fn remove_self(self) {
        if let Some(ref parent) = self.GetParentNode() {
            Node::remove(self, parent.r(), SuppressObserver::Unsuppressed);
        }
    }

    fn get_unique_id(self) -> String {
        if self.unique_id.borrow().is_empty() {
            let mut unique_id = self.unique_id.borrow_mut();
            *unique_id = uuid::Uuid::new_v4().to_simple_string();
        }
        self.unique_id.borrow().clone()
    }

    fn summarize(self) -> NodeInfo {
        NodeInfo {
            uniqueId: self.get_unique_id(),
            baseURI: self.BaseURI(),
            parent: self.GetParentNode().map(|node| node.r().get_unique_id()).unwrap_or("".to_owned()),
            nodeType: self.NodeType(),
            namespaceURI: "".to_owned(), //FIXME
            nodeName: self.NodeName(),
            numChildren: self.ChildNodes().r().Length() as usize,

            //FIXME doctype nodes only
            name: "".to_owned(),
            publicId: "".to_owned(),
            systemId: "".to_owned(),

            attrs: {
                let e: Option<&Element> = ElementCast::to_ref(self);
                match e {
                    Some(element) => element.summarize(),
                    None => vec!(),
                }
            },

            isDocumentElement:
                self.owner_doc()
                    .r()
                    .GetDocumentElement()
                    .map(|elem| NodeCast::from_ref(elem.r()) == self)
                    .unwrap_or(false),

            shortValue: self.GetNodeValue().unwrap_or("".to_owned()), //FIXME: truncate
            incompleteValue: false, //FIXME: reflect truncation
        }
    }

    // https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#dfn-concept-parse-fragment
    fn parse_fragment(self, markup: DOMString) -> Fallible<Root<DocumentFragment>> {
        let context_node: &Node = NodeCast::from_ref(self);
        let context_document = document_from_node(self);
        let fragment = DocumentFragment::new(context_document.r());
        if context_document.r().is_html_document() {
            let fragment_node = NodeCast::from_ref(fragment.r());
            parse_html_fragment(context_node, markup, fragment_node);
        } else {
            // FIXME: XML case
            unimplemented!();
        }
        Ok(fragment)
    }
}


/// Iterate through `nodes` until we find a `Node` that is not in `not_in`
fn first_node_not_in<I>(mut nodes: I, not_in: &[NodeOrString]) -> Option<Root<Node>>
        where I: Iterator<Item=Root<Node>>
{
    nodes.find(|node| {
        not_in.iter().all(|n| {
            match n {
                &NodeOrString::eNode(ref n) => n != node,
                _ => true,
            }
        })
    })
}

/// If the given untrusted node address represents a valid DOM node in the given runtime,
/// returns it.
#[allow(unsafe_code)]
pub fn from_untrusted_node_address(_runtime: *mut JSRuntime, candidate: UntrustedNodeAddress)
    -> Root<Node> {
    unsafe {
        // https://github.com/servo/servo/issues/6383
        let candidate: uintptr_t = mem::transmute(candidate.0);
//        let object: *mut JSObject = jsfriendapi::bindgen::JS_GetAddressableObject(runtime,
//                                                                                  candidate);
        let object: *mut JSObject = mem::transmute(candidate);
        if object.is_null() {
            panic!("Attempted to create a `JS<Node>` from an invalid pointer!")
        }
        let boxed_node: *const Node = conversions::native_from_reflector(object);
        Root::from_ref(&*boxed_node)
    }
}

#[allow(unsafe_code)]
pub trait LayoutNodeHelpers {
    unsafe fn type_id_for_layout(&self) -> NodeTypeId;

    unsafe fn parent_node_ref(&self) -> Option<LayoutJS<Node>>;
    unsafe fn first_child_ref(&self) -> Option<LayoutJS<Node>>;
    unsafe fn last_child_ref(&self) -> Option<LayoutJS<Node>>;
    unsafe fn prev_sibling_ref(&self) -> Option<LayoutJS<Node>>;
    unsafe fn next_sibling_ref(&self) -> Option<LayoutJS<Node>>;

    unsafe fn owner_doc_for_layout(&self) -> LayoutJS<Document>;

    unsafe fn is_element_for_layout(&self) -> bool;
    unsafe fn get_flag(&self, flag: NodeFlags) -> bool;
    unsafe fn set_flag(&self, flag: NodeFlags, value: bool);

    unsafe fn children_count(&self) -> u32;

    unsafe fn layout_data(&self) -> Ref<Option<LayoutData>>;
    unsafe fn layout_data_mut(&self) -> RefMut<Option<LayoutData>>;
    unsafe fn layout_data_unchecked(&self) -> *const Option<LayoutData>;

    fn get_hover_state_for_layout(&self) -> bool;
    fn get_focus_state_for_layout(&self) -> bool;
    fn get_disabled_state_for_layout(&self) -> bool;
    fn get_enabled_state_for_layout(&self) -> bool;
}

impl LayoutNodeHelpers for LayoutJS<Node> {
    #[inline]
    #[allow(unsafe_code)]
    unsafe fn type_id_for_layout(&self) -> NodeTypeId {
        (*self.unsafe_get()).type_id
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn is_element_for_layout(&self) -> bool {
        (*self.unsafe_get()).is_element()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn parent_node_ref(&self) -> Option<LayoutJS<Node>> {
        (*self.unsafe_get()).parent_node.get_inner_as_layout()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn first_child_ref(&self) -> Option<LayoutJS<Node>> {
        (*self.unsafe_get()).first_child.get_inner_as_layout()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn last_child_ref(&self) -> Option<LayoutJS<Node>> {
        (*self.unsafe_get()).last_child.get_inner_as_layout()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn prev_sibling_ref(&self) -> Option<LayoutJS<Node>> {
        (*self.unsafe_get()).prev_sibling.get_inner_as_layout()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn next_sibling_ref(&self) -> Option<LayoutJS<Node>> {
        (*self.unsafe_get()).next_sibling.get_inner_as_layout()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn owner_doc_for_layout(&self) -> LayoutJS<Document> {
        (*self.unsafe_get()).owner_doc.get_inner_as_layout().unwrap()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn get_flag(&self, flag: NodeFlags) -> bool {
        (*self.unsafe_get()).flags.get().contains(flag)
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn set_flag(&self, flag: NodeFlags, value: bool) {
        let this = self.unsafe_get();
        let mut flags = (*this).flags.get();

        if value {
            flags.insert(flag);
        } else {
            flags.remove(flag);
        }

        (*this).flags.set(flags);
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn children_count(&self) -> u32 {
        (*self.unsafe_get()).children_count.get()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn layout_data(&self) -> Ref<Option<LayoutData>> {
        (*self.unsafe_get()).layout_data.borrow()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn layout_data_mut(&self) -> RefMut<Option<LayoutData>> {
        (*self.unsafe_get()).layout_data.borrow_mut()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn layout_data_unchecked(&self) -> *const Option<LayoutData> {
        (*self.unsafe_get()).layout_data.borrow_unchecked()
    }

    #[inline]
    #[allow(unsafe_code)]
    fn get_hover_state_for_layout(&self) -> bool {
        unsafe {
            self.get_flag(IN_HOVER_STATE)
        }
    }
    #[inline]
    #[allow(unsafe_code)]
    fn get_focus_state_for_layout(&self) -> bool {
        unsafe {
            self.get_flag(IN_FOCUS_STATE)
        }
    }
    #[inline]
    #[allow(unsafe_code)]
    fn get_disabled_state_for_layout(&self) -> bool {
        unsafe {
            self.get_flag(IN_DISABLED_STATE)
        }
    }
    #[inline]
    #[allow(unsafe_code)]
    fn get_enabled_state_for_layout(&self) -> bool {
        unsafe {
            self.get_flag(IN_ENABLED_STATE)
        }
    }
}


//
// Iteration and traversal
//

pub type ChildElementIterator =
    Peekable<FilterMap<NodeSiblingIterator,
                       fn(Root<Node>) -> Option<Root<Element>>>>;

pub struct NodeSiblingIterator {
    current: Option<Root<Node>>,
}

impl Iterator for NodeSiblingIterator {
    type Item = Root<Node>;

    fn next(&mut self) -> Option<Root<Node>> {
        let current = match self.current.take() {
            None => return None,
            Some(current) => current,
        };
        self.current = current.r().GetNextSibling();
        Some(current)
    }
}

pub struct ReverseSiblingIterator {
    current: Option<Root<Node>>,
}

impl Iterator for ReverseSiblingIterator {
    type Item = Root<Node>;

    fn next(&mut self) -> Option<Root<Node>> {
        let current = match self.current.take() {
            None => return None,
            Some(current) => current,
        };
        self.current = current.r().GetPreviousSibling();
        Some(current)
    }
}

pub struct FollowingNodeIterator {
    current: Option<Root<Node>>,
    root: Root<Node>,
}

impl Iterator for FollowingNodeIterator {
    type Item = Root<Node>;

    // https://dom.spec.whatwg.org/#concept-tree-following
    fn next(&mut self) -> Option<Root<Node>> {
        let current = match self.current.take() {
            None => return None,
            Some(current) => current,
        };

        if let Some(first_child) = current.r().GetFirstChild() {
            self.current = Some(first_child);
            return current.r().GetFirstChild()
        }

        if self.root == current {
            self.current = None;
            return None;
        }

        if let Some(next_sibling) = current.r().GetNextSibling() {
            self.current = Some(next_sibling);
            return current.r().GetNextSibling()
        }

        for ancestor in current.r().inclusive_ancestors() {
            if self.root == ancestor {
                break;
            }
            if let Some(next_sibling) = ancestor.r().GetNextSibling() {
                self.current = Some(next_sibling);
                return ancestor.r().GetNextSibling()
            }
        }
        self.current = None;
        return None
    }
}

pub struct PrecedingNodeIterator {
    current: Option<Root<Node>>,
    root: Root<Node>,
}

impl Iterator for PrecedingNodeIterator {
    type Item = Root<Node>;

    // https://dom.spec.whatwg.org/#concept-tree-preceding
    fn next(&mut self) -> Option<Root<Node>> {
        let current = match self.current.take() {
            None => return None,
            Some(current) => current,
        };

        if self.root == current {
            self.current = None;
            return None
        }

        let node = current;
        if let Some(previous_sibling) = node.r().GetPreviousSibling() {
            if self.root == previous_sibling {
                self.current = None;
                return None
            }

            if let Some(last_child) = previous_sibling.r().descending_last_children().last() {
                self.current = Some(last_child);
                return previous_sibling.r().descending_last_children().last()
            }

            self.current = Some(previous_sibling);
            return node.r().GetPreviousSibling()
        };

        if let Some(parent_node) = node.r().GetParentNode() {
            self.current = Some(parent_node);
            return node.r().GetParentNode()
        }

        self.current = None;
        return None
    }
}

pub struct LastChildIterator {
    current: Option<Root<Node>>,
}

impl Iterator for LastChildIterator {
    type Item = Root<Node>;

    fn next(&mut self) -> Option<Root<Node>> {
        let current = match self.current.take() {
            None => return None,
            Some(current) => current,
        };
        self.current = current.r().GetLastChild();
        Some(current)
    }
}

pub struct AncestorIterator {
    current: Option<Root<Node>>,
}

impl Iterator for AncestorIterator {
    type Item = Root<Node>;

    fn next(&mut self) -> Option<Root<Node>> {
        let current = match self.current.take() {
            None => return None,
            Some(current) => current,
        };
        self.current = current.r().GetParentNode();
        Some(current)
    }
}

pub struct TreeIterator {
    current: Option<Root<Node>>,
    depth: usize,
}

impl TreeIterator {
    fn new(root: &Node) -> TreeIterator {
        TreeIterator {
            current: Some(Root::from_ref(root)),
            depth: 0,
        }
    }
}

impl Iterator for TreeIterator {
    type Item = Root<Node>;

    // https://dom.spec.whatwg.org/#concept-tree-order
    fn next(&mut self) -> Option<Root<Node>> {
        let current = match self.current.take() {
            None => return None,
            Some(current) => current,
        };
        if let Some(first_child) = current.r().GetFirstChild() {
            self.current = Some(first_child);
            self.depth += 1;
            return Some(current);
        };
        for ancestor in current.r().inclusive_ancestors() {
            if self.depth == 0 {
                break;
            }
            if let Some(next_sibling) = ancestor.r().GetNextSibling() {
                self.current = Some(next_sibling);
                return Some(current);
            }
            self.depth -= 1;
        }
        debug_assert!(self.depth == 0);
        self.current = None;
        Some(current)
    }
}

/// Specifies whether children must be recursively cloned or not.
#[derive(Copy, Clone, PartialEq, HeapSizeOf)]
pub enum CloneChildrenFlag {
    CloneChildren,
    DoNotCloneChildren
}

fn as_uintptr<T>(t: &T) -> uintptr_t { t as *const T as uintptr_t }

impl Node {
    pub fn reflect_node<N: Reflectable+NodeBase>
            (node:      Box<N>,
             document:  &Document,
             wrap_fn:   extern "Rust" fn(*mut JSContext, GlobalRef, Box<N>) -> Root<N>)
             -> Root<N> {
        let window = document.window();
        reflect_dom_object(node, GlobalRef::Window(window.r()), wrap_fn)
    }

    pub fn new_inherited(type_id: NodeTypeId, doc: &Document) -> Node {
        Node::new_(type_id, Some(doc.clone()))
    }

    pub fn new_without_doc(type_id: NodeTypeId) -> Node {
        Node::new_(type_id, None)
    }

    fn new_(type_id: NodeTypeId, doc: Option<&Document>) -> Node {
        Node {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::Node(type_id)),
            type_id: type_id,

            parent_node: Default::default(),
            first_child: Default::default(),
            last_child: Default::default(),
            next_sibling: Default::default(),
            prev_sibling: Default::default(),
            owner_doc: MutNullableHeap::new(doc.map(JS::from_ref)),
            child_list: Default::default(),
            children_count: Cell::new(0u32),
            flags: Cell::new(NodeFlags::new(type_id)),

            layout_data: LayoutDataRef::new(),

            unique_id: DOMRefCell::new(String::new()),
        }
    }

    // https://dom.spec.whatwg.org/#concept-node-adopt
    pub fn adopt(node: &Node, document: &Document) {
        // Step 1.
        let parent_node = node.GetParentNode();
        match parent_node {
            Some(ref parent) => {
                Node::remove(node, parent, SuppressObserver::Unsuppressed);
            }
            None => (),
        }

        // Step 2.
        let node_doc = document_from_node(node);
        if node_doc.r() != document {
            for descendant in node.traverse_preorder() {
                descendant.r().set_owner_doc(document);
            }
        }

        // Step 3.
        // If node is an element, it is _affected by a base URL change_.
    }

    // https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity
    pub fn ensure_pre_insertion_validity(node: &Node,
                                         parent: &Node,
                                         child: Option<&Node>) -> ErrorResult {
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
        if let Some(child) = child {
            if !parent.is_parent_of(child) {
                return Err(NotFound);
            }
        }

        // Step 4-5.
        match node.type_id() {
            NodeTypeId::CharacterData(CharacterDataTypeId::Text) => {
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
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) |
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => (),
            NodeTypeId::Document => return Err(HierarchyRequest)
        }

        // Step 6.
        if parent.type_id() == NodeTypeId::Document {
            match node.type_id() {
                // Step 6.1
                NodeTypeId::DocumentFragment => {
                    // Step 6.1.1(b)
                    if node.children()
                           .any(|c| c.r().is_text())
                    {
                        return Err(HierarchyRequest);
                    }
                    match node.child_elements().count() {
                        0 => (),
                        // Step 6.1.2
                        1 => {
                            if !parent.child_elements().is_empty() {
                                return Err(HierarchyRequest);
                            }
                            if let Some(child) = child {
                                if child.inclusively_following_siblings()
                                    .any(|child| child.r().is_doctype()) {
                                        return Err(HierarchyRequest);
                                }
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
                    if let Some(ref child) = child {
                        if child.inclusively_following_siblings()
                            .any(|child| child.r().is_doctype()) {
                                return Err(HierarchyRequest);
                        }
                    }
                },
                // Step 6.3
                NodeTypeId::DocumentType => {
                    if parent.children()
                             .any(|c| c.r().is_doctype())
                    {
                        return Err(HierarchyRequest);
                    }
                    match child {
                        Some(child) => {
                            if parent.children()
                                     .take_while(|c| c.r() != child)
                                     .any(|c| c.r().is_element())
                            {
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
                NodeTypeId::CharacterData(_) => (),
                NodeTypeId::Document => unreachable!(),
            }
        }
        Ok(())
    }

    // https://dom.spec.whatwg.org/#concept-node-pre-insert
    pub fn pre_insert(node: &Node, parent: &Node, child: Option<&Node>)
                      -> Fallible<Root<Node>> {
        // Step 1.
        try!(Node::ensure_pre_insertion_validity(node, parent, child));

        // Steps 2-3.
        let reference_child_root;
        let reference_child = match child {
            Some(child) if child == node => {
                reference_child_root = node.GetNextSibling();
                reference_child_root.r()
            },
            _ => child
        };

        // Step 4.
        let document = document_from_node(parent);
        Node::adopt(node, document.r());

        // Step 5.
        Node::insert(node, parent, reference_child, SuppressObserver::Unsuppressed);

        // Step 6.
        return Ok(Root::from_ref(node))
    }

    // https://dom.spec.whatwg.org/#concept-node-insert
    fn insert(node: &Node,
              parent: &Node,
              child: Option<&Node>,
              suppress_observers: SuppressObserver) {
        debug_assert!(&*node.owner_doc() == &*parent.owner_doc());
        debug_assert!(child.map_or(true, |child| Some(parent) == child.GetParentNode().r()));

        // Steps 1-2: ranges.
        let mut new_nodes = RootedVec::new();
        let new_nodes = if let NodeTypeId::DocumentFragment = node.type_id() {
            // Step 3.
            new_nodes.extend(node.children().map(|kid| JS::from_rooted(&kid)));
            // Step 4: mutation observers.
            // Step 5.
            for kid in new_nodes.r() {
                Node::remove(*kid, node, SuppressObserver::Suppressed);
            }
            vtable_for(&node).children_changed(&ChildrenMutation::replace_all(new_nodes.r(), &[]));
            new_nodes.r()
        } else {
            // Step 3.
            ref_slice(&node)
        };
        // Step 6: mutation observers.
        let previous_sibling = match suppress_observers {
            SuppressObserver::Unsuppressed => {
                match child {
                    Some(child) => child.GetPreviousSibling(),
                    None => parent.GetLastChild(),
                }
            },
            SuppressObserver::Suppressed => None,
        };
        // Step 7.
        for kid in new_nodes {
            // Step 7.1.
            parent.add_child(*kid, child);
            // Step 7.2: insertion steps.
        }
        if let SuppressObserver::Unsuppressed = suppress_observers {
            vtable_for(&parent).children_changed(
                &ChildrenMutation::insert(previous_sibling.r(), new_nodes, child));
        }
    }

    // https://dom.spec.whatwg.org/#concept-node-replace-all
    pub fn replace_all(node: Option<&Node>, parent: &Node) {
        // Step 1.
        if let Some(node) = node {
            Node::adopt(node, &*parent.owner_doc());
        }
        // Step 2.
        let mut removed_nodes = RootedVec::new();
        removed_nodes.extend(parent.children().map(|child| JS::from_rooted(&child)));
        // Step 3.
        let mut added_nodes = RootedVec::new();
        let added_nodes = if let Some(node) = node.as_ref() {
            if let NodeTypeId::DocumentFragment = node.type_id() {
                added_nodes.extend(node.children().map(|child| JS::from_rooted(&child)));
                added_nodes.r()
            } else {
                ref_slice(node)
            }
        } else {
            &[] as &[&Node]
        };
        // Step 4.
        for child in removed_nodes.r() {
            Node::remove(*child, parent, SuppressObserver::Suppressed);
        }
        // Step 5.
        if let Some(node) = node {
            Node::insert(node, parent, None, SuppressObserver::Suppressed);
        }
        // Step 6: mutation observers.
        vtable_for(&parent).children_changed(
            &ChildrenMutation::replace_all(removed_nodes.r(), added_nodes));
    }

    // https://dom.spec.whatwg.org/#concept-node-pre-remove
    fn pre_remove(child: &Node, parent: &Node) -> Fallible<Root<Node>> {
        // Step 1.
        match child.GetParentNode() {
            Some(ref node) if node.r() != parent => return Err(NotFound),
            None => return Err(NotFound),
            _ => ()
        }

        // Step 2.
        Node::remove(child, parent, SuppressObserver::Unsuppressed);

        // Step 3.
        Ok(Root::from_ref(child))
    }

    // https://dom.spec.whatwg.org/#concept-node-remove
    fn remove(node: &Node, parent: &Node, suppress_observers: SuppressObserver) {
        assert!(node.GetParentNode().map_or(false, |node_parent| node_parent.r() == parent));

        // Step 1-5: ranges.
        // Step 6.
        let old_previous_sibling = node.GetPreviousSibling();
        // Steps 7-8: mutation observers.
        // Step 9.
        let old_next_sibling = node.GetNextSibling();
        parent.remove_child(node);
        if let SuppressObserver::Unsuppressed = suppress_observers {
            vtable_for(&parent).children_changed(
                &ChildrenMutation::replace(old_previous_sibling.r(),
                                           &node, &[],
                                           old_next_sibling.r()));
        }
    }

    // https://dom.spec.whatwg.org/#concept-node-clone
    pub fn clone(node: &Node, maybe_doc: Option<&Document>,
                 clone_children: CloneChildrenFlag) -> Root<Node> {

        // Step 1.
        let document = match maybe_doc {
            Some(doc) => Root::from_ref(doc),
            None => node.owner_doc()
        };

        // Step 2.
        // XXXabinader: clone() for each node as trait?
        let copy: Root<Node> = match node.type_id() {
            NodeTypeId::DocumentType => {
                let doctype: &DocumentType = DocumentTypeCast::to_ref(node).unwrap();
                let doctype = DocumentType::new(doctype.name().clone(),
                                                Some(doctype.public_id().clone()),
                                                Some(doctype.system_id().clone()), document.r());
                NodeCast::from_root(doctype)
            },
            NodeTypeId::DocumentFragment => {
                let doc_fragment = DocumentFragment::new(document.r());
                NodeCast::from_root(doc_fragment)
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => {
                let cdata = CharacterDataCast::to_ref(node).unwrap();
                let comment = Comment::new(cdata.Data(), document.r());
                NodeCast::from_root(comment)
            },
            NodeTypeId::Document => {
                let document = DocumentCast::to_ref(node).unwrap();
                let is_html_doc = match document.is_html_document() {
                    true => IsHTMLDocument::HTMLDocument,
                    false => IsHTMLDocument::NonHTMLDocument,
                };
                let window = document.window();
                let loader = DocumentLoader::new(&*document.loader());
                let document = Document::new(window.r(), Some(document.url()),
                                             is_html_doc, None,
                                             None, DocumentSource::NotFromParser, loader);
                NodeCast::from_root(document)
            },
            NodeTypeId::Element(..) => {
                let element = ElementCast::to_ref(node).unwrap();
                let name = QualName {
                    ns: element.namespace().clone(),
                    local: element.local_name().clone()
                };
                let element = Element::create(name,
                    element.prefix().as_ref().map(|p| Atom::from_slice(&p)),
                    document.r(), ElementCreator::ScriptCreated);
                NodeCast::from_root(element)
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::Text) => {
                let cdata = CharacterDataCast::to_ref(node).unwrap();
                let text = Text::new(cdata.Data(), document.r());
                NodeCast::from_root(text)
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) => {
                let pi = ProcessingInstructionCast::to_ref(node).unwrap();
                let pi = ProcessingInstruction::new(pi.Target(),
                                                    CharacterDataCast::from_ref(pi).Data(), document.r());
                NodeCast::from_root(pi)
            },
        };

        // Step 3.
        let document = match DocumentCast::to_ref(copy.r()) {
            Some(doc) => Root::from_ref(doc),
            None => Root::from_ref(document.r()),
        };
        assert!(copy.r().owner_doc() == document);

        // Step 4 (some data already copied in step 2).
        match node.type_id() {
            NodeTypeId::Document => {
                let node_doc = DocumentCast::to_ref(node).unwrap();
                let copy_doc = DocumentCast::to_ref(copy.r()).unwrap();
                copy_doc.set_encoding_name(node_doc.encoding_name().clone());
                copy_doc.set_quirks_mode(node_doc.quirks_mode());
            },
            NodeTypeId::Element(..) => {
                let node_elem = ElementCast::to_ref(node).unwrap();
                let copy_elem = ElementCast::to_ref(copy.r()).unwrap();

                let window = document.r().window();
                for ref attr in node_elem.attrs().iter() {
                    let attr = attr.root();
                    let newattr =
                        Attr::new(window.r(),
                                  attr.r().local_name().clone(), attr.r().value().clone(),
                                  attr.r().name().clone(), attr.r().namespace().clone(),
                                  attr.r().prefix().clone(), Some(copy_elem));
                    copy_elem.attrs_mut().push(JS::from_rooted(&newattr));
                }
            },
            _ => ()
        }

        // Step 5: cloning steps.
        vtable_for(&node).cloning_steps(copy.r(), maybe_doc, clone_children);

        // Step 6.
        if clone_children == CloneChildrenFlag::CloneChildren {
            for child in node.children() {
                let child_copy = Node::clone(child.r(), Some(document.r()),
                                             clone_children);
                let _inserted_node = Node::pre_insert(child_copy.r(), copy.r(), None);
            }
        }

        // Step 7.
        copy
    }

    pub fn collect_text_contents<T: Iterator<Item=Root<Node>>>(iterator: T) -> String {
        let mut content = String::new();
        for node in iterator {
            let text = TextCast::to_ref(node.r());
            match text {
                Some(text) => content.push_str(&CharacterDataCast::from_ref(text).Data()),
                None => (),
            }
        }
        content
    }

    pub fn namespace_to_string(namespace: Namespace) -> Option<DOMString> {
        match namespace {
            ns!("") => None,
            Namespace(ref ns) => Some((**ns).to_owned())
        }
    }

    // https://dom.spec.whatwg.org/#locate-a-namespace
    pub fn locate_namespace(node: &Node, prefix: Option<DOMString>) -> Namespace {
        fn attr_defines_namespace(attr: &Attr,
                                  prefix: &Option<Atom>) -> bool {
            *attr.namespace() == ns!(XMLNS) &&
                match (attr.prefix(), prefix) {
                    (&Some(ref attr_prefix), &Some(ref prefix)) =>
                        attr_prefix == &atom!("xmlns") &&
                            attr.local_name() == prefix,
                    (&None, &None) => *attr.local_name() == atom!("xmlns"),
                    _ => false
                }
        }

        match node.type_id {
            NodeTypeId::Element(_) => {
                let element = ElementCast::to_ref(node).unwrap();
                // Step 1.
                if *element.namespace() != ns!("") && *element.prefix() == prefix {
                    return element.namespace().clone()
                }


                let prefix_atom = prefix.as_ref().map(|s| Atom::from_slice(s));

                // Step 2.
                let namespace_attr =
                    element.attrs()
                           .iter()
                           .map(|attr| attr.root())
                           .find(|attr| attr_defines_namespace(attr.r(),
                                                               &prefix_atom));

                // Steps 2.1-2.
                if let Some(attr) = namespace_attr {
                    return namespace_from_domstring(Some(attr.Value()));
                }

                match node.GetParentElement() {
                    // Step 3.
                    None => ns!(""),
                    // Step 4.
                    Some(parent) => Node::locate_namespace(NodeCast::from_ref(parent.r()), prefix)
                }
            },
            NodeTypeId::Document => {
                match DocumentCast::to_ref(node).unwrap().GetDocumentElement().r() {
                    // Step 1.
                    None => ns!(""),
                    // Step 2.
                    Some(document_element) => {
                        Node::locate_namespace(NodeCast::from_ref(document_element), prefix)
                    }
                }
            },
            NodeTypeId::DocumentType => ns!(""),
            NodeTypeId::DocumentFragment => ns!(""),
            _ => match node.GetParentElement() {
                     // Step 1.
                     None => ns!(""),
                     // Step 2.
                     Some(parent) => Node::locate_namespace(NodeCast::from_ref(parent.r()), prefix)
                 }
        }
    }
}

impl<'a> NodeMethods for &'a Node {
    // https://dom.spec.whatwg.org/#dom-node-nodetype
    fn NodeType(self) -> u16 {
        match self.type_id {
            NodeTypeId::CharacterData(CharacterDataTypeId::Text) =>
                NodeConstants::TEXT_NODE,
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) =>
                NodeConstants::PROCESSING_INSTRUCTION_NODE,
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) =>
                NodeConstants::COMMENT_NODE,
            NodeTypeId::Document =>
                NodeConstants::DOCUMENT_NODE,
            NodeTypeId::DocumentType =>
                NodeConstants::DOCUMENT_TYPE_NODE,
            NodeTypeId::DocumentFragment =>
                NodeConstants::DOCUMENT_FRAGMENT_NODE,
            NodeTypeId::Element(_) =>
                NodeConstants::ELEMENT_NODE,
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-nodename
    fn NodeName(self) -> DOMString {
        match self.type_id {
            NodeTypeId::Element(..) => {
                let elem: &Element = ElementCast::to_ref(self).unwrap();
                elem.TagName()
            }
            NodeTypeId::CharacterData(CharacterDataTypeId::Text) => "#text".to_owned(),
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) => {
                let processing_instruction: &ProcessingInstruction =
                    ProcessingInstructionCast::to_ref(self).unwrap();
                processing_instruction.Target()
            }
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => "#comment".to_owned(),
            NodeTypeId::DocumentType => {
                let doctype: &DocumentType = DocumentTypeCast::to_ref(self).unwrap();
                doctype.name().clone()
            },
            NodeTypeId::DocumentFragment => "#document-fragment".to_owned(),
            NodeTypeId::Document => "#document".to_owned()
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-baseuri
    fn BaseURI(self) -> DOMString {
        self.owner_doc().URL()
    }

    // https://dom.spec.whatwg.org/#dom-node-ownerdocument
    fn GetOwnerDocument(self) -> Option<Root<Document>> {
        match self.type_id {
            NodeTypeId::CharacterData(..) |
            NodeTypeId::Element(..) |
            NodeTypeId::DocumentType |
            NodeTypeId::DocumentFragment => Some(self.owner_doc()),
            NodeTypeId::Document => None
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-parentnode
    fn GetParentNode(self) -> Option<Root<Node>> {
        self.parent_node.get().map(Root::from_rooted)
    }

    // https://dom.spec.whatwg.org/#dom-node-parentelement
    fn GetParentElement(self) -> Option<Root<Element>> {
        self.GetParentNode().and_then(ElementCast::to_root)
    }

    // https://dom.spec.whatwg.org/#dom-node-haschildnodes
    fn HasChildNodes(self) -> bool {
        self.first_child.get().is_some()
    }

    // https://dom.spec.whatwg.org/#dom-node-childnodes
    fn ChildNodes(self) -> Root<NodeList> {
        self.child_list.or_init(|| {
            let doc = self.owner_doc();
            let window = doc.r().window();
            NodeList::new_child_list(window.r(), self)
        })
    }

    // https://dom.spec.whatwg.org/#dom-node-firstchild
    fn GetFirstChild(self) -> Option<Root<Node>> {
        self.first_child.get().map(Root::from_rooted)
    }

    // https://dom.spec.whatwg.org/#dom-node-lastchild
    fn GetLastChild(self) -> Option<Root<Node>> {
        self.last_child.get().map(Root::from_rooted)
    }

    // https://dom.spec.whatwg.org/#dom-node-previoussibling
    fn GetPreviousSibling(self) -> Option<Root<Node>> {
        self.prev_sibling.get().map(Root::from_rooted)
    }

    // https://dom.spec.whatwg.org/#dom-node-nextsibling
    fn GetNextSibling(self) -> Option<Root<Node>> {
        self.next_sibling.get().map(Root::from_rooted)
    }

    // https://dom.spec.whatwg.org/#dom-node-nodevalue
    fn GetNodeValue(self) -> Option<DOMString> {
        match self.type_id {
            NodeTypeId::CharacterData(..) => {
                let chardata: &CharacterData = CharacterDataCast::to_ref(self).unwrap();
                Some(chardata.Data())
            }
            _ => {
                None
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-nodevalue
    fn SetNodeValue(self, val: Option<DOMString>) {
        match self.type_id {
            NodeTypeId::CharacterData(..) => {
                self.SetTextContent(val)
            }
            _ => {}
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-textcontent
    fn GetTextContent(self) -> Option<DOMString> {
        match self.type_id {
            NodeTypeId::DocumentFragment |
            NodeTypeId::Element(..) => {
                let content = Node::collect_text_contents(self.traverse_preorder());
                Some(content)
            }
            NodeTypeId::CharacterData(..) => {
                let characterdata: &CharacterData = CharacterDataCast::to_ref(self).unwrap();
                Some(characterdata.Data())
            }
            NodeTypeId::DocumentType |
            NodeTypeId::Document => {
                None
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-textcontent
    fn SetTextContent(self, value: Option<DOMString>) {
        let value = value.unwrap_or(String::new());
        match self.type_id {
            NodeTypeId::DocumentFragment |
            NodeTypeId::Element(..) => {
                // Step 1-2.
                let node = if value.is_empty() {
                    None
                } else {
                    let document = self.owner_doc();
                    Some(NodeCast::from_root(document.r().CreateTextNode(value)))
                };

                // Step 3.
                Node::replace_all(node.r(), self);
            }
            NodeTypeId::CharacterData(..) => {
                let characterdata: &CharacterData = CharacterDataCast::to_ref(self).unwrap();
                characterdata.SetData(value);

                // Notify the document that the content of this node is different
                let document = self.owner_doc();
                document.r().content_changed(self, NodeDamage::OtherNodeDamage);
            }
            NodeTypeId::DocumentType |
            NodeTypeId::Document => {}
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-insertbefore
    fn InsertBefore(self, node: &Node, child: Option<&Node>) -> Fallible<Root<Node>> {
        Node::pre_insert(node, self, child)
    }

    // https://dom.spec.whatwg.org/#dom-node-appendchild
    fn AppendChild(self, node: &Node) -> Fallible<Root<Node>> {
        Node::pre_insert(node, self, None)
    }

    // https://dom.spec.whatwg.org/#concept-node-replace
    fn ReplaceChild(self, node: &Node, child: &Node) -> Fallible<Root<Node>> {

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
            NodeTypeId::CharacterData(CharacterDataTypeId::Text) if self.is_document() =>
                return Err(HierarchyRequest),
            NodeTypeId::DocumentType if !self.is_document() => return Err(HierarchyRequest),
            NodeTypeId::Document => return Err(HierarchyRequest),
            _ => ()
        }

        // Step 6.
        if self.is_document() {
            match node.type_id() {
                // Step 6.1
                NodeTypeId::DocumentFragment => {
                    // Step 6.1.1(b)
                    if node.children()
                           .any(|c| c.is_text())
                    {
                        return Err(HierarchyRequest);
                    }
                    match node.child_elements().count() {
                        0 => (),
                        // Step 6.1.2
                        1 => {
                            if self.child_elements()
                                   .any(|c| NodeCast::from_ref(c.r()) != child) {
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
                    if self.child_elements()
                           .any(|c| NodeCast::from_ref(c.r()) != child) {
                        return Err(HierarchyRequest);
                    }
                    if child.following_siblings()
                            .any(|child| child.is_doctype())
                    {
                        return Err(HierarchyRequest);
                    }
                },
                // Step 6.3
                NodeTypeId::DocumentType => {
                    if self.children()
                           .any(|c| c.is_doctype() &&
                                c.r() != child)
                    {
                        return Err(HierarchyRequest);
                    }
                    if self.children()
                           .take_while(|c| c.r() != child)
                           .any(|c| c.is_element())
                    {
                        return Err(HierarchyRequest);
                    }
                },
                NodeTypeId::CharacterData(..) => (),
                NodeTypeId::Document => unreachable!()
            }
        }

        // Ok if not caught by previous error checks.
        if node == child {
            return Ok(Root::from_ref(child));
        }

        // Step 7-8.
        let child_next_sibling = child.GetNextSibling();
        let node_next_sibling = node.GetNextSibling();
        let reference_child = if child_next_sibling.r() == Some(node) {
            node_next_sibling.r()
        } else {
            child_next_sibling.r()
        };

        // Step 9.
        let document = document_from_node(self);
        Node::adopt(node, document.r());

        // Step 10.
        let previous_sibling = child.GetPreviousSibling();

        // Step 11.
        Node::remove(child, self, SuppressObserver::Suppressed);

        // Step 12.
        let mut nodes = RootedVec::new();
        let nodes = if node.type_id() == NodeTypeId::DocumentFragment {
            nodes.extend(node.children().map(|node| JS::from_rooted(&node)));
            nodes.r()
        } else {
            ref_slice(&node)
        };

        // Step 13.
        Node::insert(node, self, reference_child, SuppressObserver::Suppressed);

        // Step 14.
        vtable_for(&self).children_changed(
            &ChildrenMutation::replace(previous_sibling.r(),
                                       &child, nodes,
                                       reference_child));

        // Step 15.
        Ok(Root::from_ref(child))
    }

    // https://dom.spec.whatwg.org/#dom-node-removechild
    fn RemoveChild(self, node: &Node)
                       -> Fallible<Root<Node>> {
        Node::pre_remove(node, self)
    }

    // https://dom.spec.whatwg.org/#dom-node-normalize
    fn Normalize(self) {
        let mut prev_text: Option<Root<Text>> = None;
        for child in self.children() {
            match TextCast::to_ref(child.r()) {
                Some(text) => {
                    let characterdata: &CharacterData = CharacterDataCast::from_ref(text);
                    if characterdata.Length() == 0 {
                        Node::remove(&*child, self, SuppressObserver::Unsuppressed);
                    } else {
                        match prev_text {
                            Some(ref text_node) => {
                                let prev_characterdata =
                                    CharacterDataCast::from_ref(text_node.r());
                                prev_characterdata.append_data(&**characterdata.data());
                                Node::remove(&*child, self, SuppressObserver::Unsuppressed);
                            },
                            None => prev_text = Some(Root::from_ref(text))
                        }
                    }
                },
                None => {
                    child.r().Normalize();
                    prev_text = None;
                }
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-clonenode
    fn CloneNode(self, deep: bool) -> Root<Node> {
        Node::clone(self, None, if deep {
            CloneChildrenFlag::CloneChildren
        } else {
            CloneChildrenFlag::DoNotCloneChildren
        })
    }

    // https://dom.spec.whatwg.org/#dom-node-isequalnode
    fn IsEqualNode(self, maybe_node: Option<&Node>) -> bool {
        fn is_equal_doctype(node: &Node, other: &Node) -> bool {
            let doctype: &DocumentType = DocumentTypeCast::to_ref(node).unwrap();
            let other_doctype: &DocumentType = DocumentTypeCast::to_ref(other).unwrap();
            (*doctype.name() == *other_doctype.name()) &&
            (*doctype.public_id() == *other_doctype.public_id()) &&
            (*doctype.system_id() == *other_doctype.system_id())
        }
        fn is_equal_element(node: &Node, other: &Node) -> bool {
            let element: &Element = ElementCast::to_ref(node).unwrap();
            let other_element: &Element = ElementCast::to_ref(other).unwrap();
            (*element.namespace() == *other_element.namespace()) &&
            (*element.prefix() == *other_element.prefix()) &&
            (*element.local_name() == *other_element.local_name()) &&
            (element.attrs().len() == other_element.attrs().len())
        }
        fn is_equal_processinginstruction(node: &Node, other: &Node) -> bool {
            let pi: &ProcessingInstruction = ProcessingInstructionCast::to_ref(node).unwrap();
            let other_pi: &ProcessingInstruction = ProcessingInstructionCast::to_ref(other).unwrap();
            (*pi.target() == *other_pi.target()) &&
            (*CharacterDataCast::from_ref(pi).data() == *CharacterDataCast::from_ref(other_pi).data())
        }
        fn is_equal_characterdata(node: &Node, other: &Node) -> bool {
            let characterdata: &CharacterData = CharacterDataCast::to_ref(node).unwrap();
            let other_characterdata: &CharacterData = CharacterDataCast::to_ref(other).unwrap();
            *characterdata.data() == *other_characterdata.data()
        }
        fn is_equal_element_attrs(node: &Node, other: &Node) -> bool {
            let element: &Element = ElementCast::to_ref(node).unwrap();
            let other_element: &Element = ElementCast::to_ref(other).unwrap();
            assert!(element.attrs().len() == other_element.attrs().len());
            // FIXME(https://github.com/rust-lang/rust/issues/23338)
            let attrs = element.attrs();
            attrs.iter().all(|attr| {
                let attr = attr.root();
                other_element.attrs().iter().any(|other_attr| {
                    let other_attr = other_attr.root();
                    (*attr.r().namespace() == *other_attr.r().namespace()) &&
                    (attr.r().local_name() == other_attr.r().local_name()) &&
                    (**attr.r().value() == **other_attr.r().value())
                })
            })
        }
        fn is_equal_node(this: &Node, node: &Node) -> bool {
            // Step 2.
            if this.type_id() != node.type_id() {
                return false;
            }

            match node.type_id() {
                // Step 3.
                NodeTypeId::DocumentType
                    if !is_equal_doctype(this, node) => return false,
                NodeTypeId::Element(..)
                    if !is_equal_element(this, node) => return false,
                NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction)
                    if !is_equal_processinginstruction(this, node) => return false,
                NodeTypeId::CharacterData(CharacterDataTypeId::Text) |
                NodeTypeId::CharacterData(CharacterDataTypeId::Comment)
                    if !is_equal_characterdata(this, node) => return false,
                // Step 4.
                NodeTypeId::Element(..)
                    if !is_equal_element_attrs(this, node) => return false,
                _ => ()
            }

            // Step 5.
            if this.children_count() != node.children_count() {
                return false;
            }

            // Step 6.
            this.children().zip(node.children()).all(|(child, other_child)| {
                is_equal_node(child.r(), other_child.r())
            })
        }
        match maybe_node {
            // Step 1.
            None => false,
            // Step 2-6.
            Some(node) => is_equal_node(self, node)
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-comparedocumentposition
    fn CompareDocumentPosition(self, other: &Node) -> u16 {
        if self == other {
            // step 2.
            0
        } else {
            let mut lastself = Root::from_ref(self);
            let mut lastother = Root::from_ref(other);
            for ancestor in self.ancestors() {
                if ancestor.r() == other {
                    // step 4.
                    return NodeConstants::DOCUMENT_POSITION_CONTAINS +
                           NodeConstants::DOCUMENT_POSITION_PRECEDING;
                }
                lastself = ancestor;
            }
            for ancestor in other.ancestors() {
                if ancestor.r() == self {
                    // step 5.
                    return NodeConstants::DOCUMENT_POSITION_CONTAINED_BY +
                           NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                }
                lastother = ancestor;
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

            for child in lastself.r().traverse_preorder() {
                if child.r() == other {
                    // step 6.
                    return NodeConstants::DOCUMENT_POSITION_PRECEDING;
                }
                if child.r() == self {
                    // step 7.
                    return NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                }
            }
            unreachable!()
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-contains
    fn Contains(self, maybe_other: Option<&Node>) -> bool {
        match maybe_other {
            None => false,
            Some(other) => self.is_inclusive_ancestor_of(other)
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-lookupprefix
    fn LookupPrefix(self, namespace: Option<DOMString>) -> Option<DOMString> {
        let namespace = namespace_from_domstring(namespace);

        // Step 1.
        if namespace == ns!("") {
            return None;
        }

        // Step 2.
        match self.type_id() {
            NodeTypeId::Element(..) => ElementCast::to_ref(self).unwrap().lookup_prefix(namespace),
            NodeTypeId::Document => {
                DocumentCast::to_ref(self).unwrap().GetDocumentElement().and_then(|element| {
                    element.r().lookup_prefix(namespace)
                })
            },
            NodeTypeId::DocumentType | NodeTypeId::DocumentFragment => None,
            _ => {
                self.GetParentElement().and_then(|element| {
                    element.r().lookup_prefix(namespace)
                })
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-lookupnamespaceuri
    fn LookupNamespaceURI(self, prefix: Option<DOMString>) -> Option<DOMString> {
        // Step 1.
        let prefix = match prefix {
            Some(ref p) if p.is_empty() => None,
            pre => pre
        };

        // Step 2.
        Node::namespace_to_string(Node::locate_namespace(self, prefix))
     }

    // https://dom.spec.whatwg.org/#dom-node-isdefaultnamespace
    fn IsDefaultNamespace(self, namespace: Option<DOMString>) -> bool {
        // Step 1.
        let namespace = namespace_from_domstring(namespace);
        // Steps 2 and 3.
        Node::locate_namespace(self, None) == namespace
    }
}



/// The address of a node known to be valid. These are sent from script to layout,
/// and are also used in the HTML parser interface.

#[allow(raw_pointer_derive)]
#[derive(Clone, PartialEq, Eq, Copy)]
pub struct TrustedNodeAddress(pub *const c_void);

#[allow(unsafe_code)]
unsafe impl Send for TrustedNodeAddress {}

pub fn document_from_node<T: NodeBase+Reflectable>(derived: &T) -> Root<Document> {
    let node: &Node = NodeCast::from_ref(derived);
    node.owner_doc()
}

pub fn window_from_node<T: NodeBase+Reflectable>(derived: &T) -> Root<Window> {
    let document = document_from_node(derived);
    document.r().window()
}

impl<'a> VirtualMethods for &'a Node {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let eventtarget: &&EventTarget = EventTargetCast::from_borrowed_ref(self);
        Some(eventtarget as &VirtualMethods)
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
        match *mutation {
            ChildrenMutation::Append { added, .. } |
            ChildrenMutation::Insert { added, .. } |
            ChildrenMutation::Prepend { added, .. } => {
                self.children_count.set(
                    self.children_count.get() + added.len() as u32);
            },
            ChildrenMutation::Replace { added, .. } => {
                self.children_count.set(
                    self.children_count.get() - 1u32 + added.len() as u32);
            },
            ChildrenMutation::ReplaceAll { added, .. } => {
                self.children_count.set(added.len() as u32);
            },
        }
        if let Some(list) = self.child_list.get().map(|list| list.root()) {
            list.as_children_list().children_changed(mutation);
        }
    }
}

pub trait DisabledStateHelpers {
    fn check_ancestors_disabled_state_for_form_control(self);
    fn check_parent_disabled_state_for_option(self);
    fn check_disabled_attribute(self);
}

impl<'a> DisabledStateHelpers for &'a Node {
    fn check_ancestors_disabled_state_for_form_control(self) {
        if self.get_disabled_state() { return; }
        for ancestor in self.ancestors() {
            let ancestor = ancestor;
            let ancestor = ancestor.r();
            if !ancestor.is_htmlfieldsetelement() { continue; }
            if !ancestor.get_disabled_state() { continue; }
            if ancestor.is_parent_of(self) {
                self.set_disabled_state(true);
                self.set_enabled_state(false);
                return;
            }
            match ancestor.children()
                          .find(|child| child.r().is_htmllegendelement())
            {
                Some(ref legend) => {
                    // XXXabinader: should we save previous ancestor to avoid this iteration?
                    if self.ancestors().any(|ancestor| ancestor == *legend) { continue; }
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
        if let Some(ref parent) = self.GetParentNode() {
            if parent.r().is_htmloptgroupelement() && parent.r().get_disabled_state() {
                self.set_disabled_state(true);
                self.set_enabled_state(false);
            }
        }
    }

    fn check_disabled_attribute(self) {
        let elem = ElementCast::to_ref(self).unwrap();
        let has_disabled_attrib = elem.has_attribute(&atom!("disabled"));
        self.set_disabled_state(has_disabled_attrib);
        self.set_enabled_state(!has_disabled_attrib);
    }
}

/// A summary of the changes that happened to a node.
#[derive(Copy, Clone, PartialEq, HeapSizeOf)]
pub enum NodeDamage {
    /// The node's `style` attribute changed.
    NodeStyleDamaged,
    /// Other parts of a node changed; attributes, text content, etc.
    OtherNodeDamage,
}

pub enum ChildrenMutation<'a> {
    Append { prev: &'a Node, added: &'a [&'a Node] },
    Insert { prev: &'a Node, added: &'a [&'a Node], next: &'a Node },
    Prepend { added: &'a [&'a Node], next: &'a Node },
    Replace {
        prev: Option<&'a Node>,
        removed: &'a Node,
        added: &'a [&'a Node],
        next: Option<&'a Node>,
    },
    ReplaceAll { removed: &'a [&'a Node], added: &'a [&'a Node] },
}

impl<'a> ChildrenMutation<'a> {
    fn insert(prev: Option<&'a Node>, added: &'a [&'a Node], next: Option<&'a Node>)
              -> ChildrenMutation<'a> {
        match (prev, next) {
            (None, None) => {
                ChildrenMutation::ReplaceAll { removed: &[], added: added }
            },
            (Some(prev), None) => {
                ChildrenMutation::Append { prev: prev, added: added }
            },
            (None, Some(next)) => {
                ChildrenMutation::Prepend { added: added, next: next }
            },
            (Some(prev), Some(next)) => {
                ChildrenMutation::Insert { prev: prev, added: added, next: next }
            },
        }
    }

    fn replace(prev: Option<&'a Node>,
               removed: &'a &'a Node,
               added: &'a [&'a Node],
               next: Option<&'a Node>)
               -> ChildrenMutation<'a> {
        if let (None, None) = (prev, next) {
            ChildrenMutation::ReplaceAll {
                removed: ref_slice(removed),
                added: added,
            }
        } else {
            ChildrenMutation::Replace {
                prev: prev,
                removed: *removed,
                added: added,
                next: next,
            }
        }
    }

    fn replace_all(removed: &'a [&'a Node], added: &'a [&'a Node])
                   -> ChildrenMutation<'a> {
        ChildrenMutation::ReplaceAll { removed: removed, added: added }
    }
}
