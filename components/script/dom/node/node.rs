/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use std::cell::{Cell, LazyCell, UnsafeCell};
use std::cmp::Ordering;
use std::default::Default;
use std::f64::consts::PI;
use std::ops::Deref;
use std::rc::Rc;
use std::slice::from_ref;
use std::{cmp, fmt, iter};

use app_units::Au;
use bitflags::bitflags;
use devtools_traits::NodeInfo;
use dom_struct::dom_struct;
use embedder_traits::UntrustedNodeAddress;
use euclid::default::Size2D;
use euclid::{Point2D, Rect};
use html5ever::serialize::HtmlSerializer;
use html5ever::{Namespace, Prefix, QualName, ns, serialize as html_serialize};
use js::context::{JSContext, NoGC};
use js::jsapi::JSObject;
use js::rust::HandleObject;
use keyboard_types::Modifiers;
use layout_api::{
    AccessibilityDamage, AxesOverflow, BoxAreaType, CSSPixelRectVec, GenericLayoutData,
    NodeRenderingType, PhysicalSides, TrustedNodeAddress, with_layout_state,
};
use libc::{self, uintptr_t};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use script_bindings::cell::{DomRefCell, Ref, RefMut};
use script_bindings::codegen::GenericBindings::ElementBinding::ElementMethods;
use script_bindings::codegen::GenericBindings::EventBinding::EventMethods;
use script_bindings::codegen::GenericBindings::ProcessingInstructionBinding::ProcessingInstructionMethods;
use script_bindings::codegen::InheritTypes::{DocumentFragmentTypeId, TextTypeId};
use script_bindings::reflector::{
    DomObject, DomObjectWrap, WeakReferenceableDomObjectWrap, reflect_dom_object_with_proto_and_cx,
    reflect_weak_referenceable_dom_object_with_proto,
};
use script_traits::DocumentActivity;
use servo_base::id::PipelineId;
use servo_config::pref;
use smallvec::SmallVec;
use style::Atom;
use style::context::QuirksMode;
use style::dom::OpaqueNode;
use style::dom_apis::{QueryAll, QueryFirst};
use style::selector_parser::PseudoElement;
use style_traits::CSSPixel;
use uuid::Uuid;
use xml5ever::{local_name, serialize as xml_serialize};

use crate::conversions::Convert;
use crate::document_loader::DocumentLoader;
use crate::dom::ChildrenMutation;
use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::{
    GetRootNodeOptions, NodeConstants, NodeMethods,
};
use crate::dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::ShadowRoot_Binding::ShadowRootMethods;
use crate::dom::bindings::codegen::Bindings::ShadowRootBinding::{
    ShadowRootMode, SlotAssignmentMode,
};
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::codegen::UnionTypes::NodeOrString;
use crate::dom::bindings::conversions::{self, DerivedFrom};
use crate::dom::bindings::domname::namespace_from_domstring;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::{
    Castable, CharacterDataTypeId, EventTargetTypeId, NodeTypeId,
};
use crate::dom::bindings::root::{
    Dom, DomRoot, DomSlice, LayoutDom, MutNullableDom, ToLayout, UnrootedDom,
};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::characterdata::CharacterData;
use crate::dom::context::{BindContext, IsShadowTree, MoveContext, UnbindContext};
use crate::dom::css::cssstylesheet::CSSStyleSheet;
use crate::dom::css::stylesheetlist::StyleSheetListOwner;
use crate::dom::customelementregistry::{
    CallbackReaction, CustomElementRegistry, try_upgrade_element,
};
use crate::dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::documenttype::DocumentType;
use crate::dom::element::{CustomElementCreationMode, Element, ElementCreator};
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventFlags};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlcollection::HTMLCollection;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmllinkelement::HTMLLinkElement;
use crate::dom::html::htmlslotelement::{HTMLSlotElement, Slottable};
use crate::dom::html::htmlstyleelement::HTMLStyleElement;
use crate::dom::iterators::{
    ShadowIncluding, UnrootedFollowingNodeIterator, UnrootedPrecedingNodeIterator,
};
use crate::dom::mutationobserver::{Mutation, MutationObserver, RegisteredObserver};
use crate::dom::node::iterators::{
    FollowingNodeIterator, PrecedingNodeIterator, SimpleNodeIterator, TreeIterator,
    UnrootedSimpleNodeIterator, UnrootedTreeIterator,
};
use crate::dom::node::nodelist::NodeList;
use crate::dom::node::virtualmethods::{VirtualMethods, vtable_for};
use crate::dom::pointerevent::{PointerEvent, PointerId};
use crate::dom::range::WeakRangeVec;
use crate::dom::raredata::NodeRareData;
use crate::dom::servoparser::html::HtmlSerialize;
use crate::dom::servoparser::serialize_html_fragment;
use crate::dom::shadowroot::{IsUserAgentWidget, ShadowRoot};
use crate::dom::text::Text;
use crate::dom::types::{CDATASection, KeyboardEvent, ProcessingInstruction};
use crate::dom::window::Window;
use crate::layout_dom::{ServoDangerousStyleElement, ServoDangerousStyleNode};
use crate::script_thread::ScriptThread;

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

    /// The live count of children of this node.
    children_count: Cell<u32>,

    /// A bitfield of flags for node items.
    flags: Cell<NodeFlags>,

    /// The maximum version of any inclusive descendant of this node.
    inclusive_descendants_version: Cell<u64>,

    /// Layout data for this node. This is populated during layout and can
    /// be used for incremental relayout and script queries.
    #[no_trace]
    layout_data: DomRefCell<Option<Box<GenericLayoutData>>>,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(element) = self.downcast::<Element>() {
            element.fmt(f)
        } else if let Some(character_data) = self.downcast::<CharacterData>() {
            write!(f, "[Text({})]", character_data.data())
        } else {
            write!(f, "[Node({:?})]", self.type_id())
        }
    }
}

/// Flags for node items
#[derive(Clone, Copy, JSTraceable, MallocSizeOf)]
pub(crate) struct NodeFlags(u16);

bitflags! {
    impl NodeFlags: u16 {
        /// Specifies whether this node is in a document.
        ///
        /// <https://dom.spec.whatwg.org/#in-a-document-tree>
        const IS_IN_A_DOCUMENT_TREE = 1 << 0;

        /// Specifies whether this node needs style recalc on next reflow.
        const HAS_DIRTY_DESCENDANTS = 1 << 1;

        /// Specifies whether or not there is an authentic click in progress on
        /// this element.
        const CLICK_IN_PROGRESS = 1 << 2;

        // There are three free bits here.

        /// Specifies whether the parser has set an associated form owner for
        /// this element. Only applicable for form-associatable elements.
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
        ///
        /// <https://dom.spec.whatwg.org/#connected>
        const IS_CONNECTED = 1 << 10;

        /// Whether this node has a weird parser insertion mode. i.e whether setting innerHTML
        /// needs extra work or not
        const HAS_WEIRD_PARSER_INSERTION_MODE = 1 << 11;

        /// Whether this node resides in UA shadow DOM. Element within UA Shadow DOM
        /// will have a different style computation behavior
        const IS_IN_UA_WIDGET = 1 << 12;

        /// Whether this node has a pseudo-element style which uses `attr()` in the `content` attribute.
        const USES_ATTR_IN_CONTENT_ATTRIBUTE = 1 << 13;
    }
}

/// suppress observers flag
/// <https://dom.spec.whatwg.org/#insert-suppressobservers>
/// <https://dom.spec.whatwg.org/#remove-suppressobservers>
#[derive(Clone, Copy, MallocSizeOf)]
pub(crate) enum SuppressObserver {
    Suppressed,
    Unsuppressed,
}

pub(crate) enum ForceSlottableNodeReconciliation {
    Force,
    Skip,
}

impl Node {
    // Getters for internal values
    pub(super) fn parent_node(&self) -> &MutNullableDom<Node> {
        &self.parent_node
    }

    pub(super) fn first_child(&self) -> &MutNullableDom<Node> {
        &self.first_child
    }

    pub(super) fn last_child(&self) -> &MutNullableDom<Node> {
        &self.last_child
    }

    pub(super) fn next_sibling(&self) -> &MutNullableDom<Node> {
        &self.next_sibling
    }

    pub(super) fn prev_sibling(&self) -> &MutNullableDom<Node> {
        &self.prev_sibling
    }

    pub(super) fn get_owner_doc(&self) -> &MutNullableDom<Document> {
        &self.owner_doc
    }

    pub(super) fn get_rare_data(&self) -> &DomRefCell<Option<Box<NodeRareData>>> {
        &self.rare_data
    }

    pub(super) fn flags(&self) -> &Cell<NodeFlags> {
        &self.flags
    }

    pub(super) fn layout_data(&self) -> &DomRefCell<Option<Box<GenericLayoutData>>> {
        &self.layout_data
    }

    /// Adds a new child to the end of this node's list of children.
    ///
    /// Fails unless `new_child` is disconnected from the tree.
    fn add_child(&self, cx: &mut JSContext, new_child: &Node, before: Option<&Node>) {
        assert!(new_child.parent_node.get().is_none());
        assert!(new_child.prev_sibling.get().is_none());
        assert!(new_child.next_sibling.get().is_none());

        self.add_pending_accessibility_damage(AccessibilityDamage::Children);

        match before {
            Some(before) => {
                assert!(before.parent_node.get().as_deref() == Some(self));
                let prev_sibling = before.GetPreviousSibling();
                match prev_sibling {
                    None => {
                        assert!(self.first_child.get().as_deref() == Some(before));
                        self.first_child.set(Some(new_child));
                    },
                    Some(ref prev_sibling) => {
                        prev_sibling.next_sibling.set(Some(new_child));
                        new_child.prev_sibling.set(Some(prev_sibling));
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
                        new_child.prev_sibling.set(Some(last_child));
                    },
                }

                self.last_child.set(Some(new_child));
            },
        }

        new_child.parent_node.set(Some(self));
        self.children_count.set(self.children_count.get() + 1);

        let parent_is_in_a_document_tree = self.is_in_a_document_tree();
        let parent_in_shadow_tree = self.is_in_a_shadow_tree();
        let parent_is_connected = self.is_connected();
        let parent_is_in_ua_widget = self.is_in_ua_widget();

        let context = BindContext::new(self, IsShadowTree::No);

        for node in new_child.traverse_preorder(ShadowIncluding::No) {
            if parent_in_shadow_tree {
                if let Some(shadow_root) = self.containing_shadow_root() {
                    node.set_containing_shadow_root(Some(&*shadow_root));
                }
                debug_assert!(node.containing_shadow_root().is_some());
            }
            node.set_flag(
                NodeFlags::IS_IN_A_DOCUMENT_TREE,
                parent_is_in_a_document_tree,
            );
            node.set_flag(NodeFlags::IS_IN_SHADOW_TREE, parent_in_shadow_tree);
            node.set_flag(NodeFlags::IS_CONNECTED, parent_is_connected);
            node.set_flag(NodeFlags::IS_IN_UA_WIDGET, parent_is_in_ua_widget);

            // Out-of-document elements never have the descendants flag set.
            debug_assert!(!node.get_flag(NodeFlags::HAS_DIRTY_DESCENDANTS));
            vtable_for(&node).bind_to_tree(cx, &context);
        }
    }

    /// Clear style and layout data on this [`Node`] and all descendants. This is used to clean
    /// up the data when a [`Node`] becomes detached from the flat tree. Note that this
    /// operates on both DOM and flat tree descendants.
    pub(crate) fn remove_style_and_layout_data_from_subtree(&self, no_gc: &NoGC) {
        for node in self.traverse_preorder_non_rooting(no_gc, ShadowIncluding::Yes) {
            node.clean_up_style_and_layout_data();
        }
    }

    fn clean_up_style_and_layout_data(&self) {
        self.layout_data.borrow_mut().take();
        if let Some(element) = self.downcast::<Element>() {
            element.clean_up_style_data();
        }
    }

    /// Clean up flags and runs steps 11-14 of remove a node.
    /// <https://dom.spec.whatwg.org/#concept-node-remove>
    pub(crate) fn complete_remove_subtree(
        cx: &mut JSContext,
        root: &Node,
        context: &UnbindContext,
    ) {
        // Flags that reset when a node is disconnected
        const RESET_FLAGS: NodeFlags = NodeFlags::IS_IN_A_DOCUMENT_TREE
            .union(NodeFlags::IS_CONNECTED)
            .union(NodeFlags::HAS_DIRTY_DESCENDANTS)
            .union(NodeFlags::HAS_SNAPSHOT)
            .union(NodeFlags::HANDLED_SNAPSHOT);

        for node in root.traverse_preorder_non_rooting(cx.no_gc(), ShadowIncluding::No) {
            node.set_flag(RESET_FLAGS | NodeFlags::IS_IN_SHADOW_TREE, false);

            // If the element has a shadow root attached to it then we traverse that as well,
            // but without touching the IS_IN_SHADOW_TREE flags of the children
            if let Some(shadow_root) = node.downcast::<Element>().and_then(Element::shadow_root) {
                for node in shadow_root
                    .upcast::<Node>()
                    .traverse_preorder_non_rooting(cx.no_gc(), ShadowIncluding::Yes)
                {
                    node.set_flag(RESET_FLAGS, false);
                }
            }
        }

        // Step 12.
        let is_parent_connected = context.parent.is_connected();
        let custom_element_reaction_stack = ScriptThread::custom_element_reaction_stack();

        // Since both the initial traversal in light dom and the inner traversal
        // in shadow DOM share the same code, we define a closure to prevent omissions.
        let cleanup_node = |cx: &mut JSContext, node: &Node| {
            node.owner_doc().cancel_animations_for_node(node);
            node.clean_up_style_and_layout_data();

            // Step 11 & 14.1. Run the removing steps.
            // This needs to be in its own loop, because unbind_from_tree may
            // rely on the state of IS_IN_DOC of the context node's descendants,
            // e.g. when removing a <form>.
            vtable_for(node).unbind_from_tree(cx, context);

            // Step 12 & 14.2. Enqueue disconnected custom element reactions.
            if is_parent_connected && let Some(element) = node.as_custom_element() {
                custom_element_reaction_stack.enqueue_callback_reaction(
                    cx,
                    &element,
                    CallbackReaction::Disconnected,
                    None,
                );
            }
        };

        for node in root.traverse_preorder(ShadowIncluding::No) {
            cleanup_node(cx, &node);

            // Make sure that we don't accidentally initialize the rare data for this node
            // by setting it to None
            if node.containing_shadow_root().is_some() {
                // Reset the containing shadowRoot after we unbind the node, since some elements
                // require the containing shadowRoot for cleanup logic (e.g. <style>).
                node.set_containing_shadow_root(None);
            }

            // If the element has a shadow root attached to it then we traverse that as well,
            // but without resetting the contained shadow root
            if let Some(shadow_root) = node.downcast::<Element>().and_then(Element::shadow_root) {
                for node in shadow_root
                    .upcast::<Node>()
                    .traverse_preorder(ShadowIncluding::Yes)
                {
                    cleanup_node(cx, &node);
                }
            }
        }

        // Make sure the node and its subtree aren't GCed until the accessibility tree has had a
        // chance to remove them.
        if root.owner_document().accessibility_active() {
            root.owner_document()
                .accessibility_data_mut()
                .root_removed_node(cx.no_gc(), root);
        }
    }

    pub(crate) fn complete_move_subtree(cx: &mut JSContext, root: &Node) {
        // Flags that reset when a node is moved
        const RESET_FLAGS: NodeFlags = NodeFlags::IS_IN_A_DOCUMENT_TREE
            .union(NodeFlags::IS_CONNECTED)
            .union(NodeFlags::HAS_DIRTY_DESCENDANTS)
            .union(NodeFlags::HAS_SNAPSHOT)
            .union(NodeFlags::HANDLED_SNAPSHOT);

        for node in root.traverse_preorder(ShadowIncluding::No) {
            node.set_flag(RESET_FLAGS | NodeFlags::IS_IN_SHADOW_TREE, false);
            node.clean_up_style_and_layout_data();

            // Unregister the `id` and `name` attributes for this node. Note that they
            // will be re-registered when added to the tree again.
            if let Some(element) = node.downcast::<Element>() {
                element.unregister_current_id_and_name_attribute(cx);
            }

            // Make sure that we don't accidentally initialize the rare data for this node
            // by setting it to None
            if node.containing_shadow_root().is_some() {
                // Reset the containing shadowRoot after we unbind the node, since some elements
                // require the containing shadowRoot for cleanup logic (e.g. <style>).
                node.set_containing_shadow_root(None);
            }

            // If the element has a shadow root attached to it then we traverse that as well,
            // but without touching the IS_IN_SHADOW_TREE flags of the children,
            // and without resetting the contained shadow root
            if let Some(shadow_root) = node.downcast::<Element>().and_then(Element::shadow_root) {
                for node in shadow_root
                    .upcast::<Node>()
                    .traverse_preorder(ShadowIncluding::Yes)
                {
                    node.set_flag(RESET_FLAGS, false);
                    node.clean_up_style_and_layout_data();
                }
            }
        }
    }

    /// Removes the given child from this node's list of children.
    ///
    /// Fails unless `child` is a child of this node.
    fn remove_child(&self, cx: &mut JSContext, child: &Node, cached_index: Option<u32>) {
        assert!(child.parent_node.get().as_deref() == Some(self));
        self.note_dirty_descendants(cx.no_gc());
        self.add_pending_accessibility_damage(AccessibilityDamage::Children);

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

        Self::complete_remove_subtree(cx, child, &context);
    }

    fn move_child(&self, cx: &mut JSContext, child: &Node) {
        assert!(child.parent_node.get().as_deref() == Some(self));
        self.dirty(NodeDamage::ContentOrHeritage);
        self.note_dirty_descendants(cx.no_gc());
        self.add_pending_accessibility_damage(AccessibilityDamage::Children);

        child.prev_sibling.set(None);
        child.next_sibling.set(None);
        child.parent_node.set(None);
        self.children_count.set(self.children_count.get() - 1);
        Self::complete_move_subtree(cx, child)
    }

    pub(crate) fn to_opaque(&self) -> OpaqueNode {
        OpaqueNode(self.reflector().get_jsobject().get() as usize)
    }

    pub(crate) fn as_custom_element(&self) -> Option<DomRoot<Element>> {
        self.downcast::<Element>().and_then(|element| {
            if element.is_custom() {
                assert!(element.get_custom_element_definition().is_some());
                Some(DomRoot::from_ref(element))
            } else {
                None
            }
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#fire-a-synthetic-pointer-event>
    pub(crate) fn fire_synthetic_pointer_event_not_trusted(
        &self,
        cx: &mut JSContext,
        event_type: Atom,
    ) {
        // Spec says the choice of which global to create the pointer event
        // on is not well-defined,
        // and refers to heycam/webidl#135
        let window = self.owner_window();

        // <https://w3c.github.io/pointerevents/#the-click-auxclick-and-contextmenu-events>
        let pointer_event = PointerEvent::new(
            cx,
            &window, // ambiguous in spec
            event_type,
            EventBubbles::Bubbles,              // Step 3: bubbles
            EventCancelable::Cancelable,        // Step 3: cancelable
            Some(&window),                      // Step 7: view
            0,                                  // detail uninitialized
            Point2D::zero(),                    // coordinates uninitialized
            Point2D::zero(),                    // coordinates uninitialized
            Point2D::zero(),                    // coordinates uninitialized
            Modifiers::empty(),                 // empty modifiers
            0,                                  // button, left mouse button
            0,                                  // buttons
            None,                               // related_target
            None,                               // point_in_target
            PointerId::NonPointerDevice as i32, // pointer_id
            1,                                  // width
            1,                                  // height
            0.5,                                // pressure
            0.0,                                // tangential_pressure
            0,                                  // tilt_x
            0,                                  // tilt_y
            0,                                  // twist
            PI / 2.0,                           // altitude_angle
            0.0,                                // azimuth_angle
            DOMString::from(""),                // pointer_type
            false,                              // is_primary
            vec![],                             // coalesced_events
            vec![],                             // predicted_events
        );

        // Step 4. Set event's composed flag.
        pointer_event.upcast::<Event>().set_composed(true);

        // Step 5. If the not trusted flag is set, initialize event's isTrusted attribute to false.
        pointer_event.upcast::<Event>().set_trusted(false);

        // Step 6,8. TODO keyboard modifiers

        pointer_event
            .upcast::<Event>()
            .dispatch(cx, self.upcast::<EventTarget>(), false);
    }

    pub(crate) fn parent_directionality(&self) -> String {
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

    /// Implements the combination of:
    ///  - <https://html.spec.whatwg.org/multipage/#being-rendered>
    ///  - <https://html.spec.whatwg.org/multipage/#delegating-its-rendering-to-its-children>
    pub(crate) fn is_being_rendered_or_delegates_rendering(
        &self,
        pseudo_element: Option<PseudoElement>,
    ) -> bool {
        matches!(
            self.owner_window()
                .layout()
                .node_rendering_type(self.to_trusted_node_address(), pseudo_element),
            NodeRenderingType::Rendered | NodeRenderingType::DelegatesRendering
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#being-rendered>
    pub(crate) fn is_being_rendered(&self, pseudo_element: Option<PseudoElement>) -> bool {
        matches!(
            self.owner_window()
                .layout()
                .node_rendering_type(self.to_trusted_node_address(), pseudo_element),
            NodeRenderingType::Rendered
        )
    }

    fn add_pending_accessibility_damage(&self, damage: AccessibilityDamage) {
        if !self.owner_doc().accessibility_active() {
            return;
        }

        self.owner_doc()
            .accessibility_data_mut()
            .add_pending_accessibility_damage_for_node(self, damage);

        self.owner_window()
            .layout()
            .set_needs_accessibility_update();
    }
}

impl Node {
    fn ensure_rare_data(&self) -> RefMut<'_, Box<NodeRareData>> {
        let mut rare_data = self.rare_data.borrow_mut();
        if rare_data.is_none() {
            *rare_data = Some(Default::default());
        }
        RefMut::map(rare_data, |rare_data| rare_data.as_mut().unwrap())
    }

    /// Returns true if this node is before `other` in the same connected DOM
    /// tree.
    pub(crate) fn is_before(&self, other: &Node) -> bool {
        let cmp = other.CompareDocumentPosition(self);
        if cmp & NodeConstants::DOCUMENT_POSITION_DISCONNECTED != 0 {
            return false;
        }

        cmp & NodeConstants::DOCUMENT_POSITION_PRECEDING != 0
    }

    /// Return all registered mutation observers for this node. Lazily initialize the
    /// raredata if it does not exist.
    pub(crate) fn registered_mutation_observers_mut(&self) -> RefMut<'_, Vec<RegisteredObserver>> {
        RefMut::map(self.ensure_rare_data(), |rare_data| {
            &mut rare_data.mutation_observers
        })
    }

    pub(crate) fn registered_mutation_observers(&self) -> Option<Ref<'_, Vec<RegisteredObserver>>> {
        let rare_data = self.rare_data.borrow();
        if rare_data.is_none() {
            return None;
        }
        Some(Ref::map(rare_data, |rare_data| {
            &rare_data.as_ref().unwrap().mutation_observers
        }))
    }

    /// Add a new mutation observer for a given node.
    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn add_mutation_observer(&self, observer: RegisteredObserver) {
        self.ensure_rare_data().mutation_observers.push(observer);
    }

    /// Removes the mutation observer for a given node.
    pub(crate) fn remove_mutation_observer(&self, observer: &MutationObserver) {
        let mut rare_data = self.rare_data.borrow_mut();
        let Some(rare_data) = rare_data.as_mut() else {
            return;
        };
        rare_data
            .mutation_observers
            .retain(|registered_observer| &*registered_observer.observer != observer)
    }

    /// Returns a string that describes this node.
    pub(crate) fn debug_str(&self) -> String {
        format!("{:?}", self.type_id())
    }

    /// <https://dom.spec.whatwg.org/#in-a-document-tree>
    pub(crate) fn is_in_a_document_tree(&self) -> bool {
        self.flags.get().contains(NodeFlags::IS_IN_A_DOCUMENT_TREE)
    }

    /// Return true iff node's root is a shadow-root.
    pub(crate) fn is_in_a_shadow_tree(&self) -> bool {
        self.flags.get().contains(NodeFlags::IS_IN_SHADOW_TREE)
    }

    pub(crate) fn has_weird_parser_insertion_mode(&self) -> bool {
        self.flags
            .get()
            .contains(NodeFlags::HAS_WEIRD_PARSER_INSERTION_MODE)
    }

    pub(crate) fn set_weird_parser_insertion_mode(&self) {
        self.set_flag(NodeFlags::HAS_WEIRD_PARSER_INSERTION_MODE, true)
    }

    /// <https://dom.spec.whatwg.org/#connected>
    pub(crate) fn is_connected(&self) -> bool {
        self.flags.get().contains(NodeFlags::IS_CONNECTED)
    }

    pub(crate) fn set_in_ua_widget(&self, in_ua_widget: bool) {
        self.set_flag(NodeFlags::IS_IN_UA_WIDGET, in_ua_widget)
    }

    pub(crate) fn is_in_ua_widget(&self) -> bool {
        self.flags.get().contains(NodeFlags::IS_IN_UA_WIDGET)
    }

    /// Returns the type ID of this node.
    pub(crate) fn type_id(&self) -> NodeTypeId {
        match *self.eventtarget.type_id() {
            EventTargetTypeId::Node(type_id) => type_id,
            _ => unreachable!(),
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-node-length>
    pub(crate) fn len(&self) -> u32 {
        match self.type_id() {
            NodeTypeId::DocumentType => 0,
            NodeTypeId::CharacterData(_) => self.downcast::<CharacterData>().unwrap().Length(),
            _ => self.children_count(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        // A node is considered empty if its length is 0.
        self.len() == 0
    }

    /// <https://dom.spec.whatwg.org/#concept-tree-index>
    pub(crate) fn index(&self) -> u32 {
        self.preceding_siblings().count() as u32
    }

    /// Returns true if this node has a parent.
    pub(crate) fn has_parent(&self) -> bool {
        self.parent_node.get().is_some()
    }

    pub(crate) fn children_count(&self) -> u32 {
        self.children_count.get()
    }

    pub(crate) fn weak_ranges_mut(&self) -> Option<RefMut<'_, WeakRangeVec>> {
        let rare_data = self.rare_data.borrow_mut();
        if rare_data.is_none() {
            return None;
        }
        Some(RefMut::map(rare_data, |rare_data| {
            &mut rare_data.as_mut().unwrap().weak_ranges
        }))
    }

    pub(crate) fn ensure_weak_ranges(&self) -> RefMut<'_, WeakRangeVec> {
        RefMut::map(self.ensure_rare_data(), |rare_data| {
            &mut rare_data.weak_ranges
        })
    }

    pub(crate) fn weak_ranges_is_empty(&self) -> bool {
        self.rare_data
            .borrow()
            .as_ref()
            .is_none_or(|data| data.weak_ranges.is_empty())
    }

    #[inline]
    pub(crate) fn is_doctype(&self) -> bool {
        self.type_id() == NodeTypeId::DocumentType
    }

    pub(crate) fn get_flag(&self, flag: NodeFlags) -> bool {
        self.flags.get().contains(flag)
    }

    pub(crate) fn set_flag(&self, flag: NodeFlags, value: bool) {
        let mut flags = self.flags.get();

        if value {
            flags.insert(flag);
        } else {
            flags.remove(flag);
        }

        self.flags.set(flags);
    }

    // FIXME(emilio): This and the function below should move to Element.
    pub(crate) fn note_dirty_descendants(&self, no_gc: &NoGC) {
        self.owner_doc_unrooted(no_gc)
            .note_node_with_dirty_descendants(self);
    }

    pub(crate) fn has_dirty_descendants(&self) -> bool {
        self.get_flag(NodeFlags::HAS_DIRTY_DESCENDANTS)
    }

    pub(crate) fn rev_version(&self) {
        // The new version counter is 1 plus the max of the node's current version counter,
        // its descendants version, and the document's version. Normally, this will just be
        // the document's version, but we do have to deal with the case where the node has moved
        // document, so may have a higher version count than its owning document.
        let doc: DomRoot<Node> = DomRoot::upcast(self.owner_doc());
        let version = cmp::max(
            self.inclusive_descendants_version(),
            doc.inclusive_descendants_version(),
        ) + 1;

        // This `while` loop is equivalent to iterating over the non-shadow-inclusive ancestors
        // without creating intermediate rooted DOM objects.
        let mut node = &MutNullableDom::new(Some(self));
        while let Some(p) = node.if_is_some(|p| {
            p.inclusive_descendants_version.set(version);
            &p.parent_node
        }) {
            node = p
        }
        doc.inclusive_descendants_version.set(version);
    }

    pub(crate) fn dirty(&self, damage: NodeDamage) {
        self.rev_version();
        if !self.is_connected() {
            return;
        }

        match self.type_id() {
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(..)) => {
                // This drops the cached `TextRun` that is stored here, ultimately meaning that
                // shaped text will no longer be reused for this text node.
                *self.layout_data.borrow_mut() = None;

                // For content changes in text nodes, we should accurately use
                // [`NodeDamage::ContentOrHeritage`] to mark the parent node, thereby
                // reducing the scope of incremental box tree construction.
                self.parent_node
                    .get()
                    .unwrap()
                    .dirty(NodeDamage::ContentOrHeritage);

                if damage == NodeDamage::Other {
                    self.add_pending_accessibility_damage(AccessibilityDamage::Text);
                }
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
    pub(crate) fn inclusive_descendants_version(&self) -> u64 {
        self.inclusive_descendants_version.get()
    }

    /// Iterates over this node and all its descendants, in preorder.
    pub(crate) fn traverse_preorder(&self, shadow_including: ShadowIncluding) -> TreeIterator {
        TreeIterator::new(self, shadow_including)
    }

    /// Iterates over this node and all its descendants, in preorder. We take &NoGC to prevent GC which allows us to avoid rooting.
    pub(crate) fn traverse_preorder_non_rooting<'b>(
        &self,
        no_gc: &'b NoGC,
        shadow_including: ShadowIncluding,
    ) -> UnrootedTreeIterator<'b> {
        UnrootedTreeIterator::new(self, shadow_including, no_gc)
    }

    pub(crate) fn inclusively_following_siblings(
        &self,
    ) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator::new(Some(DomRoot::from_ref(self)), |n| n.GetNextSibling())
    }

    pub(crate) fn inclusively_following_siblings_unrooted<'b>(
        &self,
        no_gc: &'b NoGC,
    ) -> impl Iterator<Item = UnrootedDom<'b, Node>> + use<'b> {
        UnrootedSimpleNodeIterator::new(
            Some(UnrootedDom::from_dom(Dom::from_ref(self), no_gc)),
            |n, no_gc| n.get_next_sibling_unrooted(no_gc),
            no_gc,
        )
    }

    pub(crate) fn inclusively_preceding_siblings_unrooted<'b>(
        &self,
        no_gc: &'b NoGC,
    ) -> impl Iterator<Item = UnrootedDom<'b, Node>> + use<'b> {
        UnrootedSimpleNodeIterator::new(
            Some(UnrootedDom::from_dom(Dom::from_ref(self), no_gc)),
            |n, no_gc| n.get_previous_sibling_unrooted(no_gc),
            no_gc,
        )
    }

    pub(crate) fn common_ancestor(
        &self,
        other: &Node,
        shadow_including: ShadowIncluding,
    ) -> Option<DomRoot<Node>> {
        self.inclusive_ancestors(shadow_including).find(|ancestor| {
            other
                .inclusive_ancestors(shadow_including)
                .any(|node| node == *ancestor)
        })
    }

    pub(crate) fn common_ancestor_in_flat_tree(&self, other: &Node) -> Option<DomRoot<Node>> {
        self.inclusive_ancestors_in_flat_tree().find(|ancestor| {
            other
                .inclusive_ancestors_in_flat_tree()
                .any(|node| node == *ancestor)
        })
    }

    /// <https://dom.spec.whatwg.org/#concept-tree-inclusive-ancestor>
    pub(crate) fn is_inclusive_ancestor_of(&self, child: &Node) -> bool {
        // > An inclusive ancestor is an object or one of its ancestors.
        self == child || self.is_ancestor_of(child)
    }

    /// <https://dom.spec.whatwg.org/#concept-tree-ancestor>
    pub(crate) fn is_ancestor_of(&self, possible_descendant: &Node) -> bool {
        // > An object A is called an ancestor of an object B if and only if B is a descendant of A.
        let mut current = &possible_descendant.parent_node;
        let mut done = false;

        while let Some(node) = current.if_is_some(|node| {
            done = node == self;
            &node.parent_node
        }) {
            if done {
                break;
            }
            current = node
        }
        done
    }

    /// <https://dom.spec.whatwg.org/#concept-tree-host-including-inclusive-ancestor>
    fn is_host_including_inclusive_ancestor(&self, child: &Node) -> bool {
        // An object A is a host-including inclusive ancestor of an object B, if either A is an inclusive ancestor of B,
        // or if B’s root has a non-null host and A is a host-including inclusive ancestor of B’s root’s host.
        self.is_inclusive_ancestor_of(child) ||
            child
                .GetRootNode(&GetRootNodeOptions::empty())
                .downcast::<DocumentFragment>()
                .and_then(|fragment| fragment.host())
                .is_some_and(|host| self.is_host_including_inclusive_ancestor(host.upcast()))
    }

    /// <https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-ancestor>
    pub(crate) fn is_shadow_including_inclusive_ancestor_of(&self, node: &Node) -> bool {
        node.inclusive_ancestors(ShadowIncluding::Yes)
            .any(|ancestor| &*ancestor == self)
    }

    pub(crate) fn following_siblings(&self) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator::new(self.GetNextSibling(), |n| n.GetNextSibling())
    }

    pub(crate) fn preceding_siblings(&self) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator::new(self.GetPreviousSibling(), |n| n.GetPreviousSibling())
    }

    pub(crate) fn following_nodes(
        &self,
        root: &Node,
        shadow_including: ShadowIncluding,
    ) -> FollowingNodeIterator {
        FollowingNodeIterator::new(
            Some(DomRoot::from_ref(self)),
            DomRoot::from_ref(root),
            shadow_including,
        )
    }

    pub(crate) fn following_nodes_unrooted<'b>(
        &self,
        no_gc: &'b NoGC,
        root: &Node,
        shadow_including: ShadowIncluding,
    ) -> UnrootedFollowingNodeIterator<'b> {
        UnrootedFollowingNodeIterator::new(
            Some(UnrootedDom::from_dom(Dom::from_ref(self), no_gc)),
            UnrootedDom::from_dom(Dom::from_ref(root), no_gc),
            shadow_including,
            no_gc,
        )
    }

    pub(crate) fn preceding_nodes(&self, root: &Node) -> PrecedingNodeIterator {
        PrecedingNodeIterator::new(Some(DomRoot::from_ref(self)), DomRoot::from_ref(root))
    }

    pub(crate) fn preceding_nodes_unrooted<'b>(
        &self,
        no_gc: &'b NoGC,
        root: &Node,
    ) -> UnrootedPrecedingNodeIterator<'b> {
        UnrootedPrecedingNodeIterator::new(
            Some(UnrootedDom::from_dom(Dom::from_ref(self), no_gc)),
            UnrootedDom::from_dom(Dom::from_ref(root), no_gc),
            no_gc,
        )
    }

    /// Return an iterator that moves from `self` down the tree, choosing the last child
    /// at each step of the way.
    pub(crate) fn descending_last_children(&self) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator::new(self.GetLastChild(), |n| n.GetLastChild())
    }

    pub(crate) fn descending_last_children_unrooted<'b>(
        &self,
        no_gc: &'b NoGC,
    ) -> impl Iterator<Item = UnrootedDom<'b, Node>> {
        UnrootedSimpleNodeIterator::new(
            self.get_last_child_unrooted(no_gc),
            |n, no_gc| n.get_last_child_unrooted(no_gc),
            no_gc,
        )
    }

    pub(crate) fn is_parent_of(&self, child: &Node) -> bool {
        child
            .parent_node
            .get()
            .is_some_and(|parent| &*parent == self)
    }

    pub(crate) fn to_trusted_node_address(&self) -> TrustedNodeAddress {
        TrustedNodeAddress(self as *const Node as *const libc::c_void)
    }

    /// Return the node that establishes a containing block for this node.
    pub(crate) fn containing_block_node_without_reflow(&self) -> Option<DomRoot<Node>> {
        self.owner_window()
            .containing_block_node_query_without_reflow(self)
    }

    pub(crate) fn padding(&self) -> Option<PhysicalSides> {
        self.owner_window().padding_query_without_reflow(self)
    }

    pub(crate) fn content_box(&self) -> Option<Rect<Au, CSSPixel>> {
        self.owner_window()
            .box_area_query(self, BoxAreaType::Content, false)
    }

    pub(crate) fn border_box(&self) -> Option<Rect<Au, CSSPixel>> {
        self.owner_window()
            .box_area_query(self, BoxAreaType::Border, false)
    }

    pub(crate) fn border_box_without_reflow(&self) -> Option<Rect<Au, CSSPixel>> {
        self.owner_window()
            .box_area_query_without_reflow(self, BoxAreaType::Border, false)
    }

    pub(crate) fn padding_box(&self) -> Option<Rect<Au, CSSPixel>> {
        self.owner_window()
            .box_area_query(self, BoxAreaType::Padding, false)
    }

    pub(crate) fn padding_box_without_reflow(&self) -> Option<Rect<Au, CSSPixel>> {
        self.owner_window()
            .box_area_query_without_reflow(self, BoxAreaType::Padding, false)
    }

    pub(crate) fn border_boxes(&self) -> CSSPixelRectVec {
        self.owner_window()
            .box_areas_query(self, BoxAreaType::Border)
    }

    pub(crate) fn client_rect(&self) -> Rect<i32, CSSPixel> {
        self.owner_window().client_rect_query(self)
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-element-scrollwidth>
    /// <https://drafts.csswg.org/cssom-view/#dom-element-scrollheight>
    pub(crate) fn scroll_area(&self) -> Rect<i32, CSSPixel> {
        // "1. Let document be the element’s node document.""
        let document = self.owner_doc();

        // "2. If document is not the active document, return zero and terminate these steps.""
        if !document.is_active() {
            return Rect::zero();
        }

        // "3. Let viewport width/height be the width of the viewport excluding the width/height of the
        // scroll bar, if any, or zero if there is no viewport."
        let window = document.window();
        let viewport = Size2D::new(window.InnerWidth(), window.InnerHeight()).cast_unit();

        let in_quirks_mode = document.quirks_mode() == QuirksMode::Quirks;
        let is_root = self.downcast::<Element>().is_some_and(|e| e.is_root());
        let is_body_element = self
            .downcast::<HTMLElement>()
            .is_some_and(|e| e.is_body_element());

        // "4. If the element is the root element and document is not in quirks mode
        // return max(viewport scrolling area width/height, viewport width/height)."
        // "5. If the element is the body element, document is in quirks mode and the
        // element is not potentially scrollable, return max(viewport scrolling area
        // width, viewport width)."
        if (is_root && !in_quirks_mode) || (is_body_element && in_quirks_mode) {
            let viewport_scrolling_area = window.scrolling_area_query(None);
            return Rect::new(
                viewport_scrolling_area.origin,
                viewport_scrolling_area.size.max(viewport),
            );
        }

        // "6. If the element does not have any associated box return zero and terminate
        // these steps."
        // "7. Return the width of the element’s scrolling area."
        window.scrolling_area_query(Some(self))
    }

    pub(crate) fn effective_overflow(&self) -> Option<AxesOverflow> {
        self.owner_window().query_effective_overflow(self)
    }

    pub(crate) fn effective_overflow_without_reflow(&self) -> Option<AxesOverflow> {
        self.owner_window()
            .query_effective_overflow_without_reflow(self)
    }

    /// <https://dom.spec.whatwg.org/#dom-childnode-before>
    pub(crate) fn before(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
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
        let node = self.owner_doc().node_from_nodes_and_strings(cx, nodes)?;

        // Step 5.
        let viable_previous_sibling = match viable_previous_sibling {
            Some(ref viable_previous_sibling) => viable_previous_sibling.next_sibling.get(),
            None => parent.first_child.get(),
        };

        // Step 6.
        Node::pre_insert(cx, &node, &parent, viable_previous_sibling.as_deref())?;

        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-childnode-after>
    pub(crate) fn after(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
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
        let node = self.owner_doc().node_from_nodes_and_strings(cx, nodes)?;

        // Step 5.
        Node::pre_insert(cx, &node, &parent, viable_next_sibling.as_deref())?;

        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-childnode-replacewith>
    pub(crate) fn replace_with(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1. Let parent be this’s parent.
        let Some(parent) = self.GetParentNode() else {
            // Step 2. If parent is null, then return.
            return Ok(());
        };

        // Step 3. Let viableNextSibling be this’s first following sibling not in nodes; otherwise null.
        let viable_next_sibling = first_node_not_in(self.following_siblings(), &nodes);

        // Step 4. Let node be the result of converting nodes into a node, given nodes and this’s node document.
        let node = self.owner_doc().node_from_nodes_and_strings(cx, nodes)?;

        if self.parent_node == Some(&*parent) {
            // Step 5. If this’s parent is parent, replace this with node within parent.
            parent.ReplaceChild(cx, &node, self)?;
        } else {
            // Step 6. Otherwise, pre-insert node into parent before viableNextSibling.
            Node::pre_insert(cx, &node, &parent, viable_next_sibling.as_deref())?;
        }
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-prepend>
    pub(crate) fn prepend(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let doc = self.owner_doc();
        let node = doc.node_from_nodes_and_strings(cx, nodes)?;
        // Step 2.
        let first_child = self.first_child.get();
        Node::pre_insert(cx, &node, self, first_child.as_deref()).map(|_| ())
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-append>
    pub(crate) fn append(&self, cx: &mut JSContext, nodes: Vec<NodeOrString>) -> ErrorResult {
        // Step 1.
        let doc = self.owner_doc();
        let node = doc.node_from_nodes_and_strings(cx, nodes)?;
        // Step 2.
        self.AppendChild(cx, &node).map(|_| ())
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-replacechildren>
    pub(crate) fn replace_children(
        &self,
        cx: &mut JSContext,
        nodes: Vec<NodeOrString>,
    ) -> ErrorResult {
        // Step 1. Let node be the result of converting nodes into a node given nodes and this’s
        // node document.
        let doc = self.owner_doc();
        let node = doc.node_from_nodes_and_strings(cx, nodes)?;

        // Step 2. Ensure pre-insert validity of node into this before null.
        Node::ensure_pre_insertion_validity(cx.no_gc(), &node, self, None)?;

        // Step 3. Replace all with node within this.
        Node::replace_all(cx, Some(&node), self);
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-movebefore>
    pub(crate) fn move_before(
        &self,
        cx: &mut JSContext,
        node: &Node,
        child: Option<&Node>,
    ) -> ErrorResult {
        // Step 1. Let referenceChild be child.
        // Step 2. If referenceChild is node, then set referenceChild to node’s next sibling.
        let reference_child_root;
        let reference_child = match child {
            Some(child) if child == node => {
                reference_child_root = node.GetNextSibling();
                reference_child_root.as_deref()
            },
            _ => child,
        };

        // Step 3. Move node into this before referenceChild.
        Node::move_fn(cx, node, self, reference_child)
    }

    /// <https://dom.spec.whatwg.org/#move>
    fn move_fn(
        cx: &mut JSContext,
        node: &Node,
        new_parent: &Node,
        child: Option<&Node>,
    ) -> ErrorResult {
        // Step 1. If newParent’s shadow-including root is not the same as node’s shadow-including
        // root, then throw a "HierarchyRequestError" DOMException.
        // This has the side effect of ensuring that a move is only performed if newParent’s
        // connected is node’s connected.
        let mut options = GetRootNodeOptions::empty();
        options.composed = true;
        if new_parent.GetRootNode(&options) != node.GetRootNode(&options) {
            return Err(Error::HierarchyRequest(None));
        }

        // Step 2. If node is a host-including inclusive ancestor of newParent, then throw a
        // "HierarchyRequestError" DOMException.
        if node.is_inclusive_ancestor_of(new_parent) {
            return Err(Error::HierarchyRequest(None));
        }

        // Step 3. If child is non-null and its parent is not newParent, then throw a
        // "NotFoundError" DOMException.
        if let Some(child) = child &&
            !new_parent.is_parent_of(child)
        {
            return Err(Error::NotFound(None));
        }

        // Step 4. If node is not an Element or a CharacterData node, then throw a
        // "HierarchyRequestError" DOMException.
        // Step 5. If node is a Text node and newParent is a document, then throw a
        // "HierarchyRequestError" DOMException.
        match node.type_id() {
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => {
                if new_parent.is::<Document>() {
                    return Err(Error::HierarchyRequest(None));
                }
            },
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) |
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) |
            NodeTypeId::Element(_) => (),
            NodeTypeId::DocumentFragment(_) |
            NodeTypeId::DocumentType |
            NodeTypeId::Document(_) |
            NodeTypeId::Attr => {
                return Err(Error::HierarchyRequest(None));
            },
        }

        // Step 6. If newParent is a document, node is an Element node, and either newParent has an
        // element child, child is a doctype, or child is non-null and a doctype is following child
        // then throw a "HierarchyRequestError" DOMException.
        if new_parent.is::<Document>() && node.is::<Element>() {
            // either newParent has an element child
            if new_parent.child_elements().next().is_some() {
                return Err(Error::HierarchyRequest(None));
            }

            // child is a doctype
            // or child is non-null and a doctype is following child
            if child.is_some_and(|child| {
                child
                    .inclusively_following_siblings_unrooted(cx.no_gc())
                    .any(|child| child.is_doctype())
            }) {
                return Err(Error::HierarchyRequest(None));
            }
        }

        // Step 7. Let oldParent be node’s parent.
        // Step 8. Assert: oldParent is non-null.
        let old_parent = node
            .parent_node
            .get()
            .expect("old_parent should always be initialized");

        // Step 9. Run the live range pre-remove steps, given node.
        let cached_index = Node::live_range_pre_remove_steps(node, &old_parent);

        // TODO Step 10. For each NodeIterator object iterator whose root’s node document is node’s
        // node document: run the NodeIterator pre-remove steps given node and iterator.

        // Step 11. Let oldPreviousSibling be node’s previous sibling.
        let old_previous_sibling = node.prev_sibling.get();

        // Step 12. Let oldNextSibling be node’s next sibling.
        let old_next_sibling = node.next_sibling.get();

        let prev_sibling = node.GetPreviousSibling();
        match prev_sibling {
            None => {
                old_parent
                    .first_child
                    .set(node.next_sibling.get().as_deref());
            },
            Some(ref prev_sibling) => {
                prev_sibling
                    .next_sibling
                    .set(node.next_sibling.get().as_deref());
            },
        }
        let next_sibling = node.GetNextSibling();
        match next_sibling {
            None => {
                old_parent
                    .last_child
                    .set(node.prev_sibling.get().as_deref());
            },
            Some(ref next_sibling) => {
                next_sibling
                    .prev_sibling
                    .set(node.prev_sibling.get().as_deref());
            },
        }

        let mut context =
            MoveContext::new(Some(&old_parent), prev_sibling.as_deref(), cached_index);

        // Step 13. Remove node from oldParent’s children.
        old_parent.move_child(cx, node);

        // Step 14. If node is assigned, then run assign slottables for node’s assigned slot.
        if let Some(slot) = node.assigned_slot() {
            slot.assign_slottables(cx);
        }

        // Step 15. If oldParent’s root is a shadow root, and oldParent is a slot whose assigned
        // nodes is empty, then run signal a slot change for oldParent.
        if old_parent.is_in_a_shadow_tree() &&
            let Some(slot_element) = old_parent.downcast::<HTMLSlotElement>() &&
            !slot_element.has_assigned_nodes()
        {
            slot_element.signal_a_slot_change(cx);
        }

        // Step 16. If node has an inclusive descendant that is a slot:
        let has_slot_descendant = node
            .traverse_preorder_non_rooting(cx.no_gc(), ShadowIncluding::No)
            .any(|element| element.is::<HTMLSlotElement>());
        if has_slot_descendant {
            // Step 16.1. Run assign slottables for a tree with oldParent’s root.
            old_parent
                .GetRootNode(&GetRootNodeOptions::empty())
                .assign_slottables_for_a_tree(cx, ForceSlottableNodeReconciliation::Skip);

            // Step 16.2. Run assign slottables for a tree with node.
            node.assign_slottables_for_a_tree(cx, ForceSlottableNodeReconciliation::Skip);
        }

        // Step 17. If child is non-null:
        if let Some(child) = child &&
            let Some(new_parent_ranges) = new_parent.weak_ranges_mut()
        {
            // Step 17.1. For each live range whose start node is newParent and start offset is
            // greater than child’s index: increase its start offset by 1.
            // Step 17.2. For each live range whose end node is newParent and end offset is greater
            // than child’s index: increase its end offset by 1.
            new_parent_ranges.increase_above(new_parent, child.index(), 1)
        }

        // Step 18. Let newPreviousSibling be child’s previous sibling if child is non-null, and
        // newParent’s last child otherwise.
        let new_previous_sibling = child.map_or_else(
            || new_parent.last_child.get(),
            |child| child.prev_sibling.get(),
        );

        // Step 19. If child is null, then append node to newParent’s children.
        // Step 20. Otherwise, insert node into newParent’s children before child’s index.
        new_parent.add_child(cx, node, child);

        // Step 21. If newParent is a shadow host whose shadow root’s slot assignment is "named" and
        // node is a slottable, then assign a slot for node.
        if let Some(shadow_root) = new_parent
            .downcast::<Element>()
            .and_then(Element::shadow_root) &&
            shadow_root.SlotAssignment() == SlotAssignmentMode::Named &&
            (node.is::<Element>() || node.is::<Text>())
        {
            rooted!(&in(cx) let slottable = Slottable(Dom::from_ref(node)));
            slottable.assign_a_slot(cx);
        }

        // Step 22. If newParent’s root is a shadow root, and newParent is a slot whose assigned
        // nodes is empty, then run signal a slot change for newParent.
        if new_parent.is_in_a_shadow_tree() &&
            let Some(slot_element) = new_parent.downcast::<HTMLSlotElement>() &&
            !slot_element.has_assigned_nodes()
        {
            slot_element.signal_a_slot_change(cx);
        }

        // Step 23. Run assign slottables for a tree with node’s root.
        node.GetRootNode(&GetRootNodeOptions::empty())
            .assign_slottables_for_a_tree(cx, ForceSlottableNodeReconciliation::Skip);

        // Step 24. For each shadow-including inclusive descendant inclusiveDescendant of node, in
        // shadow-including tree order:
        for descendant in node.traverse_preorder(ShadowIncluding::Yes) {
            // Step 24.1. If inclusiveDescendant is node, then run the moving steps with
            // inclusiveDescendant and oldParent.
            // Otherwise, run the moving steps with inclusiveDescendant and null.
            if descendant.deref() == node {
                vtable_for(&descendant).moving_steps(cx, &context);
            } else {
                context.old_parent = None;
                vtable_for(&descendant).moving_steps(cx, &context);
            }

            // Step 24.2. If inclusiveDescendant is custom and newParent is connected,
            if let Some(descendant) = descendant.downcast::<Element>() &&
                descendant.is_custom() &&
                new_parent.is_connected()
            {
                // then enqueue a custom element callback reaction with
                // inclusiveDescendant, callback name "connectedMoveCallback", and « ».
                let custom_element_reaction_stack = ScriptThread::custom_element_reaction_stack();
                custom_element_reaction_stack.enqueue_callback_reaction(
                    cx,
                    descendant,
                    CallbackReaction::ConnectedMove,
                    None,
                );
            }
        }

        // Step 25. Queue a tree mutation record for oldParent with « », « node »,
        // oldPreviousSibling, and oldNextSibling.
        let moved = [node];
        let mutation = LazyCell::new(|| Mutation::ChildList {
            added: None,
            removed: Some(&moved),
            prev: old_previous_sibling.as_deref(),
            next: old_next_sibling.as_deref(),
        });
        MutationObserver::queue_a_mutation_record(cx, &old_parent, mutation);

        // Step 26. Queue a tree mutation record for newParent with « node », « »,
        // newPreviousSibling, and child.
        let mutation = LazyCell::new(|| Mutation::ChildList {
            added: Some(&moved),
            removed: None,
            prev: new_previous_sibling.as_deref(),
            next: child,
        });
        MutationObserver::queue_a_mutation_record(cx, new_parent, mutation);

        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-queryselector>
    #[allow(unsafe_code)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn query_selector(
        &self,
        no_gc: &NoGC,
        selectors: DOMString,
    ) -> Fallible<Option<DomRoot<Element>>> {
        // > The querySelector(selectors) method steps are to return the first result of running scope-match
        // > a selectors string selectors against this, if the result is not an empty list; otherwise null.
        let document_url = self.owner_document().url().get_arc();

        // If there are any duplicate ids, their targets may need to be updated in the id map before
        // layout runs, so that the map can gather their elements in DOM order.
        self.owner_document()
            .id_map()
            .resolve_all(no_gc, self.owner_doc().upcast());

        // SAFETY: traced_node is unrooted, but we have a reference to "self" so it won't be freed.
        let traced_node = Dom::from_ref(self);

        let first_matching_element = with_layout_state(|| {
            let layout_node: LayoutDom<'_, _> = unsafe { traced_node.to_layout() };
            ServoDangerousStyleNode::from(layout_node)
                .scope_match_a_selectors_string::<QueryFirst>(document_url, &selectors.str())
        })?;

        Ok(first_matching_element.map(ServoDangerousStyleElement::rooted))
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall>
    #[allow(unsafe_code)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn query_selector_all(
        &self,
        cx: &mut JSContext,
        selectors: DOMString,
    ) -> Fallible<DomRoot<NodeList>> {
        // > The querySelectorAll(selectors) method steps are to return the static result of running scope-match
        // > a selectors string selectors against this.
        let document_url = self.owner_document().url().get_arc();

        // If there are any duplicate ids, their targets may need to be updated in the id map before
        // layout runs, so that the map can gather their elements in DOM order.
        self.owner_document()
            .id_map()
            .resolve_all(cx.no_gc(), self.owner_doc().upcast());

        let traced_node = UnrootedDom::from_dom(Dom::from_ref(self), cx.no_gc());
        let matching_elements = with_layout_state(|| {
            let layout_node: LayoutDom<'_, _> = unsafe { traced_node.to_layout() };
            ServoDangerousStyleNode::from(layout_node)
                .scope_match_a_selectors_string::<QueryAll>(document_url, &selectors.str())
        })?;
        let iter = matching_elements
            .into_iter()
            .map(ServoDangerousStyleElement::rooted)
            .map(DomRoot::upcast::<Node>);

        // NodeList::new_simple_list immediately collects the iterator, so we're not leaking LayoutDom
        // elements here.
        Ok(NodeList::new_simple_list(cx, &self.owner_window(), iter))
    }

    pub(crate) fn ancestors(&self) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator::new(self.GetParentNode(), |n| n.GetParentNode())
    }

    /// <https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-ancestor>
    pub(crate) fn inclusive_ancestors(
        &self,
        shadow_including: ShadowIncluding,
    ) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator::new(Some(DomRoot::from_ref(self)), move |n| {
            if shadow_including == ShadowIncluding::Yes &&
                let Some(shadow_root) = n.downcast::<ShadowRoot>()
            {
                return Some(DomRoot::from_ref(shadow_root.Host().upcast::<Node>()));
            }
            n.GetParentNode()
        })
    }

    pub(crate) fn inclusive_ancestors_unrooted<'a>(
        &self,
        no_gc: &'a NoGC,
        shadow_including: ShadowIncluding,
    ) -> impl Iterator<Item = UnrootedDom<'a, Node>> + use<'a> {
        UnrootedSimpleNodeIterator::new(
            Some(UnrootedDom::from_dom(Dom::from_ref(self), no_gc)),
            move |n, no_gc| {
                if shadow_including == ShadowIncluding::Yes &&
                    let Some(shadow_root) = n.downcast::<ShadowRoot>()
                {
                    return Some(UnrootedDom::from_dom(
                        Dom::from_ref(shadow_root.Host().upcast::<Node>()),
                        no_gc,
                    ));
                }
                n.get_parent_node_unrooted(no_gc)
            },
            no_gc,
        )
    }

    pub(crate) fn owner_doc(&self) -> DomRoot<Document> {
        self.owner_doc.get().unwrap()
    }

    pub(crate) fn owner_doc_unrooted<'a>(&self, no_gc: &'a NoGC) -> UnrootedDom<'a, Document> {
        self.owner_doc.get_unrooted(no_gc).unwrap()
    }

    pub(crate) fn set_owner_doc(&self, document: &Document) {
        self.owner_doc.set(Some(document));
    }

    pub(crate) fn containing_shadow_root(&self) -> Option<DomRoot<ShadowRoot>> {
        self.rare_data
            .borrow()
            .as_ref()?
            .containing_shadow_root
            .as_ref()
            .map(|sr| DomRoot::from_ref(&**sr))
    }

    pub(crate) fn set_containing_shadow_root(&self, shadow_root: Option<&ShadowRoot>) {
        self.ensure_rare_data().containing_shadow_root = shadow_root.map(Dom::from_ref);
    }

    pub(crate) fn is_in_html_doc(&self) -> bool {
        self.owner_doc().is_html_document()
    }

    pub(crate) fn is_connected_with_browsing_context(&self) -> bool {
        self.is_connected() && self.owner_doc().browsing_context().is_some()
    }

    pub(crate) fn children(&self) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator::new(self.GetFirstChild(), |n| n.GetNextSibling())
    }

    pub(crate) fn children_unrooted<'a>(
        &self,
        no_gc: &'a NoGC,
    ) -> impl Iterator<Item = UnrootedDom<'a, Node>> + use<'a> {
        UnrootedSimpleNodeIterator::new(
            self.get_first_child_unrooted(no_gc),
            |n, no_gc| n.get_next_sibling_unrooted(no_gc),
            no_gc,
        )
    }

    pub(crate) fn rev_children(&self) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator::new(self.GetLastChild(), |n| n.GetPreviousSibling())
    }

    /// Returns the children that are Elements
    pub(crate) fn child_elements(&self) -> impl Iterator<Item = DomRoot<Element>> + use<> {
        self.children()
            .filter_map(DomRoot::downcast as fn(_) -> _)
            .peekable()
    }

    pub(crate) fn child_elements_unrooted<'a>(
        &self,
        no_gc: &'a NoGC,
    ) -> impl Iterator<Item = UnrootedDom<'a, Element>> + use<'a> {
        self.children_unrooted(no_gc)
            .filter_map(UnrootedDom::downcast)
            .peekable()
    }

    pub(crate) fn remove_self(&self, cx: &mut JSContext) {
        if let Some(ref parent) = self.GetParentNode() {
            Node::remove(cx, self, parent, SuppressObserver::Unsuppressed);
        }
    }

    /// Returns the node's `unique_id` if it has been computed before and `None` otherwise.
    pub(crate) fn unique_id_if_already_present(&self) -> Option<String> {
        Ref::filter_map(self.rare_data.borrow(), |rare_data| {
            rare_data
                .as_ref()
                .and_then(|rare_data| rare_data.unique_id.as_ref())
        })
        .ok()
        .map(|unique_id| unique_id.borrow().simple().to_string())
    }

    pub(crate) fn unique_id(&self, pipeline: PipelineId) -> String {
        let mut rare_data = self.ensure_rare_data();

        if rare_data.unique_id.is_none() {
            let node_id = UniqueId::new();
            ScriptThread::save_node_id(pipeline, node_id.borrow().simple().to_string());
            rare_data.unique_id = Some(node_id);
        }
        rare_data
            .unique_id
            .as_ref()
            .unwrap()
            .borrow()
            .simple()
            .to_string()
    }

    pub(crate) fn summarize(&self, cx: &mut JSContext) -> NodeInfo {
        let USVString(base_uri) = self.BaseURI();
        let node_type = self.NodeType();
        let pipeline = self.owner_window().pipeline_id();

        let maybe_shadow_root = self.downcast::<ShadowRoot>();
        let shadow_root_mode = maybe_shadow_root
            .map(ShadowRoot::Mode)
            .map(ShadowRootMode::convert);
        let host = maybe_shadow_root
            .map(ShadowRoot::Host)
            .map(|host| host.upcast::<Node>().unique_id(pipeline));
        let is_shadow_host = self.downcast::<Element>().is_some_and(|potential_host| {
            let Some(root) = potential_host.shadow_root() else {
                return false;
            };
            !root.is_user_agent_widget() || pref!(inspector_show_servo_internal_shadow_roots)
        });

        let num_children = if is_shadow_host {
            // Shadow roots count as children
            self.ChildNodes(cx).Length() as usize + 1
        } else {
            self.ChildNodes(cx).Length() as usize
        };

        let window = self.owner_window();
        let element = self.downcast::<Element>();
        let display = element
            .map(|elem| window.GetComputedStyle(cx, elem, None))
            .map(|style| style.Display().into());

        // It is not entirely clear when this should be set to false.
        // Firefox considers nodes with "display: contents" to be displayed.
        // The doctype node is displayed despite being `display: none`.
        //
        // TODO: Should this be false if the node is in a `display: none` subtree?
        let is_displayed =
            element.is_none_or(|element| !element.is_display_none()) || self.is::<DocumentType>();
        let attrs = element.map(Element::summarize).unwrap_or_default();

        NodeInfo {
            unique_id: self.unique_id(pipeline),
            host,
            base_uri,
            parent: self
                .GetParentNode()
                .map_or("".to_owned(), |node| node.unique_id(pipeline)),
            node_type,
            is_top_level_document: node_type == NodeConstants::DOCUMENT_NODE,
            node_name: String::from(self.NodeName()),
            node_value: self.GetNodeValue().map(|v| v.into()),
            num_children,
            attrs,
            is_shadow_host,
            shadow_root_mode,
            display,
            is_displayed,
            doctype_name: self
                .downcast::<DocumentType>()
                .map(DocumentType::name)
                .cloned()
                .map(String::from),
            doctype_public_identifier: self
                .downcast::<DocumentType>()
                .map(DocumentType::public_id)
                .cloned()
                .map(String::from),
            doctype_system_identifier: self
                .downcast::<DocumentType>()
                .map(DocumentType::system_id)
                .cloned()
                .map(String::from),
            has_event_listeners: self.upcast::<EventTarget>().has_handlers(),
        }
    }

    /// Used by `HTMLTableSectionElement::InsertRow` and `HTMLTableRowElement::InsertCell`
    pub(crate) fn insert_cell_or_row<F, G, I>(
        &self,
        cx: &mut JSContext,
        index: i32,
        get_items: F,
        new_child: G,
    ) -> Fallible<DomRoot<HTMLElement>>
    where
        F: Fn(&mut JSContext) -> DomRoot<HTMLCollection>,
        G: Fn(&mut JSContext) -> DomRoot<I>,
        I: DerivedFrom<Node> + DerivedFrom<HTMLElement> + DomObject,
    {
        if index < -1 {
            return Err(Error::IndexSize(None));
        }

        let tr = new_child(cx);

        {
            let tr_node = tr.upcast::<Node>();
            if index == -1 {
                self.InsertBefore(cx, tr_node, None)?;
            } else {
                let items = get_items(cx);
                let node = match items
                    .elements_iter(cx.no_gc())
                    .map(UnrootedDom::upcast::<Node>)
                    .map(Some)
                    .chain(iter::once(None))
                    .nth(index as usize)
                {
                    None => return Err(Error::IndexSize(None)),
                    Some(node) => node,
                };
                self.InsertBefore(cx, tr_node, node.map(|node| node.as_rooted()).as_deref())?;
            }
        }

        Ok(DomRoot::upcast::<HTMLElement>(tr))
    }

    /// Used by `HTMLTableSectionElement::DeleteRow` and `HTMLTableRowElement::DeleteCell`
    pub(crate) fn delete_cell_or_row<F, G>(
        &self,
        cx: &mut JSContext,
        index: i32,
        get_items: F,
        is_delete_type: G,
    ) -> ErrorResult
    where
        F: Fn(&mut JSContext) -> DomRoot<HTMLCollection>,
        G: Fn(&Element) -> bool,
    {
        let element = match index {
            index if index < -1 => return Err(Error::IndexSize(None)),
            -1 => {
                let last_child = self.upcast::<Node>().GetLastChild();
                match last_child.and_then(|node| {
                    node.inclusively_preceding_siblings_unrooted(cx.no_gc())
                        .filter_map(UnrootedDom::downcast::<Element>)
                        .find(|elem| is_delete_type(elem))
                        .map(|elem| elem.as_rooted())
                }) {
                    Some(element) => element,
                    None => return Ok(()),
                }
            },
            index => match get_items(cx).Item(cx, index as u32) {
                Some(element) => element,
                None => return Err(Error::IndexSize(None)),
            },
        };

        element.upcast::<Node>().remove_self(cx);
        Ok(())
    }

    pub(crate) fn get_cssom_stylesheet(
        &self,
        cx: &mut JSContext,
    ) -> Option<DomRoot<CSSStyleSheet>> {
        if let Some(node) = self.downcast::<HTMLStyleElement>() {
            node.get_cssom_stylesheet(cx)
        } else if let Some(node) = self.downcast::<HTMLLinkElement>() {
            node.get_cssom_stylesheet(cx)
        } else {
            None
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#language>
    pub(crate) fn get_lang(&self) -> Option<String> {
        self.inclusive_ancestors(ShadowIncluding::Yes)
            .find_map(|node| {
                node.downcast::<Element>().and_then(|el| {
                    el.get_attribute_string_value_with_namespace(&ns!(xml), &local_name!("lang"))
                        .or_else(|| el.get_attribute_string_value(&local_name!("lang")))
                })
                // TODO: Check meta tags for a pragma-set default language
                // TODO: Check HTTP Content-Language header
            })
    }

    /// <https://dom.spec.whatwg.org/#assign-slotables-for-a-tree>
    pub(crate) fn assign_slottables_for_a_tree(
        &self,
        cx: &JSContext,
        force: ForceSlottableNodeReconciliation,
    ) {
        // NOTE: This method traverses all descendants of the node and is potentially very
        // expensive. If the node is neither a shadowroot nor a slot then assigning slottables
        // for it won't have any effect, so we take a fast path out.
        // In the case of node removal, we need to force re-assignment of slottables
        // even if the node is not a shadow root or slot, this allows us to clear assigned
        // slots from any slottables that were assigned to slots in the removed subtree.
        let is_shadow_root_with_slots = self
            .downcast::<ShadowRoot>()
            .is_some_and(|shadow_root| shadow_root.has_slot_descendants());
        if !is_shadow_root_with_slots &&
            !self.is::<HTMLSlotElement>() &&
            matches!(force, ForceSlottableNodeReconciliation::Skip)
        {
            return;
        }

        // > To assign slottables for a tree, given a node root, run assign slottables for each slot
        // > slot in root’s inclusive descendants, in tree order.
        for node in self.traverse_preorder_non_rooting(cx, ShadowIncluding::No) {
            if let Some(slot) = node.downcast::<HTMLSlotElement>() {
                slot.assign_slottables(cx);
            }
        }
    }

    pub(crate) fn assigned_slot(&self) -> Option<DomRoot<HTMLSlotElement>> {
        let assigned_slot = self
            .rare_data
            .borrow()
            .as_ref()?
            .slottable_data
            .assigned_slot
            .as_ref()?
            .as_rooted();
        Some(assigned_slot)
    }

    pub(crate) fn set_assigned_slot(&self, assigned_slot: Option<&HTMLSlotElement>) {
        self.ensure_rare_data().slottable_data.assigned_slot = assigned_slot.map(Dom::from_ref);
    }

    pub(crate) fn manual_slot_assignment(&self) -> Option<DomRoot<HTMLSlotElement>> {
        let manually_assigned_slot = self
            .rare_data
            .borrow()
            .as_ref()?
            .slottable_data
            .manual_slot_assignment
            .as_ref()?
            .as_rooted();
        Some(manually_assigned_slot)
    }

    pub(crate) fn set_manual_slot_assignment(
        &self,
        manually_assigned_slot: Option<&HTMLSlotElement>,
    ) {
        self.ensure_rare_data()
            .slottable_data
            .manual_slot_assignment = manually_assigned_slot.map(Dom::from_ref);
    }

    /// Gets the parent of this node from the perspective of layout and style.
    ///
    /// If the node and its parent have a flat tree relationship, this returns:
    ///  - The node's assigned slot.
    ///  - The parent node's shadow host if it's a shadow root.
    ///  - Or the node's parent.
    ///
    /// The parent might not have a flat tree relationship with the node if
    ///  - It's a light tree child of a shadow host.
    ///  - It's fallback content for an assigned slot.
    pub(crate) fn parent_in_flat_tree(&self) -> FlatTreeParent {
        if let Some(assigned_slot) = self.assigned_slot() {
            return FlatTreeParent::Parent(DomRoot::upcast(assigned_slot));
        }

        let Some(parent) = self.GetParentNode() else {
            return FlatTreeParent::RootNode;
        };

        if let Some(shadow_root) = parent.downcast::<ShadowRoot>() {
            return FlatTreeParent::Parent(DomRoot::from_ref(shadow_root.Host().upcast::<Node>()));
        }

        if parent
            .downcast::<Element>()
            .is_some_and(|element| element.is_shadow_host())
        {
            return FlatTreeParent::NotInFlatTree;
        }

        if parent
            .downcast::<HTMLSlotElement>()
            .is_some_and(|slot| slot.has_assigned_nodes())
        {
            return FlatTreeParent::NotInFlatTree;
        }

        FlatTreeParent::Parent(parent)
    }

    pub(crate) fn inclusive_ancestors_in_flat_tree(
        &self,
    ) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator::new(Some(DomRoot::from_ref(self)), move |node| {
            match node.parent_in_flat_tree() {
                FlatTreeParent::Parent(parent) => Some(parent),
                FlatTreeParent::NotInFlatTree | FlatTreeParent::RootNode => None,
            }
        })
    }

    /// We are marking this as an implemented pseudo element.
    pub(crate) fn set_implemented_pseudo_element(&self, pseudo_element: PseudoElement) {
        // Implemented pseudo element should exist only in the UA shadow DOM.
        debug_assert!(self.is_in_ua_widget());
        debug_assert!(pseudo_element.is_element_backed());
        self.ensure_rare_data().implemented_pseudo_element = Some(pseudo_element);
    }

    pub(crate) fn implemented_pseudo_element(&self) -> Option<PseudoElement> {
        self.rare_data
            .borrow()
            .as_ref()
            .and_then(|rare_data| rare_data.implemented_pseudo_element)
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#editing-host-of>
    pub(crate) fn editing_host_of(&self) -> Option<DomRoot<Node>> {
        // > The editing host of node is null if node is neither editable nor an editing host;
        // > node itself, if node is an editing host;
        // > or the nearest ancestor of node that is an editing host, if node is editable.
        for ancestor in self.inclusive_ancestors(ShadowIncluding::No) {
            if ancestor.is_editing_host() {
                return Some(ancestor);
            }
            if ancestor
                .downcast::<HTMLElement>()
                .is_some_and(|el| el.ContentEditable().str() == "false")
            {
                return None;
            }
        }
        None
    }

    pub(crate) fn is_editable_or_editing_host(&self) -> bool {
        self.editing_host_of().is_some()
    }

    /// <https://html.spec.whatwg.org/multipage/#editing-host>
    pub(crate) fn is_editing_host(&self) -> bool {
        self.downcast::<HTMLElement>()
            .is_some_and(HTMLElement::is_editing_host)
    }

    /// <https://w3c.github.io/editing/docs/execCommand/#editable>
    pub(crate) fn is_editable(&self) -> bool {
        // > Something is editable if it is a node; it is not an editing host;
        if self.is_editing_host() {
            return false;
        }
        // > it does not have a contenteditable attribute set to the false state;
        let html_element = self.downcast::<HTMLElement>();
        if html_element.is_some_and(|el| el.ContentEditable().str() == "false") {
            return false;
        }
        // > its parent is an editing host or editable;
        let Some(parent) = self.GetParentNode() else {
            return false;
        };
        if !parent.is_editable_or_editing_host() {
            return false;
        }
        // > and either it is an HTML element, or it is an svg or math element, or it is not an Element and its parent is an HTML element.
        html_element.is_some() || (!self.is::<Element>() && parent.is::<HTMLElement>())
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
#[expect(unsafe_code)]
pub(crate) unsafe fn from_untrusted_node_address(candidate: UntrustedNodeAddress) -> DomRoot<Node> {
    let node = unsafe { Node::from_untrusted_node_address(candidate) };
    DomRoot::from_ref(node)
}

/// Specifies whether children must be recursively cloned or not.
#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub(crate) enum CloneChildrenFlag {
    CloneChildren,
    DoNotCloneChildren,
}

impl From<bool> for CloneChildrenFlag {
    fn from(boolean: bool) -> Self {
        if boolean {
            CloneChildrenFlag::CloneChildren
        } else {
            CloneChildrenFlag::DoNotCloneChildren
        }
    }
}

pub(super) fn as_uintptr<T>(t: &T) -> uintptr_t {
    t as *const T as uintptr_t
}

impl Node {
    pub(crate) fn reflect_node<N>(
        cx: &mut JSContext,
        node: Box<N>,
        document: &Document,
    ) -> DomRoot<N>
    where
        N: DerivedFrom<Node> + DomObject + DomObjectWrap<crate::DomTypeHolder>,
    {
        Self::reflect_node_with_proto(cx, node, document, None)
    }

    pub(crate) fn reflect_node_with_proto<N>(
        cx: &mut JSContext,
        node: Box<N>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<N>
    where
        N: DerivedFrom<Node> + DomObject + DomObjectWrap<crate::DomTypeHolder>,
    {
        let window = document.window();
        reflect_dom_object_with_proto_and_cx(node, window, proto, cx)
    }

    pub(crate) fn reflect_weak_referenceable_node_with_proto<N>(
        cx: &mut JSContext,
        node: Rc<N>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<N>
    where
        N: DerivedFrom<Node> + DomObject + WeakReferenceableDomObjectWrap<crate::DomTypeHolder>,
    {
        let window = document.window();
        reflect_weak_referenceable_dom_object_with_proto(cx, node, window, proto)
    }

    pub(crate) fn new_inherited(doc: &Document) -> Node {
        Node::new_(NodeFlags::empty(), Some(doc))
    }

    pub(crate) fn new_document_node() -> Node {
        Node::new_(
            NodeFlags::IS_IN_A_DOCUMENT_TREE | NodeFlags::IS_CONNECTED,
            None,
        )
    }

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
            children_count: Cell::new(0u32),
            flags: Cell::new(flags),
            inclusive_descendants_version: Cell::new(0),
            layout_data: Default::default(),
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-node-adopt>
    pub(crate) fn adopt(cx: &mut JSContext, node: &Node, document: &Document) {
        document.add_script_and_layout_blocker();

        // Step 1. Let oldDocument be node’s node document.
        let old_doc = node.owner_doc();
        old_doc.add_script_and_layout_blocker();

        // Step 2. If node’s parent is non-null, then remove node.
        node.remove_self(cx);

        // Step 3. If document is not oldDocument:
        if &*old_doc != document {
            // Step 3.1. For each inclusiveDescendant in node’s shadow-including inclusive descendants:
            for descendant in node.traverse_preorder_non_rooting(cx.no_gc(), ShadowIncluding::Yes) {
                // Step 3.1.1 Set inclusiveDescendant’s node document to document.
                descendant.set_owner_doc(document);

                // Step 3.1.2 If inclusiveDescendant is an element, then set the node document of each
                // attribute in inclusiveDescendant’s attribute list to document.
                if let Some(element) = descendant.downcast::<Element>() {
                    for attribute in element.attrs().borrow().iter() {
                        if let Some(attr) = attribute.as_attr() {
                            attr.upcast::<Node>().set_owner_doc(document);
                        }
                    }
                }
            }

            // Step 3.2 For each inclusiveDescendant in node’s shadow-including inclusive descendants
            // that is custom, enqueue a custom element callback reaction with inclusiveDescendant,
            // callback name "adoptedCallback", and « oldDocument, document ».
            let custom_element_reaction_stack = ScriptThread::custom_element_reaction_stack();
            for descendant in node
                .traverse_preorder(ShadowIncluding::Yes)
                .filter_map(|d| d.as_custom_element())
            {
                custom_element_reaction_stack.enqueue_callback_reaction(
                    cx,
                    &descendant,
                    CallbackReaction::Adopted(old_doc.clone(), DomRoot::from_ref(document)),
                    None,
                );
            }

            // Step 3.3 For each inclusiveDescendant in node’s shadow-including inclusive descendants,
            // in shadow-including tree order, run the adopting steps with inclusiveDescendant and oldDocument.
            for descendant in node.traverse_preorder(ShadowIncluding::Yes) {
                vtable_for(&descendant).adopting_steps(cx, &old_doc);
            }
        }

        old_doc.remove_script_and_layout_blocker(cx);
        document.remove_script_and_layout_blocker(cx);
    }

    /// <https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity>
    pub(crate) fn ensure_pre_insertion_validity(
        no_gc: &NoGC,
        node: &Node,
        parent: &Node,
        child: Option<&Node>,
    ) -> ErrorResult {
        // Step 1. If parent is not a Document, DocumentFragment, or Element node, then throw a "HierarchyRequestError" DOMException.
        match parent.type_id() {
            NodeTypeId::Document(_) | NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(..) => {
            },
            _ => {
                return Err(Error::HierarchyRequest(Some(
                    "Parent is not a Document, DocumentFragment, or Element node".to_owned(),
                )));
            },
        }

        // Step 2. If node is a host-including inclusive ancestor of parent, then throw a "HierarchyRequestError" DOMException.
        if node.is_host_including_inclusive_ancestor(parent) {
            return Err(Error::HierarchyRequest(Some(
                "Node is a host-including inclusive ancestor of parent".to_owned(),
            )));
        }

        // Step 3. If child is non-null and its parent is not parent, then throw a "NotFoundError" DOMException.
        if let Some(child) = child &&
            !parent.is_parent_of(child)
        {
            return Err(Error::NotFound(Some(
                "Child is non-null and its parent is not parent".to_owned(),
            )));
        }

        match node.type_id() {
            // Step 5. If either node is a Text node and parent is a document,
            // or node is a doctype and parent is not a document,
            // then throw a "HierarchyRequestError" DOMException.
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => {
                if parent.is::<Document>() {
                    return Err(Error::HierarchyRequest(Some(
                        "Node is a Text node and parent is a document".to_owned(),
                    )));
                }
            },
            NodeTypeId::DocumentType => {
                if !parent.is::<Document>() {
                    return Err(Error::HierarchyRequest(Some(
                        "Node is a doctype and parent is not a document".to_owned(),
                    )));
                }
            },
            NodeTypeId::DocumentFragment(_) |
            NodeTypeId::Element(_) |
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) |
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => (),
            // Step 4. If node is not a DocumentFragment, DocumentType, Element,
            // or CharacterData node, then throw a "HierarchyRequestError" DOMException.
            NodeTypeId::Document(_) | NodeTypeId::Attr => {
                return Err(Error::HierarchyRequest(Some(
                    "Node is not a DocumentFragment, DocumentType, Element, or CharacterData node"
                        .to_owned(),
                )));
            },
        }

        // Step 6. If parent is a document, and any of the statements below, switched on the interface node implements,
        // are true, then throw a "HierarchyRequestError" DOMException.
        if parent.is::<Document>() {
            match node.type_id() {
                NodeTypeId::DocumentFragment(_) => {
                    // Step 6."DocumentFragment". If node has more than one element child or has a Text node child.
                    if node.children_unrooted(no_gc).any(|c| c.is::<Text>()) {
                        return Err(Error::HierarchyRequest(None));
                    }
                    match node.child_elements_unrooted(no_gc).count() {
                        0 => (),
                        // Step 6."DocumentFragment". Otherwise, if node has one element child and either parent has an element child,
                        // child is a doctype, or child is non-null and a doctype is following child.
                        1 => {
                            if parent.child_elements_unrooted(no_gc).next().is_some() {
                                return Err(Error::HierarchyRequest(None));
                            }
                            if let Some(child) = child &&
                                child
                                    .inclusively_following_siblings_unrooted(no_gc)
                                    .any(|child| child.is_doctype())
                            {
                                return Err(Error::HierarchyRequest(None));
                            }
                        },
                        _ => return Err(Error::HierarchyRequest(None)),
                    }
                },
                NodeTypeId::Element(_) => {
                    // Step 6."Element". parent has an element child, child is a doctype, or child is non-null and a doctype is following child.
                    if parent.child_elements_unrooted(no_gc).next().is_some() {
                        return Err(Error::HierarchyRequest(Some(
                            "Parent has an element child".to_owned(),
                        )));
                    }
                    if let Some(child) = child &&
                        child
                            .inclusively_following_siblings_unrooted(no_gc)
                            .any(|following| following.is_doctype())
                    {
                        return Err(Error::HierarchyRequest(Some(
                                "Child is a doctype, or child is non-null and a doctype is following child".to_owned(),
                            )));
                    }
                },
                NodeTypeId::DocumentType => {
                    // Step 6."DocumentType". parent has a doctype child, child is non-null and an element is preceding child,
                    // or child is null and parent has an element child.
                    if parent.children_unrooted(no_gc).any(|c| c.is_doctype()) {
                        return Err(Error::HierarchyRequest(None));
                    }
                    match child {
                        Some(child) => {
                            if parent
                                .children_unrooted(no_gc)
                                .take_while(|c| **c != child)
                                .any(|c| c.is::<Element>())
                            {
                                return Err(Error::HierarchyRequest(None));
                            }
                        },
                        None => {
                            if parent.child_elements_unrooted(no_gc).next().is_some() {
                                return Err(Error::HierarchyRequest(None));
                            }
                        },
                    }
                },
                NodeTypeId::CharacterData(_) => (),
                // Because Document and Attr should already throw `HierarchyRequest`
                // error, both of them are unreachable here.
                NodeTypeId::Document(_) | NodeTypeId::Attr => unreachable!(),
            }
        }
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#concept-node-pre-insert>
    pub(crate) fn pre_insert(
        cx: &mut JSContext,
        node: &Node,
        parent: &Node,
        child: Option<&Node>,
    ) -> Fallible<DomRoot<Node>> {
        // Step 1. Ensure pre-insert validity of node into parent before child.
        Node::ensure_pre_insertion_validity(cx.no_gc(), node, parent, child)?;

        // Step 2. Let referenceChild be child.
        let reference_child_root;
        let reference_child = match child {
            // Step 3. If referenceChild is node, then set referenceChild to node’s next sibling.
            Some(child) if child == node => {
                reference_child_root = node.GetNextSibling();
                reference_child_root.as_deref()
            },
            _ => child,
        };

        // Step 4. Insert node into parent before referenceChild.
        Node::insert(
            cx,
            node,
            parent,
            reference_child,
            SuppressObserver::Unsuppressed,
        );

        // Step 5. Return node.
        Ok(DomRoot::from_ref(node))
    }

    /// <https://dom.spec.whatwg.org/#concept-node-insert>
    pub(crate) fn insert(
        cx: &mut JSContext,
        node: &Node,
        parent: &Node,
        child: Option<&Node>,
        suppress_observers: SuppressObserver,
    ) {
        debug_assert!(child.is_none_or(|child| Some(parent) == child.GetParentNode().as_deref()));

        // Step 1. Let nodes be node’s children, if node is a DocumentFragment node; otherwise « node ».
        rooted_vec!(let mut new_nodes);
        let new_nodes = if let NodeTypeId::DocumentFragment(_) = node.type_id() {
            new_nodes.extend(
                node.children_unrooted(cx.no_gc())
                    .map(|node| Dom::from_ref(&**node)),
            );
            new_nodes.r()
        } else {
            from_ref(&node)
        };

        // Step 2. Let count be nodes’s size.
        let count = new_nodes.len();

        // Step 3. If count is 0, then return.
        if count == 0 {
            return;
        }

        // Script and layout blockers must be added after any early return.
        // `node.owner_doc()` may change during the algorithm.
        let parent_document = parent.owner_doc();
        let from_document = node.owner_doc();
        from_document.add_script_and_layout_blocker();
        parent_document.add_script_and_layout_blocker();

        // Step 4. If node is a DocumentFragment node:
        if let NodeTypeId::DocumentFragment(_) = node.type_id() {
            // Step 4.1. Remove its children with the suppress observers flag set.
            for kid in new_nodes {
                Node::remove(cx, kid, node, SuppressObserver::Suppressed);
            }
            vtable_for(node).children_changed(cx, &ChildrenMutation::ReplaceAll);

            // Step 4.2. Queue a tree mutation record for node with « », nodes, null, and null.
            let mutation = LazyCell::new(|| Mutation::ChildList {
                added: None,
                removed: Some(new_nodes),
                prev: None,
                next: None,
            });
            MutationObserver::queue_a_mutation_record(cx, node, mutation);
        }

        // Step 5. If child is non-null:
        //     1. For each live range whose start node is parent and start offset is
        //        greater than child’s index, increase its start offset by count.
        //     2. For each live range whose end node is parent and end offset is
        //        greater than child’s index, increase its end offset by count.
        if let Some(child) = child &&
            let Some(parent_weak_ranges) = parent.weak_ranges_mut()
        {
            parent_weak_ranges.increase_above(parent, child.index(), count.try_into().unwrap());
        }

        // Step 6. Let previousSibling be child’s previous sibling or parent’s last child if child is null.
        let previous_sibling = match suppress_observers {
            SuppressObserver::Unsuppressed => match child {
                Some(child) => child.GetPreviousSibling(),
                None => parent.GetLastChild(),
            },
            SuppressObserver::Suppressed => None,
        };

        // Step 10. Let staticNodeList be a list of nodes, initially « ».
        let mut static_node_list: SmallVec<[_; 4]> = Default::default();

        let parent_shadow_root = parent.downcast::<Element>().and_then(Element::shadow_root);
        let parent_in_shadow_tree = parent.is_in_a_shadow_tree();
        let parent_as_slot = parent.downcast::<HTMLSlotElement>();

        // Step 7. For each node in nodes, in tree order:
        for kid in new_nodes {
            // Step 7.1. Adopt node into parent’s node document.
            Node::adopt(cx, kid, &parent.owner_document());

            // Step 7.2. If child is null, then append node to parent’s children.
            // Step 7.3. Otherwise, insert node into parent’s children before child’s index.
            parent.add_child(cx, kid, child);

            // Step 7.4 If parent is a shadow host whose shadow root’s slot assignment is "named"
            // and node is a slottable, then assign a slot for node.
            if let Some(ref shadow_root) = parent_shadow_root &&
                shadow_root.SlotAssignment() == SlotAssignmentMode::Named &&
                (kid.is::<Element>() || kid.is::<Text>())
            {
                rooted!(&in(cx) let slottable = Slottable(Dom::from_ref(kid)));
                slottable.assign_a_slot(cx);
            }

            // Step 7.5 If parent’s root is a shadow root, and parent is a slot whose assigned nodes
            // is the empty list, then run signal a slot change for parent.
            if parent_in_shadow_tree &&
                let Some(slot_element) = parent_as_slot &&
                !slot_element.has_assigned_nodes()
            {
                slot_element.signal_a_slot_change(cx);
            }

            // Step 7.6 Run assign slottables for a tree with node’s root.
            kid.GetRootNode(&GetRootNodeOptions::empty())
                .assign_slottables_for_a_tree(cx, ForceSlottableNodeReconciliation::Skip);

            // Step 7.7. For each shadow-including inclusive descendant inclusiveDescendant of node,
            // in shadow-including tree order:
            for descendant in kid.traverse_preorder(ShadowIncluding::Yes) {
                // Step 7.7.1. Run the insertion steps with inclusiveDescendant.
                // This is done in `parent.add_child()`.

                // From <https://github.com/whatwg/dom/issues/833>:
                // try_upgrade_element fires even for disconnected elements.
                if let Some(element) = DomRoot::downcast::<Element>(descendant.clone()) &&
                    !element.is_custom()
                {
                    try_upgrade_element(cx, &element);
                }

                // Step 7.7.2. If inclusiveDescendant is not connected, then continue.
                if !descendant.is_connected() {
                    continue;
                }

                // Step 7.7.3. If inclusiveDescendant is an element
                if let Some(element) = DomRoot::downcast::<Element>(descendant.clone()) {
                    // and inclusiveDescendant’s custom element registry is non-null:
                    if let Some(registry) = element.custom_element_registry() {
                        // Step 7.7.3.1. If inclusiveDescendant’s custom element
                        // registry’s is scoped is true, then append
                        // inclusiveDescendant’s node document to inclusiveDescendant’s
                        // custom element registry’s scoped document set.
                        if registry.is_scoped() {
                            registry.add_scoped_document(&element.owner_document());
                        }
                    }
                    // TODO: As per the spec, following steps should only be
                    // executed for non-null custom element registry. But, it
                    // causes some WPT tests to fail. Needs Investigation.
                    //
                    // Step 7.7.3.2. If inclusiveDescendant is custom, then enqueue
                    // a custom element callback reaction with inclusiveDescendant,
                    // callback name "connectedCallback", and « ».
                    if element.is_custom() {
                        ScriptThread::custom_element_reaction_stack().enqueue_callback_reaction(
                            cx,
                            &element,
                            CallbackReaction::Connected,
                            None,
                        );
                    }
                    // Step 7.7.3.3. Otherwise, try to upgrade inclusiveDescendant.
                    else {
                        try_upgrade_element(cx, &element);
                    }
                }
                // Step 7.7.4. Otherwise, if inclusiveDescendant is a shadow
                // root, inclusiveDescendant’s custom element registry is
                // non-null, and inclusiveDescendant’s custom element registry’s
                // is scoped is true, then append inclusiveDescendant’s node
                // document to inclusiveDescendant’s custom element registry’s
                // scoped document set.
                else if let Some(shadow_root) =
                    DomRoot::downcast::<ShadowRoot>(descendant.clone()) &&
                    let Some(custom_element_registry) = shadow_root.custom_element_registry() &&
                    custom_element_registry.is_scoped()
                {
                    custom_element_registry.add_scoped_document(shadow_root.owner_doc());
                }

                // Step 11.1 For each shadow-including inclusive descendant inclusiveDescendant of node,
                //           in shadow-including tree order, append inclusiveDescendant to staticNodeList.
                static_node_list.push(descendant.clone());
            }
        }

        if let SuppressObserver::Unsuppressed = suppress_observers {
            // Step 9. Run the children changed steps for parent.
            // TODO(xiaochengh): If we follow the spec and move it out of the if block, some WPT fail. Investigate.
            vtable_for(parent).children_changed(
                cx,
                &ChildrenMutation::insert(previous_sibling.as_deref(), child),
            );

            // Step 8. If suppress observers flag is unset, then queue a tree mutation record for parent
            // with nodes, « », previousSibling, and child.
            let mutation = LazyCell::new(|| Mutation::ChildList {
                added: Some(new_nodes),
                removed: None,
                prev: previous_sibling.as_deref(),
                next: child,
            });
            MutationObserver::queue_a_mutation_record(cx, parent, mutation);
        }

        // We use a delayed task for this step to work around an awkward interaction between
        // script/layout blockers, Node::replace_all, and the children_changed vtable method.
        // Any node with a post connection step that triggers layout (such as iframes) needs
        // to be marked as dirty before doing so. This is handled by Node's children_changed
        // callback, but when Node::insert is called as part of Node::replace_all then the
        // callback is suppressed until we return to Node::replace_all. To ensure the sequence:
        // 1) children_changed in Node::replace_all,
        // 2) post_connection_steps from Node::insert,
        // we use a delayed task that will run as soon as Node::insert removes its
        // script/layout blocker.
        parent_document.add_delayed_task(
            task!(PostConnectionSteps: |cx, static_node_list: SmallVec<[DomRoot<Node>; 4]>| {
                // Step 12. For each node of staticNodeList, if node is connected, then run the
                //          post-connection steps with node.
                //
                // Note: We only add the nodes to the static_node_list which are connected.
                for node in static_node_list {
                    vtable_for(&node).post_connection_steps(cx);
                }
            }),
        );

        parent_document.remove_script_and_layout_blocker(cx);
        from_document.remove_script_and_layout_blocker(cx);
    }

    /// <https://dom.spec.whatwg.org/#concept-node-replace-all>
    pub(crate) fn replace_all(cx: &mut JSContext, node: Option<&Node>, parent: &Node) {
        parent.owner_doc().add_script_and_layout_blocker();

        // Step 1. Let removedNodes be parent’s children.
        rooted_vec!(let removed_nodes <- parent.children().map(|child| DomRoot::as_traced(&child)));

        // Step 2. Let addedNodes be the empty set.
        // Step 3. If node is a DocumentFragment node, then set addedNodes to node’s children.
        // Step 4. Otherwise, if node is non-null, set addedNodes to « node ».
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

        // Step 5. Remove all parent’s children, in tree order, with suppressObservers set to true.
        for child in &*removed_nodes {
            Node::remove(cx, child, parent, SuppressObserver::Suppressed);
        }

        // Step 6. If node is non-null, then insert node into parent before null with suppressObservers set to true.
        if let Some(node) = node {
            Node::insert(cx, node, parent, None, SuppressObserver::Suppressed);
        }

        vtable_for(parent).children_changed(cx, &ChildrenMutation::ReplaceAll);

        // Step 7. If either addedNodes or removedNodes is not empty, then queue a tree mutation record
        // for parent with addedNodes, removedNodes, null, and null.
        if !removed_nodes.is_empty() || !added_nodes.is_empty() {
            let mutation = LazyCell::new(|| Mutation::ChildList {
                added: Some(added_nodes),
                removed: Some(removed_nodes.r()),
                prev: None,
                next: None,
            });
            MutationObserver::queue_a_mutation_record(cx, parent, mutation);
        }
        parent.owner_doc().remove_script_and_layout_blocker(cx);
    }

    /// <https://dom.spec.whatwg.org/multipage/#string-replace-all>
    pub(crate) fn string_replace_all(cx: &mut JSContext, string: DOMString, parent: &Node) {
        if string.is_empty() {
            Node::replace_all(cx, None, parent);
        } else {
            let text = Text::new(cx, string, &parent.owner_document());
            Node::replace_all(cx, Some(text.upcast::<Node>()), parent);
        };
    }

    /// <https://dom.spec.whatwg.org/#concept-node-pre-remove>
    pub(super) fn pre_remove(
        cx: &mut JSContext,
        child: &Node,
        parent: &Node,
    ) -> Fallible<DomRoot<Node>> {
        // Step 1.
        match child.GetParentNode() {
            Some(ref node) if &**node != parent => return Err(Error::NotFound(None)),
            None => return Err(Error::NotFound(None)),
            _ => (),
        }

        // Step 2.
        Node::remove(cx, child, parent, SuppressObserver::Unsuppressed);

        // Step 3.
        Ok(DomRoot::from_ref(child))
    }

    /// <https://dom.spec.whatwg.org/#concept-node-remove>
    pub(super) fn remove(
        cx: &mut JSContext,
        node: &Node,
        parent: &Node,
        suppress_observers: SuppressObserver,
    ) {
        parent.owner_doc().add_script_and_layout_blocker();

        // Step 1. Let parent be node’s parent.
        // Step 2. Assert: parent is non-null.
        // NOTE: We get parent as an argument instead
        assert!(
            node.GetParentNode()
                .is_some_and(|node_parent| &*node_parent == parent)
        );

        // Step 3. Run the live range pre-remove steps.
        // https://dom.spec.whatwg.org/#live-range-pre-remove-steps
        let cached_index = Node::live_range_pre_remove_steps(node, parent);

        // TODO: Step 4. Pre-removing steps for node iterators

        // Step 5.
        let old_previous_sibling = node.GetPreviousSibling();

        // Step 6.
        let old_next_sibling = node.GetNextSibling();

        // Step 7. Remove node from its parent's children.
        // Step 11-14. Run removing steps and enqueue disconnected custom element reactions for the subtree.
        parent.remove_child(cx, node, cached_index);

        // Step 8. If node is assigned, then run assign slottables for node’s assigned slot.
        if let Some(slot) = node.assigned_slot() {
            slot.assign_slottables(cx);
        }

        // Step 9. If parent’s root is a shadow root, and parent is a slot whose assigned nodes is the empty list,
        // then run signal a slot change for parent.
        if parent.is_in_a_shadow_tree() &&
            let Some(slot_element) = parent.downcast::<HTMLSlotElement>() &&
            !slot_element.has_assigned_nodes()
        {
            slot_element.signal_a_slot_change(cx);
        }

        // Step 10. If node has an inclusive descendant that is a slot:
        let has_slot_descendant = node
            .traverse_preorder_non_rooting(cx.no_gc(), ShadowIncluding::No)
            .any(|elem| elem.is::<HTMLSlotElement>());
        if has_slot_descendant {
            // Step 10.1 Run assign slottables for a tree with parent’s root.
            parent
                .GetRootNode(&GetRootNodeOptions::empty())
                .assign_slottables_for_a_tree(cx, ForceSlottableNodeReconciliation::Skip);

            // Step 10.2 Run assign slottables for a tree with node.
            node.assign_slottables_for_a_tree(cx, ForceSlottableNodeReconciliation::Force);
        }

        // TODO: Step 15. transient registered observers

        // Step 16.
        if let SuppressObserver::Unsuppressed = suppress_observers {
            vtable_for(parent).children_changed(
                cx,
                &ChildrenMutation::replace(
                    old_previous_sibling.as_deref(),
                    &Some(node),
                    old_next_sibling.as_deref(),
                ),
            );

            let removed = [node];
            let mutation = LazyCell::new(|| Mutation::ChildList {
                added: None,
                removed: Some(&removed),
                prev: old_previous_sibling.as_deref(),
                next: old_next_sibling.as_deref(),
            });
            MutationObserver::queue_a_mutation_record(cx, parent, mutation);
        }
        parent.owner_doc().remove_script_and_layout_blocker(cx);
    }

    /// <https://dom.spec.whatwg.org/#live-range-pre-remove-steps>
    fn live_range_pre_remove_steps(node: &Node, parent: &Node) -> Option<u32> {
        if parent.weak_ranges_is_empty() {
            return None;
        }

        // Step 1. Let parent be node’s parent.
        // Step 2. Assert: parent is not null.
        // NOTE: We already have the parent.

        // Step 3. Let index be node’s index.
        let index = node.index();

        // Steps 4-5 are handled in Node::unbind_from_tree.

        // Step 6. For each live range whose start node is parent and start offset is greater than index,
        // decrease its start offset by 1.
        // Step 7. For each live range whose end node is parent and end offset is greater than index,
        // decrease its end offset by 1.
        if let Some(parent_weak_ranges) = parent.weak_ranges_mut() {
            parent_weak_ranges.decrease_above(parent, index, 1);
        }

        // Parent had ranges, we needed the index, let's keep track of
        // it to avoid computing it for other ranges when calling
        // unbind_from_tree recursively.
        Some(index)
    }

    /// <https://dom.spec.whatwg.org/#concept-node-clone>
    pub(crate) fn clone(
        cx: &mut JSContext,
        node: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
        registry: Option<DomRoot<CustomElementRegistry>>,
    ) -> DomRoot<Node> {
        // Step 1. If document is not given, let document be node’s node document.
        let document = match maybe_doc {
            Some(doc) => DomRoot::from_ref(doc),
            None => node.owner_doc(),
        };

        // Step 2. / Step 3.
        // XXXabinader: clone() for each node as trait?
        let copy: DomRoot<Node> = match node.type_id() {
            NodeTypeId::DocumentType => {
                let doctype = node.downcast::<DocumentType>().unwrap();
                let doctype = DocumentType::new(
                    cx,
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
                    cx,
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
                let doc_fragment = DocumentFragment::new(cx, &document);
                DomRoot::upcast::<Node>(doc_fragment)
            },
            NodeTypeId::CharacterData(_) => {
                let cdata = node.downcast::<CharacterData>().unwrap();
                cdata.clone_with_data(cx, cdata.Data(), &document)
            },
            NodeTypeId::Document(_) => {
                // Step 1. Set copy’s encoding, content type, URL, origin, type, mode,
                // and allow declarative shadow roots, to those of node.
                let document = node.downcast::<Document>().unwrap();
                let is_html_doc = if document.is_html_document() {
                    IsHTMLDocument::HTMLDocument
                } else {
                    IsHTMLDocument::NonHTMLDocument
                };
                let window = document.window();
                let loader = DocumentLoader::new(&document.loader());
                let document = Document::new(
                    cx,
                    window,
                    HasBrowsingContext::No,
                    Some(document.url()),
                    None,
                    // https://github.com/whatwg/dom/issues/378
                    document.origin().clone(),
                    is_html_doc,
                    None,
                    None,
                    DocumentActivity::Inactive,
                    DocumentSource::NotFromParser,
                    loader,
                    None,
                    document.status_code(),
                    Default::default(),
                    false,
                    document.allow_declarative_shadow_roots(),
                    Some(document.insecure_requests_policy()),
                    document.has_trustworthy_ancestor_or_current_origin(),
                    document.custom_element_reaction_stack(),
                    document.creation_sandboxing_flag_set(),
                    document.pipeline_id(),
                    document.image_cache(),
                );
                // Step 2. If node’s custom element registry’s is scoped is true,
                // then set copy’s custom element registry to node’s custom element registry.
                // TODO
                DomRoot::upcast::<Node>(document)
            },
            // Step 2. If node is an element:
            NodeTypeId::Element(..) => {
                let element = node.downcast::<Element>().unwrap();
                // Step 2.1. Let registry be node’s custom element registry.
                // Step 2.2. If registry is null, then set registry to fallbackRegistry.
                let registry = element.custom_element_registry().or(registry);
                // Step 2.3. If registry is a global custom element registry, then
                // set registry to document’s effective global custom element registry.
                let registry =
                    if CustomElementRegistry::is_a_global_element_registry(registry.as_deref()) {
                        document.custom_element_registry()
                    } else {
                        registry
                    };
                // Step 2.4. Set copy to the result of creating an element,
                // given document, node’s local name, node’s namespace,
                // node’s namespace prefix, node’s is value, false, and registry.
                let name = QualName {
                    prefix: element.prefix().as_ref().map(|p| Prefix::from(&**p)),
                    ns: element.namespace().clone(),
                    local: element.local_name().clone(),
                };
                let element = Element::create(
                    cx,
                    name,
                    element.get_is(),
                    &document,
                    ElementCreator::ScriptCreated,
                    CustomElementCreationMode::Asynchronous,
                    None,
                );
                // TODO: Move this into `Element::create`
                element.set_custom_element_registry(registry.as_deref());
                DomRoot::upcast::<Node>(element)
            },
        };

        // Step 4. Set copy’s node document and document to copy, if copy is a document,
        // and set copy’s node document to document otherwise.
        let document = match copy.downcast::<Document>() {
            Some(doc) => DomRoot::from_ref(doc),
            None => DomRoot::from_ref(&*document),
        };
        assert!(copy.owner_doc() == document);

        // TODO: The spec tells us to do this in step 3.
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

                // Step 2.5. For each attribute of node’s attribute list:
                node_elem.copy_all_attributes_to_other_element(cx, copy_elem);
            },
            _ => (),
        }

        // Step 5: Run any cloning steps defined for node in other applicable specifications and pass copy,
        // node, document, and the clone children flag if set, as parameters.
        vtable_for(node).cloning_steps(cx, &copy, maybe_doc, clone_children);

        // Step 6. If the clone children flag is set, then for each child child of node, in tree order: append the
        // result of cloning child with document and the clone children flag set, to copy.
        if clone_children == CloneChildrenFlag::CloneChildren {
            for child in node.children() {
                let child_copy = Node::clone(cx, &child, Some(&document), clone_children, None);
                let _inserted_node = Node::pre_insert(cx, &child_copy, &copy, None);
            }
        }

        // Step 7. If node is a shadow host whose shadow root’s clonable is true:
        // NOTE: Only elements can be shadow hosts
        if matches!(node.type_id(), NodeTypeId::Element(_)) {
            let node_elem = node.downcast::<Element>().unwrap();
            let copy_elem = copy.downcast::<Element>().unwrap();

            if let Some(shadow_root) = node_elem.shadow_root().filter(|r| r.Clonable()) {
                // Step 7.1 Assert: copy is not a shadow host.
                assert!(!copy_elem.is_shadow_host());

                // Step 7.2 Run attach a shadow root with copy, node’s shadow root’s mode, true,
                // node’s shadow root’s serializable, node’s shadow root’s delegates focus,
                // and node’s shadow root’s slot assignment.
                let copy_shadow_root =
                    copy_elem.attach_shadow(
                        cx,
                        IsUserAgentWidget::No,
                        shadow_root.Mode(),
                        shadow_root.Clonable(),
                        shadow_root.Serializable(),
                        shadow_root.DelegatesFocus(),
                        shadow_root.SlotAssignment(),
                    )
                    .expect("placement of attached shadow root must be valid, as this is a copy of an existing one");

                // Step 7.3 Set copy’s shadow root’s declarative to node’s shadow root’s declarative.
                copy_shadow_root.set_declarative(shadow_root.is_declarative());

                // Step 7.4 For each child child of node’s shadow root, in tree order: append the result of
                // cloning child with document and the clone children flag set, to copy’s shadow root.
                for child in shadow_root.upcast::<Node>().children() {
                    let child_copy = Node::clone(
                        cx,
                        &child,
                        Some(&document),
                        CloneChildrenFlag::CloneChildren,
                        None,
                    );

                    // TODO: Should we handle the error case here and in step 6?
                    let _inserted_node =
                        Node::pre_insert(cx, &child_copy, copy_shadow_root.upcast::<Node>(), None);
                }
            }
        }

        // Step 8. Return copy.
        copy
    }

    /// <https://html.spec.whatwg.org/multipage/#child-text-content>
    pub(crate) fn child_text_content(&self) -> DOMString {
        Node::collect_text_contents(self.children())
    }

    /// <https://html.spec.whatwg.org/multipage/#descendant-text-content>
    pub(crate) fn descendant_text_content(&self) -> DOMString {
        Node::collect_text_contents(self.traverse_preorder(ShadowIncluding::No))
    }

    pub(crate) fn collect_text_contents<T: Iterator<Item = DomRoot<Node>>>(
        iterator: T,
    ) -> DOMString {
        let mut content = String::new();
        for node in iterator {
            if let Some(text) = node.downcast::<Text>() {
                content.push_str(&text.upcast::<CharacterData>().data());
            }
        }
        DOMString::from(content)
    }

    /// <https://dom.spec.whatwg.org/#string-replace-all>
    pub(crate) fn set_text_content_for_element(
        &self,
        cx: &mut JSContext,
        value: Option<DOMString>,
    ) {
        // This should only be called for elements and document fragments when setting the
        // text content: https://dom.spec.whatwg.org/#set-text-content
        assert!(matches!(
            self.type_id(),
            NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(..)
        ));
        let value = value.unwrap_or_default();
        let node = if value.is_empty() {
            // Step 1. Let node be null.
            None
        } else {
            // Step 2. If string is not the empty string, then set node to
            // a new Text node whose data is string and node document is parent’s node document.
            Some(DomRoot::upcast(self.owner_doc().CreateTextNode(cx, value)))
        };

        // Step 3. Replace all with node within parent.
        Self::replace_all(cx, node.as_deref(), self);
    }

    pub(crate) fn namespace_to_string(namespace: Namespace) -> Option<DOMString> {
        match namespace {
            ns!() => None,
            // FIXME(ajeffrey): convert directly from Namespace to DOMString
            _ => Some(DOMString::from(&*namespace)),
        }
    }

    /// <https://dom.spec.whatwg.org/#locate-a-namespace>
    pub(crate) fn locate_namespace(node: &Node, prefix: Option<DOMString>) -> Namespace {
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

    /// If the given untrusted node address represents a valid DOM node in the given runtime,
    /// returns it.
    ///
    /// # Safety
    ///
    /// Callers should ensure they pass an UntrustedNodeAddress that points to a valid [`JSObject`]
    /// in memory that represents a [`Node`].
    #[expect(unsafe_code)]
    pub(crate) unsafe fn from_untrusted_node_address(
        candidate: UntrustedNodeAddress,
    ) -> &'static Self {
        // https://github.com/servo/servo/issues/6383
        let candidate = candidate.0 as usize;
        let object = candidate as *mut JSObject;
        if object.is_null() {
            panic!("Attempted to create a `Node` from an invalid pointer!")
        }

        unsafe { &*(conversions::private_from_object(object) as *const Self) }
    }

    pub(crate) fn html_serialize(
        &self,
        cx: &mut JSContext,
        traversal_scope: html_serialize::TraversalScope,
        serialize_shadow_roots: bool,
        shadow_roots: Vec<DomRoot<ShadowRoot>>,
    ) -> DOMString {
        let mut writer = vec![];
        let mut serializer = HtmlSerializer::new(
            &mut writer,
            html_serialize::SerializeOpts {
                traversal_scope: traversal_scope.clone(),
                ..Default::default()
            },
        );

        serialize_html_fragment(
            cx,
            self,
            &mut serializer,
            traversal_scope,
            serialize_shadow_roots,
            shadow_roots,
        )
        .expect("Serializing node failed");

        // FIXME(ajeffrey): Directly convert UTF8 to DOMString
        DOMString::from(String::from_utf8(writer).unwrap())
    }

    /// <https://w3c.github.io/DOM-Parsing/#dfn-xml-serialization>
    pub(crate) fn xml_serialize(
        &self,
        traversal_scope: xml_serialize::TraversalScope,
    ) -> Fallible<DOMString> {
        let mut writer = vec![];
        xml_serialize::serialize(
            &mut writer,
            &HtmlSerialize::new(self),
            xml_serialize::SerializeOpts { traversal_scope },
        )
        .map_err(|error| {
            error!("Cannot serialize node: {error}");
            Error::InvalidState(None)
        })?;

        // FIXME(ajeffrey): Directly convert UTF8 to DOMString
        let string = DOMString::from(String::from_utf8(writer).map_err(|error| {
            error!("Cannot serialize node: {error}");
            Error::InvalidState(None)
        })?);

        Ok(string)
    }

    /// <https://html.spec.whatwg.org/multipage/#fragment-serializing-algorithm-steps>
    pub(crate) fn fragment_serialization_algorithm(
        &self,
        cx: &mut JSContext,
        require_well_formed: bool,
    ) -> Fallible<DOMString> {
        // Step 1. Let context document be node's node document.
        let context_document = self.owner_document();

        // Step 2. If context document is an HTML document, return the result of HTML fragment serialization algorithm
        // with node, false, and « ».
        if context_document.is_html_document() {
            return Ok(self.html_serialize(
                cx,
                html_serialize::TraversalScope::ChildrenOnly(None),
                false,
                vec![],
            ));
        }

        // Step 3. Return the XML serialization of node given require well-formed.
        // TODO: xml5ever doesn't seem to want require_well_formed
        let _ = require_well_formed;
        self.xml_serialize(xml_serialize::TraversalScope::ChildrenOnly(None))
    }

    pub(crate) fn get_next_sibling_unrooted<'a>(
        &self,
        no_gc: &'a NoGC,
    ) -> Option<UnrootedDom<'a, Node>> {
        self.next_sibling.get_unrooted(no_gc)
    }

    pub(crate) fn get_previous_sibling_unrooted<'a>(
        &self,
        no_gc: &'a NoGC,
    ) -> Option<UnrootedDom<'a, Node>> {
        self.prev_sibling.get_unrooted(no_gc)
    }

    pub(crate) fn get_first_child_unrooted<'a>(
        &self,
        no_gc: &'a NoGC,
    ) -> Option<UnrootedDom<'a, Node>> {
        self.first_child.get_unrooted(no_gc)
    }

    fn get_last_child_unrooted<'b>(&self, no_gc: &'b NoGC) -> Option<UnrootedDom<'b, Node>> {
        self.last_child.get_unrooted(no_gc)
    }

    pub(crate) fn get_parent_node_unrooted<'a>(
        &self,
        no_gc: &'a NoGC,
    ) -> Option<UnrootedDom<'a, Node>> {
        self.parent_node.get_unrooted(no_gc)
    }

    /// Compares `other` with `self` in [tree order](https://dom.spec.whatwg.org/#concept-tree-order).
    pub(crate) fn compare_dom_tree_position(
        &self,
        other: &Node,
        common_ancestor: &Node,
        shadow_including: ShadowIncluding,
    ) -> Ordering {
        debug_assert!(
            self.inclusive_ancestors(shadow_including)
                .any(|ancestor| &*ancestor == common_ancestor)
        );
        debug_assert!(
            other
                .inclusive_ancestors(shadow_including)
                .any(|ancestor| &*ancestor == common_ancestor)
        );

        if self == other {
            return Ordering::Equal;
        }

        if self == common_ancestor {
            return Ordering::Less;
        }
        if other == common_ancestor {
            return Ordering::Greater;
        }

        let my_ancestors: Vec<_> = self
            .inclusive_ancestors(shadow_including)
            .take_while(|ancestor| &**ancestor != common_ancestor)
            .collect();
        let other_ancestors: Vec<_> = other
            .inclusive_ancestors(shadow_including)
            .take_while(|ancestor| &**ancestor != common_ancestor)
            .collect();

        // Consume any ancestors that are shared between a and b
        let mut i = my_ancestors.len() - 1;
        let mut j = other_ancestors.len() - 1;

        while my_ancestors[i] == other_ancestors[j] {
            if i == 0 {
                // self is an ancestor of other
                debug_assert_ne!(j, 0, "Equal inclusive ancestors but nodes are not equal?");
                return Ordering::Less;
            }
            if j == 0 {
                // other is an ancestor of self
                return Ordering::Greater;
            }

            i -= 1;
            j -= 1;
        }

        // Now a_ancestors[i] and b_ancestors[j] have a common parent, but are not themselves equal
        // => They are siblings.
        if my_ancestors[i]
            .preceding_siblings()
            .any(|sibling| sibling == other_ancestors[j])
        {
            // other or an ancestor is a preceding sibling of self or one of its ancestors.
            Ordering::Greater
        } else {
            // self or an ancestor is a preceding sibling of other or one of its ancestors.
            debug_assert!(
                other_ancestors[j]
                    .preceding_siblings()
                    .any(|sibling| sibling == my_ancestors[i])
            );
            Ordering::Less
        }
    }
}

impl NodeMethods<crate::DomTypeHolder> for Node {
    /// <https://dom.spec.whatwg.org/#dom-node-nodetype>
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

    /// <https://dom.spec.whatwg.org/#dom-node-nodename>
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

    /// <https://dom.spec.whatwg.org/#dom-node-baseuri>
    fn BaseURI(&self) -> USVString {
        USVString(String::from(self.owner_doc().base_url().as_str()))
    }

    /// <https://dom.spec.whatwg.org/#dom-node-isconnected>
    fn IsConnected(&self) -> bool {
        self.is_connected()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-ownerdocument>
    fn GetOwnerDocument(&self) -> Option<DomRoot<Document>> {
        match self.type_id() {
            NodeTypeId::Document(_) => None,
            _ => Some(self.owner_doc()),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-getrootnode>
    fn GetRootNode(&self, options: &GetRootNodeOptions) -> DomRoot<Node> {
        if !options.composed &&
            let Some(shadow_root) = self.containing_shadow_root()
        {
            return DomRoot::upcast(shadow_root);
        }

        if self.is_connected() {
            DomRoot::from_ref(self.owner_doc().upcast::<Node>())
        } else {
            self.inclusive_ancestors(ShadowIncluding::Yes)
                .last()
                .unwrap()
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-parentnode>
    fn GetParentNode(&self) -> Option<DomRoot<Node>> {
        self.parent_node().get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-parentelement>
    fn GetParentElement(&self) -> Option<DomRoot<Element>> {
        self.GetParentNode().and_then(DomRoot::downcast)
    }

    /// <https://dom.spec.whatwg.org/#dom-node-haschildnodes>
    fn HasChildNodes(&self) -> bool {
        self.first_child().get().is_some()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-childnodes>
    fn ChildNodes(&self, cx: &mut JSContext) -> DomRoot<NodeList> {
        if let Some(list) = self.ensure_rare_data().child_list.get() {
            return list;
        }

        let doc = self.owner_doc();
        let window = doc.window();
        let list = NodeList::new_child_list(cx, window, self);
        self.ensure_rare_data().child_list.set(Some(&list));
        list
    }

    /// <https://dom.spec.whatwg.org/#dom-node-firstchild>
    fn GetFirstChild(&self) -> Option<DomRoot<Node>> {
        self.first_child().get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-lastchild>
    fn GetLastChild(&self) -> Option<DomRoot<Node>> {
        self.last_child().get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-previoussibling>
    fn GetPreviousSibling(&self) -> Option<DomRoot<Node>> {
        self.prev_sibling().get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-nextsibling>
    fn GetNextSibling(&self) -> Option<DomRoot<Node>> {
        self.next_sibling().get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-nodevalue>
    fn GetNodeValue(&self) -> Option<DOMString> {
        match self.type_id() {
            NodeTypeId::Attr => Some(self.downcast::<Attr>().unwrap().Value()),
            NodeTypeId::CharacterData(_) => {
                self.downcast::<CharacterData>().map(CharacterData::Data)
            },
            _ => None,
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-nodevalue>
    fn SetNodeValue(&self, cx: &mut JSContext, val: Option<DOMString>) -> Fallible<()> {
        match self.type_id() {
            NodeTypeId::Attr => {
                let attr = self.downcast::<Attr>().unwrap();
                attr.SetValue(cx, val.unwrap_or_default())?;
            },
            NodeTypeId::CharacterData(_) => {
                let character_data = self.downcast::<CharacterData>().unwrap();
                character_data.SetData(cx, val.unwrap_or_default());
            },
            _ => {},
        };
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-node-textcontent>
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

    /// <https://dom.spec.whatwg.org/#set-text-content>
    fn SetTextContent(&self, cx: &mut JSContext, value: Option<DOMString>) -> Fallible<()> {
        match self.type_id() {
            NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(..) => {
                self.set_text_content_for_element(cx, value);
            },
            NodeTypeId::Attr => {
                let attr = self.downcast::<Attr>().unwrap();
                attr.SetValue(cx, value.unwrap_or_default())?;
            },
            NodeTypeId::CharacterData(..) => {
                let characterdata = self.downcast::<CharacterData>().unwrap();
                characterdata.SetData(cx, value.unwrap_or_default());
            },
            NodeTypeId::DocumentType | NodeTypeId::Document(_) => {},
        };
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-node-insertbefore>
    fn InsertBefore(
        &self,
        cx: &mut JSContext,
        node: &Node,
        child: Option<&Node>,
    ) -> Fallible<DomRoot<Node>> {
        Node::pre_insert(cx, node, self, child)
    }

    /// <https://dom.spec.whatwg.org/#dom-node-appendchild>
    fn AppendChild(&self, cx: &mut JSContext, node: &Node) -> Fallible<DomRoot<Node>> {
        Node::pre_insert(cx, node, self, None)
    }

    /// <https://dom.spec.whatwg.org/#concept-node-replace>
    fn ReplaceChild(
        &self,
        cx: &mut JSContext,
        node: &Node,
        child: &Node,
    ) -> Fallible<DomRoot<Node>> {
        // Step 1. If parent is not a Document, DocumentFragment, or Element node,
        // then throw a "HierarchyRequestError" DOMException.
        match self.type_id() {
            NodeTypeId::Document(_) | NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(..) => {
            },
            _ => return Err(Error::HierarchyRequest(None)),
        }

        // Step 2. If node is a host-including inclusive ancestor of parent,
        // then throw a "HierarchyRequestError" DOMException.
        if node.is_inclusive_ancestor_of(self) {
            return Err(Error::HierarchyRequest(None));
        }

        // Step 3. If child’s parent is not parent, then throw a "NotFoundError" DOMException.
        if !self.is_parent_of(child) {
            return Err(Error::NotFound(None));
        }

        // Step 4. If node is not a DocumentFragment, DocumentType, Element, or CharacterData node,
        // then throw a "HierarchyRequestError" DOMException.
        // Step 5. If either node is a Text node and parent is a document,
        // or node is a doctype and parent is not a document, then throw a "HierarchyRequestError" DOMException.
        match node.type_id() {
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) if self.is::<Document>() => {
                return Err(Error::HierarchyRequest(None));
            },
            NodeTypeId::DocumentType if !self.is::<Document>() => {
                return Err(Error::HierarchyRequest(None));
            },
            NodeTypeId::Document(_) | NodeTypeId::Attr => {
                return Err(Error::HierarchyRequest(None));
            },
            _ => (),
        }

        // Step 6. If parent is a document, and any of the statements below, switched on the interface node implements,
        // are true, then throw a "HierarchyRequestError" DOMException.
        if self.is::<Document>() {
            match node.type_id() {
                // Step 6.1
                NodeTypeId::DocumentFragment(_) => {
                    // Step 6.1.1(b)
                    if node.children_unrooted(cx.no_gc()).any(|c| c.is::<Text>()) {
                        return Err(Error::HierarchyRequest(None));
                    }
                    match node.child_elements_unrooted(cx.no_gc()).count() {
                        0 => (),
                        // Step 6.1.2
                        1 => {
                            if self
                                .child_elements_unrooted(cx.no_gc())
                                .any(|c| c.upcast::<Node>() != child)
                            {
                                return Err(Error::HierarchyRequest(None));
                            }
                            if child.following_siblings().any(|child| child.is_doctype()) {
                                return Err(Error::HierarchyRequest(None));
                            }
                        },
                        // Step 6.1.1(a)
                        _ => return Err(Error::HierarchyRequest(None)),
                    }
                },
                // Step 6.2
                NodeTypeId::Element(..) => {
                    if self
                        .child_elements_unrooted(cx.no_gc())
                        .any(|c| c.upcast::<Node>() != child)
                    {
                        return Err(Error::HierarchyRequest(None));
                    }
                    if child.following_siblings().any(|child| child.is_doctype()) {
                        return Err(Error::HierarchyRequest(None));
                    }
                },
                // Step 6.3
                NodeTypeId::DocumentType => {
                    if self
                        .children_unrooted(cx.no_gc())
                        .any(|c| c.is_doctype() && *c != child)
                    {
                        return Err(Error::HierarchyRequest(None));
                    }
                    if self
                        .children_unrooted(cx.no_gc())
                        .take_while(|c| **c != child)
                        .any(|c| c.is::<Element>())
                    {
                        return Err(Error::HierarchyRequest(None));
                    }
                },
                NodeTypeId::CharacterData(..) => (),
                // Because Document and Attr should already throw `HierarchyRequest`
                // error, both of them are unreachable here.
                NodeTypeId::Document(_) => unreachable!(),
                NodeTypeId::Attr => unreachable!(),
            }
        }

        // Step 7. Let referenceChild be child’s next sibling.
        // Step 8. If referenceChild is node, then set referenceChild to node’s next sibling.
        let child_next_sibling = child.GetNextSibling();
        let node_next_sibling = node.GetNextSibling();
        let reference_child = if child_next_sibling.as_deref() == Some(node) {
            node_next_sibling.as_deref()
        } else {
            child_next_sibling.as_deref()
        };

        // Step 9. Let previousSibling be child’s previous sibling.
        let previous_sibling = child.GetPreviousSibling();

        // NOTE: All existing browsers assume that adoption is performed here, which does not follow the DOM spec.
        // However, if we follow the spec and delay adoption to inside `Node::insert()`, then the mutation records will
        // be different, and we will fail WPT dom/nodes/MutationObserver-childList.html.
        let document = self.owner_document();
        Node::adopt(cx, node, &document);

        // Step 10. Let removedNodes be the empty set.
        // Step 11. If child’s parent is non-null:
        //     1. Set removedNodes to « child ».
        //     2. Remove child with the suppress observers flag set.
        let removed_child = if node != child {
            // Step 11.
            Node::remove(cx, child, self, SuppressObserver::Suppressed);
            Some(child)
        } else {
            None
        };

        // Step 12. Let nodes be node’s children if node is a DocumentFragment node; otherwise « node ».
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

        // Step 13. Insert node into parent before referenceChild with the suppress observers flag set.
        Node::insert(
            cx,
            node,
            self,
            reference_child,
            SuppressObserver::Suppressed,
        );

        vtable_for(self).children_changed(
            cx,
            &ChildrenMutation::replace(
                previous_sibling.as_deref(),
                &removed_child,
                reference_child,
            ),
        );

        // Step 14. Queue a tree mutation record for parent with nodes, removedNodes,
        // previousSibling, and referenceChild.
        let removed = removed_child.map(|r| [r]);
        let mutation = LazyCell::new(|| Mutation::ChildList {
            added: Some(nodes),
            removed: removed.as_ref().map(|r| &r[..]),
            prev: previous_sibling.as_deref(),
            next: reference_child,
        });

        MutationObserver::queue_a_mutation_record(cx, self, mutation);

        // Step 15. Return child.
        Ok(DomRoot::from_ref(child))
    }

    /// <https://dom.spec.whatwg.org/#dom-node-removechild>
    fn RemoveChild(&self, cx: &mut JSContext, node: &Node) -> Fallible<DomRoot<Node>> {
        Node::pre_remove(cx, node, self)
    }

    /// <https://dom.spec.whatwg.org/#dom-node-normalize>
    fn Normalize(&self, cx: &mut JSContext) {
        let mut children = self.children().enumerate().peekable();
        while let Some((_, node)) = children.next() {
            if let Some(text) = node.downcast::<Text>() {
                if text.is::<CDATASection>() {
                    continue;
                }
                let cdata = text.upcast::<CharacterData>();
                let mut length = cdata.Length();
                if length == 0 {
                    Node::remove(cx, &node, self, SuppressObserver::Unsuppressed);
                    continue;
                }
                while children.peek().is_some_and(|(_, sibling)| {
                    sibling.is::<Text>() && !sibling.is::<CDATASection>()
                }) {
                    let (index, sibling) = children.next().unwrap();
                    if let Some(sibling_weak_ranges) = sibling.weak_ranges_mut() {
                        sibling_weak_ranges
                            .drain_to_preceding_text_sibling(&sibling, &node, length);
                    }
                    if let Some(weak_ranges) = self.weak_ranges_mut() {
                        weak_ranges.move_to_text_child_at(self, index as u32, &node, length);
                    }
                    let sibling_cdata = sibling.downcast::<CharacterData>().unwrap();
                    length += sibling_cdata.Length();
                    cdata.append_data(cx, &sibling_cdata.data());
                    Node::remove(cx, &sibling, self, SuppressObserver::Unsuppressed);
                }
            } else {
                node.Normalize(cx);
            }
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-clonenode>
    fn CloneNode(&self, cx: &mut JSContext, subtree: bool) -> Fallible<DomRoot<Node>> {
        // Step 1. If this is a shadow root, then throw a "NotSupportedError" DOMException.
        if self.is::<ShadowRoot>() {
            return Err(Error::NotSupported(None));
        }

        // Step 2. Return the result of cloning a node given this with subtree set to subtree.
        let result = Node::clone(
            cx,
            self,
            None,
            if subtree {
                CloneChildrenFlag::CloneChildren
            } else {
                CloneChildrenFlag::DoNotCloneChildren
            },
            None,
        );
        Ok(result)
    }

    /// <https://dom.spec.whatwg.org/#dom-node-isequalnode>
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
                (element.attrs().borrow().len() == other_element.attrs().borrow().len())
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
            assert!(element.attrs().borrow().len() == other_element.attrs().borrow().len());
            element.attrs().borrow().iter().all(|attr| {
                other_element.attrs().borrow().iter().any(|other_attr| {
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
                },
                NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) |
                NodeTypeId::CharacterData(CharacterDataTypeId::Comment)
                    if !is_equal_characterdata(this, node) =>
                {
                    return false;
                },
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

    /// <https://dom.spec.whatwg.org/#dom-node-issamenode>
    fn IsSameNode(&self, other_node: Option<&Node>) -> bool {
        match other_node {
            Some(node) => self == node,
            None => false,
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-comparedocumentposition>
    fn CompareDocumentPosition(&self, other: &Node) -> u16 {
        // Step 1. If this is other, then return zero.
        if self == other {
            return 0;
        }

        // Step 2. Let node1 be other and node2 be this.
        let mut node1 = Some(other);
        let mut node2 = Some(self);

        // Step 3. Let attr1 and attr2 be null.
        let mut attr1: Option<&Attr> = None;
        let mut attr2: Option<&Attr> = None;

        // step 4: spec says to operate on node1 here,
        // node1 is definitely Some(other) going into this step
        // The compiler doesn't know the lifetime of attr1.GetOwnerElement
        // is guaranteed by the lifetime of attr1, so we hold it explicitly
        let attr1owner;
        if let Some(a) = other.downcast::<Attr>() {
            attr1 = Some(a);
            attr1owner = a.GetOwnerElement();
            node1 = match attr1owner {
                Some(ref e) => Some(e.upcast()),
                None => None,
            }
        }

        // step 5.1: spec says to operate on node2 here,
        // node2 is definitely just Some(self) going into this step
        let attr2owner;
        if let Some(a) = self.downcast::<Attr>() {
            attr2 = Some(a);
            attr2owner = a.GetOwnerElement();
            node2 = match attr2owner {
                Some(ref e) => Some(e.upcast()),
                None => None,
            }
        }

        // Step 5.2
        // This substep seems lacking in test coverage.
        // We hit this when comparing two attributes that have the
        // same owner element.
        if let Some(node2) = node2 &&
            Some(node2) == node1 &&
            let (Some(a1), Some(a2)) = (attr1, attr2)
        {
            let attrs = node2.downcast::<Element>().unwrap().attrs();
            // go through the attrs in order to see if self
            // or other is first; spec is clear that we
            // want value-equality, not reference-equality
            for attr in attrs.borrow().iter() {
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
        }

        // Step 6
        match (node1, node2) {
            (None, _) => {
                // node1 is null
                NodeConstants::DOCUMENT_POSITION_FOLLOWING +
                    NodeConstants::DOCUMENT_POSITION_DISCONNECTED +
                    NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC
            },
            (_, None) => {
                // node2 is null
                NodeConstants::DOCUMENT_POSITION_PRECEDING +
                    NodeConstants::DOCUMENT_POSITION_DISCONNECTED +
                    NodeConstants::DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC
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
                        for child in parent.children() {
                            if child == child_1 {
                                // `other` is following `self`.
                                return NodeConstants::DOCUMENT_POSITION_FOLLOWING;
                            }
                            if child == child_2 {
                                // `other` is preceding `self`.
                                return NodeConstants::DOCUMENT_POSITION_PRECEDING;
                            }
                        }
                    }

                    parent = child_1;
                }

                // We hit the end of one of the parent chains, so one node needs to be
                // contained in the other.
                //
                // If we're the container, return that `other` is contained by us.
                if self_and_ancestors.len() < other_and_ancestors.len() {
                    NodeConstants::DOCUMENT_POSITION_FOLLOWING +
                        NodeConstants::DOCUMENT_POSITION_CONTAINED_BY
                } else {
                    NodeConstants::DOCUMENT_POSITION_PRECEDING +
                        NodeConstants::DOCUMENT_POSITION_CONTAINS
                }
            },
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-contains>
    fn Contains(&self, maybe_other: Option<&Node>) -> bool {
        match maybe_other {
            None => false,
            Some(other) => self.is_inclusive_ancestor_of(other),
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-lookupprefix>
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

    /// <https://dom.spec.whatwg.org/#dom-node-lookupnamespaceuri>
    fn LookupNamespaceURI(&self, prefix: Option<DOMString>) -> Option<DOMString> {
        // Step 1. If prefix is the empty string, then set it to null.
        let prefix = prefix.filter(|prefix| !prefix.is_empty());

        // Step 2. Return the result of running locate a namespace for this using prefix.
        Node::namespace_to_string(Node::locate_namespace(self, prefix))
    }

    /// <https://dom.spec.whatwg.org/#dom-node-isdefaultnamespace>
    fn IsDefaultNamespace(&self, namespace: Option<DOMString>) -> bool {
        // Step 1.
        let namespace = namespace_from_domstring(namespace);
        // Steps 2 and 3.
        Node::locate_namespace(self, None) == namespace
    }
}

pub(crate) trait NodeTraits {
    /// Get the [`Document`] that owns this node. Note that this may differ from the
    /// [`Document`] that the node was created in if it was adopted by a different
    /// [`Document`] (the owner).
    fn owner_document(&self) -> DomRoot<Document>;
    /// Get the [`Window`] of the [`Document`] that owns this node. Note that this may
    /// differ from the [`Document`] that the node was created in if it was adopted by a
    /// different [`Document`] (the owner).
    fn owner_window(&self) -> DomRoot<Window>;
    /// Get the [`GlobalScope`] of the [`Document`] that owns this node. Note that this may
    /// differ from the [`GlobalScope`] that the node was created in if it was adopted by a
    /// different [`Document`] (the owner).
    fn owner_global(&self) -> DomRoot<GlobalScope>;
    /// If this [`Node`] is contained in a [`ShadowRoot`] return it, otherwise `None`.
    fn containing_shadow_root(&self) -> Option<DomRoot<ShadowRoot>>;
    /// Get the stylesheet owner for this node: either the [`Document`] or the [`ShadowRoot`]
    /// of the node.
    fn stylesheet_list_owner(&self) -> StyleSheetListOwner;
}

impl<T: DerivedFrom<Node> + DomObject> NodeTraits for T {
    fn owner_document(&self) -> DomRoot<Document> {
        self.upcast().owner_doc()
    }

    fn owner_window(&self) -> DomRoot<Window> {
        DomRoot::from_ref(self.owner_document().window())
    }

    fn owner_global(&self) -> DomRoot<GlobalScope> {
        DomRoot::from_ref(self.owner_window().upcast())
    }

    fn containing_shadow_root(&self) -> Option<DomRoot<ShadowRoot>> {
        Node::containing_shadow_root(self.upcast())
    }

    fn stylesheet_list_owner(&self) -> StyleSheetListOwner {
        self.containing_shadow_root()
            .map(|shadow_root| StyleSheetListOwner::ShadowRoot(Dom::from_ref(&*shadow_root)))
            .unwrap_or_else(|| {
                StyleSheetListOwner::Document(Dom::from_ref(&*self.owner_document()))
            })
    }
}

impl VirtualMethods for Node {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<EventTarget>() as &dyn VirtualMethods)
    }

    fn children_changed(&self, cx: &mut JSContext, mutation: &ChildrenMutation) {
        if let Some(s) = self.super_type() {
            s.children_changed(cx, mutation);
        }

        if let Some(data) = self.rare_data.borrow().as_ref() &&
            let Some(list) = data.child_list.get()
        {
            list.as_children_list().children_changed(mutation);
        }

        self.owner_doc_unrooted(cx.no_gc())
            .content_and_heritage_changed(cx.no_gc(), self);
    }

    // This handles the ranges mentioned in steps 2-3 when removing a node.
    /// <https://dom.spec.whatwg.org/#concept-node-remove>
    fn unbind_from_tree(&self, cx: &mut JSContext, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(cx, context);

        // Ranges should only drain to the parent from inclusive non-shadow
        // including descendants. If we're in a shadow tree at this point then the
        // unbind operation happened further up in the tree and we should not
        // drain any ranges.
        if !self.is_in_a_shadow_tree() &&
            let Some(weak_ranges) = self.weak_ranges_mut() &&
            !weak_ranges.is_empty()
        {
            weak_ranges.drain_to_parent(context.parent, context.index(), self);
        }
    }

    fn moving_steps(&self, cx: &mut JSContext, context: &MoveContext) {
        if let Some(super_type) = self.super_type() {
            super_type.moving_steps(cx, context);
        }

        // Ranges should only drain to the parent from inclusive non-shadow
        // including descendants. If we're in a shadow tree at this point then the
        // unbind operation happened further up in the tree and we should not
        // drain any ranges.
        if let Some(old_parent) = context.old_parent &&
            !self.is_in_a_shadow_tree() &&
            let Some(weak_ranges) = self.weak_ranges_mut() &&
            !weak_ranges.is_empty()
        {
            weak_ranges.drain_to_parent(old_parent, context.index(), self);
        }

        self.owner_doc()
            .content_and_heritage_changed(cx.no_gc(), self);
    }

    fn handle_event(&self, cx: &mut JSContext, event: &Event) {
        if event.DefaultPrevented() || event.flags().contains(EventFlags::Handled) {
            return;
        }

        if let Some(event) = event.downcast::<KeyboardEvent>() {
            self.owner_document()
                .event_handler()
                .run_default_keyboard_event_handler(cx, self, event);
        }
    }
}

/// A summary of the changes that happened to a node.
#[derive(Clone, Copy, MallocSizeOf, PartialEq)]
pub(crate) enum NodeDamage {
    /// The node's `style` attribute changed.
    Style,
    /// The node's content or heritage changed, such as the addition or removal of
    /// children.
    ContentOrHeritage,
    /// Other parts of a node changed; attributes, text content, etc.
    Other,
}

/// A node's unique ID, for devtools.
pub(crate) struct UniqueId {
    cell: UnsafeCell<Option<Box<Uuid>>>,
}

unsafe_no_jsmanaged_fields!(UniqueId);

impl MallocSizeOf for UniqueId {
    #[expect(unsafe_code)]
    fn size_of(&self, ops: &mut MallocSizeOfOps) -> usize {
        if let Some(uuid) = unsafe { &*self.cell.get() } {
            unsafe { ops.malloc_size_of(&**uuid) }
        } else {
            0
        }
    }
}

impl UniqueId {
    /// Create a new `UniqueId` value. The underlying `Uuid` is lazily created.
    pub(super) fn new() -> UniqueId {
        UniqueId {
            cell: UnsafeCell::new(None),
        }
    }

    /// The Uuid of that unique ID.
    #[expect(unsafe_code)]
    pub(super) fn borrow(&self) -> &Uuid {
        unsafe {
            let ptr = self.cell.get();
            if (*ptr).is_none() {
                *ptr = Some(Box::new(Uuid::new_v4()));
            }
            (*ptr).as_ref().unwrap()
        }
    }
}

/// Helper trait to insert an element into vector whose elements
/// are maintained in tree order
pub(crate) trait VecPreOrderInsertionHelper<T> {
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
    fn insert_pre_order(&mut self, node: &T, tree_root: &Node) {
        let Err(insertion_index) = self.binary_search_by(|candidate| {
            candidate.upcast().compare_dom_tree_position(
                node.upcast(),
                tree_root,
                ShadowIncluding::No,
            )
        }) else {
            // The element is already in the vector. We assume that users of this method generally
            // expect no duplicates, so there's nothing more to do.
            return;
        };

        self.insert(insertion_index, Dom::from_ref(node));
    }
}

/// The return value of [`Node::parent_in_flat_tree`].
pub(crate) enum FlatTreeParent {
    /// The parent in the flat tree.
    Parent(DomRoot<Node>),
    /// This node has a parent (it's not the root), but it does not share a flat tree
    /// relationship with its parent.
    NotInFlatTree,
    /// This node is in the flat tree, but has no parent node.
    RootNode,
}
