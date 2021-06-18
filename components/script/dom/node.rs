/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use crate::document_loader::DocumentLoader;
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::{DomRefCell, Ref, RefMut};
use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::{
    GetRootNodeOptions, NodeConstants, NodeMethods,
};
use crate::dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use crate::dom::bindings::codegen::Bindings::ProcessingInstructionBinding::ProcessingInstructionMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRootBinding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::InheritTypes::DocumentFragmentTypeId;
use crate::dom::bindings::codegen::UnionTypes::NodeOrString;
use crate::dom::bindings::conversions::{self, DerivedFrom};
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::{Castable, CharacterDataTypeId, ElementTypeId};
use crate::dom::bindings::inheritance::{EventTargetTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::inheritance::{SVGElementTypeId, SVGGraphicsElementTypeId, TextTypeId};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, DomObjectWrap};
use crate::dom::bindings::root::{Dom, DomRoot, DomSlice, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::xmlname::namespace_from_domstring;
use crate::dom::characterdata::{CharacterData, LayoutCharacterDataHelpers};
use crate::dom::cssstylesheet::CSSStyleSheet;
use crate::dom::customelementregistry::{try_upgrade_element, CallbackReaction};
use crate::dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::documenttype::DocumentType;
use crate::dom::element::{CustomElementCreationMode, Element, ElementCreator};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlbodyelement::HTMLBodyElement;
use crate::dom::htmlcanvaselement::{HTMLCanvasElement, LayoutHTMLCanvasElementHelpers};
use crate::dom::htmlcollection::HTMLCollection;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmliframeelement::{HTMLIFrameElement, HTMLIFrameElementLayoutMethods};
use crate::dom::htmlimageelement::{HTMLImageElement, LayoutHTMLImageElementHelpers};
use crate::dom::htmlinputelement::{HTMLInputElement, LayoutHTMLInputElementHelpers};
use crate::dom::htmllinkelement::HTMLLinkElement;
use crate::dom::htmlmediaelement::{HTMLMediaElement, LayoutHTMLMediaElementHelpers};
use crate::dom::htmlmetaelement::HTMLMetaElement;
use crate::dom::htmlstyleelement::HTMLStyleElement;
use crate::dom::htmltextareaelement::{HTMLTextAreaElement, LayoutHTMLTextAreaElementHelpers};
use crate::dom::mouseevent::MouseEvent;
use crate::dom::mutationobserver::{Mutation, MutationObserver, RegisteredObserver};
use crate::dom::nodelist::NodeList;
use crate::dom::processinginstruction::ProcessingInstruction;
use crate::dom::range::WeakRangeVec;
use crate::dom::raredata::NodeRareData;
use crate::dom::shadowroot::{LayoutShadowRootHelpers, ShadowRoot};
use crate::dom::stylesheetlist::StyleSheetListOwner;
use crate::dom::svgsvgelement::{LayoutSVGSVGElementHelpers, SVGSVGElement};
use crate::dom::text::Text;
use crate::dom::virtualmethods::{vtable_for, VirtualMethods};
use crate::dom::window::Window;
use crate::script_thread::ScriptThread;
use app_units::Au;
use devtools_traits::NodeInfo;
use dom_struct::dom_struct;
use euclid::default::{Point2D, Rect, Size2D, Vector2D};
use html5ever::{Namespace, Prefix, QualName};
use js::jsapi::JSObject;
use libc::{self, c_void, uintptr_t};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use net_traits::image::base::{Image, ImageMetadata};
use script_layout_interface::message::QueryMsg;
use script_layout_interface::{HTMLCanvasData, HTMLMediaData, LayoutElementType, LayoutNodeType};
use script_layout_interface::{SVGSVGData, StyleAndOpaqueLayoutData, TrustedNodeAddress};
use script_traits::DocumentActivity;
use script_traits::UntrustedNodeAddress;
use selectors::matching::{matches_selector_list, MatchingContext, MatchingMode};
use selectors::parser::SelectorList;
use servo_arc::Arc;
use servo_atoms::Atom;
use servo_url::ServoUrl;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::cell::{Cell, UnsafeCell};
use std::cmp;
use std::default::Default;
use std::iter;
use std::mem;
use std::ops::Range;
use std::slice::from_ref;
use std::sync::Arc as StdArc;
use style::context::QuirksMode;
use style::dom::OpaqueNode;
use style::properties::ComputedValues;
use style::selector_parser::{SelectorImpl, SelectorParser};
use style::stylesheets::Stylesheet;
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
    parent_node: MutNullableDom<Node>,

    /// The first child of this node.
    first_child: MutNullableDom<Node>,

    /// The last child of this node.
    last_child: MutNullableDom<Node>,

    /// The next sibling of this node.
    next_sibling: MutNullableDom<Node>,

    /// The previous sibling of this node.
    prev_sibling: MutNullableDom<Node>,

    /// The document that this node belongs to.
    owner_doc: MutNullableDom<Document>,

    /// Rare node data.
    rare_data: DomRefCell<Option<Box<NodeRareData>>>,

    /// The live list of children return by .childNodes.
    child_list: MutNullableDom<NodeList>,

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

    /// Style+Layout information.
    #[ignore_malloc_size_of = "trait object"]
    style_and_layout_data: DomRefCell<Option<Box<StyleAndOpaqueLayoutData>>>,
}

bitflags! {
    #[doc = "Flags for node items."]
    #[derive(JSTraceable, MallocSizeOf)]
    pub struct NodeFlags: u16 {
        #[doc = "Specifies whether this node is in a document."]
        const IS_IN_DOC = 1 << 0;

        #[doc = "Specifies whether this node needs style recalc on next reflow."]
        const HAS_DIRTY_DESCENDANTS = 1 << 1;

        #[doc = "Specifies whether or not there is an authentic click in progress on \
                 this element."]
        const CLICK_IN_PROGRESS = 1 << 2;

        #[doc = "Specifies whether this node is focusable and whether it is supposed \
                 to be reachable with using sequential focus navigation."]
        const SEQUENTIALLY_FOCUSABLE = 1 << 3;

        // There are two free bits here.

        #[doc = "Specifies whether the parser has set an associated form owner for \
                 this element. Only applicable for form-associatable elements."]
        const PARSER_ASSOCIATED_FORM_OWNER = 1 << 6;

        /// Whether this element has a snapshot stored due to a style or
        /// attribute change.
        ///
        /// See the `style::restyle_hints` module.
        const HAS_SNAPSHOT = 1 << 7;

        /// Whether this element has already handled the stored snapshot.
        const HANDLED_SNAPSHOT = 1 << 8;

        /// Whether this node participates in a shadow tree.
        const IS_IN_SHADOW_TREE = 1 << 9;

        /// Specifies whether this node's shadow-including root is a document.
        const IS_CONNECTED = 1 << 10;

        /// Whether this node has a weird parser insertion mode. i.e whether setting innerHTML
        /// needs extra work or not
        const HAS_WEIRD_PARSER_INSERTION_MODE = 1 << 11;
    }
}

impl NodeFlags {
    pub fn new() -> NodeFlags {
        NodeFlags::empty()
    }
}

/// suppress observers flag
/// <https://dom.spec.whatwg.org/#concept-node-insert>
/// <https://dom.spec.whatwg.org/#concept-node-remove>
#[derive(Clone, Copy, MallocSizeOf)]
enum SuppressObserver {
    Suppressed,
    Unsuppressed,
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
                assert!(before.parent_node.get().as_deref() == Some(self));
                let prev_sibling = before.GetPreviousSibling();
                match prev_sibling {
                    None => {
                        assert!(self.first_child.get().as_deref() == Some(*before));
                        self.first_child.set(Some(new_child));
                    },
                    Some(ref prev_sibling) => {
                        prev_sibling.next_sibling.set(Some(new_child));
                        new_child.prev_sibling.set(Some(&prev_sibling));
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
                    },
                }

                self.last_child.set(Some(new_child));
            },
        }

        new_child.parent_node.set(Some(self));
        self.children_count.set(self.children_count.get() + 1);

        let parent_in_doc = self.is_in_doc();
        let parent_in_shadow_tree = self.is_in_shadow_tree();
        let parent_is_connected = self.is_connected();

        for node in new_child.traverse_preorder(ShadowIncluding::No) {
            if parent_in_shadow_tree {
                if let Some(shadow_root) = self.containing_shadow_root() {
                    node.set_containing_shadow_root(Some(&*shadow_root));
                }
                debug_assert!(node.containing_shadow_root().is_some());
            }
            node.set_flag(NodeFlags::IS_IN_DOC, parent_in_doc);
            node.set_flag(NodeFlags::IS_IN_SHADOW_TREE, parent_in_shadow_tree);
            node.set_flag(NodeFlags::IS_CONNECTED, parent_is_connected);
            // Out-of-document elements never have the descendants flag set.
            debug_assert!(!node.get_flag(NodeFlags::HAS_DIRTY_DESCENDANTS));
            vtable_for(&&*node).bind_to_tree(&BindContext {
                tree_connected: parent_is_connected,
                tree_in_doc: parent_in_doc,
            });
        }
    }

    pub fn clean_up_layout_data(&self) {
        self.owner_doc().cancel_animations_for_node(self);
        self.style_and_layout_data.borrow_mut().take();
    }

    /// Clean up flags and unbind from tree.
    pub fn complete_remove_subtree(root: &Node, context: &UnbindContext) {
        for node in root.traverse_preorder(ShadowIncluding::Yes) {
            // Out-of-document elements never have the descendants flag set.
            node.set_flag(
                NodeFlags::IS_IN_DOC |
                    NodeFlags::IS_CONNECTED |
                    NodeFlags::HAS_DIRTY_DESCENDANTS |
                    NodeFlags::HAS_SNAPSHOT |
                    NodeFlags::HANDLED_SNAPSHOT,
                false,
            );
        }
        for node in root.traverse_preorder(ShadowIncluding::Yes) {
            node.clean_up_layout_data();

            // This needs to be in its own loop, because unbind_from_tree may
            // rely on the state of IS_IN_DOC of the context node's descendants,
            // e.g. when removing a <form>.
            vtable_for(&&*node).unbind_from_tree(&context);
            // https://dom.spec.whatwg.org/#concept-node-remove step 14
            if let Some(element) = node.as_custom_element() {
                ScriptThread::enqueue_callback_reaction(
                    &*element,
                    CallbackReaction::Disconnected,
                    None,
                );
            }
        }
    }

    /// Removes the given child from this node's list of children.
    ///
    /// Fails unless `child` is a child of this node.
    fn remove_child(&self, child: &Node, cached_index: Option<u32>) {
        assert!(child.parent_node.get().as_deref() == Some(self));
        self.note_dirty_descendants();

        let prev_sibling = child.GetPreviousSibling();
        match prev_sibling {
            None => {
                self.first_child.set(child.next_sibling.get().as_deref());
            },
            Some(ref prev_sibling) => {
                prev_sibling
                    .next_sibling
                    .set(child.next_sibling.get().as_deref());
            },
        }
        let next_sibling = child.GetNextSibling();
        match next_sibling {
            None => {
                self.last_child.set(child.prev_sibling.get().as_deref());
            },
            Some(ref next_sibling) => {
                next_sibling
                    .prev_sibling
                    .set(child.prev_sibling.get().as_deref());
            },
        }

        let context = UnbindContext::new(
            self,
            prev_sibling.as_deref(),
            next_sibling.as_deref(),
            cached_index,
        );

        child.prev_sibling.set(None);
        child.next_sibling.set(None);
        child.parent_node.set(None);
        self.children_count.set(self.children_count.get() - 1);

        Self::complete_remove_subtree(child, &context);
    }

    pub fn to_untrusted_node_address(&self) -> UntrustedNodeAddress {
        UntrustedNodeAddress(self.reflector().get_jsobject().get() as *const c_void)
    }

    pub fn to_opaque(&self) -> OpaqueNode {
        OpaqueNode(self.reflector().get_jsobject().get() as usize)
    }

    pub fn as_custom_element(&self) -> Option<DomRoot<Element>> {
        self.downcast::<Element>().and_then(|element| {
            if element.get_custom_element_definition().is_some() {
                Some(DomRoot::from_ref(element))
            } else {
                None
            }
        })
    }

    // https://html.spec.whatg.org/#fire_a_synthetic_mouse_event
    pub fn fire_synthetic_mouse_event_not_trusted(&self, name: DOMString) {
        // Spec says the choice of which global to create
        // the mouse event on is not well-defined,
        // and refers to heycam/webidl#135
        let win = window_from_node(self);

        let mouse_event = MouseEvent::new(
            &win, // ambiguous in spec
            name,
            EventBubbles::Bubbles,       // Step 3: bubbles
            EventCancelable::Cancelable, // Step 3: cancelable,
            Some(&win),                  // Step 7: view (this is unambiguous in spec)
            0,                           // detail uninitialized
            0,                           // coordinates uninitialized
            0,                           // coordinates uninitialized
            0,                           // coordinates uninitialized
            0,                           // coordinates uninitialized
            false,
            false,
            false,
            false, // Step 6 modifier keys TODO compositor hook needed
            0,     // button uninitialized (and therefore left)
            0,     // buttons uninitialized (and therefore none)
            None,  // related_target uninitialized,
            None,  // point_in_target uninitialized,
        );

        // Step 4: TODO composed flag for shadow root

        // Step 5
        mouse_event.upcast::<Event>().set_trusted(false);

        // Step 8: TODO keyboard modifiers

        mouse_event
            .upcast::<Event>()
            .dispatch(self.upcast::<EventTarget>(), false);
    }

    pub fn parent_directionality(&self) -> String {
        let mut current = self.GetParentNode();

        loop {
            match current {
                Some(node) => {
                    if let Some(directionality) = node
                        .downcast::<HTMLElement>()
                        .and_then(|html_element| html_element.directionality())
                    {
                        return directionality;
                    } else {
                        current = node.GetParentNode();
                    }
                },
                None => return "ltr".to_owned(),
            }
        }
    }
}

pub struct QuerySelectorIterator {
    selectors: SelectorList<SelectorImpl>,
    iterator: TreeIterator,
}

impl<'a> QuerySelectorIterator {
    fn new(iter: TreeIterator, selectors: SelectorList<SelectorImpl>) -> QuerySelectorIterator {
        QuerySelectorIterator {
            selectors: selectors,
            iterator: iter,
        }
    }
}

impl<'a> Iterator for QuerySelectorIterator {
    type Item = DomRoot<Node>;

    fn next(&mut self) -> Option<DomRoot<Node>> {
        let selectors = &self.selectors;

        self.iterator
            .by_ref()
            .filter_map(|node| {
                // TODO(cgaebel): Is it worth it to build a bloom filter here
                // (instead of passing `None`)? Probably.
                //
                // FIXME(bholley): Consider an nth-index cache here.
                let mut ctx = MatchingContext::new(
                    MatchingMode::Normal,
                    None,
                    None,
                    node.owner_doc().quirks_mode(),
                );
                if let Some(element) = DomRoot::downcast(node) {
                    if matches_selector_list(selectors, &element, &mut ctx) {
                        return Some(DomRoot::upcast(element));
                    }
                }
                None
            })
            .next()
    }
}

impl Node {
    impl_rare_data!(NodeRareData);

    /// Returns true if this node is before `other` in the same connected DOM
    /// tree.
    pub fn is_before(&self, other: &Node) -> bool {
        let cmp = other.CompareDocumentPosition(self);
        if cmp & NodeConstants::DOCUMENT_POSITION_DISCONNECTED != 0 {
            return false;
        }

        cmp & NodeConstants::DOCUMENT_POSITION_PRECEDING != 0
    }

    /// Return all registered mutation observers for this node. Lazily initialize the
    /// raredata if it does not exist.
    pub fn registered_mutation_observers_mut(&self) -> RefMut<Vec<RegisteredObserver>> {
        RefMut::map(self.ensure_rare_data(), |rare_data| {
            &mut rare_data.mutation_observers
        })
    }

    pub fn registered_mutation_observers(&self) -> Option<Ref<Vec<RegisteredObserver>>> {
        let rare_data: Ref<_> = self.rare_data.borrow();

        if rare_data.is_none() {
            return None;
        }
        Some(Ref::map(rare_data, |rare_data| {
            &rare_data.as_ref().unwrap().mutation_observers
        }))
    }

    /// Add a new mutation observer for a given node.
    pub fn add_mutation_observer(&self, observer: RegisteredObserver) {
        self.ensure_rare_data().mutation_observers.push(observer);
    }

    /// Removes the mutation observer for a given node.
    pub fn remove_mutation_observer(&self, observer: &MutationObserver) {
        self.ensure_rare_data()
            .mutation_observers
            .retain(|reg_obs| &*reg_obs.observer != observer)
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
        self.flags.get().contains(NodeFlags::IS_IN_DOC)
    }

    pub fn is_in_shadow_tree(&self) -> bool {
        self.flags.get().contains(NodeFlags::IS_IN_SHADOW_TREE)
    }

    pub fn has_weird_parser_insertion_mode(&self) -> bool {
        self.flags
            .get()
            .contains(NodeFlags::HAS_WEIRD_PARSER_INSERTION_MODE)
    }

    pub fn set_weird_parser_insertion_mode(&self) {
        self.set_flag(NodeFlags::HAS_WEIRD_PARSER_INSERTION_MODE, true)
    }

    pub fn is_connected(&self) -> bool {
        self.flags.get().contains(NodeFlags::IS_CONNECTED)
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
            NodeTypeId::CharacterData(_) => self.downcast::<CharacterData>().unwrap().Length(),
            _ => self.children_count(),
        }
    }

    // https://dom.spec.whatwg.org/#concept-tree-index
    pub fn index(&self) -> u32 {
        self.preceding_siblings().count() as u32
    }

    /// Returns true if this node has a parent.
    pub fn has_parent(&self) -> bool {
        self.parent_node.get().is_some()
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

    // FIXME(emilio): This and the function below should move to Element.
    pub fn note_dirty_descendants(&self) {
        self.owner_doc().note_node_with_dirty_descendants(self);
    }

    pub fn has_dirty_descendants(&self) -> bool {
        self.get_flag(NodeFlags::HAS_DIRTY_DESCENDANTS)
    }

    pub fn rev_version(&self) {
        // The new version counter is 1 plus the max of the node's current version counter,
        // its descendants version, and the document's version. Normally, this will just be
        // the document's version, but we do have to deal with the case where the node has moved
        // document, so may have a higher version count than its owning document.
        let doc: DomRoot<Node> = DomRoot::upcast(self.owner_doc());
        let version = cmp::max(
            self.inclusive_descendants_version(),
            doc.inclusive_descendants_version(),
        ) + 1;
        for ancestor in self.inclusive_ancestors(ShadowIncluding::No) {
            ancestor.inclusive_descendants_version.set(version);
        }
        doc.inclusive_descendants_version.set(version);
    }

    pub fn dirty(&self, damage: NodeDamage) {
        self.rev_version();
        if !self.is_connected() {
            return;
        }

        match self.type_id() {
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::Text)) => {
                self.parent_node.get().unwrap().dirty(damage)
            },
            NodeTypeId::Element(_) => self.downcast::<Element>().unwrap().restyle(damage),
            NodeTypeId::DocumentFragment(DocumentFragmentTypeId::ShadowRoot) => self
                .downcast::<ShadowRoot>()
                .unwrap()
                .Host()
                .upcast::<Element>()
                .restyle(damage),
            _ => {},
        };
    }

    /// The maximum version number of this node's descendants, including itself
    pub fn inclusive_descendants_version(&self) -> u64 {
        self.inclusive_descendants_version.get()
    }

    /// Iterates over this node and all its descendants, in preorder.
    pub fn traverse_preorder(&self, shadow_including: ShadowIncluding) -> TreeIterator {
        TreeIterator::new(self, shadow_including)
    }

    pub fn inclusively_following_siblings(&self) -> impl Iterator<Item = DomRoot<Node>> {
        SimpleNodeIterator {
            current: Some(DomRoot::from_ref(self)),
            next_node: |n| n.GetNextSibling(),
        }
    }

    pub fn inclusively_preceding_siblings(&self) -> impl Iterator<Item = DomRoot<Node>> {
        SimpleNodeIterator {
            current: Some(DomRoot::from_ref(self)),
            next_node: |n| n.GetPreviousSibling(),
        }
    }

    pub fn common_ancestor(
        &self,
        other: &Node,
        shadow_including: ShadowIncluding,
    ) -> Option<DomRoot<Node>> {
        for ancestor in self.inclusive_ancestors(shadow_including) {
            if other
                .inclusive_ancestors(shadow_including)
                .any(|node| node == ancestor)
            {
                return Some(ancestor);
            }
        }
        None
    }

    pub fn is_inclusive_ancestor_of(&self, parent: &Node) -> bool {
        self == parent || self.is_ancestor_of(parent)
    }

    pub fn is_ancestor_of(&self, parent: &Node) -> bool {
        parent.ancestors().any(|ancestor| &*ancestor == self)
    }

    fn is_shadow_including_inclusive_ancestor_of(&self, node: &Node) -> bool {
        node.inclusive_ancestors(ShadowIncluding::Yes)
            .any(|ancestor| &*ancestor == self)
    }

    pub fn following_siblings(&self) -> impl Iterator<Item = DomRoot<Node>> {
        SimpleNodeIterator {
            current: self.GetNextSibling(),
            next_node: |n| n.GetNextSibling(),
        }
    }

    pub fn preceding_siblings(&self) -> impl Iterator<Item = DomRoot<Node>> {
        SimpleNodeIterator {
            current: self.GetPreviousSibling(),
            next_node: |n| n.GetPreviousSibling(),
        }
    }

    pub fn following_nodes(&self, root: &Node) -> FollowingNodeIterator {
        FollowingNodeIterator {
            current: Some(DomRoot::from_ref(self)),
            root: DomRoot::from_ref(root),
        }
    }

    pub fn preceding_nodes(&self, root: &Node) -> PrecedingNodeIterator {
        PrecedingNodeIterator {
            current: Some(DomRoot::from_ref(self)),
            root: DomRoot::from_ref(root),
        }
    }

    pub fn descending_last_children(&self) -> impl Iterator<Item = DomRoot<Node>> {
        SimpleNodeIterator {
            current: self.GetLastChild(),
            next_node: |n| n.GetLastChild(),
        }
    }

    pub fn is_parent_of(&self, child: &Node) -> bool {
        child
            .parent_node
            .get()
            .map_or(false, |parent| &*parent == self)
    }

    pub fn to_trusted_node_address(&self) -> TrustedNodeAddress {
        TrustedNodeAddress(&*self as *const Node as *const libc::c_void)
    }

    /// Returns the rendered bounding content box if the element is rendered,
    /// and none otherwise.
    pub fn bounding_content_box(&self) -> Option<Rect<Au>> {
        window_from_node(self).content_box_query(self)
    }

    pub fn bounding_content_box_or_zero(&self) -> Rect<Au> {
        self.bounding_content_box().unwrap_or_else(Rect::zero)
    }

    pub fn content_boxes(&self) -> Vec<Rect<Au>> {
        window_from_node(self).content_boxes_query(self)
    }

    pub fn client_rect(&self) -> Rect<i32> {
        window_from_node(self).client_rect_query(self)
    }

    // https://drafts.csswg.org/cssom-view/#dom-element-scrollwidth
    // https://drafts.csswg.org/cssom-view/#dom-element-scrollheight
    pub fn scroll_area(&self) -> Rect<i32> {
        // Step 1
        let document = self.owner_doc();
        // Step 3
        let window = document.window();

        let html_element = document.GetDocumentElement();

        let is_body_element = self
            .downcast::<HTMLBodyElement>()
            .map_or(false, |e| e.is_the_html_body_element());

        let scroll_area = window.scroll_area_query(self);

        match (
            document != window.Document(),
            is_body_element,
            document.quirks_mode(),
            html_element.as_deref() == self.downcast::<Element>(),
        ) {
            // Step 2 && Step 5
            (true, _, _, _) | (_, false, QuirksMode::Quirks, true) => Rect::zero(),
            // Step 6 && Step 7
            (false, false, _, true) | (false, true, QuirksMode::Quirks, _) => Rect::new(
                Point2D::new(window.ScrollX(), window.ScrollY()),
                Size2D::new(
                    cmp::max(window.InnerWidth(), scroll_area.size.width),
                    cmp::max(window.InnerHeight(), scroll_area.size.height),
                ),
            ),
            // Step 9
            _ => scroll_area,
        }
    }

    pub fn scroll_offset(&self) -> Vector2D<f32> {
        let document = self.owner_doc();
        let window = document.window();
        window.scroll_offset_query(self).to_untyped()
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
        let node = self.owner_doc().node_from_nodes_and_strings(nodes)?;

        // Step 5.
        let viable_previous_sibling = match viable_previous_sibling {
            Some(ref viable_previous_sibling) => viable_previous_sibling.next_sibling.get(),
            None => parent.first_child.get(),
        };

        // Step 6.
        Node::pre_insert(&node, &parent, viable_previous_sibling.as_deref())?;

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
        let node = self.owner_doc().node_from_nodes_and_strings(nodes)?;

        // Step 5.
        Node::pre_insert(&node, &parent, viable_next_sibling.as_deref())?;

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
        let node = self.owner_doc().node_from_nodes_and_strings(nodes)?;
        if self.parent_node == Some(&*parent) {
            // Step 5.
            parent.ReplaceChild(&node, self)?;
        } else {
            // Step 6.
            Node::pre_insert(&node, &parent, viable_next_sibling.as_deref())?;
        }
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-prepend
    pub fn prepend(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let doc = self.owner_doc();
        let node = doc.node_from_nodes_and_strings(nodes)?;
        // Step 2.
        let first_child = self.first_child.get();
        Node::pre_insert(&node, self, first_child.as_deref()).map(|_| ())
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-append
    pub fn append(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let doc = self.owner_doc();
        let node = doc.node_from_nodes_and_strings(nodes)?;
        // Step 2.
        self.AppendChild(&node).map(|_| ())
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-replacechildren
    pub fn replace_children(&self, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let doc = self.owner_doc();
        let node = doc.node_from_nodes_and_strings(nodes)?;
        // Step 2.
        Node::ensure_pre_insertion_validity(&node, self, None)?;
        // Step 3.
        Node::replace_all(Some(&node), self);
        Ok(())
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselector
    pub fn query_selector(&self, selectors: DOMString) -> Fallible<Option<DomRoot<Element>>> {
        // Step 1.
        match SelectorParser::parse_author_origin_no_namespace(&selectors) {
            // Step 2.
            Err(_) => Err(Error::Syntax),
            // Step 3.
            Ok(selectors) => {
                // FIXME(bholley): Consider an nth-index cache here.
                let mut ctx = MatchingContext::new(
                    MatchingMode::Normal,
                    None,
                    None,
                    self.owner_doc().quirks_mode(),
                );
                Ok(self
                    .traverse_preorder(ShadowIncluding::No)
                    .filter_map(DomRoot::downcast)
                    .find(|element| matches_selector_list(&selectors, element, &mut ctx)))
            },
        }
    }

    /// <https://dom.spec.whatwg.org/#scope-match-a-selectors-string>
    /// Get an iterator over all nodes which match a set of selectors
    /// Be careful not to do anything which may manipulate the DOM tree
    /// whilst iterating, otherwise the iterator may be invalidated.
    pub fn query_selector_iter(&self, selectors: DOMString) -> Fallible<QuerySelectorIterator> {
        // Step 1.
        match SelectorParser::parse_author_origin_no_namespace(&selectors) {
            // Step 2.
            Err(_) => Err(Error::Syntax),
            // Step 3.
            Ok(selectors) => {
                let mut descendants = self.traverse_preorder(ShadowIncluding::No);
                // Skip the root of the tree.
                assert!(&*descendants.next().unwrap() == self);
                Ok(QuerySelectorIterator::new(descendants, selectors))
            },
        }
    }

    // https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
    #[allow(unsafe_code)]
    pub fn query_selector_all(&self, selectors: DOMString) -> Fallible<DomRoot<NodeList>> {
        let window = window_from_node(self);
        let iter = self.query_selector_iter(selectors)?;
        Ok(NodeList::new_simple_list(&window, iter))
    }

    pub fn ancestors(&self) -> impl Iterator<Item = DomRoot<Node>> {
        SimpleNodeIterator {
            current: self.GetParentNode(),
            next_node: |n| n.GetParentNode(),
        }
    }

    /// https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-ancestor
    pub fn inclusive_ancestors(
        &self,
        shadow_including: ShadowIncluding,
    ) -> impl Iterator<Item = DomRoot<Node>> {
        SimpleNodeIterator {
            current: Some(DomRoot::from_ref(self)),
            next_node: move |n| {
                if shadow_including == ShadowIncluding::Yes {
                    if let Some(shadow_root) = n.downcast::<ShadowRoot>() {
                        return Some(DomRoot::from_ref(shadow_root.Host().upcast::<Node>()));
                    }
                }
                n.GetParentNode()
            },
        }
    }

    pub fn owner_doc(&self) -> DomRoot<Document> {
        self.owner_doc.get().unwrap()
    }

    pub fn set_owner_doc(&self, document: &Document) {
        self.owner_doc.set(Some(document));
    }

    pub fn containing_shadow_root(&self) -> Option<DomRoot<ShadowRoot>> {
        self.rare_data()
            .as_ref()?
            .containing_shadow_root
            .as_ref()
            .map(|sr| DomRoot::from_ref(&**sr))
    }

    pub fn set_containing_shadow_root(&self, shadow_root: Option<&ShadowRoot>) {
        self.ensure_rare_data().containing_shadow_root = shadow_root.map(Dom::from_ref);
    }

    pub fn is_in_html_doc(&self) -> bool {
        self.owner_doc().is_html_document()
    }

    pub fn is_connected_with_browsing_context(&self) -> bool {
        self.is_connected() && self.owner_doc().browsing_context().is_some()
    }

    pub fn children(&self) -> impl Iterator<Item = DomRoot<Node>> {
        SimpleNodeIterator {
            current: self.GetFirstChild(),
            next_node: |n| n.GetNextSibling(),
        }
    }

    pub fn rev_children(&self) -> impl Iterator<Item = DomRoot<Node>> {
        SimpleNodeIterator {
            current: self.GetLastChild(),
            next_node: |n| n.GetPreviousSibling(),
        }
    }

    pub fn child_elements(&self) -> impl Iterator<Item = DomRoot<Element>> {
        self.children()
            .filter_map(DomRoot::downcast as fn(_) -> _)
            .peekable()
    }

    pub fn remove_self(&self) {
        if let Some(ref parent) = self.GetParentNode() {
            Node::remove(self, &parent, SuppressObserver::Unsuppressed);
        }
    }

    pub fn unique_id(&self) -> String {
        let mut rare_data = self.ensure_rare_data();

        if rare_data.unique_id.is_none() {
            let id = UniqueId::new();
            ScriptThread::save_node_id(id.borrow().to_simple().to_string());
            rare_data.unique_id = Some(id);
        }
        rare_data
            .unique_id
            .as_ref()
            .unwrap()
            .borrow()
            .to_simple()
            .to_string()
    }

    pub fn summarize(&self) -> NodeInfo {
        let USVString(base_uri) = self.BaseURI();
        NodeInfo {
            uniqueId: self.unique_id(),
            baseURI: base_uri,
            parent: self
                .GetParentNode()
                .map_or("".to_owned(), |node| node.unique_id()),
            nodeType: self.NodeType(),
            namespaceURI: String::new(), //FIXME
            nodeName: String::from(self.NodeName()),
            numChildren: self.ChildNodes().Length() as usize,

            //FIXME doctype nodes only
            name: String::new(),
            publicId: String::new(),
            systemId: String::new(),
            attrs: self.downcast().map(Element::summarize).unwrap_or(vec![]),

            isDocumentElement: self
                .owner_doc()
                .GetDocumentElement()
                .map_or(false, |elem| elem.upcast::<Node>() == self),

            shortValue: self.GetNodeValue().map(String::from).unwrap_or_default(), //FIXME: truncate
            incompleteValue: false, //FIXME: reflect truncation
        }
    }

    /// Used by `HTMLTableSectionElement::InsertRow` and `HTMLTableRowElement::InsertCell`
    pub fn insert_cell_or_row<F, G, I>(
        &self,
        index: i32,
        get_items: F,
        new_child: G,
    ) -> Fallible<DomRoot<HTMLElement>>
    where
        F: Fn() -> DomRoot<HTMLCollection>,
        G: Fn() -> DomRoot<I>,
        I: DerivedFrom<Node> + DerivedFrom<HTMLElement> + DomObject,
    {
        if index < -1 {
            return Err(Error::IndexSize);
        }

        let tr = new_child();

        {
            let tr_node = tr.upcast::<Node>();
            if index == -1 {
                self.InsertBefore(tr_node, None)?;
            } else {
                let items = get_items();
                let node = match items
                    .elements_iter()
                    .map(DomRoot::upcast::<Node>)
                    .map(Some)
                    .chain(iter::once(None))
                    .nth(index as usize)
                {
                    None => return Err(Error::IndexSize),
                    Some(node) => node,
                };
                self.InsertBefore(tr_node, node.as_deref())?;
            }
        }

        Ok(DomRoot::upcast::<HTMLElement>(tr))
    }

    /// Used by `HTMLTableSectionElement::DeleteRow` and `HTMLTableRowElement::DeleteCell`
    pub fn delete_cell_or_row<F, G>(
        &self,
        index: i32,
        get_items: F,
        is_delete_type: G,
    ) -> ErrorResult
    where
        F: Fn() -> DomRoot<HTMLCollection>,
        G: Fn(&Element) -> bool,
    {
        let element = match index {
            index if index < -1 => return Err(Error::IndexSize),
            -1 => {
                let last_child = self.upcast::<Node>().GetLastChild();
                match last_child.and_then(|node| {
                    node.inclusively_preceding_siblings()
                        .filter_map(DomRoot::downcast::<Element>)
                        .filter(|elem| is_delete_type(elem))
                        .next()
                }) {
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

    pub fn get_stylesheet(&self) -> Option<Arc<Stylesheet>> {
        if let Some(node) = self.downcast::<HTMLStyleElement>() {
            node.get_stylesheet()
        } else if let Some(node) = self.downcast::<HTMLLinkElement>() {
            node.get_stylesheet()
        } else if let Some(node) = self.downcast::<HTMLMetaElement>() {
            node.get_stylesheet()
        } else {
            None
        }
    }

    pub fn get_cssom_stylesheet(&self) -> Option<DomRoot<CSSStyleSheet>> {
        if let Some(node) = self.downcast::<HTMLStyleElement>() {
            node.get_cssom_stylesheet()
        } else if let Some(node) = self.downcast::<HTMLLinkElement>() {
            node.get_cssom_stylesheet()
        } else if let Some(node) = self.downcast::<HTMLMetaElement>() {
            node.get_cssom_stylesheet()
        } else {
            None
        }
    }

    /// https://dom.spec.whatwg.org/#retarget
    pub fn retarget(&self, b: &Node) -> DomRoot<Node> {
        let mut a = DomRoot::from_ref(&*self);
        loop {
            // Step 1.
            let a_root = a.GetRootNode(&GetRootNodeOptions::empty());
            if !a_root.is::<ShadowRoot>() || a_root.is_shadow_including_inclusive_ancestor_of(b) {
                return DomRoot::from_ref(&a);
            }

            // Step 2.
            a = DomRoot::from_ref(
                a_root
                    .downcast::<ShadowRoot>()
                    .unwrap()
                    .Host()
                    .upcast::<Node>(),
            );
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-document-nameditem-filter
    pub fn is_document_named_item(&self, name: &Atom) -> bool {
        let html_elem_type = match self.type_id() {
            NodeTypeId::Element(ElementTypeId::HTMLElement(type_)) => type_,
            _ => return false,
        };
        let elem = self
            .downcast::<Element>()
            .expect("Node with an Element::HTMLElement NodeTypeID must be an Element");
        match html_elem_type {
            HTMLElementTypeId::HTMLFormElement | HTMLElementTypeId::HTMLIFrameElement => {
                elem.get_name().map_or(false, |n| n == *name)
            },
            HTMLElementTypeId::HTMLImageElement =>
            // Images can match by id, but only when their name is non-empty.
            {
                elem.get_name().map_or(false, |n| {
                    n == *name || elem.get_id().map_or(false, |i| i == *name)
                })
            },
            // TODO: Handle <embed> and <object>; these depend on
            // whether the element is "exposed", a concept which
            // doesn't fully make sense until embed/object behaviors
            // are actually implemented.
            _ => false,
        }
    }

    pub fn is_styled(&self) -> bool {
        self.style_and_layout_data.borrow().is_some()
    }

    pub fn is_display_none(&self) -> bool {
        self.style_and_layout_data
            .borrow()
            .as_ref()
            .map_or(true, |data| {
                data.style_data
                    .element_data
                    .borrow()
                    .styles
                    .primary()
                    .get_box()
                    .display
                    .is_none()
            })
    }

    pub fn style(&self) -> Option<Arc<ComputedValues>> {
        if !window_from_node(self).layout_reflow(QueryMsg::StyleQuery) {
            return None;
        }
        self.style_and_layout_data.borrow().as_ref().map(|data| {
            data.style_data
                .element_data
                .borrow()
                .styles
                .primary()
                .clone()
        })
    }
}

/// Iterate through `nodes` until we find a `Node` that is not in `not_in`
fn first_node_not_in<I>(mut nodes: I, not_in: &[NodeOrString]) -> Option<DomRoot<Node>>
where
    I: Iterator<Item = DomRoot<Node>>,
{
    nodes.find(|node| {
        not_in.iter().all(|n| match *n {
            NodeOrString::Node(ref n) => n != node,
            _ => true,
        })
    })
}

/// If the given untrusted node address represents a valid DOM node in the given runtime,
/// returns it.
#[allow(unsafe_code)]
pub unsafe fn from_untrusted_node_address(candidate: UntrustedNodeAddress) -> DomRoot<Node> {
    // https://github.com/servo/servo/issues/6383
    let candidate: uintptr_t = mem::transmute(candidate.0);
    //        let object: *mut JSObject = jsfriendapi::bindgen::JS_GetAddressableObject(runtime,
    //                                                                                  candidate);
    let object: *mut JSObject = mem::transmute(candidate);
    if object.is_null() {
        panic!("Attempted to create a `Dom<Node>` from an invalid pointer!")
    }
    let boxed_node = conversions::private_from_object(object) as *const Node;
    DomRoot::from_ref(&*boxed_node)
}

#[allow(unsafe_code)]
pub trait LayoutNodeHelpers<'dom> {
    fn type_id_for_layout(self) -> NodeTypeId;

    fn composed_parent_node_ref(self) -> Option<LayoutDom<'dom, Node>>;
    fn first_child_ref(self) -> Option<LayoutDom<'dom, Node>>;
    fn last_child_ref(self) -> Option<LayoutDom<'dom, Node>>;
    fn prev_sibling_ref(self) -> Option<LayoutDom<'dom, Node>>;
    fn next_sibling_ref(self) -> Option<LayoutDom<'dom, Node>>;

    fn owner_doc_for_layout(self) -> LayoutDom<'dom, Document>;
    fn containing_shadow_root_for_layout(self) -> Option<LayoutDom<'dom, ShadowRoot>>;

    fn is_element_for_layout(self) -> bool;
    unsafe fn get_flag(self, flag: NodeFlags) -> bool;
    unsafe fn set_flag(self, flag: NodeFlags, value: bool);

    fn children_count(self) -> u32;

    fn get_style_and_opaque_layout_data(self) -> Option<&'dom StyleAndOpaqueLayoutData>;
    unsafe fn init_style_and_opaque_layout_data(self, data: Box<StyleAndOpaqueLayoutData>);
    unsafe fn take_style_and_opaque_layout_data(self) -> Box<StyleAndOpaqueLayoutData>;

    fn text_content(self) -> Cow<'dom, str>;
    fn selection(self) -> Option<Range<usize>>;
    fn image_url(self) -> Option<ServoUrl>;
    fn image_density(self) -> Option<f64>;
    fn image_data(self) -> Option<(Option<StdArc<Image>>, Option<ImageMetadata>)>;
    fn canvas_data(self) -> Option<HTMLCanvasData>;
    fn media_data(self) -> Option<HTMLMediaData>;
    fn svg_data(self) -> Option<SVGSVGData>;
    fn iframe_browsing_context_id(self) -> Option<BrowsingContextId>;
    fn iframe_pipeline_id(self) -> Option<PipelineId>;
    fn opaque(self) -> OpaqueNode;
}

impl<'dom> LayoutDom<'dom, Node> {
    #[inline]
    #[allow(unsafe_code)]
    fn parent_node_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().parent_node.get_inner_as_layout() }
    }
}

impl<'dom> LayoutNodeHelpers<'dom> for LayoutDom<'dom, Node> {
    #[inline]
    #[allow(unsafe_code)]
    fn type_id_for_layout(self) -> NodeTypeId {
        unsafe { self.unsafe_get().type_id() }
    }

    #[inline]
    fn is_element_for_layout(self) -> bool {
        self.is::<Element>()
    }

    #[inline]
    fn composed_parent_node_ref(self) -> Option<LayoutDom<'dom, Node>> {
        let parent = self.parent_node_ref();
        if let Some(parent) = parent {
            if let Some(shadow_root) = parent.downcast::<ShadowRoot>() {
                return Some(shadow_root.get_host_for_layout().upcast());
            }
        }
        parent
    }

    #[inline]
    #[allow(unsafe_code)]
    fn first_child_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().first_child.get_inner_as_layout() }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn last_child_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().last_child.get_inner_as_layout() }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn prev_sibling_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().prev_sibling.get_inner_as_layout() }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn next_sibling_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().next_sibling.get_inner_as_layout() }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn owner_doc_for_layout(self) -> LayoutDom<'dom, Document> {
        unsafe { self.unsafe_get().owner_doc.get_inner_as_layout().unwrap() }
    }

    #[inline]
    #[allow(unsafe_code)]
    fn containing_shadow_root_for_layout(self) -> Option<LayoutDom<'dom, ShadowRoot>> {
        unsafe {
            self.unsafe_get()
                .rare_data
                .borrow_for_layout()
                .as_ref()?
                .containing_shadow_root
                .as_ref()
                .map(|sr| sr.to_layout())
        }
    }

    // FIXME(nox): get_flag/set_flag (especially the latter) are not safe because
    // they mutate stuff while values of this type can be used from multiple
    // threads at once, this should be revisited.

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn get_flag(self, flag: NodeFlags) -> bool {
        (*self.unsafe_get()).flags.get().contains(flag)
    }

    #[inline]
    #[allow(unsafe_code)]
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

    #[inline]
    #[allow(unsafe_code)]
    fn children_count(self) -> u32 {
        unsafe { self.unsafe_get().children_count.get() }
    }

    // FIXME(nox): How we handle style and layout data needs to be completely
    // revisited so we can do that more cleanly and safely in layout 2020.

    #[inline]
    #[allow(unsafe_code)]
    fn get_style_and_opaque_layout_data(self) -> Option<&'dom StyleAndOpaqueLayoutData> {
        unsafe {
            self.unsafe_get()
                .style_and_layout_data
                .borrow_for_layout()
                .as_deref()
        }
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn init_style_and_opaque_layout_data(self, val: Box<StyleAndOpaqueLayoutData>) {
        let data = self
            .unsafe_get()
            .style_and_layout_data
            .borrow_mut_for_layout();
        debug_assert!(data.is_none());
        *data = Some(val);
    }

    #[inline]
    #[allow(unsafe_code)]
    unsafe fn take_style_and_opaque_layout_data(self) -> Box<StyleAndOpaqueLayoutData> {
        self.unsafe_get()
            .style_and_layout_data
            .borrow_mut_for_layout()
            .take()
            .unwrap()
    }

    fn text_content(self) -> Cow<'dom, str> {
        if let Some(text) = self.downcast::<Text>() {
            return text.upcast().data_for_layout().into();
        }

        if let Some(input) = self.downcast::<HTMLInputElement>() {
            return input.value_for_layout();
        }

        if let Some(area) = self.downcast::<HTMLTextAreaElement>() {
            return area.value_for_layout().into();
        }

        panic!("not text!")
    }

    fn selection(self) -> Option<Range<usize>> {
        if let Some(area) = self.downcast::<HTMLTextAreaElement>() {
            return area.selection_for_layout();
        }

        if let Some(input) = self.downcast::<HTMLInputElement>() {
            return input.selection_for_layout();
        }

        None
    }

    fn image_url(self) -> Option<ServoUrl> {
        self.downcast::<HTMLImageElement>()
            .expect("not an image!")
            .image_url()
    }

    fn image_data(self) -> Option<(Option<StdArc<Image>>, Option<ImageMetadata>)> {
        self.downcast::<HTMLImageElement>().map(|e| e.image_data())
    }

    fn image_density(self) -> Option<f64> {
        self.downcast::<HTMLImageElement>()
            .expect("not an image!")
            .image_density()
    }

    fn canvas_data(self) -> Option<HTMLCanvasData> {
        self.downcast::<HTMLCanvasElement>()
            .map(|canvas| canvas.data())
    }

    fn media_data(self) -> Option<HTMLMediaData> {
        self.downcast::<HTMLMediaElement>()
            .map(|media| media.data())
    }

    fn svg_data(self) -> Option<SVGSVGData> {
        self.downcast::<SVGSVGElement>().map(|svg| svg.data())
    }

    fn iframe_browsing_context_id(self) -> Option<BrowsingContextId> {
        let iframe_element = self
            .downcast::<HTMLIFrameElement>()
            .expect("not an iframe element!");
        iframe_element.browsing_context_id()
    }

    fn iframe_pipeline_id(self) -> Option<PipelineId> {
        let iframe_element = self
            .downcast::<HTMLIFrameElement>()
            .expect("not an iframe element!");
        iframe_element.pipeline_id()
    }

    #[allow(unsafe_code)]
    fn opaque(self) -> OpaqueNode {
        unsafe { OpaqueNode(self.get_jsobject() as usize) }
    }
}

//
// Iteration and traversal
//

pub struct FollowingNodeIterator {
    current: Option<DomRoot<Node>>,
    root: DomRoot<Node>,
}

impl FollowingNodeIterator {
    /// Skips iterating the children of the current node
    pub fn next_skipping_children(&mut self) -> Option<DomRoot<Node>> {
        let current = self.current.take()?;
        self.next_skipping_children_impl(current)
    }

    fn next_skipping_children_impl(&mut self, current: DomRoot<Node>) -> Option<DomRoot<Node>> {
        if self.root == current {
            self.current = None;
            return None;
        }

        if let Some(next_sibling) = current.GetNextSibling() {
            self.current = Some(next_sibling);
            return current.GetNextSibling();
        }

        for ancestor in current.inclusive_ancestors(ShadowIncluding::No) {
            if self.root == ancestor {
                break;
            }
            if let Some(next_sibling) = ancestor.GetNextSibling() {
                self.current = Some(next_sibling);
                return ancestor.GetNextSibling();
            }
        }
        self.current = None;
        None
    }
}

impl Iterator for FollowingNodeIterator {
    type Item = DomRoot<Node>;

    // https://dom.spec.whatwg.org/#concept-tree-following
    fn next(&mut self) -> Option<DomRoot<Node>> {
        let current = self.current.take()?;

        if let Some(first_child) = current.GetFirstChild() {
            self.current = Some(first_child);
            return current.GetFirstChild();
        }

        self.next_skipping_children_impl(current)
    }
}

pub struct PrecedingNodeIterator {
    current: Option<DomRoot<Node>>,
    root: DomRoot<Node>,
}

impl Iterator for PrecedingNodeIterator {
    type Item = DomRoot<Node>;

    // https://dom.spec.whatwg.org/#concept-tree-preceding
    fn next(&mut self) -> Option<DomRoot<Node>> {
        let current = self.current.take()?;

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

struct SimpleNodeIterator<I>
where
    I: Fn(&Node) -> Option<DomRoot<Node>>,
{
    current: Option<DomRoot<Node>>,
    next_node: I,
}

impl<I> Iterator for SimpleNodeIterator<I>
where
    I: Fn(&Node) -> Option<DomRoot<Node>>,
{
    type Item = DomRoot<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take();
        self.current = current.as_ref().and_then(|c| (self.next_node)(c));
        current
    }
}

/// Whether a tree traversal should pass shadow tree boundaries.
#[derive(Clone, Copy, PartialEq)]
pub enum ShadowIncluding {
    No,
    Yes,
}

pub struct TreeIterator {
    current: Option<DomRoot<Node>>,
    depth: usize,
    shadow_including: bool,
}

impl TreeIterator {
    fn new(root: &Node, shadow_including: ShadowIncluding) -> TreeIterator {
        TreeIterator {
            current: Some(DomRoot::from_ref(root)),
            depth: 0,
            shadow_including: shadow_including == ShadowIncluding::Yes,
        }
    }

    pub fn next_skipping_children(&mut self) -> Option<DomRoot<Node>> {
        let current = self.current.take()?;

        self.next_skipping_children_impl(current)
    }

    fn next_skipping_children_impl(&mut self, current: DomRoot<Node>) -> Option<DomRoot<Node>> {
        let iter = current.inclusive_ancestors(if self.shadow_including {
            ShadowIncluding::Yes
        } else {
            ShadowIncluding::No
        });

        for ancestor in iter {
            if self.depth == 0 {
                break;
            }
            if let Some(next_sibling) = ancestor.GetNextSibling() {
                self.current = Some(next_sibling);
                return Some(current);
            }
            self.depth -= 1;
        }
        debug_assert_eq!(self.depth, 0);
        self.current = None;
        Some(current)
    }
}

impl Iterator for TreeIterator {
    type Item = DomRoot<Node>;

    // https://dom.spec.whatwg.org/#concept-tree-order
    // https://dom.spec.whatwg.org/#concept-shadow-including-tree-order
    fn next(&mut self) -> Option<DomRoot<Node>> {
        let current = self.current.take()?;

        if !self.shadow_including {
            if let Some(element) = current.downcast::<Element>() {
                if element.is_shadow_host() {
                    return self.next_skipping_children_impl(current);
                }
            }
        }

        if let Some(first_child) = current.GetFirstChild() {
            self.current = Some(first_child);
            self.depth += 1;
            return Some(current);
        };

        self.next_skipping_children_impl(current)
    }
}

/// Specifies whether children must be recursively cloned or not.
#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub enum CloneChildrenFlag {
    CloneChildren,
    DoNotCloneChildren,
}

fn as_uintptr<T>(t: &T) -> uintptr_t {
    t as *const T as uintptr_t
}

impl Node {
    pub fn reflect_node<N>(node: Box<N>, document: &Document) -> DomRoot<N>
    where
        N: DerivedFrom<Node> + DomObject + DomObjectWrap,
    {
        let window = document.window();
        reflect_dom_object(node, window)
    }

    pub fn new_inherited(doc: &Document) -> Node {
        Node::new_(NodeFlags::new(), Some(doc))
    }

    #[allow(unrooted_must_root)]
    pub fn new_document_node() -> Node {
        Node::new_(
            NodeFlags::new() | NodeFlags::IS_IN_DOC | NodeFlags::IS_CONNECTED,
            None,
        )
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
            owner_doc: MutNullableDom::new(doc),
            rare_data: Default::default(),
            child_list: Default::default(),
            children_count: Cell::new(0u32),
            flags: Cell::new(flags),
            inclusive_descendants_version: Cell::new(0),
            ranges: WeakRangeVec::new(),

            style_and_layout_data: Default::default(),
        }
    }

    // https://dom.spec.whatwg.org/#concept-node-adopt
    pub fn adopt(node: &Node, document: &Document) {
        document.add_script_and_layout_blocker();

        // Step 1.
        let old_doc = node.owner_doc();
        old_doc.add_script_and_layout_blocker();
        // Step 2.
        node.remove_self();
        // Step 3.
        if &*old_doc != document {
            // Step 3.1.
            for descendant in node.traverse_preorder(ShadowIncluding::Yes) {
                descendant.set_owner_doc(document);
            }
            for descendant in node
                .traverse_preorder(ShadowIncluding::Yes)
                .filter_map(|d| d.as_custom_element())
            {
                // Step 3.2.
                ScriptThread::enqueue_callback_reaction(
                    &*descendant,
                    CallbackReaction::Adopted(old_doc.clone(), DomRoot::from_ref(document)),
                    None,
                );
            }
            for descendant in node.traverse_preorder(ShadowIncluding::Yes) {
                // Step 3.3.
                vtable_for(&descendant).adopting_steps(&old_doc);
            }
        }

        old_doc.remove_script_and_layout_blocker();
        document.remove_script_and_layout_blocker();
    }

    // https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity
    pub fn ensure_pre_insertion_validity(
        node: &Node,
        parent: &Node,
        child: Option<&Node>,
    ) -> ErrorResult {
        // Step 1.
        match parent.type_id() {
            NodeTypeId::Document(_) | NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(..) => {
                ()
            },
            _ => return Err(Error::HierarchyRequest),
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
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => {
                if parent.is::<Document>() {
                    return Err(Error::HierarchyRequest);
                }
            },
            NodeTypeId::DocumentType => {
                if !parent.is::<Document>() {
                    return Err(Error::HierarchyRequest);
                }
            },
            NodeTypeId::DocumentFragment(_) |
            NodeTypeId::Element(_) |
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) |
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => (),
            NodeTypeId::Document(_) => return Err(Error::HierarchyRequest),
            NodeTypeId::Attr => unreachable!(),
        }

        // Step 6.
        if parent.is::<Document>() {
            match node.type_id() {
                // Step 6.1
                NodeTypeId::DocumentFragment(_) => {
                    // Step 6.1.1(b)
                    if node.children().any(|c| c.is::<Text>()) {
                        return Err(Error::HierarchyRequest);
                    }
                    match node.child_elements().count() {
                        0 => (),
                        // Step 6.1.2
                        1 => {
                            if !parent.child_elements().next().is_none() {
                                return Err(Error::HierarchyRequest);
                            }
                            if let Some(child) = child {
                                if child
                                    .inclusively_following_siblings()
                                    .any(|child| child.is_doctype())
                                {
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
                    if !parent.child_elements().next().is_none() {
                        return Err(Error::HierarchyRequest);
                    }
                    if let Some(ref child) = child {
                        if child
                            .inclusively_following_siblings()
                            .any(|child| child.is_doctype())
                        {
                            return Err(Error::HierarchyRequest);
                        }
                    }
                },
                // Step 6.3
                NodeTypeId::DocumentType => {
                    if parent.children().any(|c| c.is_doctype()) {
                        return Err(Error::HierarchyRequest);
                    }
                    match child {
                        Some(child) => {
                            if parent
                                .children()
                                .take_while(|c| &**c != child)
                                .any(|c| c.is::<Element>())
                            {
                                return Err(Error::HierarchyRequest);
                            }
                        },
                        None => {
                            if !parent.child_elements().next().is_none() {
                                return Err(Error::HierarchyRequest);
                            }
                        },
                    }
                },
                NodeTypeId::CharacterData(_) => (),
                NodeTypeId::Document(_) => unreachable!(),
                NodeTypeId::Attr => unreachable!(),
            }
        }
        Ok(())
    }

    // https://dom.spec.whatwg.org/#concept-node-pre-insert
    pub fn pre_insert(node: &Node, parent: &Node, child: Option<&Node>) -> Fallible<DomRoot<Node>> {
        // Step 1.
        Node::ensure_pre_insertion_validity(node, parent, child)?;

        // Steps 2-3.
        let reference_child_root;
        let reference_child = match child {
            Some(child) if child == node => {
                reference_child_root = node.GetNextSibling();
                reference_child_root.as_deref()
            },
            _ => child,
        };

        // Step 4.
        let document = document_from_node(parent);
        Node::adopt(node, &document);

        // Step 5.
        Node::insert(
            node,
            parent,
            reference_child,
            SuppressObserver::Unsuppressed,
        );

        // Step 6.
        Ok(DomRoot::from_ref(node))
    }

    // https://dom.spec.whatwg.org/#concept-node-insert
    fn insert(
        node: &Node,
        parent: &Node,
        child: Option<&Node>,
        suppress_observers: SuppressObserver,
    ) {
        node.owner_doc().add_script_and_layout_blocker();
        debug_assert!(&*node.owner_doc() == &*parent.owner_doc());
        debug_assert!(child.map_or(true, |child| Some(parent) ==
            child.GetParentNode().as_deref()));

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
        rooted_vec!(let mut new_nodes);
        let new_nodes = if let NodeTypeId::DocumentFragment(_) = node.type_id() {
            // Step 3.
            new_nodes.extend(node.children().map(|kid| Dom::from_ref(&*kid)));
            // Step 4.
            for kid in &*new_nodes {
                Node::remove(kid, node, SuppressObserver::Suppressed);
            }
            // Step 5.
            vtable_for(&node).children_changed(&ChildrenMutation::replace_all(new_nodes.r(), &[]));

            let mutation = Mutation::ChildList {
                added: None,
                removed: Some(new_nodes.r()),
                prev: None,
                next: None,
            };
            MutationObserver::queue_a_mutation_record(&node, mutation);

            new_nodes.r()
        } else {
            // Step 3.
            from_ref(&node)
        };
        // Step 6.
        let previous_sibling = match suppress_observers {
            SuppressObserver::Unsuppressed => match child {
                Some(child) => child.GetPreviousSibling(),
                None => parent.GetLastChild(),
            },
            SuppressObserver::Suppressed => None,
        };
        // Step 7.
        for kid in new_nodes {
            // Step 7.1.
            parent.add_child(*kid, child);
            // Step 7.7.
            for descendant in kid
                .traverse_preorder(ShadowIncluding::Yes)
                .filter_map(DomRoot::downcast::<Element>)
            {
                // Step 7.7.2.
                if descendant.is_connected() {
                    if descendant.get_custom_element_definition().is_some() {
                        // Step 7.7.2.1.
                        ScriptThread::enqueue_callback_reaction(
                            &*descendant,
                            CallbackReaction::Connected,
                            None,
                        );
                    } else {
                        // Step 7.7.2.2.
                        try_upgrade_element(&*descendant);
                    }
                }
            }
        }
        if let SuppressObserver::Unsuppressed = suppress_observers {
            vtable_for(&parent).children_changed(&ChildrenMutation::insert(
                previous_sibling.as_deref(),
                new_nodes,
                child,
            ));

            let mutation = Mutation::ChildList {
                added: Some(new_nodes),
                removed: None,
                prev: previous_sibling.as_deref(),
                next: child,
            };
            MutationObserver::queue_a_mutation_record(&parent, mutation);
        }
        node.owner_doc().remove_script_and_layout_blocker();
    }

    // https://dom.spec.whatwg.org/#concept-node-replace-all
    pub fn replace_all(node: Option<&Node>, parent: &Node) {
        parent.owner_doc().add_script_and_layout_blocker();
        // Step 1.
        if let Some(node) = node {
            Node::adopt(node, &*parent.owner_doc());
        }
        // Step 2.
        rooted_vec!(let removed_nodes <- parent.children());
        // Step 3.
        rooted_vec!(let mut added_nodes);
        let added_nodes = if let Some(node) = node.as_ref() {
            if let NodeTypeId::DocumentFragment(_) = node.type_id() {
                added_nodes.extend(node.children().map(|child| Dom::from_ref(&*child)));
                added_nodes.r()
            } else {
                from_ref(node)
            }
        } else {
            &[] as &[&Node]
        };
        // Step 4.
        for child in &*removed_nodes {
            Node::remove(child, parent, SuppressObserver::Suppressed);
        }
        // Step 5.
        if let Some(node) = node {
            Node::insert(node, parent, None, SuppressObserver::Suppressed);
        }
        // Step 6.
        vtable_for(&parent).children_changed(&ChildrenMutation::replace_all(
            removed_nodes.r(),
            added_nodes,
        ));

        if !removed_nodes.is_empty() || !added_nodes.is_empty() {
            let mutation = Mutation::ChildList {
                added: Some(added_nodes),
                removed: Some(removed_nodes.r()),
                prev: None,
                next: None,
            };
            MutationObserver::queue_a_mutation_record(&parent, mutation);
        }
        parent.owner_doc().remove_script_and_layout_blocker();
    }

    // https://dom.spec.whatwg.org/multipage/#string-replace-all
    pub fn string_replace_all(string: DOMString, parent: &Node) {
        if string.len() == 0 {
            Node::replace_all(None, parent);
        } else {
            let text = Text::new(string, &document_from_node(parent));
            Node::replace_all(Some(text.upcast::<Node>()), parent);
        };
    }

    // https://dom.spec.whatwg.org/#concept-node-pre-remove
    fn pre_remove(child: &Node, parent: &Node) -> Fallible<DomRoot<Node>> {
        // Step 1.
        match child.GetParentNode() {
            Some(ref node) if &**node != parent => return Err(Error::NotFound),
            None => return Err(Error::NotFound),
            _ => (),
        }

        // Step 2.
        Node::remove(child, parent, SuppressObserver::Unsuppressed);

        // Step 3.
        Ok(DomRoot::from_ref(child))
    }

    // https://dom.spec.whatwg.org/#concept-node-remove
    fn remove(node: &Node, parent: &Node, suppress_observers: SuppressObserver) {
        parent.owner_doc().add_script_and_layout_blocker();
        assert!(node
            .GetParentNode()
            .map_or(false, |node_parent| &*node_parent == parent));
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
            vtable_for(&parent).children_changed(&ChildrenMutation::replace(
                old_previous_sibling.as_deref(),
                &Some(&node),
                &[],
                old_next_sibling.as_deref(),
            ));

            let removed = [node];
            let mutation = Mutation::ChildList {
                added: None,
                removed: Some(&removed),
                prev: old_previous_sibling.as_deref(),
                next: old_next_sibling.as_deref(),
            };
            MutationObserver::queue_a_mutation_record(&parent, mutation);
        }
        parent.owner_doc().remove_script_and_layout_blocker();
    }

    // https://dom.spec.whatwg.org/#concept-node-clone
    pub fn clone(
        node: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
    ) -> DomRoot<Node> {
        // Step 1.
        let document = match maybe_doc {
            Some(doc) => DomRoot::from_ref(doc),
            None => node.owner_doc(),
        };

        // Step 2.
        // XXXabinader: clone() for each node as trait?
        let copy: DomRoot<Node> = match node.type_id() {
            NodeTypeId::DocumentType => {
                let doctype = node.downcast::<DocumentType>().unwrap();
                let doctype = DocumentType::new(
                    doctype.name().clone(),
                    Some(doctype.public_id().clone()),
                    Some(doctype.system_id().clone()),
                    &document,
                );
                DomRoot::upcast::<Node>(doctype)
            },
            NodeTypeId::Attr => {
                let attr = node.downcast::<Attr>().unwrap();
                let attr = Attr::new(
                    &document,
                    attr.local_name().clone(),
                    attr.value().clone(),
                    attr.name().clone(),
                    attr.namespace().clone(),
                    attr.prefix().cloned(),
                    None,
                );
                DomRoot::upcast::<Node>(attr)
            },
            NodeTypeId::DocumentFragment(_) => {
                let doc_fragment = DocumentFragment::new(&document);
                DomRoot::upcast::<Node>(doc_fragment)
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
                let document = Document::new(
                    window,
                    HasBrowsingContext::No,
                    Some(document.url()),
                    // https://github.com/whatwg/dom/issues/378
                    document.origin().clone(),
                    is_html_doc,
                    None,
                    None,
                    DocumentActivity::Inactive,
                    DocumentSource::NotFromParser,
                    loader,
                    None,
                    None,
                    Default::default(),
                );
                DomRoot::upcast::<Node>(document)
            },
            NodeTypeId::Element(..) => {
                let element = node.downcast::<Element>().unwrap();
                let name = QualName {
                    prefix: element.prefix().as_ref().map(|p| Prefix::from(&**p)),
                    ns: element.namespace().clone(),
                    local: element.local_name().clone(),
                };
                let element = Element::create(
                    name,
                    element.get_is(),
                    &document,
                    ElementCreator::ScriptCreated,
                    CustomElementCreationMode::Asynchronous,
                );
                DomRoot::upcast::<Node>(element)
            },
        };

        // Step 3.
        let document = match copy.downcast::<Document>() {
            Some(doc) => DomRoot::from_ref(doc),
            None => DomRoot::from_ref(&*document),
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
                    copy_elem.push_new_attribute(
                        attr.local_name().clone(),
                        attr.value().clone(),
                        attr.name().clone(),
                        attr.namespace().clone(),
                        attr.prefix().cloned(),
                    );
                }
            },
            _ => (),
        }

        // Step 5: cloning steps.
        vtable_for(&node).cloning_steps(&copy, maybe_doc, clone_children);

        // Step 6.
        if clone_children == CloneChildrenFlag::CloneChildren {
            for child in node.children() {
                let child_copy = Node::clone(&child, Some(&document), clone_children);
                let _inserted_node = Node::pre_insert(&child_copy, &copy, None);
            }
        }

        // Step 7.
        copy
    }

    /// <https://html.spec.whatwg.org/multipage/#child-text-content>
    pub fn child_text_content(&self) -> DOMString {
        Node::collect_text_contents(self.children())
    }

    /// <https://html.spec.whatwg.org/multipage/#descendant-text-content>
    pub fn descendant_text_content(&self) -> DOMString {
        Node::collect_text_contents(self.traverse_preorder(ShadowIncluding::No))
    }

    pub fn collect_text_contents<T: Iterator<Item = DomRoot<Node>>>(iterator: T) -> DOMString {
        let mut content = String::new();
        for node in iterator {
            if let Some(ref text) = node.downcast::<Text>() {
                content.push_str(&text.upcast::<CharacterData>().data());
            }
        }
        DOMString::from(content)
    }

    pub fn namespace_to_string(namespace: Namespace) -> Option<DOMString> {
        match namespace {
            ns!() => None,
            // FIXME(ajeffrey): convert directly from Namespace to DOMString
            _ => Some(DOMString::from(&*namespace)),
        }
    }

    // https://dom.spec.whatwg.org/#locate-a-namespace
    pub fn locate_namespace(node: &Node, prefix: Option<DOMString>) -> Namespace {
        match node.type_id() {
            NodeTypeId::Element(_) => node.downcast::<Element>().unwrap().locate_namespace(prefix),
            NodeTypeId::Attr => node
                .downcast::<Attr>()
                .unwrap()
                .GetOwnerElement()
                .as_ref()
                .map_or(ns!(), |elem| elem.locate_namespace(prefix)),
            NodeTypeId::Document(_) => node
                .downcast::<Document>()
                .unwrap()
                .GetDocumentElement()
                .as_ref()
                .map_or(ns!(), |elem| elem.locate_namespace(prefix)),
            NodeTypeId::DocumentType | NodeTypeId::DocumentFragment(_) => ns!(),
            _ => node
                .GetParentElement()
                .as_ref()
                .map_or(ns!(), |elem| elem.locate_namespace(prefix)),
        }
    }
}

impl NodeMethods for Node {
    // https://dom.spec.whatwg.org/#dom-node-nodetype
    fn NodeType(&self) -> u16 {
        match self.type_id() {
            NodeTypeId::Attr => NodeConstants::ATTRIBUTE_NODE,
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::Text)) => {
                NodeConstants::TEXT_NODE
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::CDATASection)) => {
                NodeConstants::CDATA_SECTION_NODE
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) => {
                NodeConstants::PROCESSING_INSTRUCTION_NODE
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => NodeConstants::COMMENT_NODE,
            NodeTypeId::Document(_) => NodeConstants::DOCUMENT_NODE,
            NodeTypeId::DocumentType => NodeConstants::DOCUMENT_TYPE_NODE,
            NodeTypeId::DocumentFragment(_) => NodeConstants::DOCUMENT_FRAGMENT_NODE,
            NodeTypeId::Element(_) => NodeConstants::ELEMENT_NODE,
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-nodename
    fn NodeName(&self) -> DOMString {
        match self.type_id() {
            NodeTypeId::Attr => self.downcast::<Attr>().unwrap().qualified_name(),
            NodeTypeId::Element(..) => self.downcast::<Element>().unwrap().TagName(),
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::Text)) => {
                DOMString::from("#text")
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::CDATASection)) => {
                DOMString::from("#cdata-section")
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) => {
                self.downcast::<ProcessingInstruction>().unwrap().Target()
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => DOMString::from("#comment"),
            NodeTypeId::DocumentType => self.downcast::<DocumentType>().unwrap().name().clone(),
            NodeTypeId::DocumentFragment(_) => DOMString::from("#document-fragment"),
            NodeTypeId::Document(_) => DOMString::from("#document"),
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-baseuri
    fn BaseURI(&self) -> USVString {
        USVString(String::from(self.owner_doc().base_url().as_str()))
    }

    // https://dom.spec.whatwg.org/#dom-node-isconnected
    fn IsConnected(&self) -> bool {
        return self.is_connected();
    }

    // https://dom.spec.whatwg.org/#dom-node-ownerdocument
    fn GetOwnerDocument(&self) -> Option<DomRoot<Document>> {
        match self.type_id() {
            NodeTypeId::Document(_) => None,
            _ => Some(self.owner_doc()),
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-getrootnode
    fn GetRootNode(&self, options: &GetRootNodeOptions) -> DomRoot<Node> {
        if let Some(shadow_root) = self.containing_shadow_root() {
            return if options.composed {
                // shadow-including root.
                shadow_root.Host().upcast::<Node>().GetRootNode(options)
            } else {
                DomRoot::from_ref(shadow_root.upcast::<Node>())
            };
        }

        if self.is_in_doc() {
            DomRoot::from_ref(self.owner_doc().upcast::<Node>())
        } else {
            self.inclusive_ancestors(ShadowIncluding::No)
                .last()
                .unwrap()
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-parentnode
    fn GetParentNode(&self) -> Option<DomRoot<Node>> {
        self.parent_node.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-parentelement
    fn GetParentElement(&self) -> Option<DomRoot<Element>> {
        self.GetParentNode().and_then(DomRoot::downcast)
    }

    // https://dom.spec.whatwg.org/#dom-node-haschildnodes
    fn HasChildNodes(&self) -> bool {
        self.first_child.get().is_some()
    }

    // https://dom.spec.whatwg.org/#dom-node-childnodes
    fn ChildNodes(&self) -> DomRoot<NodeList> {
        self.child_list.or_init(|| {
            let doc = self.owner_doc();
            let window = doc.window();
            NodeList::new_child_list(window, self)
        })
    }

    // https://dom.spec.whatwg.org/#dom-node-firstchild
    fn GetFirstChild(&self) -> Option<DomRoot<Node>> {
        self.first_child.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-lastchild
    fn GetLastChild(&self) -> Option<DomRoot<Node>> {
        self.last_child.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-previoussibling
    fn GetPreviousSibling(&self) -> Option<DomRoot<Node>> {
        self.prev_sibling.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-nextsibling
    fn GetNextSibling(&self) -> Option<DomRoot<Node>> {
        self.next_sibling.get()
    }

    // https://dom.spec.whatwg.org/#dom-node-nodevalue
    fn GetNodeValue(&self) -> Option<DOMString> {
        match self.type_id() {
            NodeTypeId::Attr => Some(self.downcast::<Attr>().unwrap().Value()),
            NodeTypeId::CharacterData(_) => {
                self.downcast::<CharacterData>().map(CharacterData::Data)
            },
            _ => None,
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-nodevalue
    fn SetNodeValue(&self, val: Option<DOMString>) {
        match self.type_id() {
            NodeTypeId::Attr => {
                let attr = self.downcast::<Attr>().unwrap();
                attr.SetValue(val.unwrap_or_default());
            },
            NodeTypeId::CharacterData(_) => {
                let character_data = self.downcast::<CharacterData>().unwrap();
                character_data.SetData(val.unwrap_or_default());
            },
            _ => {},
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-textcontent
    fn GetTextContent(&self) -> Option<DOMString> {
        match self.type_id() {
            NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(..) => {
                let content =
                    Node::collect_text_contents(self.traverse_preorder(ShadowIncluding::No));
                Some(content)
            },
            NodeTypeId::Attr => Some(self.downcast::<Attr>().unwrap().Value()),
            NodeTypeId::CharacterData(..) => {
                let characterdata = self.downcast::<CharacterData>().unwrap();
                Some(characterdata.Data())
            },
            NodeTypeId::DocumentType | NodeTypeId::Document(_) => None,
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-textcontent
    fn SetTextContent(&self, value: Option<DOMString>) {
        let value = value.unwrap_or_default();
        match self.type_id() {
            NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(..) => {
                // Step 1-2.
                let node = if value.is_empty() {
                    None
                } else {
                    Some(DomRoot::upcast(self.owner_doc().CreateTextNode(value)))
                };

                // Step 3.
                Node::replace_all(node.as_deref(), self);
            },
            NodeTypeId::Attr => {
                let attr = self.downcast::<Attr>().unwrap();
                attr.SetValue(value);
            },
            NodeTypeId::CharacterData(..) => {
                let characterdata = self.downcast::<CharacterData>().unwrap();
                characterdata.SetData(value);
            },
            NodeTypeId::DocumentType | NodeTypeId::Document(_) => {},
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-insertbefore
    fn InsertBefore(&self, node: &Node, child: Option<&Node>) -> Fallible<DomRoot<Node>> {
        Node::pre_insert(node, self, child)
    }

    // https://dom.spec.whatwg.org/#dom-node-appendchild
    fn AppendChild(&self, node: &Node) -> Fallible<DomRoot<Node>> {
        Node::pre_insert(node, self, None)
    }

    // https://dom.spec.whatwg.org/#concept-node-replace
    fn ReplaceChild(&self, node: &Node, child: &Node) -> Fallible<DomRoot<Node>> {
        // Step 1.
        match self.type_id() {
            NodeTypeId::Document(_) | NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(..) => {
                ()
            },
            _ => return Err(Error::HierarchyRequest),
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
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) if self.is::<Document>() => {
                return Err(Error::HierarchyRequest);
            },
            NodeTypeId::DocumentType if !self.is::<Document>() => {
                return Err(Error::HierarchyRequest);
            },
            NodeTypeId::Document(_) => return Err(Error::HierarchyRequest),
            _ => (),
        }

        // Step 6.
        if self.is::<Document>() {
            match node.type_id() {
                // Step 6.1
                NodeTypeId::DocumentFragment(_) => {
                    // Step 6.1.1(b)
                    if node.children().any(|c| c.is::<Text>()) {
                        return Err(Error::HierarchyRequest);
                    }
                    match node.child_elements().count() {
                        0 => (),
                        // Step 6.1.2
                        1 => {
                            if self.child_elements().any(|c| c.upcast::<Node>() != child) {
                                return Err(Error::HierarchyRequest);
                            }
                            if child.following_siblings().any(|child| child.is_doctype()) {
                                return Err(Error::HierarchyRequest);
                            }
                        },
                        // Step 6.1.1(a)
                        _ => return Err(Error::HierarchyRequest),
                    }
                },
                // Step 6.2
                NodeTypeId::Element(..) => {
                    if self.child_elements().any(|c| c.upcast::<Node>() != child) {
                        return Err(Error::HierarchyRequest);
                    }
                    if child.following_siblings().any(|child| child.is_doctype()) {
                        return Err(Error::HierarchyRequest);
                    }
                },
                // Step 6.3
                NodeTypeId::DocumentType => {
                    if self.children().any(|c| c.is_doctype() && &*c != child) {
                        return Err(Error::HierarchyRequest);
                    }
                    if self
                        .children()
                        .take_while(|c| &**c != child)
                        .any(|c| c.is::<Element>())
                    {
                        return Err(Error::HierarchyRequest);
                    }
                },
                NodeTypeId::CharacterData(..) => (),
                NodeTypeId::Document(_) => unreachable!(),
                NodeTypeId::Attr => unreachable!(),
            }
        }

        // Step 7-8.
        let child_next_sibling = child.GetNextSibling();
        let node_next_sibling = node.GetNextSibling();
        let reference_child = if child_next_sibling.as_deref() == Some(node) {
            node_next_sibling.as_deref()
        } else {
            child_next_sibling.as_deref()
        };

        // Step 9.
        let previous_sibling = child.GetPreviousSibling();

        // Step 10.
        let document = document_from_node(self);
        Node::adopt(node, &document);

        let removed_child = if node != child {
            // Step 11.
            Node::remove(child, self, SuppressObserver::Suppressed);
            Some(child)
        } else {
            None
        };

        // Step 12.
        rooted_vec!(let mut nodes);
        let nodes = if node.type_id() ==
            NodeTypeId::DocumentFragment(DocumentFragmentTypeId::DocumentFragment) ||
            node.type_id() == NodeTypeId::DocumentFragment(DocumentFragmentTypeId::ShadowRoot)
        {
            nodes.extend(node.children().map(|node| Dom::from_ref(&*node)));
            nodes.r()
        } else {
            from_ref(&node)
        };

        // Step 13.
        Node::insert(node, self, reference_child, SuppressObserver::Suppressed);

        // Step 14.
        vtable_for(&self).children_changed(&ChildrenMutation::replace(
            previous_sibling.as_deref(),
            &removed_child,
            nodes,
            reference_child,
        ));
        let removed = removed_child.map(|r| [r]);
        let mutation = Mutation::ChildList {
            added: Some(nodes),
            removed: removed.as_ref().map(|r| &r[..]),
            prev: previous_sibling.as_deref(),
            next: reference_child,
        };

        MutationObserver::queue_a_mutation_record(&self, mutation);

        // Step 15.
        Ok(DomRoot::from_ref(child))
    }

    // https://dom.spec.whatwg.org/#dom-node-removechild
    fn RemoveChild(&self, node: &Node) -> Fallible<DomRoot<Node>> {
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
                while children
                    .peek()
                    .map_or(false, |&(_, ref sibling)| sibling.is::<Text>())
                {
                    let (index, sibling) = children.next().unwrap();
                    sibling
                        .ranges
                        .drain_to_preceding_text_sibling(&sibling, &node, length);
                    self.ranges
                        .move_to_text_child_at(self, index as u32, &node, length as u32);
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
    fn CloneNode(&self, deep: bool) -> Fallible<DomRoot<Node>> {
        if deep && self.is::<ShadowRoot>() {
            return Err(Error::NotSupported);
        }
        Ok(Node::clone(
            self,
            None,
            if deep {
                CloneChildrenFlag::CloneChildren
            } else {
                CloneChildrenFlag::DoNotCloneChildren
            },
        ))
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
                (*pi.upcast::<CharacterData>().data() ==
                    *other_pi.upcast::<CharacterData>().data())
        }
        fn is_equal_characterdata(node: &Node, other: &Node) -> bool {
            let characterdata = node.downcast::<CharacterData>().unwrap();
            let other_characterdata = other.downcast::<CharacterData>().unwrap();
            *characterdata.data() == *other_characterdata.data()
        }
        fn is_equal_attr(node: &Node, other: &Node) -> bool {
            let attr = node.downcast::<Attr>().unwrap();
            let other_attr = other.downcast::<Attr>().unwrap();
            (*attr.namespace() == *other_attr.namespace()) &&
                (attr.local_name() == other_attr.local_name()) &&
                (**attr.value() == **other_attr.value())
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
                NodeTypeId::DocumentType if !is_equal_doctype(this, node) => return false,
                NodeTypeId::Element(..) if !is_equal_element(this, node) => return false,
                NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction)
                    if !is_equal_processinginstruction(this, node) =>
                {
                    return false;
                }
                NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) |
                NodeTypeId::CharacterData(CharacterDataTypeId::Comment)
                    if !is_equal_characterdata(this, node) =>
                {
                    return false;
                }
                // Step 4.
                NodeTypeId::Element(..) if !is_equal_element_attrs(this, node) => return false,
                NodeTypeId::Attr if !is_equal_attr(this, node) => return false,

                _ => (),
            }

            // Step 5.
            if this.children_count() != node.children_count() {
                return false;
            }

            // Step 6.
            this.children()
                .zip(node.children())
                .all(|(child, other_child)| is_equal_node(&child, &other_child))
        }
        match maybe_node {
            // Step 1.
            None => false,
            // Step 2-6.
            Some(node) => is_equal_node(self, node),
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-issamenode
    fn IsSameNode(&self, other_node: Option<&Node>) -> bool {
        match other_node {
            Some(node) => self == node,
            None => false,
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-comparedocumentposition
    fn CompareDocumentPosition(&self, other: &Node) -> u16 {
        // step 1.
        if self == other {
            return 0;
        }

        // step 2
        let mut node1 = Some(other);
        let mut node2 = Some(self);

        // step 3
        let mut attr1: Option<&Attr> = None;
        let mut attr2: Option<&Attr> = None;

        // step 4: spec says to operate on node1 here,
        // node1 is definitely Some(other) going into this step
        // The compiler doesn't know the lifetime of attr1.GetOwnerElement
        // is guaranteed by the lifetime of attr1, so we hold it explicitly
        let attr1owner;
        if let Some(ref a) = other.downcast::<Attr>() {
            attr1 = Some(a);
            attr1owner = a.GetOwnerElement();
            node1 = match attr1owner {
                Some(ref e) => Some(&e.upcast()),
                None => None,
            }
        }

        // step 5.1: spec says to operate on node2 here,
        // node2 is definitely just Some(self) going into this step
        let attr2owner;
        if let Some(ref a) = self.downcast::<Attr>() {
            attr2 = Some(a);
            attr2owner = a.GetOwnerElement();
            node2 = match attr2owner {
                Some(ref e) => Some(&*e.upcast()),
                None => None,
            }
        }

        // Step 5.2
        // This substep seems lacking in test coverage.
        // We hit this when comparing two attributes that have the
        // same owner element.
        if let Some(node2) = node2 {
            if Some(node2) == node1 {
                match (attr1, attr2) {
                    (Some(a1), Some(a2)) => {
                        let attrs = node2.downcast::<Element>().unwrap().attrs();
                        // go through the attrs in order to see if self
                        // or other is first; spec is clear that we
                        // want value-equality, not reference-equality
                        for attr in attrs.iter() {
                            if (*attr.namespace() == *a1.namespace()) &&
                                (attr.local_name() == a1.local_name()) &&
                                (**attr.value() == **a1.value())
                            {
                                return NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC +
                                    NodeConstants::DOCUMENT_POSITION_PRECEDING;
                            }
                            if (*attr.namespace() == *a2.namespace()) &&
                                (attr.local_name() == a2.local_name()) &&
                                (**attr.value() == **a2.value())
                            {
                                return NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC +
                                    NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                            }
                        }
                        // both attrs have node2 as their owner element, so
                        // we can't have left the loop without seeing them
                        unreachable!();
                    },
                    (_, _) => {},
                }
            }
        }

        // Step 6
        match (node1, node2) {
            (None, _) => {
                // node1 is null
                return NodeConstants::DOCUMENT_POSITION_FOLLOWING +
                    NodeConstants::DOCUMENT_POSITION_DISCONNECTED +
                    NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC;
            },
            (_, None) => {
                // node2 is null
                return NodeConstants::DOCUMENT_POSITION_PRECEDING +
                    NodeConstants::DOCUMENT_POSITION_DISCONNECTED +
                    NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC;
            },
            (Some(node1), Some(node2)) => {
                // still step 6, testing if node1 and 2 share a root
                let mut self_and_ancestors = node2
                    .inclusive_ancestors(ShadowIncluding::No)
                    .collect::<SmallVec<[_; 20]>>();
                let mut other_and_ancestors = node1
                    .inclusive_ancestors(ShadowIncluding::No)
                    .collect::<SmallVec<[_; 20]>>();

                if self_and_ancestors.last() != other_and_ancestors.last() {
                    let random = as_uintptr(self_and_ancestors.last().unwrap()) <
                        as_uintptr(other_and_ancestors.last().unwrap());
                    let random = if random {
                        NodeConstants::DOCUMENT_POSITION_FOLLOWING
                    } else {
                        NodeConstants::DOCUMENT_POSITION_PRECEDING
                    };

                    // Disconnected.
                    return random +
                        NodeConstants::DOCUMENT_POSITION_DISCONNECTED +
                        NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC;
                }
                // steps 7-10
                let mut parent = self_and_ancestors.pop().unwrap();
                other_and_ancestors.pop().unwrap();

                let mut current_position =
                    cmp::min(self_and_ancestors.len(), other_and_ancestors.len());

                while current_position > 0 {
                    current_position -= 1;
                    let child_1 = self_and_ancestors.pop().unwrap();
                    let child_2 = other_and_ancestors.pop().unwrap();

                    if child_1 != child_2 {
                        let is_before = parent.children().position(|c| c == child_1).unwrap() <
                            parent.children().position(|c| c == child_2).unwrap();
                        // If I am before, `other` is following, and the other way
                        // around.
                        return if is_before {
                            NodeConstants::DOCUMENT_POSITION_FOLLOWING
                        } else {
                            NodeConstants::DOCUMENT_POSITION_PRECEDING
                        };
                    }

                    parent = child_1;
                }

                // We hit the end of one of the parent chains, so one node needs to be
                // contained in the other.
                //
                // If we're the container, return that `other` is contained by us.
                return if self_and_ancestors.len() < other_and_ancestors.len() {
                    NodeConstants::DOCUMENT_POSITION_FOLLOWING +
                        NodeConstants::DOCUMENT_POSITION_CONTAINED_BY
                } else {
                    NodeConstants::DOCUMENT_POSITION_PRECEDING +
                        NodeConstants::DOCUMENT_POSITION_CONTAINS
                };
            },
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-contains
    fn Contains(&self, maybe_other: Option<&Node>) -> bool {
        match maybe_other {
            None => false,
            Some(other) => self.is_inclusive_ancestor_of(other),
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
            NodeTypeId::Element(..) => self.downcast::<Element>().unwrap().lookup_prefix(namespace),
            NodeTypeId::Document(_) => self
                .downcast::<Document>()
                .unwrap()
                .GetDocumentElement()
                .and_then(|element| element.lookup_prefix(namespace)),
            NodeTypeId::DocumentType | NodeTypeId::DocumentFragment(_) => None,
            NodeTypeId::Attr => self
                .downcast::<Attr>()
                .unwrap()
                .GetOwnerElement()
                .and_then(|element| element.lookup_prefix(namespace)),
            _ => self
                .GetParentElement()
                .and_then(|element| element.lookup_prefix(namespace)),
        }
    }

    // https://dom.spec.whatwg.org/#dom-node-lookupnamespaceuri
    fn LookupNamespaceURI(&self, prefix: Option<DOMString>) -> Option<DOMString> {
        // Step 1.
        let prefix = match prefix {
            Some(ref p) if p.is_empty() => None,
            pre => pre,
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

pub fn document_from_node<T: DerivedFrom<Node> + DomObject>(derived: &T) -> DomRoot<Document> {
    derived.upcast().owner_doc()
}

pub fn containing_shadow_root<T: DerivedFrom<Node> + DomObject>(
    derived: &T,
) -> Option<DomRoot<ShadowRoot>> {
    derived.upcast().containing_shadow_root()
}

#[allow(unrooted_must_root)]
pub fn stylesheets_owner_from_node<T: DerivedFrom<Node> + DomObject>(
    derived: &T,
) -> StyleSheetListOwner {
    if let Some(shadow_root) = containing_shadow_root(derived) {
        StyleSheetListOwner::ShadowRoot(Dom::from_ref(&*shadow_root))
    } else {
        StyleSheetListOwner::Document(Dom::from_ref(&*document_from_node(derived)))
    }
}

pub fn window_from_node<T: DerivedFrom<Node> + DomObject>(derived: &T) -> DomRoot<Window> {
    let document = document_from_node(derived);
    DomRoot::from_ref(document.window())
}

impl VirtualMethods for Node {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<EventTarget>() as &dyn VirtualMethods)
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
        if let Some(list) = self.child_list.get() {
            list.as_children_list().children_changed(mutation);
        }
        self.owner_doc().content_and_heritage_changed(self);
    }

    // This handles the ranges mentioned in steps 2-3 when removing a node.
    // https://dom.spec.whatwg.org/#concept-node-remove
    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);
        self.ranges.drain_to_parent(context, self);
    }
}

/// A summary of the changes that happened to a node.
#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub enum NodeDamage {
    /// The node's `style` attribute changed.
    NodeStyleDamaged,
    /// Other parts of a node changed; attributes, text content, etc.
    OtherNodeDamage,
}

pub enum ChildrenMutation<'a> {
    Append {
        prev: &'a Node,
        added: &'a [&'a Node],
    },
    Insert {
        prev: &'a Node,
        added: &'a [&'a Node],
        next: &'a Node,
    },
    Prepend {
        added: &'a [&'a Node],
        next: &'a Node,
    },
    Replace {
        prev: Option<&'a Node>,
        removed: &'a Node,
        added: &'a [&'a Node],
        next: Option<&'a Node>,
    },
    ReplaceAll {
        removed: &'a [&'a Node],
        added: &'a [&'a Node],
    },
    /// Mutation for when a Text node's data is modified.
    /// This doesn't change the structure of the list, which is what the other
    /// variants' fields are stored for at the moment, so this can just have no
    /// fields.
    ChangeText,
}

impl<'a> ChildrenMutation<'a> {
    fn insert(
        prev: Option<&'a Node>,
        added: &'a [&'a Node],
        next: Option<&'a Node>,
    ) -> ChildrenMutation<'a> {
        match (prev, next) {
            (None, None) => ChildrenMutation::ReplaceAll {
                removed: &[],
                added: added,
            },
            (Some(prev), None) => ChildrenMutation::Append {
                prev: prev,
                added: added,
            },
            (None, Some(next)) => ChildrenMutation::Prepend {
                added: added,
                next: next,
            },
            (Some(prev), Some(next)) => ChildrenMutation::Insert {
                prev: prev,
                added: added,
                next: next,
            },
        }
    }

    fn replace(
        prev: Option<&'a Node>,
        removed: &'a Option<&'a Node>,
        added: &'a [&'a Node],
        next: Option<&'a Node>,
    ) -> ChildrenMutation<'a> {
        if let Some(ref removed) = *removed {
            if let (None, None) = (prev, next) {
                ChildrenMutation::ReplaceAll {
                    removed: from_ref(removed),
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

    fn replace_all(removed: &'a [&'a Node], added: &'a [&'a Node]) -> ChildrenMutation<'a> {
        ChildrenMutation::ReplaceAll {
            removed: removed,
            added: added,
        }
    }

    /// Get the child that follows the added or removed children.
    /// Currently only used when this mutation might force us to
    /// restyle later children (see HAS_SLOW_SELECTOR_LATER_SIBLINGS and
    /// Element's implementation of VirtualMethods::children_changed).
    pub fn next_child(&self) -> Option<&Node> {
        match *self {
            ChildrenMutation::Append { .. } => None,
            ChildrenMutation::Insert { next, .. } => Some(next),
            ChildrenMutation::Prepend { next, .. } => Some(next),
            ChildrenMutation::Replace { next, .. } => next,
            ChildrenMutation::ReplaceAll { .. } => None,
            ChildrenMutation::ChangeText => None,
        }
    }

    /// If nodes were added or removed at the start or end of a container, return any
    /// previously-existing child whose ":first-child" or ":last-child" status *may* have changed.
    ///
    /// NOTE: This does not check whether the inserted/removed nodes were elements, so in some
    /// cases it will return a false positive.  This doesn't matter for correctness, because at
    /// worst the returned element will be restyled unnecessarily.
    pub fn modified_edge_element(&self) -> Option<DomRoot<Node>> {
        match *self {
            // Add/remove at start of container: Return the first following element.
            ChildrenMutation::Prepend { next, .. } |
            ChildrenMutation::Replace {
                prev: None,
                next: Some(next),
                ..
            } => next
                .inclusively_following_siblings()
                .filter(|node| node.is::<Element>())
                .next(),
            // Add/remove at end of container: Return the last preceding element.
            ChildrenMutation::Append { prev, .. } |
            ChildrenMutation::Replace {
                prev: Some(prev),
                next: None,
                ..
            } => prev
                .inclusively_preceding_siblings()
                .filter(|node| node.is::<Element>())
                .next(),
            // Insert or replace in the middle:
            ChildrenMutation::Insert { prev, next, .. } |
            ChildrenMutation::Replace {
                prev: Some(prev),
                next: Some(next),
                ..
            } => {
                if prev
                    .inclusively_preceding_siblings()
                    .all(|node| !node.is::<Element>())
                {
                    // Before the first element: Return the first following element.
                    next.inclusively_following_siblings()
                        .filter(|node| node.is::<Element>())
                        .next()
                } else if next
                    .inclusively_following_siblings()
                    .all(|node| !node.is::<Element>())
                {
                    // After the last element: Return the last preceding element.
                    prev.inclusively_preceding_siblings()
                        .filter(|node| node.is::<Element>())
                        .next()
                } else {
                    None
                }
            },

            ChildrenMutation::Replace {
                prev: None,
                next: None,
                ..
            } => unreachable!(),
            ChildrenMutation::ReplaceAll { .. } => None,
            ChildrenMutation::ChangeText => None,
        }
    }
}

/// The context of the binding to tree of a node.
pub struct BindContext {
    /// Whether the tree is connected.
    pub tree_connected: bool,
    /// Whether the tree is in the document.
    pub tree_in_doc: bool,
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
    /// The next sibling of the inclusive ancestor that was removed.
    pub next_sibling: Option<&'a Node>,
    /// Whether the tree is connected.
    pub tree_connected: bool,
    /// Whether the tree is in doc.
    pub tree_in_doc: bool,
}

impl<'a> UnbindContext<'a> {
    /// Create a new `UnbindContext` value.
    pub fn new(
        parent: &'a Node,
        prev_sibling: Option<&'a Node>,
        next_sibling: Option<&'a Node>,
        cached_index: Option<u32>,
    ) -> Self {
        UnbindContext {
            index: Cell::new(cached_index),
            parent: parent,
            prev_sibling: prev_sibling,
            next_sibling: next_sibling,
            tree_connected: parent.is_connected(),
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
pub struct UniqueId {
    cell: UnsafeCell<Option<Box<Uuid>>>,
}

unsafe_no_jsmanaged_fields!(UniqueId);

impl MallocSizeOf for UniqueId {
    #[allow(unsafe_code)]
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if let &Some(ref uuid) = unsafe { &*self.cell.get() } {
            unsafe { ops.malloc_size_of(&**uuid) }
        } else {
            0
        }
    }
}

impl UniqueId {
    /// Create a new `UniqueId` value. The underlying `Uuid` is lazily created.
    fn new() -> UniqueId {
        UniqueId {
            cell: UnsafeCell::new(None),
        }
    }

    /// The Uuid of that unique ID.
    #[allow(unsafe_code)]
    fn borrow(&self) -> &Uuid {
        unsafe {
            let ptr = self.cell.get();
            if (*ptr).is_none() {
                *ptr = Some(Box::new(Uuid::new_v4()));
            }
            &(&*ptr).as_ref().unwrap()
        }
    }
}

impl Into<LayoutNodeType> for NodeTypeId {
    #[inline(always)]
    fn into(self) -> LayoutNodeType {
        match self {
            NodeTypeId::Element(e) => LayoutNodeType::Element(e.into()),
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => LayoutNodeType::Text,
            x => unreachable!("Layout should not traverse nodes of type {:?}", x),
        }
    }
}

impl Into<LayoutElementType> for ElementTypeId {
    #[inline(always)]
    fn into(self) -> LayoutElementType {
        match self {
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBodyElement) => {
                LayoutElementType::HTMLBodyElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLBRElement) => {
                LayoutElementType::HTMLBRElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLCanvasElement) => {
                LayoutElementType::HTMLCanvasElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHtmlElement) => {
                LayoutElementType::HTMLHtmlElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLIFrameElement) => {
                LayoutElementType::HTMLIFrameElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLImageElement) => {
                LayoutElementType::HTMLImageElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLMediaElement(_)) => {
                LayoutElementType::HTMLMediaElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement) => {
                LayoutElementType::HTMLInputElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement) => {
                LayoutElementType::HTMLObjectElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLParagraphElement) => {
                LayoutElementType::HTMLParagraphElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableCellElement) => {
                LayoutElementType::HTMLTableCellElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableColElement) => {
                LayoutElementType::HTMLTableColElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableElement) => {
                LayoutElementType::HTMLTableElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableRowElement) => {
                LayoutElementType::HTMLTableRowElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTableSectionElement) => {
                LayoutElementType::HTMLTableSectionElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement) => {
                LayoutElementType::HTMLTextAreaElement
            },
            ElementTypeId::SVGElement(SVGElementTypeId::SVGGraphicsElement(
                SVGGraphicsElementTypeId::SVGSVGElement,
            )) => LayoutElementType::SVGSVGElement,
            _ => LayoutElementType::Element,
        }
    }
}

/// Helper trait to insert an element into vector whose elements
/// are maintained in tree order
pub trait VecPreOrderInsertionHelper<T> {
    fn insert_pre_order(&mut self, elem: &T, tree_root: &Node);
}

impl<T> VecPreOrderInsertionHelper<T> for Vec<Dom<T>>
where
    T: DerivedFrom<Node> + DomObject,
{
    /// This algorithm relies on the following assumptions:
    /// * any elements inserted in this vector share the same tree root
    /// * any time an element is removed from the tree root, it is also removed from this array
    /// * any time an element is moved within the tree, it is removed from this array and re-inserted
    ///
    /// Under these assumptions, an element's tree-order position in this array can be determined by
    /// performing a [preorder traversal](https://dom.spec.whatwg.org/#concept-tree-order) of the tree root's children,
    /// and increasing the destination index in the array every time a node in the array is encountered during
    /// the traversal.
    fn insert_pre_order(&mut self, elem: &T, tree_root: &Node) {
        if self.is_empty() {
            self.push(Dom::from_ref(elem));
            return;
        }

        let elem_node = elem.upcast::<Node>();
        let mut head: usize = 0;
        for node in tree_root.traverse_preorder(ShadowIncluding::No) {
            let head_node = DomRoot::upcast::<Node>(DomRoot::from_ref(&*self[head]));
            if head_node == node {
                head += 1;
            }
            if elem_node == &*node || head == self.len() {
                break;
            }
        }
        self.insert(head, Dom::from_ref(elem));
    }
}
