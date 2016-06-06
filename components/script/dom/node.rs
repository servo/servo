/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use app_units::Au;
use core::nonzero::NonZero;
use devtools_traits::NodeInfo;
use document_loader::DocumentLoader;
use dom::attr::Attr;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use dom::bindings::codegen::Bindings::NodeBinding::{NodeConstants, NodeMethods};
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::codegen::Bindings::ProcessingInstructionBinding::ProcessingInstructionMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::UnionTypes::NodeOrString;
use dom::bindings::conversions::{self, DerivedFrom};
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::{Castable, CharacterDataTypeId};
use dom::bindings::inheritance::{EventTargetTypeId, NodeTypeId};
use dom::bindings::js::Root;
use dom::bindings::js::RootedReference;
use dom::bindings::js::{JS, LayoutJS, MutNullableHeap};
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::str::{DOMString, USVString};
use dom::bindings::trace::RootedVec;
use dom::bindings::xmlname::namespace_from_domstring;
use dom::characterdata::{CharacterData, LayoutCharacterDataHelpers};
use dom::document::{Document, DocumentSource, IsHTMLDocument};
use dom::documentfragment::DocumentFragment;
use dom::documenttype::DocumentType;
use dom::element::{Element, ElementCreator};
use dom::eventtarget::EventTarget;
use dom::htmlbodyelement::HTMLBodyElement;
use dom::htmlcollection::HTMLCollection;
use dom::htmlelement::HTMLElement;
use dom::htmlinputelement::{HTMLInputElement, LayoutHTMLInputElementHelpers};
use dom::htmltextareaelement::{HTMLTextAreaElement, LayoutHTMLTextAreaElementHelpers};
use dom::nodelist::NodeList;
use dom::processinginstruction::ProcessingInstruction;
use dom::range::WeakRangeVec;
use dom::text::Text;
use dom::virtualmethods::{VirtualMethods, vtable_for};
use dom::window::Window;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::size::Size2D;
use heapsize::{HeapSizeOf, heap_size_of};
use html5ever::tree_builder::QuirksMode;
use js::jsapi::{JSContext, JSObject, JSRuntime};
use layout_interface::Msg;
use libc::{self, c_void, uintptr_t};
use parse::html::parse_html_fragment;
use ref_slice::ref_slice;
use script_traits::UntrustedNodeAddress;
use selectors::matching::matches;
use selectors::parser::Selector;
use selectors::parser::parse_author_origin_selector_list_from_str;
use std::borrow::ToOwned;
use std::cell::{Cell, UnsafeCell};
use std::cmp::max;
use std::default::Default;
use std::iter::{self, FilterMap, Peekable};
use std::mem;
use string_cache::{Atom, Namespace, QualName};
use style::selector_impl::ServoSelectorImpl;
use util::thread_state;
use uuid::Uuid;

//
// The basic Node structure
//

/// An HTML node.
#[dom_struct]
pub struct Node {
    /// The JavaScript reflector for this node.
    eventtarget: EventTarget,

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

    /// The maximum version of any inclusive descendant of this node.
    inclusive_descendants_version: Cell<u64>,

    /// A vector of weak references to Range instances of which the start
    /// or end containers are this node. No range should ever be found
    /// twice in this vector, even if both the start and end containers
    /// are this node.
    ranges: WeakRangeVec,

    /// Style+Layout information. Only the layout thread may touch this data.
    ///
    /// Must be sent back to the layout thread to be destroyed when this
    /// node is finalized.
    style_and_layout_data: Cell<Option<OpaqueStyleAndLayoutData>>,

    unique_id: UniqueId,
}

bitflags! {
    #[doc = "Flags for node items."]
    #[derive(JSTraceable, HeapSizeOf)]
    pub flags NodeFlags: u8 {
        #[doc = "Specifies whether this node is in a document."]
        const IS_IN_DOC = 0x01,
        #[doc = "Specifies whether this node _must_ be reflowed regardless of style differences."]
        const HAS_CHANGED = 0x02,
        #[doc = "Specifies whether this node needs style recalc on next reflow."]
        const IS_DIRTY = 0x04,
        #[doc = "Specifies whether this node has descendants (inclusive of itself) which \
                 have changed since the last reflow."]
        const HAS_DIRTY_DESCENDANTS = 0x08,
        // TODO: find a better place to keep this (#4105)
        // https://critic.hoppipolla.co.uk/showcomment?chain=8873
        // Perhaps using a Set in Document?
        #[doc = "Specifies whether or not there is an authentic click in progress on \
                 this element."]
        const CLICK_IN_PROGRESS = 0x10,
        #[doc = "Specifies whether this node is focusable and whether it is supposed \
                 to be reachable with using sequential focus navigation."]
        const SEQUENTIALLY_FOCUSABLE = 0x20,

        /// Whether any ancestor is a fragmentation container
        const CAN_BE_FRAGMENTED = 0x40
    }
}

impl NodeFlags {
    pub fn new() -> NodeFlags {
        HAS_CHANGED | IS_DIRTY | HAS_DIRTY_DESCENDANTS
    }
}

impl Drop for Node {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        self.style_and_layout_data.get().map(|d| d.dispose(self));
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

#[derive(Copy, Clone, HeapSizeOf)]
pub struct OpaqueStyleAndLayoutData {
    #[ignore_heap_size_of = "TODO(#6910) Box value that should be counted but \
                             the type lives in layout"]
    pub ptr: NonZero<*mut ()>
}

#[allow(unsafe_code)]
unsafe impl Send for OpaqueStyleAndLayoutData {}

no_jsmanaged_fields!(OpaqueStyleAndLayoutData);

impl OpaqueStyleAndLayoutData {
    /// Sends the style and layout data, if any, back to the layout thread to be destroyed.
    pub fn dispose(self, node: &Node) {
        debug_assert!(thread_state::get().is_script());
        let win = window_from_node(node);
        node.style_and_layout_data.set(None);
        win.layout_chan().send(Msg::ReapStyleAndLayoutData(self)).unwrap();
    }
}

impl Node {
    /// Adds a new child to the end of this node's list of children.
    ///
    /// Fails unless `new_child` is disconnected from the tree.
    fn add_child(&self, new_child: &Node, before: Option<&Node>) {
        assert!(new_child.parent_node.get().is_none());
        assert!(new_child.prev_sibling.get().is_none());
        assert!(new_child.next_sibling.get().is_none());
        match before {
            Some(ref before) => {
                assert!(before.parent_node.get().r() == Some(self));
                let prev_sibling = before.GetPreviousSibling();
                match prev_sibling {
                    None => {
                        assert!(Some(*before) == self.first_child.get().r());
                        self.first_child.set(Some(new_child));
                    },
                    Some(ref prev_sibling) => {
                        prev_sibling.next_sibling.set(Some(new_child));
                        new_child.prev_sibling.set(Some(prev_sibling.r()));
                    },
                }
                before.prev_sibling.set(Some(new_child));
                new_child.next_sibling.set(Some(before));
            },
            None => {
                let last_child = self.GetLastChild();
                match last_child {
                    None => self.first_child.set(Some(new_child)),
                    Some(ref last_child) => {
                        assert!(last_child.next_sibling.get().is_none());
                        last_child.next_sibling.set(Some(new_child));
                        new_child.prev_sibling.set(Some(&last_child));
                    }
                }

                self.last_child.set(Some(new_child));
            },
        }

        new_child.parent_node.set(Some(self));
        self.children_count.set(self.children_count.get() + 1);

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
    fn remove_child(&self, child: &Node, cached_index: Option<u32>) {
        assert!(child.parent_node.get().r() == Some(self));
        let prev_sibling = child.GetPreviousSibling();
        match prev_sibling {
            None => {
                self.first_child.set(child.next_sibling.get().r());
            }
            Some(ref prev_sibling) => {
                prev_sibling.next_sibling.set(child.next_sibling.get().r());
            }
        }
        let next_sibling = child.GetNextSibling();
        match next_sibling {
            None => {
                self.last_child.set(child.prev_sibling.get().r());
            }
            Some(ref next_sibling) => {
                next_sibling.prev_sibling.set(child.prev_sibling.get().r());
            }
        }

        let context = UnbindContext::new(self, prev_sibling.r(), cached_index);

        child.prev_sibling.set(None);
        child.next_sibling.set(None);
        child.parent_node.set(None);
        self.children_count.set(self.children_count.get() - 1);

        for node in child.traverse_preorder() {
            node.set_flag(IS_IN_DOC, false);
            vtable_for(&&*node).unbind_from_tree(&context);
            node.style_and_layout_data.get().map(|d| d.dispose(&node));
        }

        self.owner_doc().content_and_heritage_changed(self, NodeDamage::OtherNodeDamage);
        child.owner_doc().content_and_heritage_changed(child, NodeDamage::OtherNodeDamage);
    }
}

pub struct QuerySelectorIterator {
    selectors: Vec<Selector<ServoSelectorImpl>>,
    iterator: TreeIterator,
}

impl<'a> QuerySelectorIterator {
     fn new(iter: TreeIterator, selectors: Vec<Selector<ServoSelectorImpl>>)
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
            if let Some(element) = Root::downcast(node) {
                if matches(selectors, &element, None) {
                    return Some(Root::upcast(element));
                }
            }
            None
        }).next()
    }
}


impl Node {
    pub fn teardown(&self) {
        self.style_and_layout_data.get().map(|d| d.dispose(self));
        for kid in self.children() {
            kid.teardown();
        }
    }

    /// Dumps the subtree rooted at this node, for debugging.
    pub fn dump(&self) {
        self.dump_indent(0);
    }

    /// Dumps the node tree, for debugging, with indentation.
    pub fn dump_indent(&self, indent: u32) {
        let mut s = String::new();
        for _ in 0..indent {
            s.push_str("    ");
        }

        s.push_str(&*self.debug_str());
        debug!("{:?}", s);

        // FIXME: this should have a pure version?
        for kid in self.children() {
            kid.dump_indent(indent + 1)
        }
    }

    /// Returns a string that describes this node.
    pub fn debug_str(&self) -> String {
        format!("{:?}", self.type_id())
    }

    pub fn is_in_doc(&self) -> bool {
        self.flags.get().contains(IS_IN_DOC)
    }

    /// Returns the type ID of this node.
    pub fn type_id(&self) -> NodeTypeId {
        match *self.eventtarget.type_id() {
            EventTargetTypeId::Node(type_id) => type_id,
            _ => unreachable!(),
        }
    }

    // https://dom.spec.whatwg.org/#concept-node-length
    pub fn len(&self) -> u32 {
        match self.type_id() {
            NodeTypeId::DocumentType => 0,
            NodeTypeId::CharacterData(_) => {
                self.downcast::<CharacterData>().unwrap().Length()
            },
            _ => self.children_count(),
        }
    }

    // https://dom.spec.whatwg.org/#concept-tree-index
    pub fn index(&self) -> u32 {
        self.preceding_siblings().count() as u32
    }

    pub fn children_count(&self) -> u32 {
        self.children_count.get()
    }

    pub fn ranges(&self) -> &WeakRangeVec {
        &self.ranges
    }

    #[inline]
    pub fn is_doctype(&self) -> bool {
        self.type_id() == NodeTypeId::DocumentType
    }

    pub fn get_flag(&self, flag: NodeFlags) -> bool {
        self.flags.get().contains(flag)
    }

    pub fn set_flag(&self, flag: NodeFlags, value: bool) {
        let mut flags = self.flags.get();

        if value {
            flags.insert(flag);
        } else {
            flags.remove(flag);
        }

        self.flags.set(flags);
    }

    pub fn has_changed(&self) -> bool {
        self.get_flag(HAS_CHANGED)
    }

    pub fn set_has_changed(&self, state: bool) {
        self.set_flag(HAS_CHANGED, state)
    }

    pub fn is_dirty(&self) -> bool {
        self.get_flag(IS_DIRTY)
    }

    pub fn set_is_dirty(&self, state: bool) {
        self.set_flag(IS_DIRTY, state)
    }

    pub fn has_dirty_descendants(&self) -> bool {
        self.get_flag(HAS_DIRTY_DESCENDANTS)
    }

    pub fn set_has_dirty_descendants(&self, state: bool) {
        self.set_flag(HAS_DIRTY_DESCENDANTS, state)
    }

    pub fn force_dirty_ancestors(&self, damage: NodeDamage) {
        self.dirty_impl(damage, true)
    }

    pub fn rev_version(&self) {
        // The new version counter is 1 plus the max of the node's current version counter,
        // its descendants version, and the document's version. Normally, this will just be
        // the document's version, but we do have to deal with the case where the node has moved
        // document, so may have a higher version count than its owning document.
        let doc: Root<Node> = Root::upcast(self.owner_doc());
        let version = max(self.inclusive_descendants_version(), doc.inclusive_descendants_version()) + 1;
        for ancestor in self.inclusive_ancestors() {
            ancestor.inclusive_descendants_version.set(version);
        }
        doc.inclusive_descendants_version.set(version);
    }

    pub fn dirty(&self, damage: NodeDamage) {
        self.dirty_impl(damage, false)
    }

    pub fn dirty_impl(&self, damage: NodeDamage, force_ancestors: bool) {
        // 0. Set version counter
        self.rev_version();

        // 1. Dirty self.
        match damage {
            NodeDamage::NodeStyleDamaged => {}
            NodeDamage::OtherNodeDamage => self.set_has_changed(true),
        }

        if self.is_dirty() && !force_ancestors {
            return
        }

        // 2. Dirty descendants.
        fn dirty_subtree(node: &Node) {
            // Stop if this subtree is already dirty.
            if node.is_dirty() { return }

            node.set_flag(IS_DIRTY | HAS_DIRTY_DESCENDANTS, true);

            for kid in node.children() {
                dirty_subtree(kid.r());
            }
        }

        dirty_subtree(self);

        // 4. Dirty ancestors.
        for ancestor in self.ancestors() {
            if !force_ancestors && ancestor.has_dirty_descendants() { break }
            ancestor.set_has_dirty_descendants(true);
        }
    }

    /// The maximum version number of this node's descendants, including itself
    pub fn inclusive_descendants_version(&self) -> u64 {
        self.inclusive_descendants_version.get()
    }

    /// Iterates over this node and all its descendants, in preorder.
    pub fn traverse_preorder(&self) -> TreeIterator {
        TreeIterator::new(self)
    }

    pub fn inclusively_following_siblings(&self) -> NodeSiblingIterator {
        NodeSiblingIterator {
            current: Some(Root::from_ref(self)),
        }
    }

    pub fn inclusively_preceding_siblings(&self) -> ReverseSiblingIterator {
        ReverseSiblingIterator {
            current: Some(Root::from_ref(self)),
        }
    }

    pub fn is_inclusive_ancestor_of(&self, parent: &Node) -> bool {
        self == parent || self.is_ancestor_of(parent)
    }

    pub fn is_ancestor_of(&self, parent: &Node) -> bool {
        parent.ancestors().any(|ancestor| ancestor.r() == self)
    }

    pub fn following_siblings(&self) -> NodeSiblingIterator {
        NodeSiblingIterator {
            current: self.GetNextSibling(),
        }
    }

    pub fn preceding_siblings(&self) -> ReverseSiblingIterator {
        ReverseSiblingIterator {
            current: self.GetPreviousSibling(),
        }
    }

    pub fn following_nodes(&self, root: &Node) -> FollowingNodeIterator {
        FollowingNodeIterator {
            current: Some(Root::from_ref(self)),
            root: Root::from_ref(root),
        }
    }

    pub fn preceding_nodes(&self, root: &Node) -> PrecedingNodeIterator {
        PrecedingNodeIterator {
            current: Some(Root::from_ref(self)),
            root: Root::from_ref(root),
        }
    }

    pub fn descending_last_children(&self) -> LastChildIterator {
        LastChildIterator {
            current: self.GetLastChild(),
        }
    }

    pub fn is_parent_of(&self, child: &Node) -> bool {
        child.parent_node.get().map_or(false, |ref parent| parent.r() == self)
    }

    pub fn to_trusted_node_address(&self) -> TrustedNodeAddress {
        TrustedNodeAddress(&*self as *const Node as *const libc::c_void)
    }

    pub fn bounding_content_box(&self) -> Rect<Au> {
        window_from_node(self).content_box_query(self.to_trusted_node_address())
    }

    pub fn content_boxes(&self) -> Vec<Rect<Au>> {
        window_from_node(self).content_boxes_query(self.to_trusted_node_address())
    }

    pub fn client_rect(&self) -> Rect<i32> {
        window_from_node(self).client_rect_query(self.to_trusted_node_address())
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollwidth
    // https://drafts.csswg.org/cssom-view/#dom-element-scrollheight
    // https://drafts.csswg.org/cssom-view/#dom-element-scrolltop
    // https://drafts.csswg.org/cssom-view/#dom-element-scrollleft
    pub fn scroll_area(&self) -> Rect<i32> {
        // Step 1
        let document = self.owner_doc();
        // Step 3
        let window = document.window();

        let html_element = document.GetDocumentElement();

        let is_body_element = self.downcast::<HTMLBodyElement>()
                                  .map_or(false, |e| e.is_the_html_body_element());

        let scroll_area = window.scroll_area_query(self.to_trusted_node_address());

        match (document != window.Document(), is_body_element, document.quirks_mode(),
               html_element.r() == self.downcast::<Element>()) {
            // Step 2 && Step 5
            (true, _, _, _) | (_, false, QuirksMode::Quirks, true) => Rect::zero(),
            // Step 6 && Step 7
            (false, false, _, true) | (false, true, QuirksMode::Quirks, _) => {
                Rect::new(Point2D::new(window.ScrollX(), window.ScrollY()),
                                       Size2D::new(max(window.InnerWidth(), scroll_area.size.width),
                                                   max(window.InnerHeight(), scroll_area.size.height)))
            },
            // Step 9
            _ => scroll_area
        }
    }

    pub fn scroll_offset(&self) -> Point2D<f32> {
        let document = self.owner_doc();
        let window = document.window();
        window.scroll_offset_query(self.to_trusted_node_address())
    }

    // https://dom.spec.whatwg.org/#dom-childnode-before
    pub fn before(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let parent = &self.parent_node;

        // Step 2.
        let parent = match parent.get() {
            None => return Ok(()),
            Some(parent) => parent,
        };

        // Step 3.
        let viable_previous_sibling = first_node_not_in(self.preceding_siblings(), &nodes);

        // Step 4.
        let node = try!(self.owner_doc().node_from_nodes_and_strings(nodes));

        // Step 5.
        let viable_previous_sibling = match viable_previous_sibling {
            Some(ref viable_previous_sibling) => viable_previous_sibling.next_sibling.get(),
            None => parent.first_child.get(),
        };

        // Step 6.
        try!(Node::pre_insert(&node, &parent, viable_previous_sibling.r()));

        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-childnode-after
    pub fn after(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let parent = &self.parent_node;

        // Step 2.
        let parent = match parent.get() {
            None => return Ok(()),
            Some(parent) => parent,
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
    pub fn replace_with(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let parent = if let Some(parent) = self.GetParentNode() {
            parent
        } else {
            // Step 2.
            return Ok(());
        };
        // Step 3.
        let viable_next_sibling = first_node_not_in(self.following_siblings(), &nodes);
        // Step 4.
        let node = try!(self.owner_doc().node_from_nodes_and_strings(nodes));
        if self.parent_node == Some(&*parent) {
            // Step 5.
            try!(parent.ReplaceChild(&node, self));
        } else {
            // Step 6.
            try!(Node::pre_insert(&node, &parent, viable_next_sibling.r()));
        }
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    pub fn prepend(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let doc = self.owner_doc();
        let node = try!(doc.node_from_nodes_and_strings(nodes));
        // Step 2.
        let first_child = self.first_child.get();
        Node::pre_insert(node.r(), self, first_child.r()).map(|_| ())
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    pub fn append(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let doc = self.owner_doc();
        let node = try!(doc.node_from_nodes_and_strings(nodes));
        // Step 2.
        self.AppendChild(node.r()).map(|_| ())
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    pub fn query_selector(&self, selectors: DOMString) -> Fallible<Option<Root<Element>>> {
        // Step 1.
        match parse_author_origin_selector_list_from_str(&selectors) {
            // Step 2.
            Err(()) => Err(Error::Syntax),
            // Step 3.
            Ok(ref selectors) => {
                Ok(self.traverse_preorder().filter_map(Root::downcast).find(|element| {
                    matches(selectors, element, None)
                }))
            }
        }
    }

    /// https://dom.spec.whatwg.org/#scope-match-a-selectors-string
    /// Get an iterator over all nodes which match a set of selectors
    /// Be careful not to do anything which may manipulate the DOM tree
    /// whilst iterating, otherwise the iterator may be invalidated.
    pub fn query_selector_iter(&self, selectors: DOMString)
                                  -> Fallible<QuerySelectorIterator> {
        // Step 1.
        match parse_author_origin_selector_list_from_str(&selectors) {
            // Step 2.
            Err(()) => Err(Error::Syntax),
            // Step 3.
            Ok(selectors) => {
                Ok(QuerySelectorIterator::new(self.traverse_preorder(), selectors))
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    #[allow(unsafe_code)]
    pub fn query_selector_all(&self, selectors: DOMString) -> Fallible<Root<NodeList>> {
        let window = window_from_node(self);
        let iter = try!(self.query_selector_iter(selectors));
        Ok(NodeList::new_simple_list(window.r(), iter))
    }

    pub fn ancestors(&self) -> AncestorIterator {
        AncestorIterator {
            current: self.GetParentNode()
        }
    }

    pub fn inclusive_ancestors(&self) -> AncestorIterator {
        AncestorIterator {
            current: Some(Root::from_ref(self))
        }
    }

    pub fn owner_doc(&self) -> Root<Document> {
        self.owner_doc.get().unwrap()
    }

    pub fn set_owner_doc(&self, document: &Document) {
        self.owner_doc.set(Some(document));
    }

    pub fn is_in_html_doc(&self) -> bool {
        self.owner_doc().is_html_document()
    }

    pub fn children(&self) -> NodeSiblingIterator {
        NodeSiblingIterator {
            current: self.GetFirstChild(),
        }
    }

    pub fn rev_children(&self) -> ReverseSiblingIterator {
        ReverseSiblingIterator {
            current: self.GetLastChild(),
        }
    }

    pub fn child_elements(&self) -> ChildElementIterator {
        self.children().filter_map(Root::downcast as fn(_) -> _).peekable()
    }

    pub fn remove_self(&self) {
        if let Some(ref parent) = self.GetParentNode() {
            Node::remove(self, parent.r(), SuppressObserver::Unsuppressed);
        }
    }

    pub fn unique_id(&self) -> String {
        self.unique_id.borrow().simple().to_string()
    }

    pub fn summarize(&self) -> NodeInfo {
        let USVString(baseURI) = self.BaseURI();
        NodeInfo {
            uniqueId: self.unique_id(),
            baseURI: baseURI,
            parent: self.GetParentNode().map_or("".to_owned(), |node| node.unique_id()),
            nodeType: self.NodeType(),
            namespaceURI: String::new(), //FIXME
            nodeName: String::from(self.NodeName()),
            numChildren: self.ChildNodes().Length() as usize,

            //FIXME doctype nodes only
            name: String::new(),
            publicId: String::new(),
            systemId: String::new(),
            attrs: self.downcast().map(Element::summarize).unwrap_or(vec![]),

            isDocumentElement:
                self.owner_doc()
                    .GetDocumentElement()
                    .map_or(false, |elem| elem.upcast::<Node>() == self),

            shortValue: self.GetNodeValue().map(String::from).unwrap_or_default(), //FIXME: truncate
            incompleteValue: false, //FIXME: reflect truncation
        }
    }

    // https://dvcs.w3.org/hg/innerhtml/raw-file/tip/index.html#dfn-concept-parse-fragment
    pub fn parse_fragment(&self, markup: DOMString) -> Fallible<Root<DocumentFragment>> {
        let context_document = document_from_node(self);
        let fragment = DocumentFragment::new(context_document.r());
        if context_document.is_html_document() {
            parse_html_fragment(self.upcast(), markup, fragment.upcast());
        } else {
            // FIXME: XML case
            unimplemented!();
        }
        Ok(fragment)
    }

    /// Used by `HTMLTableSectionElement::InsertRow` and `HTMLTableRowElement::InsertCell`
    pub fn insert_cell_or_row<F, G, I>(&self, index: i32, get_items: F, new_child: G) -> Fallible<Root<HTMLElement>>
        where F: Fn() -> Root<HTMLCollection>,
              G: Fn() -> Root<I>,
              I: DerivedFrom<Node> + DerivedFrom<HTMLElement> + Reflectable,
    {
        if index < -1 {
            return Err(Error::IndexSize);
        }

        let tr = new_child();

        let after_node = if index == -1 {
            None
        } else {
            match get_items().elements_iter()
                             .map(Root::upcast::<Node>)
                             .map(Some)
                             .chain(iter::once(None))
                             .nth(index as usize) {
                None => return Err(Error::IndexSize),
                Some(node) => node,
            }
        };

        {
            let tr_node = tr.upcast::<Node>();
            try!(self.InsertBefore(tr_node, after_node.r()));
        }

        Ok(Root::upcast::<HTMLElement>(tr))
    }

    /// Used by `HTMLTableSectionElement::DeleteRow` and `HTMLTableRowElement::DeleteCell`
    pub fn delete_cell_or_row<F, G>(&self, index: i32, get_items: F, is_delete_type: G) -> ErrorResult
        where F: Fn() -> Root<HTMLCollection>,
              G: Fn(&Element) -> bool
    {
        let element = match index {
            index if index < -1 => return Err(Error::IndexSize),
            -1 => {
                let last_child = self.upcast::<Node>().GetLastChild();
                match last_child.and_then(|node| node.inclusively_preceding_siblings()
                                                     .filter_map(Root::downcast::<Element>)
                                                     .filter(|elem| is_delete_type(elem))
                                                     .next()) {
                    Some(element) => element,
                    None => return Ok(()),
                }
            },
            index => match get_items().Item(index as u32) {
                Some(element) => element,
                None => return Err(Error::IndexSize),
            },
        };

        element.upcast::<Node>().remove_self();
        Ok(())
    }
}


/// Iterate through `nodes` until we find a `Node` that is not in `not_in`
fn first_node_not_in<I>(mut nodes: I, not_in: &[NodeOrString]) -> Option<Root<Node>>
        where I: Iterator<Item=Root<Node>>
{
    nodes.find(|node| {
        not_in.iter().all(|n| {
            match *n {
                NodeOrString::Node(ref n) => n != node,
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
        let boxed_node = conversions::private_from_object(object) as *const Node;
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

    unsafe fn get_style_and_layout_data(&self) -> Option<OpaqueStyleAndLayoutData>;
    unsafe fn init_style_and_layout_data(&self, OpaqueStyleAndLayoutData);

    fn text_content(&self) -> String;
}

impl LayoutNodeHelpers for LayoutJS<Node> {
    #[inline]
    #[allow(unsafe_code)]
    unsafe fn type_id_for_layout(&self) -> NodeTypeId {
        (*self.unsafe_get()).type_id()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn is_element_for_layout(&self) -> bool {
        (*self.unsafe_get()).is::<Element>()
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
    unsafe fn get_style_and_layout_data(&self) -> Option<OpaqueStyleAndLayoutData> {
        (*self.unsafe_get()).style_and_layout_data.get()
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn init_style_and_layout_data(&self, val: OpaqueStyleAndLayoutData) {
        debug_assert!((*self.unsafe_get()).style_and_layout_data.get().is_none());
        (*self.unsafe_get()).style_and_layout_data.set(Some(val));
    }

    #[allow(unsafe_code)]
    fn text_content(&self) -> String {
        if let Some(text) = self.downcast::<Text>() {
            return unsafe { text.upcast().data_for_layout().to_owned() };
        }

        if let Some(input) = self.downcast::<HTMLInputElement>() {
            return unsafe { input.value_for_layout() };
        }

        if let Some(area) = self.downcast::<HTMLTextAreaElement>() {
            return unsafe { area.get_value_for_layout() };
        }

        panic!("not text!")
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
        self.current = current.GetNextSibling();
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
        self.current = current.GetPreviousSibling();
        Some(current)
    }
}

pub struct FollowingNodeIterator {
    current: Option<Root<Node>>,
    root: Root<Node>,
}

impl FollowingNodeIterator {
    /// Skips iterating the children of the current node
    pub fn next_skipping_children(&mut self) -> Option<Root<Node>> {
        let current = match self.current.take() {
            None => return None,
            Some(current) => current,
        };

        self.next_skipping_children_impl(current)
    }

    fn next_skipping_children_impl(&mut self, current: Root<Node>) -> Option<Root<Node>> {
        if self.root == current {
            self.current = None;
            return None;
        }

        if let Some(next_sibling) = current.GetNextSibling() {
            self.current = Some(next_sibling);
            return current.GetNextSibling()
        }

        for ancestor in current.inclusive_ancestors() {
            if self.root == ancestor {
                break;
            }
            if let Some(next_sibling) = ancestor.GetNextSibling() {
                self.current = Some(next_sibling);
                return ancestor.GetNextSibling()
            }
        }
        self.current = None;
        None
    }
}

impl Iterator for FollowingNodeIterator {
    type Item = Root<Node>;

    // https://dom.spec.whatwg.org/#concept-tree-following
    fn next(&mut self) -> Option<Root<Node>> {
        let current = match self.current.take() {
            None => return None,
            Some(current) => current,
        };

        if let Some(first_child) = current.GetFirstChild() {
            self.current = Some(first_child);
            return current.GetFirstChild()
        }

        self.next_skipping_children_impl(current)
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

        self.current = if self.root == current {
            None
        } else if let Some(previous_sibling) = current.GetPreviousSibling() {
            if self.root == previous_sibling {
                None
            } else if let Some(last_child) = previous_sibling.descending_last_children().last() {
                Some(last_child)
            } else {
                Some(previous_sibling)
            }
        } else {
            current.GetParentNode()
        };
        self.current.clone()
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
        self.current = current.GetLastChild();
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
        self.current = current.GetParentNode();
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
        if let Some(first_child) = current.GetFirstChild() {
            self.current = Some(first_child);
            self.depth += 1;
            return Some(current);
        };
        for ancestor in current.inclusive_ancestors() {
            if self.depth == 0 {
                break;
            }
            if let Some(next_sibling) = ancestor.GetNextSibling() {
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
    pub fn reflect_node<N: DerivedFrom<Node> + Reflectable>
            (node:      Box<N>,
             document:  &Document,
             wrap_fn:   extern "Rust" fn(*mut JSContext, GlobalRef, Box<N>) -> Root<N>)
             -> Root<N> {
        let window = document.window();
        reflect_dom_object(node, GlobalRef::Window(window), wrap_fn)
    }

    pub fn new_inherited(doc: &Document) -> Node {
        Node::new_(NodeFlags::new(), Some(doc))
    }

    #[allow(unrooted_must_root)]
    pub fn new_document_node() -> Node {
        Node::new_(NodeFlags::new() | IS_IN_DOC, None)
    }

    #[allow(unrooted_must_root)]
    fn new_(flags: NodeFlags, doc: Option<&Document>) -> Node {
        Node {
            eventtarget: EventTarget::new_inherited(),

            parent_node: Default::default(),
            first_child: Default::default(),
            last_child: Default::default(),
            next_sibling: Default::default(),
            prev_sibling: Default::default(),
            owner_doc: MutNullableHeap::new(doc),
            child_list: Default::default(),
            children_count: Cell::new(0u32),
            flags: Cell::new(flags),
            inclusive_descendants_version: Cell::new(0),
            ranges: WeakRangeVec::new(),

            style_and_layout_data: Cell::new(None),

            unique_id: UniqueId::new(),
        }
    }

    // https://dom.spec.whatwg.org/#concept-node-adopt
    pub fn adopt(node: &Node, document: &Document) {
        // Step 1.
        let old_doc = node.owner_doc();
        // Step 2.
        node.remove_self();
        if &*old_doc != document {
            // Step 3.
            for descendant in node.traverse_preorder() {
                descendant.set_owner_doc(document);
            }
            // Step 4.
            for descendant in node.traverse_preorder() {
                vtable_for(&descendant).adopting_steps(&old_doc);
            }
        }
    }

    // https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity
    pub fn ensure_pre_insertion_validity(node: &Node,
                                         parent: &Node,
                                         child: Option<&Node>) -> ErrorResult {
        // Step 1.
        match parent.type_id() {
            NodeTypeId::Document(_) |
            NodeTypeId::DocumentFragment |
            NodeTypeId::Element(..) => (),
            _ => return Err(Error::HierarchyRequest)
        }

        // Step 2.
        if node.is_inclusive_ancestor_of(parent) {
            return Err(Error::HierarchyRequest);
        }

        // Step 3.
        if let Some(child) = child {
            if !parent.is_parent_of(child) {
                return Err(Error::NotFound);
            }
        }

        // Step 4-5.
        match node.type_id() {
            NodeTypeId::CharacterData(CharacterDataTypeId::Text) => {
                if parent.is::<Document>() {
                    return Err(Error::HierarchyRequest);
                }
            },
            NodeTypeId::DocumentType => {
                if !parent.is::<Document>() {
                    return Err(Error::HierarchyRequest);
                }
            },
            NodeTypeId::DocumentFragment |
            NodeTypeId::Element(_) |
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) |
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => (),
            NodeTypeId::Document(_) => return Err(Error::HierarchyRequest)
        }

        // Step 6.
        if parent.is::<Document>() {
            match node.type_id() {
                // Step 6.1
                NodeTypeId::DocumentFragment => {
                    // Step 6.1.1(b)
                    if node.children()
                           .any(|c| c.is::<Text>())
                    {
                        return Err(Error::HierarchyRequest);
                    }
                    match node.child_elements().count() {
                        0 => (),
                        // Step 6.1.2
                        1 => {
                            if !parent.child_elements().peek().is_none() {
                                return Err(Error::HierarchyRequest);
                            }
                            if let Some(child) = child {
                                if child.inclusively_following_siblings()
                                    .any(|child| child.is_doctype()) {
                                        return Err(Error::HierarchyRequest);
                                }
                            }
                        },
                        // Step 6.1.1(a)
                        _ => return Err(Error::HierarchyRequest),
                    }
                },
                // Step 6.2
                NodeTypeId::Element(_) => {
                    if !parent.child_elements().peek().is_none() {
                        return Err(Error::HierarchyRequest);
                    }
                    if let Some(ref child) = child {
                        if child.inclusively_following_siblings()
                            .any(|child| child.is_doctype()) {
                                return Err(Error::HierarchyRequest);
                        }
                    }
                },
                // Step 6.3
                NodeTypeId::DocumentType => {
                    if parent.children()
                             .any(|c| c.is_doctype())
                    {
                        return Err(Error::HierarchyRequest);
                    }
                    match child {
                        Some(child) => {
                            if parent.children()
                                     .take_while(|c| c.r() != child)
                                     .any(|c| c.is::<Element>())
                            {
                                return Err(Error::HierarchyRequest);
                            }
                        },
                        None => {
                            if !parent.child_elements().peek().is_none() {
                                return Err(Error::HierarchyRequest);
                            }
                        },
                    }
                },
                NodeTypeId::CharacterData(_) => (),
                NodeTypeId::Document(_) => unreachable!(),
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
        Ok(Root::from_ref(node))
    }

    // https://dom.spec.whatwg.org/#concept-node-insert
    fn insert(node: &Node,
              parent: &Node,
              child: Option<&Node>,
              suppress_observers: SuppressObserver) {
        debug_assert!(&*node.owner_doc() == &*parent.owner_doc());
        debug_assert!(child.map_or(true, |child| Some(parent) == child.GetParentNode().r()));

        // Step 1.
        let count = if node.is::<DocumentFragment>() {
            node.children_count()
        } else {
            1
        };
        // Step 2.
        if let Some(child) = child {
            if !parent.ranges.is_empty() {
                let index = child.index();
                // Steps 2.1-2.
                parent.ranges.increase_above(parent, index, count);
            }
        }
        let mut new_nodes = RootedVec::new();
        let new_nodes = if let NodeTypeId::DocumentFragment = node.type_id() {
            // Step 3.
            new_nodes.extend(node.children().map(|kid| JS::from_ref(&*kid)));
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
        let removed_nodes = parent.children().collect::<RootedVec<_>>();
        // Step 3.
        let mut added_nodes = RootedVec::new();
        let added_nodes = if let Some(node) = node.as_ref() {
            if let NodeTypeId::DocumentFragment = node.type_id() {
                added_nodes.extend(node.children().map(|child| JS::from_ref(&*child)));
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
            Some(ref node) if node.r() != parent => return Err(Error::NotFound),
            None => return Err(Error::NotFound),
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
        let cached_index = {
            if parent.ranges.is_empty() {
                None
            } else {
                // Step 1.
                let index = node.index();
                // Steps 2-3 are handled in Node::unbind_from_tree.
                // Steps 4-5.
                parent.ranges.decrease_above(parent, index, 1);
                // Parent had ranges, we needed the index, let's keep track of
                // it to avoid computing it for other ranges when calling
                // unbind_from_tree recursively.
                Some(index)
            }
        };
        // Step 6. pre-removing steps for node iterators
        // Step 7.
        let old_previous_sibling = node.GetPreviousSibling();
        // Step 8.
        let old_next_sibling = node.GetNextSibling();
        // Steps 9-10 are handled in unbind_from_tree.
        parent.remove_child(node, cached_index);
        // Step 11. transient registered observers
        // Step 12.
        if let SuppressObserver::Unsuppressed = suppress_observers {
            vtable_for(&parent).children_changed(
                &ChildrenMutation::replace(old_previous_sibling.r(),
                                           &Some(&node), &[],
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
                let doctype = node.downcast::<DocumentType>().unwrap();
                let doctype = DocumentType::new(doctype.name().clone(),
                                                Some(doctype.public_id().clone()),
                                                Some(doctype.system_id().clone()), document.r());
                Root::upcast::<Node>(doctype)
            },
            NodeTypeId::DocumentFragment => {
                let doc_fragment = DocumentFragment::new(document.r());
                Root::upcast::<Node>(doc_fragment)
            },
            NodeTypeId::CharacterData(_) => {
                let cdata = node.downcast::<CharacterData>().unwrap();
                cdata.clone_with_data(cdata.Data(), &document)
            },
            NodeTypeId::Document(_) => {
                let document = node.downcast::<Document>().unwrap();
                let is_html_doc = if document.is_html_document() {
                    IsHTMLDocument::HTMLDocument
                } else {
                    IsHTMLDocument::NonHTMLDocument
                };
                let window = document.window();
                let loader = DocumentLoader::new(&*document.loader());
                let document = Document::new(window, None,
                                             Some((*document.url()).clone()),
                                             is_html_doc, None,
                                             None, DocumentSource::NotFromParser, loader);
                Root::upcast::<Node>(document)
            },
            NodeTypeId::Element(..) => {
                let element = node.downcast::<Element>().unwrap();
                let name = QualName {
                    ns: element.namespace().clone(),
                    local: element.local_name().clone()
                };
                let element = Element::create(name,
                    element.prefix().as_ref().map(|p| Atom::from(&**p)),
                    document.r(), ElementCreator::ScriptCreated);
                Root::upcast::<Node>(element)
            },
        };

        // Step 3.
        let document = match copy.downcast::<Document>() {
            Some(doc) => Root::from_ref(doc),
            None => Root::from_ref(document.r()),
        };
        assert!(copy.owner_doc() == document);

        // Step 4 (some data already copied in step 2).
        match node.type_id() {
            NodeTypeId::Document(_) => {
                let node_doc = node.downcast::<Document>().unwrap();
                let copy_doc = copy.downcast::<Document>().unwrap();
                copy_doc.set_encoding(node_doc.encoding());
                copy_doc.set_quirks_mode(node_doc.quirks_mode());
            },
            NodeTypeId::Element(..) => {
                let node_elem = node.downcast::<Element>().unwrap();
                let copy_elem = copy.downcast::<Element>().unwrap();

                for attr in node_elem.attrs().iter() {
                    copy_elem.push_new_attribute(attr.local_name().clone(),
                                                 attr.value().clone(),
                                                 attr.name().clone(),
                                                 attr.namespace().clone(),
                                                 attr.prefix().clone());
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

    pub fn collect_text_contents<T: Iterator<Item=Root<Node>>>(iterator: T) -> DOMString {
        let mut content = String::new();
        for node in iterator {
            match node.downcast::<Text>() {
                Some(ref text) => content.push_str(&text.upcast::<CharacterData>().data()),
                None => (),
            }
        }
        DOMString::from(content)
    }

    pub fn namespace_to_string(namespace: Namespace) -> Option<DOMString> {
        match namespace {
            ns!() => None,
            // FIXME(ajeffrey): convert directly from &Atom to DOMString
            Namespace(ref ns) => Some(DOMString::from(&**ns))
        }
    }

    // https://dom.spec.whatwg.org/#locate-a-namespace
    pub fn locate_namespace(node: &Node, prefix: Option<DOMString>) -> Namespace {
        fn attr_defines_namespace(attr: &Attr,
                                  prefix: &Option<Atom>) -> bool {
            *attr.namespace() == ns!(xmlns) &&
                match (attr.prefix(), prefix) {
                    (&Some(ref attr_prefix), &Some(ref prefix)) =>
                        attr_prefix == &atom!("xmlns") &&
                            attr.local_name() == prefix,
                    (&None, &None) => *attr.local_name() == atom!("xmlns"),
                    _ => false
                }
        }

        match node.type_id() {
            NodeTypeId::Element(_) => {
                let element = node.downcast::<Element>().unwrap();
                // Step 1.
                if *element.namespace() != ns!() && *element.prefix() == prefix {
                    return element.namespace().clone()
                }

                // FIXME(ajeffrey): directly convert DOMString to Atom
                let prefix_atom = prefix.as_ref().map(|s| Atom::from(&**s));

                // Step 2.
                let attrs = element.attrs();
                let namespace_attr = attrs.iter().find(|attr| {
                    attr_defines_namespace(attr, &prefix_atom)
                });

                // Steps 2.1-2.
                if let Some(attr) = namespace_attr {
                    return namespace_from_domstring(Some(attr.Value()));
                }

                match node.GetParentElement() {
                    // Step 3.
                    None => ns!(),
                    // Step 4.
                    Some(parent) => Node::locate_namespace(parent.upcast(), prefix)
                }
            },
            NodeTypeId::Document(_) => {
                match node.downcast::<Document>().unwrap().GetDocumentElement().r() {
                    // Step 1.
                    None => ns!(),
                    // Step 2.
                    Some(document_element) => {
                        Node::locate_namespace(document_element.upcast(), prefix)
                    }
                }
            },
            NodeTypeId::DocumentType => ns!(),
            NodeTypeId::DocumentFragment => ns!(),
            _ => match node.GetParentElement() {
                     // Step 1.
                     None => ns!(),
                     // Step 2.
                     Some(parent) => Node::locate_namespace(parent.upcast(), prefix)
                 }
        }
    }
}

impl NodeMethods for Node {
    // https://dom.spec.whatwg.org/#dom-node-nodetype
    fn NodeType(&self) -> u16 {
        match self.type_id() {
            NodeTypeId::CharacterData(CharacterDataTypeId::Text) =>
                NodeConstants::TEXT_NODE,
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) =>
                NodeConstants::PROCESSING_INSTRUCTION_NODE,
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) =>
                NodeConstants::COMMENT_NODE,
            NodeTypeId::Document(_) =>
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
    fn NodeName(&self) -> DOMString {
        match self.type_id() {
            NodeTypeId::Element(..) => {
                self.downcast::<Element>().unwrap().TagName()
            }
            NodeTypeId::CharacterData(CharacterDataTypeId::Text) => DOMString::from("#text"),
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) => {
                self.downcast::<ProcessingInstruction>().unwrap().Target()
            }
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => DOMString::from("#comment"),
            NodeTypeId::DocumentType => {
                self.downcast::<DocumentType>().unwrap().name().clone()
            },
            NodeTypeId::DocumentFragment => DOMString::from("#document-fragment"),
            NodeTypeId::Document(_) => DOMString::from("#document")
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-baseuri
    fn BaseURI(&self) -> USVString {
        USVString(String::from(self.owner_doc().base_url().as_str()))
    }

    // https://dom.spec.whatwg.org/#dom-node-ownerdocument
    fn GetOwnerDocument(&self) -> Option<Root<Document>> {
        match self.type_id() {
            NodeTypeId::CharacterData(..) |
            NodeTypeId::Element(..) |
            NodeTypeId::DocumentType |
            NodeTypeId::DocumentFragment => Some(self.owner_doc()),
            NodeTypeId::Document(_) => None
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-rootnode
    fn RootNode(&self) -> Root<Node> {
        self.inclusive_ancestors().last().unwrap()
    }

    // https://dom.spec.whatwg.org/#dom-node-parentnode
    fn GetParentNode(&self) -> Option<Root<Node>> {
        self.parent_node.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-parentelement
    fn GetParentElement(&self) -> Option<Root<Element>> {
        self.GetParentNode().and_then(Root::downcast)
    }

    // https://dom.spec.whatwg.org/#dom-node-haschildnodes
    fn HasChildNodes(&self) -> bool {
        self.first_child.get().is_some()
    }

    // https://dom.spec.whatwg.org/#dom-node-childnodes
    fn ChildNodes(&self) -> Root<NodeList> {
        self.child_list.or_init(|| {
            let doc = self.owner_doc();
            let window = doc.window();
            NodeList::new_child_list(window, self)
        })
    }

    // https://dom.spec.whatwg.org/#dom-node-firstchild
    fn GetFirstChild(&self) -> Option<Root<Node>> {
        self.first_child.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-lastchild
    fn GetLastChild(&self) -> Option<Root<Node>> {
        self.last_child.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-previoussibling
    fn GetPreviousSibling(&self) -> Option<Root<Node>> {
        self.prev_sibling.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-nextsibling
    fn GetNextSibling(&self) -> Option<Root<Node>> {
        self.next_sibling.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-nodevalue
    fn GetNodeValue(&self) -> Option<DOMString> {
        self.downcast::<CharacterData>().map(CharacterData::Data)
    }

    // https://dom.spec.whatwg.org/#dom-node-nodevalue
    fn SetNodeValue(&self, val: Option<DOMString>) {
        if let Some(character_data) = self.downcast::<CharacterData>() {
            character_data.SetData(val.unwrap_or_default());
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-textcontent
    fn GetTextContent(&self) -> Option<DOMString> {
        match self.type_id() {
            NodeTypeId::DocumentFragment |
            NodeTypeId::Element(..) => {
                let content = Node::collect_text_contents(self.traverse_preorder());
                Some(content)
            }
            NodeTypeId::CharacterData(..) => {
                let characterdata = self.downcast::<CharacterData>().unwrap();
                Some(characterdata.Data())
            }
            NodeTypeId::DocumentType |
            NodeTypeId::Document(_) => {
                None
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-textcontent
    fn SetTextContent(&self, value: Option<DOMString>) {
        let value = value.unwrap_or_default();
        match self.type_id() {
            NodeTypeId::DocumentFragment |
            NodeTypeId::Element(..) => {
                // Step 1-2.
                let node = if value.is_empty() {
                    None
                } else {
                    Some(Root::upcast(self.owner_doc().CreateTextNode(value)))
                };

                // Step 3.
                Node::replace_all(node.r(), self);
            }
            NodeTypeId::CharacterData(..) => {
                let characterdata = self.downcast::<CharacterData>().unwrap();
                characterdata.SetData(value);
            }
            NodeTypeId::DocumentType |
            NodeTypeId::Document(_) => {}
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-insertbefore
    fn InsertBefore(&self, node: &Node, child: Option<&Node>) -> Fallible<Root<Node>> {
        Node::pre_insert(node, self, child)
    }

    // https://dom.spec.whatwg.org/#dom-node-appendchild
    fn AppendChild(&self, node: &Node) -> Fallible<Root<Node>> {
        Node::pre_insert(node, self, None)
    }

    // https://dom.spec.whatwg.org/#concept-node-replace
    fn ReplaceChild(&self, node: &Node, child: &Node) -> Fallible<Root<Node>> {
        // Step 1.
        match self.type_id() {
            NodeTypeId::Document(_) |
            NodeTypeId::DocumentFragment |
            NodeTypeId::Element(..) => (),
            _ => return Err(Error::HierarchyRequest)
        }

        // Step 2.
        if node.is_inclusive_ancestor_of(self) {
            return Err(Error::HierarchyRequest);
        }

        // Step 3.
        if !self.is_parent_of(child) {
            return Err(Error::NotFound);
        }

        // Step 4-5.
        match node.type_id() {
            NodeTypeId::CharacterData(CharacterDataTypeId::Text) if self.is::<Document>() =>
                return Err(Error::HierarchyRequest),
            NodeTypeId::DocumentType if !self.is::<Document>() => return Err(Error::HierarchyRequest),
            NodeTypeId::Document(_) => return Err(Error::HierarchyRequest),
            _ => ()
        }

        // Step 6.
        if self.is::<Document>() {
            match node.type_id() {
                // Step 6.1
                NodeTypeId::DocumentFragment => {
                    // Step 6.1.1(b)
                    if node.children()
                           .any(|c| c.is::<Text>())
                    {
                        return Err(Error::HierarchyRequest);
                    }
                    match node.child_elements().count() {
                        0 => (),
                        // Step 6.1.2
                        1 => {
                            if self.child_elements().any(|c| c.upcast::<Node>() != child) {
                                return Err(Error::HierarchyRequest);
                            }
                            if child.following_siblings()
                                    .any(|child| child.is_doctype()) {
                                return Err(Error::HierarchyRequest);
                            }
                        },
                        // Step 6.1.1(a)
                        _ => return Err(Error::HierarchyRequest)
                    }
                },
                // Step 6.2
                NodeTypeId::Element(..) => {
                    if self.child_elements()
                           .any(|c| c.upcast::<Node>() != child) {
                        return Err(Error::HierarchyRequest);
                    }
                    if child.following_siblings()
                            .any(|child| child.is_doctype())
                    {
                        return Err(Error::HierarchyRequest);
                    }
                },
                // Step 6.3
                NodeTypeId::DocumentType => {
                    if self.children()
                           .any(|c| c.is_doctype() &&
                                c.r() != child)
                    {
                        return Err(Error::HierarchyRequest);
                    }
                    if self.children()
                           .take_while(|c| c.r() != child)
                           .any(|c| c.is::<Element>())
                    {
                        return Err(Error::HierarchyRequest);
                    }
                },
                NodeTypeId::CharacterData(..) => (),
                NodeTypeId::Document(_) => unreachable!(),
            }
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
        let previous_sibling = child.GetPreviousSibling();

        // Step 10.
        let document = document_from_node(self);
        Node::adopt(node, document.r());

        let removed_child = if node != child {
            // Step 11.
            Node::remove(child, self, SuppressObserver::Suppressed);
            Some(child)
        } else {
            None
        };

        // Step 12.
        let mut nodes = RootedVec::new();
        let nodes = if node.type_id() == NodeTypeId::DocumentFragment {
            nodes.extend(node.children().map(|node| JS::from_ref(&*node)));
            nodes.r()
        } else {
            ref_slice(&node)
        };

        // Step 13.
        Node::insert(node, self, reference_child, SuppressObserver::Suppressed);

        // Step 14.
        vtable_for(&self).children_changed(
            &ChildrenMutation::replace(previous_sibling.r(),
                                       &removed_child, nodes,
                                       reference_child));

        // Step 15.
        Ok(Root::from_ref(child))
    }

    // https://dom.spec.whatwg.org/#dom-node-removechild
    fn RemoveChild(&self, node: &Node)
                       -> Fallible<Root<Node>> {
        Node::pre_remove(node, self)
    }

    // https://dom.spec.whatwg.org/#dom-node-normalize
    fn Normalize(&self) {
        let mut children = self.children().enumerate().peekable();
        while let Some((_, node)) = children.next() {
            if let Some(text) = node.downcast::<Text>() {
                let cdata = text.upcast::<CharacterData>();
                let mut length = cdata.Length();
                if length == 0 {
                    Node::remove(&node, self, SuppressObserver::Unsuppressed);
                    continue;
                }
                while children.peek().map_or(false, |&(_, ref sibling)| sibling.is::<Text>()) {
                    let (index, sibling) = children.next().unwrap();
                    sibling.ranges.drain_to_preceding_text_sibling(&sibling, &node, length);
                    self.ranges.move_to_text_child_at(self, index as u32, &node, length as u32);
                    let sibling_cdata = sibling.downcast::<CharacterData>().unwrap();
                    length += sibling_cdata.Length();
                    cdata.append_data(&sibling_cdata.data());
                    Node::remove(&sibling, self, SuppressObserver::Unsuppressed);
                }
            } else {
                node.Normalize();
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-clonenode
    fn CloneNode(&self, deep: bool) -> Root<Node> {
        Node::clone(self, None, if deep {
            CloneChildrenFlag::CloneChildren
        } else {
            CloneChildrenFlag::DoNotCloneChildren
        })
    }

    // https://dom.spec.whatwg.org/#dom-node-isequalnode
    fn IsEqualNode(&self, maybe_node: Option<&Node>) -> bool {
        fn is_equal_doctype(node: &Node, other: &Node) -> bool {
            let doctype = node.downcast::<DocumentType>().unwrap();
            let other_doctype = other.downcast::<DocumentType>().unwrap();
            (*doctype.name() == *other_doctype.name()) &&
            (*doctype.public_id() == *other_doctype.public_id()) &&
            (*doctype.system_id() == *other_doctype.system_id())
        }
        fn is_equal_element(node: &Node, other: &Node) -> bool {
            let element = node.downcast::<Element>().unwrap();
            let other_element = other.downcast::<Element>().unwrap();
            (*element.namespace() == *other_element.namespace()) &&
            (*element.prefix() == *other_element.prefix()) &&
            (*element.local_name() == *other_element.local_name()) &&
            (element.attrs().len() == other_element.attrs().len())
        }
        fn is_equal_processinginstruction(node: &Node, other: &Node) -> bool {
            let pi = node.downcast::<ProcessingInstruction>().unwrap();
            let other_pi = other.downcast::<ProcessingInstruction>().unwrap();
            (*pi.target() == *other_pi.target()) &&
            (*pi.upcast::<CharacterData>().data() == *other_pi.upcast::<CharacterData>().data())
        }
        fn is_equal_characterdata(node: &Node, other: &Node) -> bool {
            let characterdata = node.downcast::<CharacterData>().unwrap();
            let other_characterdata = other.downcast::<CharacterData>().unwrap();
            *characterdata.data() == *other_characterdata.data()
        }
        fn is_equal_element_attrs(node: &Node, other: &Node) -> bool {
            let element = node.downcast::<Element>().unwrap();
            let other_element = other.downcast::<Element>().unwrap();
            assert!(element.attrs().len() == other_element.attrs().len());
            element.attrs().iter().all(|attr| {
                other_element.attrs().iter().any(|other_attr| {
                    (*attr.namespace() == *other_attr.namespace()) &&
                    (attr.local_name() == other_attr.local_name()) &&
                    (**attr.value() == **other_attr.value())
                })
            })
        }
        fn is_equal_node(this: &Node, node: &Node) -> bool {
            // Step 2.
            if this.NodeType() != node.NodeType() {
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

    // https://dom.spec.whatwg.org/#dom-node-issamenode
    fn IsSameNode(&self, otherNode: Option<&Node>) -> bool {
        match otherNode {
            Some(node) => self == node,
            None => false,
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-comparedocumentposition
    fn CompareDocumentPosition(&self, other: &Node) -> u16 {
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

            for child in lastself.traverse_preorder() {
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
    fn Contains(&self, maybe_other: Option<&Node>) -> bool {
        match maybe_other {
            None => false,
            Some(other) => self.is_inclusive_ancestor_of(other)
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-lookupprefix
    fn LookupPrefix(&self, namespace: Option<DOMString>) -> Option<DOMString> {
        let namespace = namespace_from_domstring(namespace);

        // Step 1.
        if namespace == ns!() {
            return None;
        }

        // Step 2.
        match self.type_id() {
            NodeTypeId::Element(..) => {
                self.downcast::<Element>().unwrap().lookup_prefix(namespace)
            },
            NodeTypeId::Document(_) => {
                self.downcast::<Document>().unwrap().GetDocumentElement().and_then(|element| {
                    element.lookup_prefix(namespace)
                })
            },
            NodeTypeId::DocumentType | NodeTypeId::DocumentFragment => None,
            _ => {
                self.GetParentElement().and_then(|element| {
                    element.lookup_prefix(namespace)
                })
            }
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-lookupnamespaceuri
    fn LookupNamespaceURI(&self, prefix: Option<DOMString>) -> Option<DOMString> {
        // Step 1.
        let prefix = match prefix {
            Some(ref p) if p.is_empty() => None,
            pre => pre
        };

        // Step 2.
        Node::namespace_to_string(Node::locate_namespace(self, prefix))
     }

    // https://dom.spec.whatwg.org/#dom-node-isdefaultnamespace
    fn IsDefaultNamespace(&self, namespace: Option<DOMString>) -> bool {
        // Step 1.
        let namespace = namespace_from_domstring(namespace);
        // Steps 2 and 3.
        Node::locate_namespace(self, None) == namespace
    }
}



/// The address of a node known to be valid. These are sent from script to layout,
/// and are also used in the HTML parser interface.

#[derive(Clone, PartialEq, Eq, Copy)]
pub struct TrustedNodeAddress(pub *const c_void);

#[allow(unsafe_code)]
unsafe impl Send for TrustedNodeAddress {}

pub fn document_from_node<T: DerivedFrom<Node> + Reflectable>(derived: &T) -> Root<Document> {
    derived.upcast().owner_doc()
}

pub fn window_from_node<T: DerivedFrom<Node> + Reflectable>(derived: &T) -> Root<Window> {
    let document = document_from_node(derived);
    Root::from_ref(document.window())
}

impl VirtualMethods for Node {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<EventTarget>() as &VirtualMethods)
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
        if let Some(list) = self.child_list.get() {
            list.as_children_list().children_changed(mutation);
        }
    }

    // This handles the ranges mentioned in steps 2-3 when removing a node.
    // https://dom.spec.whatwg.org/#concept-node-remove
    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);
        self.ranges.drain_to_parent(context, self);
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
               removed: &'a Option<&'a Node>,
               added: &'a [&'a Node],
               next: Option<&'a Node>)
               -> ChildrenMutation<'a> {
        if let Some(ref removed) = *removed {
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
        } else {
            ChildrenMutation::insert(prev, added, next)
        }
    }

    fn replace_all(removed: &'a [&'a Node], added: &'a [&'a Node])
                   -> ChildrenMutation<'a> {
        ChildrenMutation::ReplaceAll { removed: removed, added: added }
    }

    /// Get the child that follows the added or removed children.
    pub fn next_child(&self) -> Option<&Node> {
        match *self {
            ChildrenMutation::Append { .. } => None,
            ChildrenMutation::Insert { next, .. } => Some(next),
            ChildrenMutation::Prepend { next, .. } => Some(next),
            ChildrenMutation::Replace { next, .. } => next,
            ChildrenMutation::ReplaceAll { .. } => None,
        }
    }

    /// If nodes were added or removed at the start or end of a container, return any
    /// previously-existing child whose ":first-child" or ":last-child" status *may* have changed.
    ///
    /// NOTE: This does not check whether the inserted/removed nodes were elements, so in some
    /// cases it will return a false positive.  This doesn't matter for correctness, because at
    /// worst the returned element will be restyled unnecessarily.
    pub fn modified_edge_element(&self) -> Option<Root<Node>> {
        match *self {
            // Add/remove at start of container: Return the first following element.
            ChildrenMutation::Prepend { next, .. } |
            ChildrenMutation::Replace { prev: None, next: Some(next), .. } => {
                next.inclusively_following_siblings().filter(|node| node.is::<Element>()).next()
            }
            // Add/remove at end of container: Return the last preceding element.
            ChildrenMutation::Append { prev, .. } |
            ChildrenMutation::Replace { prev: Some(prev), next: None, .. } => {
                prev.inclusively_preceding_siblings().filter(|node| node.is::<Element>()).next()
            }
            // Insert or replace in the middle:
            ChildrenMutation::Insert { prev, next, .. } |
            ChildrenMutation::Replace { prev: Some(prev), next: Some(next), .. } => {
                if prev.inclusively_preceding_siblings().all(|node| !node.is::<Element>()) {
                    // Before the first element: Return the first following element.
                    next.inclusively_following_siblings().filter(|node| node.is::<Element>()).next()
                } else if next.inclusively_following_siblings().all(|node| !node.is::<Element>()) {
                    // After the last element: Return the last preceding element.
                    prev.inclusively_preceding_siblings().filter(|node| node.is::<Element>()).next()
                } else {
                    None
                }
            }

            ChildrenMutation::Replace { prev: None, next: None, .. } => unreachable!(),
            ChildrenMutation::ReplaceAll { .. } => None,
        }
    }
}

/// The context of the unbinding from a tree of a node when one of its
/// inclusive ancestors is removed.
pub struct UnbindContext<'a> {
    /// The index of the inclusive ancestor that was removed.
    index: Cell<Option<u32>>,
    /// The parent of the inclusive ancestor that was removed.
    pub parent: &'a Node,
    /// The previous sibling of the inclusive ancestor that was removed.
    prev_sibling: Option<&'a Node>,
    /// Whether the tree is in a document.
    pub tree_in_doc: bool,
}

impl<'a> UnbindContext<'a> {
    /// Create a new `UnbindContext` value.
    fn new(parent: &'a Node,
           prev_sibling: Option<&'a Node>,
           cached_index: Option<u32>) -> Self {
        UnbindContext {
            index: Cell::new(cached_index),
            parent: parent,
            prev_sibling: prev_sibling,
            tree_in_doc: parent.is_in_doc(),
        }
    }

    /// The index of the inclusive ancestor that was removed from the tree.
    #[allow(unsafe_code)]
    pub fn index(&self) -> u32 {
        if let Some(index) = self.index.get() {
            return index;
        }
        let index = self.prev_sibling.map_or(0, |sibling| sibling.index() + 1);
        self.index.set(Some(index));
        index
    }
}

/// A node's unique ID, for devtools.
struct UniqueId {
    cell: UnsafeCell<Option<Box<Uuid>>>,
}

no_jsmanaged_fields!(UniqueId);

impl HeapSizeOf for UniqueId {
    #[allow(unsafe_code)]
    fn heap_size_of_children(&self) -> usize {
        if let &Some(ref uuid) = unsafe { &*self.cell.get() } {
            unsafe { heap_size_of(&** uuid as *const Uuid as *const _) }
        } else {
            0
        }
    }
}

impl UniqueId {
    /// Create a new `UniqueId` value. The underlying `Uuid` is lazily created.
    fn new() -> UniqueId {
        UniqueId { cell: UnsafeCell::new(None) }
    }

    /// The Uuid of that unique ID.
    #[allow(unsafe_code)]
    fn borrow(&self) -> &Uuid {
        unsafe {
            let ptr = self.cell.get();
            if (*ptr).is_none() {
                *ptr = Some(box Uuid::new_v4());
            }
            &(&*ptr).as_ref().unwrap()
        }
    }
}
