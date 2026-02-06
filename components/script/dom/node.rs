/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The core DOM types. Defines the basic DOM hierarchy as well as all the HTML elements.

use std::borrow::Cow;
use std::cell::{Cell, LazyCell, UnsafeCell};
use std::default::Default;
use std::f64::consts::PI;
use std::ops::Deref;
use std::slice::from_ref;
use std::{cmp, fmt, iter};

use app_units::Au;
use base::id::{BrowsingContextId, PipelineId};
use bitflags::bitflags;
use devtools_traits::NodeInfo;
use dom_struct::dom_struct;
use embedder_traits::UntrustedNodeAddress;
use euclid::default::Size2D;
use euclid::{Point2D, Rect};
use html5ever::serialize::HtmlSerializer;
use html5ever::{Namespace, Prefix, QualName, ns, serialize as html_serialize};
use js::jsapi::JSObject;
use js::rust::HandleObject;
use keyboard_types::Modifiers;
use layout_api::wrapper_traits::SharedSelection;
use layout_api::{
    BoxAreaType, CSSPixelRectIterator, GenericLayoutData, HTMLCanvasData, HTMLMediaData,
    LayoutElementType, LayoutNodeType, PhysicalSides, QueryMsg, SVGElementData, StyleData,
    TrustedNodeAddress,
};
use libc::{self, c_void, uintptr_t};
use malloc_size_of::{MallocSizeOf, MallocSizeOfOps};
use net_traits::image_cache::Image;
use pixels::ImageMetadata;
use script_bindings::codegen::InheritTypes::DocumentFragmentTypeId;
use script_traits::DocumentActivity;
use selectors::bloom::BloomFilter;
use selectors::matching::{
    MatchingContext, MatchingForInvalidation, MatchingMode, NeedsSelectorFlags,
    matches_selector_list,
};
use selectors::parser::SelectorList;
use servo_arc::Arc as ServoArc;
use servo_config::pref;
use servo_url::ServoUrl;
use smallvec::SmallVec;
use style::attr::AttrValue;
use style::context::QuirksMode;
use style::dom::OpaqueNode;
use style::properties::ComputedValues;
use style::selector_parser::{PseudoElement, SelectorImpl, SelectorParser};
use style::stylesheets::{Stylesheet, UrlExtraData};
use style_traits::CSSPixel;
use uuid::Uuid;
use xml5ever::{local_name, serialize as xml_serialize};

use super::types::CDATASection;
use crate::conversions::Convert;
use crate::document_loader::DocumentLoader;
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::{DomRefCell, Ref, RefMut};
use crate::dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use crate::dom::bindings::codegen::Bindings::CSSStyleDeclarationBinding::CSSStyleDeclarationMethods;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLCollectionBinding::HTMLCollectionMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::{
    GetRootNodeOptions, NodeConstants, NodeMethods,
};
use crate::dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use crate::dom::bindings::codegen::Bindings::ProcessingInstructionBinding::ProcessingInstructionMethods;
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
    Castable, CharacterDataTypeId, ElementTypeId, EventTargetTypeId, HTMLElementTypeId, NodeTypeId,
    SVGElementTypeId, SVGGraphicsElementTypeId, TextTypeId,
};
use crate::dom::bindings::reflector::{DomObject, DomObjectWrap, reflect_dom_object_with_proto};
use crate::dom::bindings::root::{Dom, DomRoot, DomSlice, LayoutDom, MutNullableDom, ToLayout};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::characterdata::{CharacterData, LayoutCharacterDataHelpers};
use crate::dom::css::cssstylesheet::CSSStyleSheet;
use crate::dom::css::stylesheetlist::StyleSheetListOwner;
use crate::dom::customelementregistry::{
    CallbackReaction, CustomElementRegistry, try_upgrade_element,
};
use crate::dom::document::{Document, DocumentSource, HasBrowsingContext, IsHTMLDocument};
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::documenttype::DocumentType;
use crate::dom::element::{
    AttributeMutationReason, CustomElementCreationMode, Element, ElementCreator, SelectorWrapper,
};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlcanvaselement::{HTMLCanvasElement, LayoutHTMLCanvasElementHelpers};
use crate::dom::html::htmlcollection::HTMLCollection;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmliframeelement::{HTMLIFrameElement, HTMLIFrameElementLayoutMethods};
use crate::dom::html::htmlimageelement::{HTMLImageElement, LayoutHTMLImageElementHelpers};
use crate::dom::html::htmlinputelement::{HTMLInputElement, LayoutHTMLInputElementHelpers};
use crate::dom::html::htmllinkelement::HTMLLinkElement;
use crate::dom::html::htmlslotelement::{HTMLSlotElement, Slottable};
use crate::dom::html::htmlstyleelement::HTMLStyleElement;
use crate::dom::html::htmltextareaelement::{
    HTMLTextAreaElement, LayoutHTMLTextAreaElementHelpers,
};
use crate::dom::html::htmlvideoelement::{HTMLVideoElement, LayoutHTMLVideoElementHelpers};
use crate::dom::mutationobserver::{Mutation, MutationObserver, RegisteredObserver};
use crate::dom::nodelist::NodeList;
use crate::dom::pointerevent::{PointerEvent, PointerId};
use crate::dom::processinginstruction::ProcessingInstruction;
use crate::dom::range::WeakRangeVec;
use crate::dom::raredata::NodeRareData;
use crate::dom::servoparser::html::HtmlSerialize;
use crate::dom::servoparser::{ServoParser, serialize_html_fragment};
use crate::dom::shadowroot::{IsUserAgentWidget, LayoutShadowRootHelpers, ShadowRoot};
use crate::dom::svg::svgsvgelement::{LayoutSVGSVGElementHelpers, SVGSVGElement};
use crate::dom::text::Text;
use crate::dom::types::KeyboardEvent;
use crate::dom::virtualmethods::{VirtualMethods, vtable_for};
use crate::dom::window::Window;
use crate::script_runtime::CanGc;
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

    /// Style data for this node. This is accessed and mutated by style
    /// passes and is used to lay out this node and populate layout data.
    #[no_trace]
    style_data: DomRefCell<Option<Box<StyleData>>>,

    /// Layout data for this node. This is populated during layout and can
    /// be used for incremental relayout and script queries.
    #[no_trace]
    layout_data: DomRefCell<Option<Box<GenericLayoutData>>>,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if matches!(self.type_id(), NodeTypeId::Element(_)) {
            let el = self.downcast::<Element>().unwrap();
            el.fmt(f)
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

        /// Specifies whether this node is focusable and whether it is supposed
        /// to be reachable with using sequential focus navigation."]
        const SEQUENTIALLY_FOCUSABLE = 1 << 3;

        // There are two free bits here.

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
/// <https://dom.spec.whatwg.org/#concept-node-insert>
/// <https://dom.spec.whatwg.org/#concept-node-remove>
#[derive(Clone, Copy, MallocSizeOf)]
enum SuppressObserver {
    Suppressed,
    Unsuppressed,
}

pub(crate) enum ForceSlottableNodeReconciliation {
    Force,
    Skip,
}

impl Node {
    /// Adds a new child to the end of this node's list of children.
    ///
    /// Fails unless `new_child` is disconnected from the tree.
    fn add_child(&self, new_child: &Node, before: Option<&Node>, can_gc: CanGc) {
        assert!(new_child.parent_node.get().is_none());
        assert!(new_child.prev_sibling.get().is_none());
        assert!(new_child.next_sibling.get().is_none());
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
            vtable_for(&node).bind_to_tree(&context, can_gc);
        }
    }

    /// Implements the "unsafely set HTML" algorithm as specified in:
    /// <https://html.spec.whatwg.org/multipage/#concept-unsafely-set-html>
    pub(crate) fn unsafely_set_html(
        target: &Node,
        context_element: &Element,
        html: DOMString,
        can_gc: CanGc,
    ) {
        // Step 1. Let newChildren be the result of the HTML fragment parsing algorithm.
        let new_children = ServoParser::parse_html_fragment(context_element, html, true, can_gc);

        // Step 2. Let fragment be a new DocumentFragment whose node document is contextElement's node document.

        let context_document = context_element.owner_document();
        let fragment = DocumentFragment::new(&context_document, can_gc);

        // Step 3. For each node in newChildren, append node to fragment.
        for child in new_children {
            fragment
                .upcast::<Node>()
                .AppendChild(&child, can_gc)
                .unwrap();
        }

        // Step 4. Replace all with fragment within target.
        Node::replace_all(Some(fragment.upcast()), target, can_gc);
    }

    /// Clear this [`Node`]'s layout data and also clear the layout data of all children.
    /// Note that clears layout data from all non-flat tree descendants and flat tree
    /// descendants.
    pub(crate) fn remove_layout_boxes_from_subtree(&self) {
        for node in self.traverse_preorder(ShadowIncluding::Yes) {
            node.layout_data.borrow_mut().take();
        }
    }

    pub(crate) fn clean_up_style_and_layout_data(&self) {
        self.owner_doc().cancel_animations_for_node(self);
        self.style_data.borrow_mut().take();
        self.layout_data.borrow_mut().take();
    }

    /// Clean up flags and runs steps 11-14 of remove a node.
    /// <https://dom.spec.whatwg.org/#concept-node-remove>
    pub(crate) fn complete_remove_subtree(root: &Node, context: &UnbindContext, can_gc: CanGc) {
        // Flags that reset when a node is disconnected
        const RESET_FLAGS: NodeFlags = NodeFlags::IS_IN_A_DOCUMENT_TREE
            .union(NodeFlags::IS_CONNECTED)
            .union(NodeFlags::HAS_DIRTY_DESCENDANTS)
            .union(NodeFlags::HAS_SNAPSHOT)
            .union(NodeFlags::HANDLED_SNAPSHOT);

        for node in root.traverse_preorder(ShadowIncluding::No) {
            node.set_flag(RESET_FLAGS | NodeFlags::IS_IN_SHADOW_TREE, false);

            // If the element has a shadow root attached to it then we traverse that as well,
            // but without touching the IS_IN_SHADOW_TREE flags of the children
            if let Some(shadow_root) = node.downcast::<Element>().and_then(Element::shadow_root) {
                for node in shadow_root
                    .upcast::<Node>()
                    .traverse_preorder(ShadowIncluding::Yes)
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
        let cleanup_node = |node: &Node| {
            node.clean_up_style_and_layout_data();

            // Step 11 & 14.1. Run the removing steps.
            // This needs to be in its own loop, because unbind_from_tree may
            // rely on the state of IS_IN_DOC of the context node's descendants,
            // e.g. when removing a <form>.
            vtable_for(node).unbind_from_tree(context, can_gc);

            // Step 12 & 14.2. Enqueue disconnected custom element reactions.
            if is_parent_connected {
                if let Some(element) = node.as_custom_element() {
                    custom_element_reaction_stack.enqueue_callback_reaction(
                        &element,
                        CallbackReaction::Disconnected,
                        None,
                    );
                }
            }
        };

        for node in root.traverse_preorder(ShadowIncluding::No) {
            cleanup_node(&node);

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
                    cleanup_node(&node);
                }
            }
        }
    }

    pub(crate) fn complete_move_subtree(root: &Node) {
        // Flags that reset when a node is moved
        const RESET_FLAGS: NodeFlags = NodeFlags::IS_IN_A_DOCUMENT_TREE
            .union(NodeFlags::IS_CONNECTED)
            .union(NodeFlags::HAS_DIRTY_DESCENDANTS)
            .union(NodeFlags::HAS_SNAPSHOT)
            .union(NodeFlags::HANDLED_SNAPSHOT);

        // Since both the initial traversal in light dom and the inner traversal
        // in shadow DOM share the same code, we define a closure to prevent omissions.
        let cleanup_node = |node: &Node| {
            node.style_data.borrow_mut().take();
            node.layout_data.borrow_mut().take();
        };

        for node in root.traverse_preorder(ShadowIncluding::No) {
            node.set_flag(RESET_FLAGS | NodeFlags::IS_IN_SHADOW_TREE, false);
            cleanup_node(&node);

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
                    cleanup_node(&node);
                }
            }
        }
    }

    /// Removes the given child from this node's list of children.
    ///
    /// Fails unless `child` is a child of this node.
    fn remove_child(&self, child: &Node, cached_index: Option<u32>, can_gc: CanGc) {
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

        Self::complete_remove_subtree(child, &context, can_gc);
    }

    fn move_child(&self, child: &Node) {
        assert!(child.parent_node.get().as_deref() == Some(self));
        self.note_dirty_descendants();

        child.prev_sibling.set(None);
        child.next_sibling.set(None);
        child.parent_node.set(None);
        self.children_count.set(self.children_count.get() - 1);
        Self::complete_move_subtree(child)
    }

    pub(crate) fn to_untrusted_node_address(&self) -> UntrustedNodeAddress {
        UntrustedNodeAddress(self.reflector().get_jsobject().get() as *const c_void)
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
    pub(crate) fn fire_synthetic_pointer_event_not_trusted(&self, name: DOMString, can_gc: CanGc) {
        // Spec says the choice of which global to create the pointer event
        // on is not well-defined,
        // and refers to heycam/webidl#135
        let window = self.owner_window();

        // <https://w3c.github.io/pointerevents/#the-click-auxclick-and-contextmenu-events>
        let pointer_event = PointerEvent::new(
            &window, // ambiguous in spec
            name,
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
            can_gc,
        );

        // Step 4. Set event's composed flag.
        pointer_event.upcast::<Event>().set_composed(true);

        // Step 5. If the not trusted flag is set, initialize event's isTrusted attribute to false.
        pointer_event.upcast::<Event>().set_trusted(false);

        // Step 6,8. TODO keyboard modifiers

        pointer_event
            .upcast::<Event>()
            .dispatch(self.upcast::<EventTarget>(), false, can_gc);
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
}

pub(crate) struct QuerySelectorIterator {
    selectors: SelectorList<SelectorImpl>,
    iterator: TreeIterator,
}

impl QuerySelectorIterator {
    fn new(iter: TreeIterator, selectors: SelectorList<SelectorImpl>) -> QuerySelectorIterator {
        QuerySelectorIterator {
            selectors,
            iterator: iter,
        }
    }
}

impl Iterator for QuerySelectorIterator {
    type Item = DomRoot<Node>;

    fn next(&mut self) -> Option<DomRoot<Node>> {
        let selectors = &self.selectors;

        let (quirks_mode, filter) = match self.iterator.by_ref().peek() {
            Some(node) => (node.owner_doc().quirks_mode(), BloomFilter::default()),
            None => return None,
        };

        self.iterator.by_ref().find_map(|node| {
            if let Some(element) = DomRoot::downcast(node) {
                let mut nth_index_cache = Default::default();
                let mut ctx = MatchingContext::new(
                    MatchingMode::Normal,
                    Some(&filter),
                    &mut nth_index_cache,
                    quirks_mode,
                    NeedsSelectorFlags::No,
                    MatchingForInvalidation::No,
                );
                if matches_selector_list(selectors, &SelectorWrapper::Borrowed(&element), &mut ctx)
                {
                    return Some(DomRoot::upcast(element));
                }
            }
            None
        })
    }
}

impl Node {
    fn rare_data(&self) -> Ref<'_, Option<Box<NodeRareData>>> {
        self.rare_data.borrow()
    }

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
        let rare_data: Ref<'_, _> = self.rare_data.borrow();

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
        self.ensure_rare_data()
            .mutation_observers
            .retain(|reg_obs| &*reg_obs.observer != observer)
    }

    /// Dumps the subtree rooted at this node, for debugging.
    pub(crate) fn dump(&self) {
        self.dump_indent(0);
    }

    /// Dumps the node tree, for debugging, with indentation.
    pub(crate) fn dump_indent(&self, indent: u32) {
        let mut s = String::new();
        for _ in 0..indent {
            s.push_str("    ");
        }

        s.push_str(&self.debug_str());
        debug!("{:?}", s);

        // FIXME: this should have a pure version?
        for kid in self.children() {
            kid.dump_indent(indent + 1)
        }
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

    pub(crate) fn ranges(&self) -> RefMut<'_, WeakRangeVec> {
        RefMut::map(self.ensure_rare_data(), |rare_data| &mut rare_data.ranges)
    }

    pub(crate) fn ranges_is_empty(&self) -> bool {
        self.rare_data()
            .as_ref()
            .is_none_or(|data| data.ranges.is_empty())
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
    pub(crate) fn note_dirty_descendants(&self) {
        self.owner_doc().note_node_with_dirty_descendants(self);
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
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::Text)) => {
                // For content changes in text nodes, we should accurately use
                // [`NodeDamage::ContentOrHeritage`] to mark the parent node, thereby
                // reducing the scope of incremental box tree construction.
                self.parent_node
                    .get()
                    .unwrap()
                    .dirty(NodeDamage::ContentOrHeritage)
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

    pub(crate) fn inclusively_following_siblings(
        &self,
    ) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator {
            current: Some(DomRoot::from_ref(self)),
            next_node: |n| n.GetNextSibling(),
        }
    }

    pub(crate) fn inclusively_preceding_siblings(
        &self,
    ) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator {
            current: Some(DomRoot::from_ref(self)),
            next_node: |n| n.GetPreviousSibling(),
        }
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
        SimpleNodeIterator {
            current: self.GetNextSibling(),
            next_node: |n| n.GetNextSibling(),
        }
    }

    pub(crate) fn preceding_siblings(&self) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator {
            current: self.GetPreviousSibling(),
            next_node: |n| n.GetPreviousSibling(),
        }
    }

    pub(crate) fn following_nodes(&self, root: &Node) -> FollowingNodeIterator {
        FollowingNodeIterator {
            current: Some(DomRoot::from_ref(self)),
            root: DomRoot::from_ref(root),
        }
    }

    pub(crate) fn preceding_nodes(&self, root: &Node) -> PrecedingNodeIterator {
        PrecedingNodeIterator {
            current: Some(DomRoot::from_ref(self)),
            root: DomRoot::from_ref(root),
        }
    }

    /// Return an iterator that moves from `self` down the tree, choosing the last child
    /// at each step of the way.
    pub(crate) fn descending_last_children(&self) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator {
            current: self.GetLastChild(),
            next_node: |n| n.GetLastChild(),
        }
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

    pub(crate) fn padding_box(&self) -> Option<Rect<Au, CSSPixel>> {
        self.owner_window()
            .box_area_query(self, BoxAreaType::Padding, false)
    }

    pub(crate) fn border_boxes(&self) -> CSSPixelRectIterator {
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

    /// <https://dom.spec.whatwg.org/#dom-childnode-before>
    pub(crate) fn before(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
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
        let node = self
            .owner_doc()
            .node_from_nodes_and_strings(nodes, can_gc)?;

        // Step 5.
        let viable_previous_sibling = match viable_previous_sibling {
            Some(ref viable_previous_sibling) => viable_previous_sibling.next_sibling.get(),
            None => parent.first_child.get(),
        };

        // Step 6.
        Node::pre_insert(&node, &parent, viable_previous_sibling.as_deref(), can_gc)?;

        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-childnode-after>
    pub(crate) fn after(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
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
        let node = self
            .owner_doc()
            .node_from_nodes_and_strings(nodes, can_gc)?;

        // Step 5.
        Node::pre_insert(&node, &parent, viable_next_sibling.as_deref(), can_gc)?;

        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-childnode-replacewith>
    pub(crate) fn replace_with(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        // Step 1. Let parent be this’s parent.
        let Some(parent) = self.GetParentNode() else {
            // Step 2. If parent is null, then return.
            return Ok(());
        };

        // Step 3. Let viableNextSibling be this’s first following sibling not in nodes; otherwise null.
        let viable_next_sibling = first_node_not_in(self.following_siblings(), &nodes);

        // Step 4. Let node be the result of converting nodes into a node, given nodes and this’s node document.
        let node = self
            .owner_doc()
            .node_from_nodes_and_strings(nodes, can_gc)?;

        if self.parent_node == Some(&*parent) {
            // Step 5. If this’s parent is parent, replace this with node within parent.
            parent.ReplaceChild(&node, self, can_gc)?;
        } else {
            // Step 6. Otherwise, pre-insert node into parent before viableNextSibling.
            Node::pre_insert(&node, &parent, viable_next_sibling.as_deref(), can_gc)?;
        }
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-prepend>
    pub(crate) fn prepend(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        // Step 1.
        let doc = self.owner_doc();
        let node = doc.node_from_nodes_and_strings(nodes, can_gc)?;
        // Step 2.
        let first_child = self.first_child.get();
        Node::pre_insert(&node, self, first_child.as_deref(), can_gc).map(|_| ())
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-append>
    pub(crate) fn append(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        // Step 1.
        let doc = self.owner_doc();
        let node = doc.node_from_nodes_and_strings(nodes, can_gc)?;
        // Step 2.
        self.AppendChild(&node, can_gc).map(|_| ())
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-replacechildren>
    pub(crate) fn replace_children(&self, nodes: Vec<NodeOrString>, can_gc: CanGc) -> ErrorResult {
        // Step 1. Let node be the result of converting nodes into a node given nodes and this’s
        // node document.
        let doc = self.owner_doc();
        let node = doc.node_from_nodes_and_strings(nodes, can_gc)?;

        // Step 2. Ensure pre-insert validity of node into this before null.
        Node::ensure_pre_insertion_validity(&node, self, None)?;

        // Step 3. Replace all with node within this.
        Node::replace_all(Some(&node), self, can_gc);
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-movebefore>
    pub(crate) fn move_before(
        &self,
        node: &Node,
        child: Option<&Node>,
        can_gc: CanGc,
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
        Node::move_fn(node, self, reference_child, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#move>
    pub(crate) fn move_fn(
        node: &Node,
        new_parent: &Node,
        child: Option<&Node>,
        can_gc: CanGc,
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
        if let Some(child) = child {
            if !new_parent.is_parent_of(child) {
                return Err(Error::NotFound(None));
            }
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
                    .inclusively_following_siblings()
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

        let mut context = MoveContext::new(
            Some(&old_parent),
            prev_sibling.as_deref(),
            next_sibling.as_deref(),
            cached_index,
        );

        // Step 13. Remove node from oldParent’s children.
        old_parent.move_child(node);

        // Step 14. If node is assigned, then run assign slottables for node’s assigned slot.
        if let Some(slot) = node.assigned_slot() {
            slot.assign_slottables();
        }

        // Step 15. If oldParent’s root is a shadow root, and oldParent is a slot whose assigned
        // nodes is empty, then run signal a slot change for oldParent.
        if old_parent.is_in_a_shadow_tree() {
            if let Some(slot_element) = old_parent.downcast::<HTMLSlotElement>() {
                if !slot_element.has_assigned_nodes() {
                    slot_element.signal_a_slot_change();
                }
            }
        }

        // Step 16. If node has an inclusive descendant that is a slot:
        let has_slot_descendant = node
            .traverse_preorder(ShadowIncluding::No)
            .any(|element| element.is::<HTMLSlotElement>());
        if has_slot_descendant {
            // Step 16.1. Run assign slottables for a tree with oldParent’s root.
            old_parent
                .GetRootNode(&GetRootNodeOptions::empty())
                .assign_slottables_for_a_tree();

            // Step 16.2. Run assign slottables for a tree with node.
            node.assign_slottables_for_a_tree();
        }

        // Step 17. If child is non-null:
        if let Some(child) = child {
            // Step 17.1. For each live range whose start node is newParent and start offset is
            // greater than child’s index: increase its start offset by 1.
            // Step 17.2. For each live range whose end node is newParent and end offset is greater
            // than child’s index: increase its end offset by 1.
            new_parent
                .ranges()
                .increase_above(new_parent, child.index(), 1)
        }

        // Step 18. Let newPreviousSibling be child’s previous sibling if child is non-null, and
        // newParent’s last child otherwise.
        let new_previous_sibling = child.map_or_else(
            || new_parent.last_child.get(),
            |child| child.prev_sibling.get(),
        );

        // Step 19. If child is null, then append node to newParent’s children.
        // Step 20. Otherwise, insert node into newParent’s children before child’s index.
        new_parent.add_child(node, child, can_gc);

        // Step 21. If newParent is a shadow host whose shadow root’s slot assignment is "named" and
        // node is a slottable, then assign a slot for node.
        if let Some(shadow_root) = new_parent
            .downcast::<Element>()
            .and_then(Element::shadow_root)
        {
            if shadow_root.SlotAssignment() == SlotAssignmentMode::Named {
                let cx = GlobalScope::get_cx();
                if node.is::<Element>() || node.is::<Text>() {
                    rooted!(in(*cx) let slottable = Slottable(Dom::from_ref(node)));
                    slottable.assign_a_slot();
                }
            }
        }

        // Step 22. If newParent’s root is a shadow root, and newParent is a slot whose assigned
        // nodes is empty, then run signal a slot change for newParent.
        if new_parent.is_in_a_shadow_tree() {
            if let Some(slot_element) = new_parent.downcast::<HTMLSlotElement>() {
                if !slot_element.has_assigned_nodes() {
                    slot_element.signal_a_slot_change();
                }
            }
        }

        // Step 23. Run assign slottables for a tree with node’s root.
        node.GetRootNode(&GetRootNodeOptions::empty())
            .assign_slottables_for_a_tree();

        // Step 24. For each shadow-including inclusive descendant inclusiveDescendant of node, in
        // shadow-including tree order:
        for descendant in node.traverse_preorder(ShadowIncluding::Yes) {
            // Step 24.1. If inclusiveDescendant is node, then run the moving steps with
            // inclusiveDescendant and oldParent.
            // Otherwise, run the moving steps with inclusiveDescendant and null.
            if descendant.deref() == node {
                vtable_for(&descendant).moving_steps(&context, can_gc);
            } else {
                context.old_parent = None;
                vtable_for(&descendant).moving_steps(&context, can_gc);
            }

            // Step 24.2. If inclusiveDescendant is custom and newParent is connected,
            if let Some(descendant) = descendant.downcast::<Element>() {
                if descendant.is_custom() && new_parent.is_connected() {
                    // then enqueue a custom element callback reaction with
                    // inclusiveDescendant, callback name "connectedMoveCallback", and « ».
                    let custom_element_reaction_stack =
                        ScriptThread::custom_element_reaction_stack();
                    custom_element_reaction_stack.enqueue_callback_reaction(
                        descendant,
                        CallbackReaction::ConnectedMove,
                        None,
                    );
                }
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
        MutationObserver::queue_a_mutation_record(&old_parent, mutation);

        // Step 26. Queue a tree mutation record for newParent with « node », « »,
        // newPreviousSibling, and child.
        let mutation = LazyCell::new(|| Mutation::ChildList {
            added: Some(&moved),
            removed: None,
            prev: new_previous_sibling.as_deref(),
            next: child,
        });
        MutationObserver::queue_a_mutation_record(new_parent, mutation);

        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-queryselector>
    pub(crate) fn query_selector(
        &self,
        selectors: DOMString,
    ) -> Fallible<Option<DomRoot<Element>>> {
        // Step 1.
        let doc = self.owner_doc();
        match SelectorParser::parse_author_origin_no_namespace(
            &selectors.str(),
            &UrlExtraData(doc.url().get_arc()),
        ) {
            // Step 2.
            Err(_) => Err(Error::Syntax(None)),
            // Step 3.
            Ok(selectors) => {
                let mut nth_index_cache = Default::default();
                let filter = BloomFilter::default();
                let mut ctx = MatchingContext::new(
                    MatchingMode::Normal,
                    Some(&filter),
                    &mut nth_index_cache,
                    doc.quirks_mode(),
                    NeedsSelectorFlags::No,
                    MatchingForInvalidation::No,
                );
                let mut descendants = self.traverse_preorder(ShadowIncluding::No);
                // Skip the root of the tree.
                assert!(&*descendants.next().unwrap() == self);
                Ok(descendants.filter_map(DomRoot::downcast).find(|element| {
                    matches_selector_list(&selectors, &SelectorWrapper::Borrowed(element), &mut ctx)
                }))
            },
        }
    }

    /// <https://dom.spec.whatwg.org/#scope-match-a-selectors-string>
    /// Get an iterator over all nodes which match a set of selectors
    /// Be careful not to do anything which may manipulate the DOM tree
    /// whilst iterating, otherwise the iterator may be invalidated.
    pub(crate) fn query_selector_iter(
        &self,
        selectors: DOMString,
    ) -> Fallible<QuerySelectorIterator> {
        // Step 1.
        let url = self.owner_doc().url();
        match SelectorParser::parse_author_origin_no_namespace(
            &selectors.str(),
            &UrlExtraData(url.get_arc()),
        ) {
            // Step 2.
            Err(_) => Err(Error::Syntax(None)),
            // Step 3.
            Ok(selectors) => {
                let mut descendants = self.traverse_preorder(ShadowIncluding::No);
                // Skip the root of the tree.
                assert!(&*descendants.next().unwrap() == self);
                Ok(QuerySelectorIterator::new(descendants, selectors))
            },
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall>
    pub(crate) fn query_selector_all(&self, selectors: DOMString) -> Fallible<DomRoot<NodeList>> {
        let window = self.owner_window();
        let iter = self.query_selector_iter(selectors)?;
        Ok(NodeList::new_simple_list(&window, iter, CanGc::note()))
    }

    pub(crate) fn ancestors(&self) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator {
            current: self.GetParentNode(),
            next_node: |n| n.GetParentNode(),
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-shadow-including-inclusive-ancestor>
    pub(crate) fn inclusive_ancestors(
        &self,
        shadow_including: ShadowIncluding,
    ) -> impl Iterator<Item = DomRoot<Node>> + use<> {
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

    pub(crate) fn owner_doc(&self) -> DomRoot<Document> {
        self.owner_doc.get().unwrap()
    }

    pub(crate) fn set_owner_doc(&self, document: &Document) {
        self.owner_doc.set(Some(document));
    }

    pub(crate) fn containing_shadow_root(&self) -> Option<DomRoot<ShadowRoot>> {
        self.rare_data()
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
        SimpleNodeIterator {
            current: self.GetFirstChild(),
            next_node: |n| n.GetNextSibling(),
        }
    }

    pub(crate) fn rev_children(&self) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator {
            current: self.GetLastChild(),
            next_node: |n| n.GetPreviousSibling(),
        }
    }

    pub(crate) fn child_elements(&self) -> impl Iterator<Item = DomRoot<Element>> + use<> {
        self.children()
            .filter_map(DomRoot::downcast as fn(_) -> _)
            .peekable()
    }

    pub(crate) fn remove_self(&self, can_gc: CanGc) {
        if let Some(ref parent) = self.GetParentNode() {
            Node::remove(self, parent, SuppressObserver::Unsuppressed, can_gc);
        }
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

    pub(crate) fn summarize(&self, can_gc: CanGc) -> NodeInfo {
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
            self.ChildNodes(can_gc).Length() as usize + 1
        } else {
            self.ChildNodes(can_gc).Length() as usize
        };

        let window = self.owner_window();
        let display = self
            .downcast::<Element>()
            .map(|elem| window.GetComputedStyle(elem, None))
            .map(|style| style.Display().into());

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
            attrs: self.downcast().map(Element::summarize).unwrap_or(vec![]),
            is_shadow_host,
            shadow_root_mode,
            display,
            // It is not entirely clear when this should be set to false.
            // Firefox considers nodes with "display: contents" to be displayed.
            // The doctype node is displayed despite being `display: none`.
            is_displayed: !self.is_display_none() || self.is::<DocumentType>(),
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
        }
    }

    /// Used by `HTMLTableSectionElement::InsertRow` and `HTMLTableRowElement::InsertCell`
    pub(crate) fn insert_cell_or_row<F, G, I>(
        &self,
        index: i32,
        get_items: F,
        new_child: G,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<HTMLElement>>
    where
        F: Fn() -> DomRoot<HTMLCollection>,
        G: Fn() -> DomRoot<I>,
        I: DerivedFrom<Node> + DerivedFrom<HTMLElement> + DomObject,
    {
        if index < -1 {
            return Err(Error::IndexSize(None));
        }

        let tr = new_child();

        {
            let tr_node = tr.upcast::<Node>();
            if index == -1 {
                self.InsertBefore(tr_node, None, can_gc)?;
            } else {
                let items = get_items();
                let node = match items
                    .elements_iter()
                    .map(DomRoot::upcast::<Node>)
                    .map(Some)
                    .chain(iter::once(None))
                    .nth(index as usize)
                {
                    None => return Err(Error::IndexSize(None)),
                    Some(node) => node,
                };
                self.InsertBefore(tr_node, node.as_deref(), can_gc)?;
            }
        }

        Ok(DomRoot::upcast::<HTMLElement>(tr))
    }

    /// Used by `HTMLTableSectionElement::DeleteRow` and `HTMLTableRowElement::DeleteCell`
    pub(crate) fn delete_cell_or_row<F, G>(
        &self,
        index: i32,
        get_items: F,
        is_delete_type: G,
        can_gc: CanGc,
    ) -> ErrorResult
    where
        F: Fn() -> DomRoot<HTMLCollection>,
        G: Fn(&Element) -> bool,
    {
        let element = match index {
            index if index < -1 => return Err(Error::IndexSize(None)),
            -1 => {
                let last_child = self.upcast::<Node>().GetLastChild();
                match last_child.and_then(|node| {
                    node.inclusively_preceding_siblings()
                        .filter_map(DomRoot::downcast::<Element>)
                        .find(|elem| is_delete_type(elem))
                }) {
                    Some(element) => element,
                    None => return Ok(()),
                }
            },
            index => match get_items().Item(index as u32) {
                Some(element) => element,
                None => return Err(Error::IndexSize(None)),
            },
        };

        element.upcast::<Node>().remove_self(can_gc);
        Ok(())
    }

    pub(crate) fn get_stylesheet(&self) -> Option<ServoArc<Stylesheet>> {
        if let Some(node) = self.downcast::<HTMLStyleElement>() {
            node.get_stylesheet()
        } else if let Some(node) = self.downcast::<HTMLLinkElement>() {
            node.get_stylesheet()
        } else {
            None
        }
    }

    pub(crate) fn get_cssom_stylesheet(&self) -> Option<DomRoot<CSSStyleSheet>> {
        if let Some(node) = self.downcast::<HTMLStyleElement>() {
            node.get_cssom_stylesheet()
        } else if let Some(node) = self.downcast::<HTMLLinkElement>() {
            node.get_cssom_stylesheet(CanGc::note())
        } else {
            None
        }
    }

    pub(crate) fn is_styled(&self) -> bool {
        self.style_data.borrow().is_some()
    }

    pub(crate) fn is_display_none(&self) -> bool {
        self.style_data.borrow().as_ref().is_none_or(|data| {
            data.element_data
                .borrow()
                .styles
                .primary()
                .get_box()
                .display
                .is_none()
        })
    }

    pub(crate) fn style(&self) -> Option<ServoArc<ComputedValues>> {
        self.owner_window().layout_reflow(QueryMsg::StyleQuery);
        self.style_data
            .borrow()
            .as_ref()
            .map(|data| data.element_data.borrow().styles.primary().clone())
    }

    /// <https://html.spec.whatwg.org/multipage/#language>
    pub(crate) fn get_lang(&self) -> Option<String> {
        self.inclusive_ancestors(ShadowIncluding::Yes)
            .find_map(|node| {
                node.downcast::<Element>().and_then(|el| {
                    el.get_attribute(&ns!(xml), &local_name!("lang"))
                        .or_else(|| el.get_attribute(&ns!(), &local_name!("lang")))
                        .map(|attr| String::from(attr.Value()))
                })
                // TODO: Check meta tags for a pragma-set default language
                // TODO: Check HTTP Content-Language header
            })
    }

    /// <https://dom.spec.whatwg.org/#assign-slotables-for-a-tree>
    pub(crate) fn assign_slottables_for_a_tree(&self, force: ForceSlottableNodeReconciliation) {
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
        for node in self.traverse_preorder(ShadowIncluding::No) {
            if let Some(slot) = node.downcast::<HTMLSlotElement>() {
                slot.assign_slottables();
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
    /// The returned node is the node's assigned slot, if any, or the
    /// shadow host if it's a shadow root. Otherwise, it is the node's
    /// parent.
    pub(crate) fn parent_in_flat_tree(&self) -> Option<DomRoot<Node>> {
        if let Some(assigned_slot) = self.assigned_slot() {
            return Some(DomRoot::upcast(assigned_slot));
        }

        let parent_or_none = self.GetParentNode();
        if let Some(parent) = parent_or_none.as_deref() {
            if let Some(shadow_root) = parent.downcast::<ShadowRoot>() {
                return Some(DomRoot::from_ref(shadow_root.Host().upcast::<Node>()));
            }
        }

        parent_or_none
    }

    pub(crate) fn inclusive_ancestors_in_flat_tree(
        &self,
    ) -> impl Iterator<Item = DomRoot<Node>> + use<> {
        SimpleNodeIterator {
            current: Some(DomRoot::from_ref(self)),
            next_node: move |n| n.parent_in_flat_tree(),
        }
    }

    /// We are marking this as an implemented pseudo element.
    pub(crate) fn set_implemented_pseudo_element(&self, pseudo_element: PseudoElement) {
        // Implemented pseudo element should exist only in the UA shadow DOM.
        debug_assert!(self.is_in_ua_widget());
        self.ensure_rare_data().implemented_pseudo_element = Some(pseudo_element);
    }

    pub(crate) fn implemented_pseudo_element(&self) -> Option<PseudoElement> {
        self.rare_data
            .borrow()
            .as_ref()
            .and_then(|rare_data| rare_data.implemented_pseudo_element)
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

#[expect(unsafe_code)]
pub(crate) trait LayoutNodeHelpers<'dom> {
    fn type_id_for_layout(self) -> NodeTypeId;

    fn parent_node_ref(self) -> Option<LayoutDom<'dom, Node>>;
    fn composed_parent_node_ref(self) -> Option<LayoutDom<'dom, Node>>;
    fn first_child_ref(self) -> Option<LayoutDom<'dom, Node>>;
    fn last_child_ref(self) -> Option<LayoutDom<'dom, Node>>;
    fn prev_sibling_ref(self) -> Option<LayoutDom<'dom, Node>>;
    fn next_sibling_ref(self) -> Option<LayoutDom<'dom, Node>>;

    fn owner_doc_for_layout(self) -> LayoutDom<'dom, Document>;
    fn containing_shadow_root_for_layout(self) -> Option<LayoutDom<'dom, ShadowRoot>>;
    fn assigned_slot_for_layout(self) -> Option<LayoutDom<'dom, HTMLSlotElement>>;

    fn is_element_for_layout(&self) -> bool;
    fn is_text_node_for_layout(&self) -> bool;
    unsafe fn get_flag(self, flag: NodeFlags) -> bool;
    unsafe fn set_flag(self, flag: NodeFlags, value: bool);

    fn style_data(self) -> Option<&'dom StyleData>;
    fn layout_data(self) -> Option<&'dom GenericLayoutData>;

    /// Initialize the style data of this node.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it modifies the given node during
    /// layout. Callers should ensure that no other layout thread is
    /// attempting to read or modify the opaque layout data of this node.
    unsafe fn initialize_style_data(self);

    /// Initialize the opaque layout data of this node.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it modifies the given node during
    /// layout. Callers should ensure that no other layout thread is
    /// attempting to read or modify the opaque layout data of this node.
    unsafe fn initialize_layout_data(self, data: Box<GenericLayoutData>);

    /// Clear the style and opaque layout data of this node.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it modifies the given node during
    /// layout. Callers should ensure that no other layout thread is
    /// attempting to read or modify the opaque layout data of this node.
    unsafe fn clear_style_and_layout_data(self);

    /// Whether this element serve as a container of editable text for a text input
    /// that is implemented as an UA widget.
    fn is_single_line_text_inner_editor(&self) -> bool;

    /// Whether this element serve as a container of any text inside a text input
    /// that is implemented as an UA widget.
    fn is_text_container_of_single_line_input(&self) -> bool;
    fn text_content(self) -> Cow<'dom, str>;
    fn selection(self) -> Option<SharedSelection>;
    fn image_url(self) -> Option<ServoUrl>;
    fn image_density(self) -> Option<f64>;
    fn image_data(self) -> Option<(Option<Image>, Option<ImageMetadata>)>;
    fn showing_broken_image_icon(self) -> bool;
    fn canvas_data(self) -> Option<HTMLCanvasData>;
    fn media_data(self) -> Option<HTMLMediaData>;
    fn svg_data(self) -> Option<SVGElementData<'dom>>;
    fn iframe_browsing_context_id(self) -> Option<BrowsingContextId>;
    fn iframe_pipeline_id(self) -> Option<PipelineId>;
    fn opaque(self) -> OpaqueNode;
    fn implemented_pseudo_element(&self) -> Option<PseudoElement>;
    fn is_in_ua_widget(&self) -> bool;
}

impl<'dom> LayoutDom<'dom, Node> {
    #[inline]
    #[expect(unsafe_code)]
    pub(crate) fn parent_node_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().parent_node.get_inner_as_layout() }
    }
}

impl<'dom> LayoutNodeHelpers<'dom> for LayoutDom<'dom, Node> {
    #[inline]
    fn type_id_for_layout(self) -> NodeTypeId {
        self.unsafe_get().type_id()
    }

    #[inline]
    fn is_element_for_layout(&self) -> bool {
        (*self).is::<Element>()
    }

    fn is_text_node_for_layout(&self) -> bool {
        self.type_id_for_layout() ==
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::Text))
    }

    #[inline]
    #[expect(unsafe_code)]
    fn parent_node_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().parent_node.get_inner_as_layout() }
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
    #[expect(unsafe_code)]
    fn first_child_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().first_child.get_inner_as_layout() }
    }

    #[inline]
    #[expect(unsafe_code)]
    fn last_child_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().last_child.get_inner_as_layout() }
    }

    #[inline]
    #[expect(unsafe_code)]
    fn prev_sibling_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().prev_sibling.get_inner_as_layout() }
    }

    #[inline]
    #[expect(unsafe_code)]
    fn next_sibling_ref(self) -> Option<LayoutDom<'dom, Node>> {
        unsafe { self.unsafe_get().next_sibling.get_inner_as_layout() }
    }

    #[inline]
    #[expect(unsafe_code)]
    fn owner_doc_for_layout(self) -> LayoutDom<'dom, Document> {
        unsafe { self.unsafe_get().owner_doc.get_inner_as_layout().unwrap() }
    }

    #[inline]
    #[expect(unsafe_code)]
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

    #[inline]
    #[expect(unsafe_code)]
    fn assigned_slot_for_layout(self) -> Option<LayoutDom<'dom, HTMLSlotElement>> {
        unsafe {
            self.unsafe_get()
                .rare_data
                .borrow_for_layout()
                .as_ref()?
                .slottable_data
                .assigned_slot
                .as_ref()
                .map(|assigned_slot| assigned_slot.to_layout())
        }
    }

    // FIXME(nox): get_flag/set_flag (especially the latter) are not safe because
    // they mutate stuff while values of this type can be used from multiple
    // threads at once, this should be revisited.

    #[inline]
    #[expect(unsafe_code)]
    unsafe fn get_flag(self, flag: NodeFlags) -> bool {
        (self.unsafe_get()).flags.get().contains(flag)
    }

    #[inline]
    #[expect(unsafe_code)]
    unsafe fn set_flag(self, flag: NodeFlags, value: bool) {
        let this = self.unsafe_get();
        let mut flags = (this).flags.get();

        if value {
            flags.insert(flag);
        } else {
            flags.remove(flag);
        }

        (this).flags.set(flags);
    }

    // FIXME(nox): How we handle style and layout data needs to be completely
    // revisited so we can do that more cleanly and safely in layout 2020.
    #[inline]
    #[expect(unsafe_code)]
    fn style_data(self) -> Option<&'dom StyleData> {
        unsafe { self.unsafe_get().style_data.borrow_for_layout().as_deref() }
    }

    #[inline]
    #[expect(unsafe_code)]
    fn layout_data(self) -> Option<&'dom GenericLayoutData> {
        unsafe { self.unsafe_get().layout_data.borrow_for_layout().as_deref() }
    }

    #[inline]
    #[expect(unsafe_code)]
    unsafe fn initialize_style_data(self) {
        let data = unsafe { self.unsafe_get().style_data.borrow_mut_for_layout() };
        debug_assert!(data.is_none());
        *data = Some(Box::default());
    }

    #[inline]
    #[expect(unsafe_code)]
    unsafe fn initialize_layout_data(self, new_data: Box<GenericLayoutData>) {
        let data = unsafe { self.unsafe_get().layout_data.borrow_mut_for_layout() };
        debug_assert!(data.is_none());
        *data = Some(new_data);
    }

    #[inline]
    #[expect(unsafe_code)]
    unsafe fn clear_style_and_layout_data(self) {
        unsafe {
            self.unsafe_get().style_data.borrow_mut_for_layout().take();
            self.unsafe_get().layout_data.borrow_mut_for_layout().take();
        }
    }

    fn is_single_line_text_inner_editor(&self) -> bool {
        matches!(
            self.unsafe_get().implemented_pseudo_element(),
            Some(PseudoElement::ServoTextControlInnerEditor)
        )
    }

    fn is_text_container_of_single_line_input(&self) -> bool {
        let is_single_line_text_inner_placeholder = matches!(
            self.unsafe_get().implemented_pseudo_element(),
            Some(PseudoElement::Placeholder)
        );
        // Currently `::placeholder` is only implemented for single line text input element.
        debug_assert!(
            !is_single_line_text_inner_placeholder ||
                self.containing_shadow_root_for_layout()
                    .map(|root| root.get_host_for_layout())
                    .map(|host| host.downcast::<HTMLInputElement>())
                    .is_some()
        );

        self.is_single_line_text_inner_editor() || is_single_line_text_inner_placeholder
    }

    fn text_content(self) -> Cow<'dom, str> {
        self.downcast::<Text>()
            .expect("Called LayoutDom::text_content on non-Text node!")
            .upcast()
            .data_for_layout()
            .into()
    }

    /// Get the selection for the given node. This only works for text nodes that are in
    /// the shadow DOM of user agent widgets for form controls, specifically for `<input>`
    /// and `<textarea>`.
    ///
    /// As we want to expose the selection on the inner text node of the widget's shadow
    /// DOM, we must find the shadow root and then access the containing element itself.
    fn selection(self) -> Option<SharedSelection> {
        if let Some(input) = self.downcast::<HTMLInputElement>() {
            return input.selection_for_layout();
        }
        if let Some(textarea) = self.downcast::<HTMLTextAreaElement>() {
            return Some(textarea.selection_for_layout());
        }

        let shadow_root = self
            .containing_shadow_root_for_layout()?
            .get_host_for_layout();
        if let Some(input) = shadow_root.downcast::<HTMLInputElement>() {
            return input.selection_for_layout();
        }
        shadow_root
            .downcast::<HTMLTextAreaElement>()
            .map(|textarea| textarea.selection_for_layout())
    }

    fn image_url(self) -> Option<ServoUrl> {
        self.downcast::<HTMLImageElement>()
            .expect("not an image!")
            .image_url()
    }

    fn image_data(self) -> Option<(Option<Image>, Option<ImageMetadata>)> {
        self.downcast::<HTMLImageElement>().map(|e| e.image_data())
    }

    fn image_density(self) -> Option<f64> {
        self.downcast::<HTMLImageElement>()
            .expect("not an image!")
            .image_density()
    }

    fn showing_broken_image_icon(self) -> bool {
        self.downcast::<HTMLImageElement>()
            .map(|image_element| image_element.showing_broken_image_icon())
            .unwrap_or_default()
    }

    fn canvas_data(self) -> Option<HTMLCanvasData> {
        self.downcast::<HTMLCanvasElement>()
            .map(|canvas| canvas.data())
    }

    fn media_data(self) -> Option<HTMLMediaData> {
        self.downcast::<HTMLVideoElement>()
            .map(|media| media.data())
    }

    fn svg_data(self) -> Option<SVGElementData<'dom>> {
        self.downcast::<SVGSVGElement>().map(|svg| svg.data())
    }

    fn iframe_browsing_context_id(self) -> Option<BrowsingContextId> {
        self.downcast::<HTMLIFrameElement>()
            .and_then(|iframe_element| iframe_element.browsing_context_id())
    }

    fn iframe_pipeline_id(self) -> Option<PipelineId> {
        self.downcast::<HTMLIFrameElement>()
            .and_then(|iframe_element| iframe_element.pipeline_id())
    }

    #[expect(unsafe_code)]
    fn opaque(self) -> OpaqueNode {
        unsafe { OpaqueNode(self.get_jsobject() as usize) }
    }

    fn implemented_pseudo_element(&self) -> Option<PseudoElement> {
        self.unsafe_get().implemented_pseudo_element()
    }

    fn is_in_ua_widget(&self) -> bool {
        self.unsafe_get().is_in_ua_widget()
    }
}

//
// Iteration and traversal
//

pub(crate) struct FollowingNodeIterator {
    current: Option<DomRoot<Node>>,
    root: DomRoot<Node>,
}

impl FollowingNodeIterator {
    /// Skips iterating the children of the current node
    pub(crate) fn next_skipping_children(&mut self) -> Option<DomRoot<Node>> {
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

    /// <https://dom.spec.whatwg.org/#concept-tree-following>
    fn next(&mut self) -> Option<DomRoot<Node>> {
        let current = self.current.take()?;

        if let Some(first_child) = current.GetFirstChild() {
            self.current = Some(first_child);
            return current.GetFirstChild();
        }

        self.next_skipping_children_impl(current)
    }
}

pub(crate) struct PrecedingNodeIterator {
    current: Option<DomRoot<Node>>,
    root: DomRoot<Node>,
}

impl Iterator for PrecedingNodeIterator {
    type Item = DomRoot<Node>;

    /// <https://dom.spec.whatwg.org/#concept-tree-preceding>
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
pub(crate) enum ShadowIncluding {
    No,
    Yes,
}

pub(crate) struct TreeIterator {
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

    pub(crate) fn next_skipping_children(&mut self) -> Option<DomRoot<Node>> {
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
            if let Some(shadow_root) = ancestor.downcast::<ShadowRoot>() {
                // Shadow roots don't have sibling, so after we're done traversing
                // one we jump to the first child of the host
                if let Some(child) = shadow_root.Host().upcast::<Node>().GetFirstChild() {
                    self.current = Some(child);
                    return Some(current);
                }
            }
            self.depth -= 1;
        }
        debug_assert_eq!(self.depth, 0);
        self.current = None;
        Some(current)
    }

    pub(crate) fn peek(&self) -> Option<&DomRoot<Node>> {
        self.current.as_ref()
    }
}

impl Iterator for TreeIterator {
    type Item = DomRoot<Node>;

    /// <https://dom.spec.whatwg.org/#concept-tree-order>
    /// <https://dom.spec.whatwg.org/#concept-shadow-including-tree-order>
    fn next(&mut self) -> Option<DomRoot<Node>> {
        let current = self.current.take()?;

        // Handle a potential shadow root on the element
        if let Some(element) = current.downcast::<Element>() {
            if let Some(shadow_root) = element.shadow_root() {
                if self.shadow_including {
                    self.current = Some(DomRoot::from_ref(shadow_root.upcast::<Node>()));
                    self.depth += 1;
                    return Some(current);
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

fn as_uintptr<T>(t: &T) -> uintptr_t {
    t as *const T as uintptr_t
}

impl Node {
    pub(crate) fn reflect_node<N>(node: Box<N>, document: &Document, can_gc: CanGc) -> DomRoot<N>
    where
        N: DerivedFrom<Node> + DomObject + DomObjectWrap<crate::DomTypeHolder>,
    {
        Self::reflect_node_with_proto(node, document, None, can_gc)
    }

    pub(crate) fn reflect_node_with_proto<N>(
        node: Box<N>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<N>
    where
        N: DerivedFrom<Node> + DomObject + DomObjectWrap<crate::DomTypeHolder>,
    {
        let window = document.window();
        reflect_dom_object_with_proto(node, window, proto, can_gc)
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
            style_data: Default::default(),
            layout_data: Default::default(),
        }
    }

    /// <https://dom.spec.whatwg.org/#concept-node-adopt>
    pub(crate) fn adopt(node: &Node, document: &Document, can_gc: CanGc) {
        document.add_script_and_layout_blocker();

        // Step 1. Let oldDocument be node’s node document.
        let old_doc = node.owner_doc();
        old_doc.add_script_and_layout_blocker();

        // Step 2. If node’s parent is non-null, then remove node.
        node.remove_self(can_gc);

        // Step 3. If document is not oldDocument:
        if &*old_doc != document {
            // Step 3.1. For each inclusiveDescendant in node’s shadow-including inclusive descendants:
            for descendant in node.traverse_preorder(ShadowIncluding::Yes) {
                // Step 3.1.1 Set inclusiveDescendant’s node document to document.
                descendant.set_owner_doc(document);

                // Step 3.1.2 If inclusiveDescendant is an element, then set the node document of each
                // attribute in inclusiveDescendant’s attribute list to document.
                if let Some(element) = descendant.downcast::<Element>() {
                    for attribute in element.attrs().iter() {
                        attribute.upcast::<Node>().set_owner_doc(document);
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
                    &descendant,
                    CallbackReaction::Adopted(old_doc.clone(), DomRoot::from_ref(document)),
                    None,
                );
            }

            // Step 3.3 For each inclusiveDescendant in node’s shadow-including inclusive descendants,
            // in shadow-including tree order, run the adopting steps with inclusiveDescendant and oldDocument.
            for descendant in node.traverse_preorder(ShadowIncluding::Yes) {
                vtable_for(&descendant).adopting_steps(&old_doc, can_gc);
            }
        }

        old_doc.remove_script_and_layout_blocker();
        document.remove_script_and_layout_blocker();
    }

    /// <https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity>
    pub(crate) fn ensure_pre_insertion_validity(
        node: &Node,
        parent: &Node,
        child: Option<&Node>,
    ) -> ErrorResult {
        // Step 1. If parent is not a Document, DocumentFragment, or Element node, then throw a "HierarchyRequestError" DOMException.
        match parent.type_id() {
            NodeTypeId::Document(_) | NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(..) => {
            },
            _ => return Err(Error::HierarchyRequest(None)),
        }

        // Step 2. If node is a host-including inclusive ancestor of parent, then throw a "HierarchyRequestError" DOMException.
        if node.is_host_including_inclusive_ancestor(parent) {
            return Err(Error::HierarchyRequest(None));
        }

        // Step 3. If child is non-null and its parent is not parent, then throw a "NotFoundError" DOMException.
        if let Some(child) = child {
            if !parent.is_parent_of(child) {
                return Err(Error::NotFound(None));
            }
        }

        // Step 4. If node is not a DocumentFragment, DocumentType, Element, or CharacterData node, then throw a "HierarchyRequestError" DOMException.
        match node.type_id() {
            // Step 5. If either node is a Text node and parent is a document,
            // or node is a doctype and parent is not a document, then throw a "HierarchyRequestError" DOMException.
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => {
                if parent.is::<Document>() {
                    return Err(Error::HierarchyRequest(None));
                }
            },
            NodeTypeId::DocumentType => {
                if !parent.is::<Document>() {
                    return Err(Error::HierarchyRequest(None));
                }
            },
            NodeTypeId::DocumentFragment(_) |
            NodeTypeId::Element(_) |
            NodeTypeId::CharacterData(CharacterDataTypeId::ProcessingInstruction) |
            NodeTypeId::CharacterData(CharacterDataTypeId::Comment) => (),
            NodeTypeId::Document(_) | NodeTypeId::Attr => {
                return Err(Error::HierarchyRequest(None));
            },
        }

        // Step 6. If parent is a document, and any of the statements below, switched on the interface node implements,
        // are true, then throw a "HierarchyRequestError" DOMException.
        if parent.is::<Document>() {
            match node.type_id() {
                NodeTypeId::DocumentFragment(_) => {
                    // Step 6."DocumentFragment". If node has more than one element child or has a Text node child.
                    if node.children().any(|c| c.is::<Text>()) {
                        return Err(Error::HierarchyRequest(None));
                    }
                    match node.child_elements().count() {
                        0 => (),
                        // Step 6."DocumentFragment". Otherwise, if node has one element child and either parent has an element child,
                        // child is a doctype, or child is non-null and a doctype is following child.
                        1 => {
                            if parent.child_elements().next().is_some() {
                                return Err(Error::HierarchyRequest(None));
                            }
                            if let Some(child) = child {
                                if child
                                    .inclusively_following_siblings()
                                    .any(|child| child.is_doctype())
                                {
                                    return Err(Error::HierarchyRequest(None));
                                }
                            }
                        },
                        _ => return Err(Error::HierarchyRequest(None)),
                    }
                },
                NodeTypeId::Element(_) => {
                    // Step 6."Element". parent has an element child, child is a doctype, or child is non-null and a doctype is following child.
                    if parent.child_elements().next().is_some() {
                        return Err(Error::HierarchyRequest(None));
                    }
                    if let Some(child) = child {
                        if child
                            .inclusively_following_siblings()
                            .any(|child| child.is_doctype())
                        {
                            return Err(Error::HierarchyRequest(None));
                        }
                    }
                },
                NodeTypeId::DocumentType => {
                    // Step 6."DocumentType". parent has a doctype child, child is non-null and an element is preceding child,
                    // or child is null and parent has an element child.
                    if parent.children().any(|c| c.is_doctype()) {
                        return Err(Error::HierarchyRequest(None));
                    }
                    match child {
                        Some(child) => {
                            if parent
                                .children()
                                .take_while(|c| &**c != child)
                                .any(|c| c.is::<Element>())
                            {
                                return Err(Error::HierarchyRequest(None));
                            }
                        },
                        None => {
                            if parent.child_elements().next().is_some() {
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
        node: &Node,
        parent: &Node,
        child: Option<&Node>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Node>> {
        // Step 1. Ensure pre-insert validity of node into parent before child.
        Node::ensure_pre_insertion_validity(node, parent, child)?;

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
            node,
            parent,
            reference_child,
            SuppressObserver::Unsuppressed,
            can_gc,
        );

        // Step 5. Return node.
        Ok(DomRoot::from_ref(node))
    }

    /// <https://dom.spec.whatwg.org/#concept-node-insert>
    fn insert(
        node: &Node,
        parent: &Node,
        child: Option<&Node>,
        suppress_observers: SuppressObserver,
        can_gc: CanGc,
    ) {
        debug_assert!(child.is_none_or(|child| Some(parent) == child.GetParentNode().as_deref()));

        // Step 1. Let nodes be node’s children, if node is a DocumentFragment node; otherwise « node ».
        rooted_vec!(let mut new_nodes);
        let new_nodes = if let NodeTypeId::DocumentFragment(_) = node.type_id() {
            new_nodes.extend(node.children().map(|node| Dom::from_ref(&*node)));
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
                Node::remove(kid, node, SuppressObserver::Suppressed, can_gc);
            }
            vtable_for(node)
                .children_changed(&ChildrenMutation::replace_all(new_nodes, &[]), can_gc);

            // Step 4.2. Queue a tree mutation record for node with « », nodes, null, and null.
            let mutation = LazyCell::new(|| Mutation::ChildList {
                added: None,
                removed: Some(new_nodes),
                prev: None,
                next: None,
            });
            MutationObserver::queue_a_mutation_record(node, mutation);
        }

        // Step 5. If child is non-null:
        //     1. For each live range whose start node is parent and start offset is
        //        greater than child’s index, increase its start offset by count.
        //     2. For each live range whose end node is parent and end offset is
        //        greater than child’s index, increase its end offset by count.
        if let Some(child) = child {
            if !parent.ranges_is_empty() {
                parent
                    .ranges()
                    .increase_above(parent, child.index(), count.try_into().unwrap());
            }
        }

        // Step 6. Let previousSibling be child’s previous sibling or parent’s last child if child is null.
        let previous_sibling = match suppress_observers {
            SuppressObserver::Unsuppressed => match child {
                Some(child) => child.GetPreviousSibling(),
                None => parent.GetLastChild(),
            },
            SuppressObserver::Suppressed => None,
        };

        let custom_element_reaction_stack = ScriptThread::custom_element_reaction_stack();

        // Step 10. Let staticNodeList be a list of nodes, initially « ».
        let mut static_node_list: SmallVec<[_; 4]> = Default::default();

        let parent_shadow_root = parent.downcast::<Element>().and_then(Element::shadow_root);
        let parent_in_shadow_tree = parent.is_in_a_shadow_tree();
        let parent_as_slot = parent.downcast::<HTMLSlotElement>();

        // Step 7. For each node in nodes, in tree order:
        for kid in new_nodes {
            // Step 7.1. Adopt node into parent’s node document.
            Node::adopt(kid, &parent.owner_document(), can_gc);

            // Step 7.2. If child is null, then append node to parent’s children.
            // Step 7.3. Otherwise, insert node into parent’s children before child’s index.
            parent.add_child(kid, child, can_gc);

            // Step 7.4 If parent is a shadow host whose shadow root’s slot assignment is "named"
            // and node is a slottable, then assign a slot for node.
            if let Some(ref shadow_root) = parent_shadow_root {
                if shadow_root.SlotAssignment() == SlotAssignmentMode::Named {
                    let cx = GlobalScope::get_cx();
                    if kid.is::<Element>() || kid.is::<Text>() {
                        rooted!(in(*cx) let slottable = Slottable(Dom::from_ref(kid)));
                        slottable.assign_a_slot();
                    }
                }
            }

            // Step 7.5 If parent’s root is a shadow root, and parent is a slot whose assigned nodes
            // is the empty list, then run signal a slot change for parent.
            if parent_in_shadow_tree {
                if let Some(slot_element) = parent_as_slot {
                    if !slot_element.has_assigned_nodes() {
                        slot_element.signal_a_slot_change();
                    }
                }
            }

            // Step 7.6 Run assign slottables for a tree with node’s root.
            kid.GetRootNode(&GetRootNodeOptions::empty())
                .assign_slottables_for_a_tree(ForceSlottableNodeReconciliation::Skip);

            // Step 7.7. For each shadow-including inclusive descendant inclusiveDescendant of node,
            // in shadow-including tree order:
            for descendant in kid.traverse_preorder(ShadowIncluding::Yes) {
                // Step 11.1 For each shadow-including inclusive descendant inclusiveDescendant of node,
                //           in shadow-including tree order, append inclusiveDescendant to staticNodeList.
                if descendant.is_connected() {
                    static_node_list.push(descendant.clone());
                }

                // Step 7.7.1. Run the insertion steps with inclusiveDescendant.
                // This is done in `parent.add_child()`.

                // Step 7.7.2, whatwg/dom#833
                // Enqueue connected reactions for custom elements or try upgrade.
                if let Some(descendant) = DomRoot::downcast::<Element>(descendant) {
                    if descendant.is_custom() {
                        if descendant.is_connected() {
                            custom_element_reaction_stack.enqueue_callback_reaction(
                                &descendant,
                                CallbackReaction::Connected,
                                None,
                            );
                        }
                    } else {
                        try_upgrade_element(&descendant);
                    }
                }
            }
        }

        if let SuppressObserver::Unsuppressed = suppress_observers {
            // Step 9. Run the children changed steps for parent.
            // TODO(xiaochengh): If we follow the spec and move it out of the if block, some WPT fail. Investigate.
            vtable_for(parent).children_changed(
                &ChildrenMutation::insert(previous_sibling.as_deref(), new_nodes, child),
                can_gc,
            );

            // Step 8. If suppress observers flag is unset, then queue a tree mutation record for parent
            // with nodes, « », previousSibling, and child.
            let mutation = LazyCell::new(|| Mutation::ChildList {
                added: Some(new_nodes),
                removed: None,
                prev: previous_sibling.as_deref(),
                next: child,
            });
            MutationObserver::queue_a_mutation_record(parent, mutation);
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
                for node in static_node_list {
                    vtable_for(&node).post_connection_steps(CanGc::from_cx(cx));
                }
            }),
        );

        parent_document.remove_script_and_layout_blocker();
        from_document.remove_script_and_layout_blocker();
    }

    /// <https://dom.spec.whatwg.org/#concept-node-replace-all>
    pub(crate) fn replace_all(node: Option<&Node>, parent: &Node, can_gc: CanGc) {
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
            Node::remove(child, parent, SuppressObserver::Suppressed, can_gc);
        }

        // Step 6. If node is non-null, then insert node into parent before null with suppressObservers set to true.
        if let Some(node) = node {
            Node::insert(node, parent, None, SuppressObserver::Suppressed, can_gc);
        }

        vtable_for(parent).children_changed(
            &ChildrenMutation::replace_all(removed_nodes.r(), added_nodes),
            can_gc,
        );

        // Step 7. If either addedNodes or removedNodes is not empty, then queue a tree mutation record
        // for parent with addedNodes, removedNodes, null, and null.
        if !removed_nodes.is_empty() || !added_nodes.is_empty() {
            let mutation = LazyCell::new(|| Mutation::ChildList {
                added: Some(added_nodes),
                removed: Some(removed_nodes.r()),
                prev: None,
                next: None,
            });
            MutationObserver::queue_a_mutation_record(parent, mutation);
        }
        parent.owner_doc().remove_script_and_layout_blocker();
    }

    /// <https://dom.spec.whatwg.org/multipage/#string-replace-all>
    pub(crate) fn string_replace_all(string: DOMString, parent: &Node, can_gc: CanGc) {
        if string.is_empty() {
            Node::replace_all(None, parent, can_gc);
        } else {
            let text = Text::new(string, &parent.owner_document(), can_gc);
            Node::replace_all(Some(text.upcast::<Node>()), parent, can_gc);
        };
    }

    /// <https://dom.spec.whatwg.org/#concept-node-pre-remove>
    fn pre_remove(child: &Node, parent: &Node, can_gc: CanGc) -> Fallible<DomRoot<Node>> {
        // Step 1.
        match child.GetParentNode() {
            Some(ref node) if &**node != parent => return Err(Error::NotFound(None)),
            None => return Err(Error::NotFound(None)),
            _ => (),
        }

        // Step 2.
        Node::remove(child, parent, SuppressObserver::Unsuppressed, can_gc);

        // Step 3.
        Ok(DomRoot::from_ref(child))
    }

    /// <https://dom.spec.whatwg.org/#concept-node-remove>
    fn remove(node: &Node, parent: &Node, suppress_observers: SuppressObserver, can_gc: CanGc) {
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
        parent.remove_child(node, cached_index, can_gc);

        // Step 8. If node is assigned, then run assign slottables for node’s assigned slot.
        if let Some(slot) = node.assigned_slot() {
            slot.assign_slottables();
        }

        // Step 9. If parent’s root is a shadow root, and parent is a slot whose assigned nodes is the empty list,
        // then run signal a slot change for parent.
        if parent.is_in_a_shadow_tree() {
            if let Some(slot_element) = parent.downcast::<HTMLSlotElement>() {
                if !slot_element.has_assigned_nodes() {
                    slot_element.signal_a_slot_change();
                }
            }
        }

        // Step 10. If node has an inclusive descendant that is a slot:
        let has_slot_descendant = node
            .traverse_preorder(ShadowIncluding::No)
            .any(|elem| elem.is::<HTMLSlotElement>());
        if has_slot_descendant {
            // Step 10.1 Run assign slottables for a tree with parent’s root.
            parent
                .GetRootNode(&GetRootNodeOptions::empty())
                .assign_slottables_for_a_tree(ForceSlottableNodeReconciliation::Skip);

            // Step 10.2 Run assign slottables for a tree with node.
            node.assign_slottables_for_a_tree(ForceSlottableNodeReconciliation::Force);
        }

        // TODO: Step 15. transient registered observers

        // Step 16.
        if let SuppressObserver::Unsuppressed = suppress_observers {
            vtable_for(parent).children_changed(
                &ChildrenMutation::replace(
                    old_previous_sibling.as_deref(),
                    &Some(node),
                    &[],
                    old_next_sibling.as_deref(),
                ),
                can_gc,
            );

            let removed = [node];
            let mutation = LazyCell::new(|| Mutation::ChildList {
                added: None,
                removed: Some(&removed),
                prev: old_previous_sibling.as_deref(),
                next: old_next_sibling.as_deref(),
            });
            MutationObserver::queue_a_mutation_record(parent, mutation);
        }
        parent.owner_doc().remove_script_and_layout_blocker();
    }

    /// <https://dom.spec.whatwg.org/#live-range-pre-remove-steps>
    fn live_range_pre_remove_steps(node: &Node, parent: &Node) -> Option<u32> {
        if parent.ranges_is_empty() {
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
        parent.ranges().decrease_above(parent, index, 1);

        // Parent had ranges, we needed the index, let's keep track of
        // it to avoid computing it for other ranges when calling
        // unbind_from_tree recursively.
        Some(index)
    }

    /// Ensure that for styles, we clone the already-parsed property declaration block.
    /// This does two things:
    /// 1. it uses the same fast-path as CSSStyleDeclaration
    /// 2. it also avoids the CSP checks when cloning (it shouldn't run any when cloning
    ///    existing valid attributes)
    fn compute_attribute_value_with_style_fast_path(attr: &Dom<Attr>, elem: &Element) -> AttrValue {
        if *attr.local_name() == local_name!("style") {
            if let Some(ref pdb) = *elem.style_attribute().borrow() {
                let document = elem.owner_document();
                let shared_lock = document.style_shared_lock();
                let new_pdb = pdb.read_with(&shared_lock.read()).clone();
                return AttrValue::Declaration(
                    (**attr.value()).to_owned(),
                    ServoArc::new(shared_lock.wrap(new_pdb)),
                );
            }
        }

        attr.value().clone()
    }

    /// <https://dom.spec.whatwg.org/#concept-node-clone>
    pub(crate) fn clone(
        node: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
        registry: Option<DomRoot<CustomElementRegistry>>,
        can_gc: CanGc,
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
                    doctype.name().clone(),
                    Some(doctype.public_id().clone()),
                    Some(doctype.system_id().clone()),
                    &document,
                    can_gc,
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
                    can_gc,
                );
                DomRoot::upcast::<Node>(attr)
            },
            NodeTypeId::DocumentFragment(_) => {
                let doc_fragment = DocumentFragment::new(&document, can_gc);
                DomRoot::upcast::<Node>(doc_fragment)
            },
            NodeTypeId::CharacterData(_) => {
                let cdata = node.downcast::<CharacterData>().unwrap();
                cdata.clone_with_data(cdata.Data(), &document, can_gc)
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
                    can_gc,
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
                        Some(document.custom_element_registry())
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
                    name,
                    element.get_is(),
                    &document,
                    ElementCreator::ScriptCreated,
                    CustomElementCreationMode::Asynchronous,
                    None,
                    can_gc,
                );
                // TODO: Move this into `Element::create`
                element.set_custom_element_registry(registry);
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
                for attr in node_elem.attrs().iter() {
                    // Step 2.5.1. Let copyAttribute be the result of cloning a single node given attribute, document, and null.
                    let new_value =
                        Node::compute_attribute_value_with_style_fast_path(attr, node_elem);
                    // Step 2.5.2. Append copyAttribute to copy.
                    copy_elem.push_new_attribute(
                        attr.local_name().clone(),
                        new_value,
                        attr.name().clone(),
                        attr.namespace().clone(),
                        attr.prefix().cloned(),
                        AttributeMutationReason::ByCloning,
                        can_gc,
                    );
                }
            },
            _ => (),
        }

        // Step 5: Run any cloning steps defined for node in other applicable specifications and pass copy,
        // node, document, and the clone children flag if set, as parameters.
        vtable_for(node).cloning_steps(&copy, maybe_doc, clone_children, can_gc);

        // Step 6. If the clone children flag is set, then for each child child of node, in tree order: append the
        // result of cloning child with document and the clone children flag set, to copy.
        if clone_children == CloneChildrenFlag::CloneChildren {
            for child in node.children() {
                let child_copy = Node::clone(&child, Some(&document), clone_children, None, can_gc);
                let _inserted_node = Node::pre_insert(&child_copy, &copy, None, can_gc);
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
                        IsUserAgentWidget::No,
                        shadow_root.Mode(),
                        shadow_root.Clonable(),
                        shadow_root.Serializable(),
                        shadow_root.DelegatesFocus(),
                        shadow_root.SlotAssignment(),
                        can_gc
                    )
                    .expect("placement of attached shadow root must be valid, as this is a copy of an existing one");

                // Step 7.3 Set copy’s shadow root’s declarative to node’s shadow root’s declarative.
                copy_shadow_root.set_declarative(shadow_root.is_declarative());

                // Step 7.4 For each child child of node’s shadow root, in tree order: append the result of
                // cloning child with document and the clone children flag set, to copy’s shadow root.
                for child in shadow_root.upcast::<Node>().children() {
                    let child_copy = Node::clone(
                        &child,
                        Some(&document),
                        CloneChildrenFlag::CloneChildren,
                        None,
                        can_gc,
                    );

                    // TODO: Should we handle the error case here and in step 6?
                    let _inserted_node = Node::pre_insert(
                        &child_copy,
                        copy_shadow_root.upcast::<Node>(),
                        None,
                        can_gc,
                    );
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
    pub(crate) fn set_text_content_for_element(&self, value: Option<DOMString>, can_gc: CanGc) {
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
            Some(DomRoot::upcast(
                self.owner_doc().CreateTextNode(value, can_gc),
            ))
        };

        // Step 3. Replace all with node within parent.
        Self::replace_all(node.as_deref(), self, can_gc);
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
        traversal_scope: html_serialize::TraversalScope,
        serialize_shadow_roots: bool,
        shadow_roots: Vec<DomRoot<ShadowRoot>>,
        can_gc: CanGc,
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
            self,
            &mut serializer,
            traversal_scope,
            serialize_shadow_roots,
            shadow_roots,
            can_gc,
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
        require_well_formed: bool,
        can_gc: CanGc,
    ) -> Fallible<DOMString> {
        // Step 1. Let context document be node's node document.
        let context_document = self.owner_document();

        // Step 2. If context document is an HTML document, return the result of HTML fragment serialization algorithm
        // with node, false, and « ».
        if context_document.is_html_document() {
            return Ok(self.html_serialize(
                html_serialize::TraversalScope::ChildrenOnly(None),
                false,
                vec![],
                can_gc,
            ));
        }

        // Step 3. Return the XML serialization of node given require well-formed.
        // TODO: xml5ever doesn't seem to want require_well_formed
        let _ = require_well_formed;
        self.xml_serialize(xml_serialize::TraversalScope::ChildrenOnly(None))
    }

    /// Return true if this node establishes a "scrolling box" for the purposes of `scrollIntoView`.
    pub(crate) fn establishes_scrolling_box(&self) -> bool {
        // For now, `Document` represents the viewport.
        //
        // TODO: Is this the right thing to do? Maybe `Document` should be ignored and viewport
        // should be represented by the root of the DOM flat tree.
        if self.is::<Document>() {
            return true;
        }
        let Some(element) = self.downcast::<Element>() else {
            // Shadow roots and other nodes are not scrolling boxes.
            return false;
        };
        // TODO: This should ask layout whether or not the element establishes a scrolling
        // box. This heuristic is wrong.
        element.style().is_some_and(|style| {
            let overflow_x = style.get_box().clone_overflow_x();
            let overflow_y = style.get_box().clone_overflow_y();
            overflow_x.is_scrollable() || overflow_y.is_scrollable()
        })
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
        if !options.composed {
            if let Some(shadow_root) = self.containing_shadow_root() {
                return DomRoot::upcast(shadow_root);
            }
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
        self.parent_node.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-parentelement>
    fn GetParentElement(&self) -> Option<DomRoot<Element>> {
        self.GetParentNode().and_then(DomRoot::downcast)
    }

    /// <https://dom.spec.whatwg.org/#dom-node-haschildnodes>
    fn HasChildNodes(&self) -> bool {
        self.first_child.get().is_some()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-childnodes>
    fn ChildNodes(&self, can_gc: CanGc) -> DomRoot<NodeList> {
        if let Some(list) = self.ensure_rare_data().child_list.get() {
            return list;
        }

        let doc = self.owner_doc();
        let window = doc.window();
        let list = NodeList::new_child_list(window, self, can_gc);
        self.ensure_rare_data().child_list.set(Some(&list));
        list
    }

    /// <https://dom.spec.whatwg.org/#dom-node-firstchild>
    fn GetFirstChild(&self) -> Option<DomRoot<Node>> {
        self.first_child.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-lastchild>
    fn GetLastChild(&self) -> Option<DomRoot<Node>> {
        self.last_child.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-previoussibling>
    fn GetPreviousSibling(&self) -> Option<DomRoot<Node>> {
        self.prev_sibling.get()
    }

    /// <https://dom.spec.whatwg.org/#dom-node-nextsibling>
    fn GetNextSibling(&self) -> Option<DomRoot<Node>> {
        self.next_sibling.get()
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
    fn SetNodeValue(&self, val: Option<DOMString>, can_gc: CanGc) -> Fallible<()> {
        match self.type_id() {
            NodeTypeId::Attr => {
                let attr = self.downcast::<Attr>().unwrap();
                attr.SetValue(val.unwrap_or_default(), can_gc)?;
            },
            NodeTypeId::CharacterData(_) => {
                let character_data = self.downcast::<CharacterData>().unwrap();
                character_data.SetData(val.unwrap_or_default());
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
    fn SetTextContent(&self, value: Option<DOMString>, can_gc: CanGc) -> Fallible<()> {
        match self.type_id() {
            NodeTypeId::DocumentFragment(_) | NodeTypeId::Element(..) => {
                self.set_text_content_for_element(value, can_gc);
            },
            NodeTypeId::Attr => {
                let attr = self.downcast::<Attr>().unwrap();
                attr.SetValue(value.unwrap_or_default(), can_gc)?;
            },
            NodeTypeId::CharacterData(..) => {
                let characterdata = self.downcast::<CharacterData>().unwrap();
                characterdata.SetData(value.unwrap_or_default());
            },
            NodeTypeId::DocumentType | NodeTypeId::Document(_) => {},
        };
        Ok(())
    }

    /// <https://dom.spec.whatwg.org/#dom-node-insertbefore>
    fn InsertBefore(
        &self,
        node: &Node,
        child: Option<&Node>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<Node>> {
        Node::pre_insert(node, self, child, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-node-appendchild>
    fn AppendChild(&self, node: &Node, can_gc: CanGc) -> Fallible<DomRoot<Node>> {
        Node::pre_insert(node, self, None, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#concept-node-replace>
    fn ReplaceChild(&self, node: &Node, child: &Node, can_gc: CanGc) -> Fallible<DomRoot<Node>> {
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
                    if node.children().any(|c| c.is::<Text>()) {
                        return Err(Error::HierarchyRequest(None));
                    }
                    match node.child_elements().count() {
                        0 => (),
                        // Step 6.1.2
                        1 => {
                            if self.child_elements().any(|c| c.upcast::<Node>() != child) {
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
                    if self.child_elements().any(|c| c.upcast::<Node>() != child) {
                        return Err(Error::HierarchyRequest(None));
                    }
                    if child.following_siblings().any(|child| child.is_doctype()) {
                        return Err(Error::HierarchyRequest(None));
                    }
                },
                // Step 6.3
                NodeTypeId::DocumentType => {
                    if self.children().any(|c| c.is_doctype() && &*c != child) {
                        return Err(Error::HierarchyRequest(None));
                    }
                    if self
                        .children()
                        .take_while(|c| &**c != child)
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
        Node::adopt(node, &document, can_gc);

        // Step 10. Let removedNodes be the empty set.
        // Step 11. If child’s parent is non-null:
        //     1. Set removedNodes to « child ».
        //     2. Remove child with the suppress observers flag set.
        let removed_child = if node != child {
            // Step 11.
            Node::remove(child, self, SuppressObserver::Suppressed, can_gc);
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
            node,
            self,
            reference_child,
            SuppressObserver::Suppressed,
            can_gc,
        );

        vtable_for(self).children_changed(
            &ChildrenMutation::replace(
                previous_sibling.as_deref(),
                &removed_child,
                nodes,
                reference_child,
            ),
            can_gc,
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

        MutationObserver::queue_a_mutation_record(self, mutation);

        // Step 15. Return child.
        Ok(DomRoot::from_ref(child))
    }

    /// <https://dom.spec.whatwg.org/#dom-node-removechild>
    fn RemoveChild(&self, node: &Node, can_gc: CanGc) -> Fallible<DomRoot<Node>> {
        Node::pre_remove(node, self, can_gc)
    }

    /// <https://dom.spec.whatwg.org/#dom-node-normalize>
    fn Normalize(&self, can_gc: CanGc) {
        let mut children = self.children().enumerate().peekable();
        while let Some((_, node)) = children.next() {
            if let Some(text) = node.downcast::<Text>() {
                if text.is::<CDATASection>() {
                    continue;
                }
                let cdata = text.upcast::<CharacterData>();
                let mut length = cdata.Length();
                if length == 0 {
                    Node::remove(&node, self, SuppressObserver::Unsuppressed, can_gc);
                    continue;
                }
                while children.peek().is_some_and(|(_, sibling)| {
                    sibling.is::<Text>() && !sibling.is::<CDATASection>()
                }) {
                    let (index, sibling) = children.next().unwrap();
                    sibling
                        .ranges()
                        .drain_to_preceding_text_sibling(&sibling, &node, length);
                    self.ranges()
                        .move_to_text_child_at(self, index as u32, &node, length);
                    let sibling_cdata = sibling.downcast::<CharacterData>().unwrap();
                    length += sibling_cdata.Length();
                    cdata.append_data(&sibling_cdata.data());
                    Node::remove(&sibling, self, SuppressObserver::Unsuppressed, can_gc);
                }
            } else {
                node.Normalize(can_gc);
            }
        }
    }

    /// <https://dom.spec.whatwg.org/#dom-node-clonenode>
    fn CloneNode(&self, subtree: bool, can_gc: CanGc) -> Fallible<DomRoot<Node>> {
        // Step 1. If this is a shadow root, then throw a "NotSupportedError" DOMException.
        if self.is::<ShadowRoot>() {
            return Err(Error::NotSupported(None));
        }

        // Step 2. Return the result of cloning a node given this with subtree set to subtree.
        let result = Node::clone(
            self,
            None,
            if subtree {
                CloneChildrenFlag::CloneChildren
            } else {
                CloneChildrenFlag::DoNotCloneChildren
            },
            None,
            can_gc,
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
        if let Some(node2) = node2 {
            if Some(node2) == node1 {
                if let (Some(a1), Some(a2)) = (attr1, attr2) {
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
                }
            }
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

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
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

    fn children_changed(&self, mutation: &ChildrenMutation, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.children_changed(mutation, can_gc);
        }

        if let Some(data) = self.rare_data().as_ref() {
            if let Some(list) = data.child_list.get() {
                list.as_children_list().children_changed(mutation);
            }
        }

        self.owner_doc().content_and_heritage_changed(self);
    }

    // This handles the ranges mentioned in steps 2-3 when removing a node.
    /// <https://dom.spec.whatwg.org/#concept-node-remove>
    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);

        // Ranges should only drain to the parent from inclusive non-shadow
        // including descendants. If we're in a shadow tree at this point then the
        // unbind operation happened further up in the tree and we should not
        // drain any ranges.
        if !self.is_in_a_shadow_tree() && !self.ranges_is_empty() {
            self.ranges()
                .drain_to_parent(context.parent, context.index(), self);
        }
    }

    fn moving_steps(&self, context: &MoveContext, can_gc: CanGc) {
        if let Some(super_type) = self.super_type() {
            super_type.moving_steps(context, can_gc);
        }

        // Ranges should only drain to the parent from inclusive non-shadow
        // including descendants. If we're in a shadow tree at this point then the
        // unbind operation happened further up in the tree and we should not
        // drain any ranges.
        if let Some(old_parent) = context.old_parent &&
            !self.is_in_a_shadow_tree() &&
            !self.ranges_is_empty()
        {
            self.ranges()
                .drain_to_parent(old_parent, context.index(), self);
        }
    }

    fn handle_event(&self, event: &Event, _: CanGc) {
        if let Some(event) = event.downcast::<KeyboardEvent>() {
            self.owner_document()
                .event_handler()
                .run_default_keyboard_event_handler(event);
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

pub(crate) enum ChildrenMutation<'a> {
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
                added,
            },
            (Some(prev), None) => ChildrenMutation::Append { prev, added },
            (None, Some(next)) => ChildrenMutation::Prepend { added, next },
            (Some(prev), Some(next)) => ChildrenMutation::Insert { prev, added, next },
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
                    added,
                }
            } else {
                ChildrenMutation::Replace {
                    prev,
                    removed,
                    added,
                    next,
                }
            }
        } else {
            ChildrenMutation::insert(prev, added, next)
        }
    }

    fn replace_all(removed: &'a [&'a Node], added: &'a [&'a Node]) -> ChildrenMutation<'a> {
        ChildrenMutation::ReplaceAll { removed, added }
    }

    /// Get the child that follows the added or removed children.
    /// Currently only used when this mutation might force us to
    /// restyle later children (see HAS_SLOW_SELECTOR_LATER_SIBLINGS and
    /// Element's implementation of VirtualMethods::children_changed).
    pub(crate) fn next_child(&self) -> Option<&Node> {
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
    pub(crate) fn modified_edge_element(&self) -> Option<DomRoot<Node>> {
        match *self {
            // Add/remove at start of container: Return the first following element.
            ChildrenMutation::Prepend { next, .. } |
            ChildrenMutation::Replace {
                prev: None,
                next: Some(next),
                ..
            } => next
                .inclusively_following_siblings()
                .find(|node| node.is::<Element>()),
            // Add/remove at end of container: Return the last preceding element.
            ChildrenMutation::Append { prev, .. } |
            ChildrenMutation::Replace {
                prev: Some(prev),
                next: None,
                ..
            } => prev
                .inclusively_preceding_siblings()
                .find(|node| node.is::<Element>()),
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
                        .find(|node| node.is::<Element>())
                } else if next
                    .inclusively_following_siblings()
                    .all(|node| !node.is::<Element>())
                {
                    // After the last element: Return the last preceding element.
                    prev.inclusively_preceding_siblings()
                        .find(|node| node.is::<Element>())
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
pub(crate) struct BindContext<'a> {
    /// The parent of the inclusive ancestor that was inserted.
    pub(crate) parent: &'a Node,

    /// Whether the tree is connected.
    ///
    /// <https://dom.spec.whatwg.org/#connected>
    pub(crate) tree_connected: bool,

    /// Whether the tree's root is a document.
    ///
    /// <https://dom.spec.whatwg.org/#in-a-document-tree>
    pub(crate) tree_is_in_a_document_tree: bool,

    /// Whether the tree's root is a shadow root
    pub(crate) tree_is_in_a_shadow_tree: bool,

    /// Whether the root of the subtree that is being bound to the parent is a shadow root.
    ///
    /// This implies that all elements whose "bind_to_tree" method are called were already
    /// in a shadow tree beforehand.
    pub(crate) is_shadow_tree: IsShadowTree,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum IsShadowTree {
    Yes,
    No,
}

impl<'a> BindContext<'a> {
    /// Create a new `BindContext` value.
    pub(crate) fn new(parent: &'a Node, is_shadow_tree: IsShadowTree) -> Self {
        BindContext {
            parent,
            tree_connected: parent.is_connected(),
            tree_is_in_a_document_tree: parent.is_in_a_document_tree(),
            tree_is_in_a_shadow_tree: parent.is_in_a_shadow_tree(),
            is_shadow_tree,
        }
    }

    /// Return true iff the tree is inside either a document- or a shadow tree.
    pub(crate) fn is_in_tree(&self) -> bool {
        self.tree_is_in_a_document_tree || self.tree_is_in_a_shadow_tree
    }
}

/// The context of the unbinding from a tree of a node when one of its
/// inclusive ancestors is removed.
pub(crate) struct UnbindContext<'a> {
    /// The index of the inclusive ancestor that was removed.
    index: Cell<Option<u32>>,
    /// The parent of the inclusive ancestor that was removed.
    pub(crate) parent: &'a Node,
    /// The previous sibling of the inclusive ancestor that was removed.
    prev_sibling: Option<&'a Node>,
    /// The next sibling of the inclusive ancestor that was removed.
    pub(crate) next_sibling: Option<&'a Node>,

    /// Whether the tree is connected.
    ///
    /// <https://dom.spec.whatwg.org/#connected>
    pub(crate) tree_connected: bool,

    /// Whether the tree's root is a document.
    ///
    /// <https://dom.spec.whatwg.org/#in-a-document-tree>
    pub(crate) tree_is_in_a_document_tree: bool,

    /// Whether the tree's root is a shadow root
    pub(crate) tree_is_in_a_shadow_tree: bool,
}

impl<'a> UnbindContext<'a> {
    /// Create a new `UnbindContext` value.
    pub(crate) fn new(
        parent: &'a Node,
        prev_sibling: Option<&'a Node>,
        next_sibling: Option<&'a Node>,
        cached_index: Option<u32>,
    ) -> Self {
        UnbindContext {
            index: Cell::new(cached_index),
            parent,
            prev_sibling,
            next_sibling,
            tree_connected: parent.is_connected(),
            tree_is_in_a_document_tree: parent.is_in_a_document_tree(),
            tree_is_in_a_shadow_tree: parent.is_in_a_shadow_tree(),
        }
    }

    /// The index of the inclusive ancestor that was removed from the tree.
    pub(crate) fn index(&self) -> u32 {
        if let Some(index) = self.index.get() {
            return index;
        }
        let index = self.prev_sibling.map_or(0, |sibling| sibling.index() + 1);
        self.index.set(Some(index));
        index
    }
}

/// The context of the moving from a tree of a node when one of its
/// inclusive ancestors is moved.
pub(crate) struct MoveContext<'a> {
    /// The index of the inclusive ancestor that was moved.
    index: Cell<Option<u32>>,
    /// The old parent, if any, of the inclusive ancestor that was moved.
    pub(crate) old_parent: Option<&'a Node>,
    /// The previous sibling of the inclusive ancestor that was moved.
    prev_sibling: Option<&'a Node>,
    /// The next sibling of the inclusive ancestor that was moved.
    pub(crate) next_sibling: Option<&'a Node>,
}

impl<'a> MoveContext<'a> {
    /// Create a new `MoveContext` value.
    pub(crate) fn new(
        old_parent: Option<&'a Node>,
        prev_sibling: Option<&'a Node>,
        next_sibling: Option<&'a Node>,
        cached_index: Option<u32>,
    ) -> Self {
        MoveContext {
            index: Cell::new(cached_index),
            old_parent,
            prev_sibling,
            next_sibling,
        }
    }

    /// The index of the inclusive ancestor that was moved from the tree.
    pub(crate) fn index(&self) -> u32 {
        if let Some(index) = self.index.get() {
            return index;
        }
        let index = self.prev_sibling.map_or(0, |sibling| sibling.index() + 1);
        self.index.set(Some(index));
        index
    }
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
    fn new() -> UniqueId {
        UniqueId {
            cell: UnsafeCell::new(None),
        }
    }

    /// The Uuid of that unique ID.
    #[expect(unsafe_code)]
    fn borrow(&self) -> &Uuid {
        unsafe {
            let ptr = self.cell.get();
            if (*ptr).is_none() {
                *ptr = Some(Box::new(Uuid::new_v4()));
            }
            (*ptr).as_ref().unwrap()
        }
    }
}

pub(crate) struct NodeTypeIdWrapper(pub(crate) NodeTypeId);

impl From<NodeTypeIdWrapper> for LayoutNodeType {
    #[inline(always)]
    fn from(node_type: NodeTypeIdWrapper) -> LayoutNodeType {
        match node_type.0 {
            NodeTypeId::Element(e) => LayoutNodeType::Element(ElementTypeIdWrapper(e).into()),
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(_)) => LayoutNodeType::Text,
            x => unreachable!("Layout should not traverse nodes of type {:?}", x),
        }
    }
}

struct ElementTypeIdWrapper(ElementTypeId);

impl From<ElementTypeIdWrapper> for LayoutElementType {
    #[inline(always)]
    fn from(element_type: ElementTypeIdWrapper) -> LayoutElementType {
        match element_type.0 {
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
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptGroupElement) => {
                LayoutElementType::HTMLOptGroupElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement) => {
                LayoutElementType::HTMLOptionElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLObjectElement) => {
                LayoutElementType::HTMLObjectElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLParagraphElement) => {
                LayoutElementType::HTMLParagraphElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLPreElement) => {
                LayoutElementType::HTMLPreElement
            },
            ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement) => {
                LayoutElementType::HTMLSelectElement
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
                SVGGraphicsElementTypeId::SVGImageElement,
            )) => LayoutElementType::SVGImageElement,
            ElementTypeId::SVGElement(SVGElementTypeId::SVGGraphicsElement(
                SVGGraphicsElementTypeId::SVGSVGElement,
            )) => LayoutElementType::SVGSVGElement,
            _ => LayoutElementType::Element,
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
