/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::hash::Hash;
use std::sync::atomic::Ordering;

use atomic_refcell::{AtomicRef, AtomicRefMut};
use html5ever::{local_name, namespace_url, ns, LocalName, Namespace};
use script_layout_interface::wrapper_traits::{
    LayoutNode, PseudoElementType, ThreadSafeLayoutElement, ThreadSafeLayoutNode,
};
use script_layout_interface::{LayoutNodeType, StyleData};
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::bloom::{BloomFilter, BLOOM_HASH_MASK};
use selectors::matching::{ElementSelectorFlags, MatchingContext, VisitedHandlingMode};
use selectors::sink::Push;
use servo_arc::{Arc, ArcBorrow};
use servo_atoms::Atom;
use style::animation::AnimationSetKey;
use style::applicable_declarations::ApplicableDeclarationBlock;
use style::attr::AttrValue;
use style::bloom::each_relevant_element_hash;
use style::context::SharedStyleContext;
use style::data::ElementData;
use style::dom::{DomChildren, LayoutIterator, TDocument, TElement, TNode, TShadowRoot};
use style::properties::PropertyDeclarationBlock;
use style::selector_parser::{
    extended_filtering, AttrValue as SelectorAttrValue, Lang, NonTSPseudoClass, PseudoElement,
    SelectorImpl,
};
use style::shared_lock::Locked as StyleLocked;
use style::values::computed::Display;
use style::values::{AtomIdent, AtomString};
use style::CaseSensitivityExt;
use style_dom::ElementState;

use crate::dom::attr::AttrHelpersForLayout;
use crate::dom::bindings::inheritance::{
    CharacterDataTypeId, DocumentFragmentTypeId, ElementTypeId, HTMLElementTypeId, NodeTypeId,
    TextTypeId,
};
use crate::dom::bindings::root::LayoutDom;
use crate::dom::characterdata::LayoutCharacterDataHelpers;
use crate::dom::element::{Element, LayoutElementHelpers};
use crate::dom::node::{LayoutNodeHelpers, NodeFlags};
use crate::layout_dom::{ServoLayoutNode, ServoShadowRoot, ServoThreadSafeLayoutNode};

/// A wrapper around elements that ensures layout can only ever access safe properties.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ServoLayoutElement<'dom> {
    /// The wrapped private DOM Element.
    element: LayoutDom<'dom, Element>,
}

impl<'dom> fmt::Debug for ServoLayoutElement<'dom> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}", self.element.local_name())?;
        if let Some(id) = self.id() {
            write!(f, " id={}", id)?;
        }
        write!(f, "> ({:#x})", self.as_node().opaque().0)
    }
}

impl<'dom> ServoLayoutElement<'dom> {
    pub(super) fn from_layout_js(el: LayoutDom<'dom, Element>) -> Self {
        ServoLayoutElement { element: el }
    }

    pub(super) fn is_html_element(&self) -> bool {
        self.element.is_html_element()
    }

    #[inline]
    fn get_attr_enum(&self, namespace: &Namespace, name: &LocalName) -> Option<&AttrValue> {
        self.element.get_attr_for_layout(namespace, name)
    }

    #[inline]
    fn get_attr(&self, namespace: &Namespace, name: &LocalName) -> Option<&str> {
        self.element.get_attr_val_for_layout(namespace, name)
    }

    fn get_style_data(&self) -> Option<&StyleData> {
        self.as_node().style_data()
    }

    pub unsafe fn unset_snapshot_flags(&self) {
        self.as_node()
            .node
            .set_flag(NodeFlags::HAS_SNAPSHOT | NodeFlags::HANDLED_SNAPSHOT, false);
    }

    pub unsafe fn set_has_snapshot(&self) {
        self.as_node().node.set_flag(NodeFlags::HAS_SNAPSHOT, true);
    }

    /// Returns true if this element is the body child of an html element root element.
    fn is_body_element_of_html_element_root(&self) -> bool {
        if self.element.local_name() != &local_name!("body") {
            return false;
        }

        self.parent_element()
            .map(|element| {
                element.is_root() && element.element.local_name() == &local_name!("html")
            })
            .unwrap_or(false)
    }

    /// Returns the parent element of this element, if it has one.
    fn parent_element(&self) -> Option<Self> {
        self.element
            .upcast()
            .composed_parent_node_ref()
            .and_then(|node| node.downcast().map(ServoLayoutElement::from_layout_js))
    }

    fn is_root(&self) -> bool {
        match self.as_node().parent_node() {
            None => false,
            Some(node) => matches!(node.script_type_id(), NodeTypeId::Document(_)),
        }
    }
}

impl<'dom> style::dom::TElement for ServoLayoutElement<'dom> {
    type ConcreteNode = ServoLayoutNode<'dom>;
    type TraversalChildrenIterator = DomChildren<Self::ConcreteNode>;

    fn as_node(&self) -> ServoLayoutNode<'dom> {
        ServoLayoutNode::from_layout_js(self.element.upcast())
    }

    fn traversal_children(&self) -> LayoutIterator<Self::TraversalChildrenIterator> {
        LayoutIterator(if let Some(shadow) = self.shadow_root() {
            shadow.as_node().dom_children()
        } else {
            self.as_node().dom_children()
        })
    }

    fn is_html_element(&self) -> bool {
        ServoLayoutElement::is_html_element(self)
    }

    fn is_mathml_element(&self) -> bool {
        *self.element.namespace() == ns!(mathml)
    }

    fn is_svg_element(&self) -> bool {
        *self.element.namespace() == ns!(svg)
    }

    fn has_part_attr(&self) -> bool {
        false
    }

    fn exports_any_part(&self) -> bool {
        false
    }

    fn style_attribute(&self) -> Option<ArcBorrow<StyleLocked<PropertyDeclarationBlock>>> {
        unsafe {
            (*self.element.style_attribute())
                .as_ref()
                .map(|x| x.borrow_arc())
        }
    }

    fn may_have_animations(&self) -> bool {
        true
    }

    fn animation_rule(
        &self,
        context: &SharedStyleContext,
    ) -> Option<Arc<StyleLocked<PropertyDeclarationBlock>>> {
        let node = self.as_node();
        let document = node.owner_doc();
        context.animations.get_animation_declarations(
            &AnimationSetKey::new_for_non_pseudo(node.opaque()),
            context.current_time_for_animations,
            document.style_shared_lock(),
        )
    }

    fn transition_rule(
        &self,
        context: &SharedStyleContext,
    ) -> Option<Arc<StyleLocked<PropertyDeclarationBlock>>> {
        let node = self.as_node();
        let document = node.owner_doc();
        context.animations.get_transition_declarations(
            &AnimationSetKey::new_for_non_pseudo(node.opaque()),
            context.current_time_for_animations,
            document.style_shared_lock(),
        )
    }

    fn state(&self) -> ElementState {
        self.element.get_state_for_layout()
    }

    #[inline]
    fn id(&self) -> Option<&Atom> {
        unsafe { (*self.element.id_attribute()).as_ref() }
    }

    #[inline(always)]
    fn each_class<F>(&self, mut callback: F)
    where
        F: FnMut(&AtomIdent),
    {
        if let Some(classes) = self.element.get_classes_for_layout() {
            for class in classes {
                callback(AtomIdent::cast(class))
            }
        }
    }

    #[inline(always)]
    fn each_attr_name<F>(&self, mut callback: F)
    where
        F: FnMut(&style::LocalName),
    {
        for attr in self.element.attrs() {
            callback(style::values::GenericAtomIdent::cast(attr.local_name()))
        }
    }

    fn has_dirty_descendants(&self) -> bool {
        unsafe {
            self.as_node()
                .node
                .get_flag(NodeFlags::HAS_DIRTY_DESCENDANTS)
        }
    }

    fn has_snapshot(&self) -> bool {
        unsafe { self.as_node().node.get_flag(NodeFlags::HAS_SNAPSHOT) }
    }

    fn handled_snapshot(&self) -> bool {
        unsafe { self.as_node().node.get_flag(NodeFlags::HANDLED_SNAPSHOT) }
    }

    unsafe fn set_handled_snapshot(&self) {
        self.as_node()
            .node
            .set_flag(NodeFlags::HANDLED_SNAPSHOT, true);
    }

    unsafe fn set_dirty_descendants(&self) {
        debug_assert!(self.as_node().is_connected());
        self.as_node()
            .node
            .set_flag(NodeFlags::HAS_DIRTY_DESCENDANTS, true)
    }

    unsafe fn unset_dirty_descendants(&self) {
        self.as_node()
            .node
            .set_flag(NodeFlags::HAS_DIRTY_DESCENDANTS, false)
    }

    fn store_children_to_process(&self, n: isize) {
        let data = self.get_style_data().unwrap();
        data.parallel
            .children_to_process
            .store(n, Ordering::Relaxed);
    }

    fn did_process_child(&self) -> isize {
        let data = self.get_style_data().unwrap();
        let old_value = data
            .parallel
            .children_to_process
            .fetch_sub(1, Ordering::Relaxed);
        debug_assert!(old_value >= 1);
        old_value - 1
    }

    unsafe fn clear_data(&self) {
        self.as_node().get_jsmanaged().clear_style_and_layout_data()
    }

    unsafe fn ensure_data(&self) -> AtomicRefMut<ElementData> {
        self.as_node().get_jsmanaged().initialize_style_data();
        self.mutate_data().unwrap()
    }

    /// Whether there is an ElementData container.
    fn has_data(&self) -> bool {
        self.get_style_data().is_some()
    }

    /// Immutably borrows the ElementData.
    fn borrow_data(&self) -> Option<AtomicRef<ElementData>> {
        self.get_style_data().map(|data| data.element_data.borrow())
    }

    /// Mutably borrows the ElementData.
    fn mutate_data(&self) -> Option<AtomicRefMut<ElementData>> {
        self.get_style_data()
            .map(|data| data.element_data.borrow_mut())
    }

    fn skip_item_display_fixup(&self) -> bool {
        false
    }

    fn has_animations(&self, context: &SharedStyleContext) -> bool {
        // This is not used for pseudo elements currently so we can pass None.
        self.has_css_animations(context, /* pseudo_element = */ None) ||
            self.has_css_transitions(context, /* pseudo_element = */ None)
    }

    fn has_css_animations(
        &self,
        context: &SharedStyleContext,
        pseudo_element: Option<PseudoElement>,
    ) -> bool {
        let key = AnimationSetKey::new(self.as_node().opaque(), pseudo_element);
        context.animations.has_active_animations(&key)
    }

    fn has_css_transitions(
        &self,
        context: &SharedStyleContext,
        pseudo_element: Option<PseudoElement>,
    ) -> bool {
        let key = AnimationSetKey::new(self.as_node().opaque(), pseudo_element);
        context.animations.has_active_transitions(&key)
    }

    #[inline]
    fn lang_attr(&self) -> Option<SelectorAttrValue> {
        self.get_attr(&ns!(xml), &local_name!("lang"))
            .or_else(|| self.get_attr(&ns!(), &local_name!("lang")))
            .map(|v| SelectorAttrValue::from(v as &str))
    }

    fn match_element_lang(
        &self,
        override_lang: Option<Option<SelectorAttrValue>>,
        value: &Lang,
    ) -> bool {
        // Servo supports :lang() from CSS Selectors 4, which can take a comma-
        // separated list of language tags in the pseudo-class, and which
        // performs RFC 4647 extended filtering matching on them.
        //
        // FIXME(heycam): This is wrong, since extended_filtering accepts
        // a string containing commas (separating each language tag in
        // a list) but the pseudo-class instead should be parsing and
        // storing separate <ident> or <string>s for each language tag.
        //
        // FIXME(heycam): Look at `element`'s document's Content-Language
        // HTTP header for language tags to match `value` against.  To
        // do this, we should make `get_lang_for_layout` return an Option,
        // so we can decide when to fall back to the Content-Language check.
        let element_lang = match override_lang {
            Some(Some(lang)) => lang,
            Some(None) => AtomString::default(),
            None => AtomString::from(&*self.element.get_lang_for_layout()),
        };
        extended_filtering(&element_lang, value)
    }

    fn is_html_document_body_element(&self) -> bool {
        self.is_body_element_of_html_element_root()
    }

    fn synthesize_presentational_hints_for_legacy_attributes<V>(
        &self,
        _visited_handling: VisitedHandlingMode,
        hints: &mut V,
    ) where
        V: Push<ApplicableDeclarationBlock>,
    {
        self.element
            .synthesize_presentational_hints_for_legacy_attributes(hints);
    }

    /// The shadow root this element is a host of.
    fn shadow_root(&self) -> Option<ServoShadowRoot<'dom>> {
        self.element
            .get_shadow_root_for_layout()
            .map(ServoShadowRoot::from_layout_js)
    }

    /// The shadow root which roots the subtree this element is contained in.
    fn containing_shadow(&self) -> Option<ServoShadowRoot<'dom>> {
        self.element
            .upcast()
            .containing_shadow_root_for_layout()
            .map(ServoShadowRoot::from_layout_js)
    }

    fn local_name(&self) -> &LocalName {
        self.element.local_name()
    }

    fn namespace(&self) -> &Namespace {
        self.element.namespace()
    }

    fn query_container_size(
        &self,
        _display: &Display,
    ) -> euclid::default::Size2D<Option<app_units::Au>> {
        todo!();
    }

    fn has_selector_flags(&self, flags: ElementSelectorFlags) -> bool {
        self.element.get_selector_flags().contains(flags)
    }

    fn relative_selector_search_direction(&self) -> ElementSelectorFlags {
        self.element
            .get_selector_flags()
            .intersection(ElementSelectorFlags::RELATIVE_SELECTOR_SEARCH_DIRECTION_ANCESTOR_SIBLING)
    }

    fn each_custom_state<F>(&self, _callback: F)
    where
        F: FnMut(&AtomIdent),
    {
    }
}

impl<'dom> ::selectors::Element for ServoLayoutElement<'dom> {
    type Impl = SelectorImpl;

    fn opaque(&self) -> ::selectors::OpaqueElement {
        ::selectors::OpaqueElement::new(unsafe { &*(self.as_node().opaque().0 as *const ()) })
    }

    fn parent_element(&self) -> Option<Self> {
        ServoLayoutElement::parent_element(self)
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        match self.as_node().parent_node() {
            None => false,
            Some(node) => {
                node.script_type_id() ==
                    NodeTypeId::DocumentFragment(DocumentFragmentTypeId::ShadowRoot)
            },
        }
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        self.containing_shadow().map(|s| s.host())
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        let mut node = self.as_node();
        while let Some(sibling) = node.prev_sibling() {
            if let Some(element) = sibling.as_element() {
                return Some(element);
            }
            node = sibling;
        }
        None
    }

    fn next_sibling_element(&self) -> Option<ServoLayoutElement<'dom>> {
        let mut node = self.as_node();
        while let Some(sibling) = node.next_sibling() {
            if let Some(element) = sibling.as_element() {
                return Some(element);
            }
            node = sibling;
        }
        None
    }

    fn first_element_child(&self) -> Option<Self> {
        self.as_node()
            .dom_children()
            .find_map(|child| child.as_element())
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&style::Namespace>,
        local_name: &style::LocalName,
        operation: &AttrSelectorOperation<&AtomString>,
    ) -> bool {
        match *ns {
            NamespaceConstraint::Specific(ns) => self
                .get_attr_enum(ns, local_name)
                .map_or(false, |value| value.eval_selector(operation)),
            NamespaceConstraint::Any => self
                .element
                .get_attr_vals_for_layout(local_name)
                .iter()
                .any(|value| value.eval_selector(operation)),
        }
    }

    fn is_root(&self) -> bool {
        ServoLayoutElement::is_root(self)
    }

    fn is_empty(&self) -> bool {
        self.as_node()
            .dom_children()
            .all(|node| match node.script_type_id() {
                NodeTypeId::Element(..) => false,
                NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::Text)) => {
                    node.node.downcast().unwrap().data_for_layout().is_empty()
                },
                _ => true,
            })
    }

    #[inline]
    fn has_local_name(&self, name: &LocalName) -> bool {
        self.element.local_name() == name
    }

    #[inline]
    fn has_namespace(&self, ns: &Namespace) -> bool {
        self.element.namespace() == ns
    }

    #[inline]
    fn is_same_type(&self, other: &Self) -> bool {
        self.element.local_name() == other.element.local_name() &&
            self.element.namespace() == other.element.namespace()
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn match_pseudo_element(
        &self,
        _pseudo: &PseudoElement,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn match_non_ts_pseudo_class(
        &self,
        pseudo_class: &NonTSPseudoClass,
        _: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        match *pseudo_class {
            // https://github.com/servo/servo/issues/8718
            NonTSPseudoClass::Link | NonTSPseudoClass::AnyLink => self.is_link(),
            NonTSPseudoClass::Visited => false,

            NonTSPseudoClass::CustomState(ref state) => self.has_custom_state(&state.0),
            NonTSPseudoClass::Lang(ref lang) => self.match_element_lang(None, lang),

            NonTSPseudoClass::ServoNonZeroBorder => !matches!(
                self.element
                    .get_attr_for_layout(&ns!(), &local_name!("border")),
                None | Some(&AttrValue::UInt(_, 0))
            ),
            NonTSPseudoClass::ReadOnly => !self
                .element
                .get_state_for_layout()
                .contains(pseudo_class.state_flag()),

            NonTSPseudoClass::Active |
            NonTSPseudoClass::Focus |
            NonTSPseudoClass::Fullscreen |
            NonTSPseudoClass::Hover |
            NonTSPseudoClass::Defined |
            NonTSPseudoClass::Enabled |
            NonTSPseudoClass::Disabled |
            NonTSPseudoClass::Checked |
            NonTSPseudoClass::Valid |
            NonTSPseudoClass::Invalid |
            NonTSPseudoClass::Indeterminate |
            NonTSPseudoClass::ReadWrite |
            NonTSPseudoClass::PlaceholderShown |
            NonTSPseudoClass::Target => self
                .element
                .get_state_for_layout()
                .contains(pseudo_class.state_flag()),
        }
    }

    #[inline]
    fn is_link(&self) -> bool {
        match self.as_node().script_type_id() {
            // https://html.spec.whatwg.org/multipage/#selector-link
            NodeTypeId::Element(ElementTypeId::HTMLElement(
                HTMLElementTypeId::HTMLAnchorElement,
            )) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) => {
                self.element
                    .get_attr_val_for_layout(&ns!(), &local_name!("href"))
                    .is_some()
            },
            _ => false,
        }
    }

    #[inline]
    fn has_id(&self, id: &AtomIdent, case_sensitivity: CaseSensitivity) -> bool {
        unsafe {
            (*self.element.id_attribute())
                .as_ref()
                .map_or(false, |atom| case_sensitivity.eq_atom(atom, id))
        }
    }

    #[inline]
    fn is_part(&self, _name: &AtomIdent) -> bool {
        false
    }

    fn imported_part(&self, _: &AtomIdent) -> Option<AtomIdent> {
        None
    }

    #[inline]
    fn has_class(&self, name: &AtomIdent, case_sensitivity: CaseSensitivity) -> bool {
        self.element.has_class_for_layout(name, case_sensitivity)
    }

    fn is_html_slot_element(&self) -> bool {
        self.element.is_html_element() && self.local_name() == &local_name!("slot")
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.element.is_html_element() && self.as_node().owner_doc().is_html_document()
    }

    fn apply_selector_flags(&self, flags: ElementSelectorFlags) {
        // Handle flags that apply to the element.
        let self_flags = flags.for_self();
        if !self_flags.is_empty() {
            self.element.insert_selector_flags(flags);
        }

        // Handle flags that apply to the parent.
        let parent_flags = flags.for_parent();
        if !parent_flags.is_empty() {
            if let Some(p) = self.as_node().parent_element() {
                p.element.insert_selector_flags(flags);
            }
        }
    }

    fn add_element_unique_hashes(&self, filter: &mut BloomFilter) -> bool {
        each_relevant_element_hash(*self, |hash| filter.insert_hash(hash & BLOOM_HASH_MASK));
        true
    }

    fn has_custom_state(&self, _name: &AtomIdent) -> bool {
        false
    }
}

/// A wrapper around elements that ensures layout can only
/// ever access safe properties and cannot race on elements.
#[derive(Clone, Copy, Debug)]
pub struct ServoThreadSafeLayoutElement<'dom> {
    pub(super) element: ServoLayoutElement<'dom>,

    /// The pseudo-element type, with (optionally)
    /// a specified display value to override the stylesheet.
    pub(super) pseudo: PseudoElementType,
}

impl<'dom> ThreadSafeLayoutElement<'dom> for ServoThreadSafeLayoutElement<'dom> {
    type ConcreteThreadSafeLayoutNode = ServoThreadSafeLayoutNode<'dom>;
    type ConcreteElement = ServoLayoutElement<'dom>;

    fn as_node(&self) -> ServoThreadSafeLayoutNode<'dom> {
        ServoThreadSafeLayoutNode {
            node: self.element.as_node(),
            pseudo: self.pseudo,
        }
    }

    fn get_pseudo_element_type(&self) -> PseudoElementType {
        self.pseudo
    }

    fn with_pseudo(&self, pseudo: PseudoElementType) -> Self {
        ServoThreadSafeLayoutElement {
            element: self.element,
            pseudo,
        }
    }

    fn type_id(&self) -> Option<LayoutNodeType> {
        self.as_node().type_id()
    }

    fn unsafe_get(self) -> ServoLayoutElement<'dom> {
        self.element
    }

    fn get_local_name(&self) -> &LocalName {
        self.element.local_name()
    }

    fn get_attr_enum(&self, namespace: &Namespace, name: &LocalName) -> Option<&AttrValue> {
        self.element.get_attr_enum(namespace, name)
    }

    fn get_attr<'a>(&'a self, namespace: &Namespace, name: &LocalName) -> Option<&'a str> {
        self.element.get_attr(namespace, name)
    }

    fn style_data(&self) -> AtomicRef<ElementData> {
        self.element.borrow_data().expect("Unstyled layout node?")
    }

    fn is_shadow_host(&self) -> bool {
        self.element.shadow_root().is_some()
    }

    fn is_body_element_of_html_element_root(&self) -> bool {
        self.element.is_html_document_body_element()
    }
}

/// This implementation of `::selectors::Element` is used for implementing lazy
/// pseudo-elements.
///
/// Lazy pseudo-elements in Servo only allows selectors using safe properties,
/// i.e., local_name, attributes, so they can only be used for **private**
/// pseudo-elements (like `::-servo-details-content`).
///
/// Probably a few more of this functions can be implemented (like `has_class`, etc.),
/// but they have no use right now.
///
/// Note that the element implementation is needed only for selector matching,
/// not for inheritance (styles are inherited appropriately).
impl<'dom> ::selectors::Element for ServoThreadSafeLayoutElement<'dom> {
    type Impl = SelectorImpl;

    fn opaque(&self) -> ::selectors::OpaqueElement {
        ::selectors::OpaqueElement::new(unsafe { &*(self.as_node().opaque().0 as *const ()) })
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn parent_element(&self) -> Option<Self> {
        warn!("ServoThreadSafeLayoutElement::parent_element called");
        None
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    // Skips non-element nodes
    fn prev_sibling_element(&self) -> Option<Self> {
        warn!("ServoThreadSafeLayoutElement::prev_sibling_element called");
        None
    }

    // Skips non-element nodes
    fn next_sibling_element(&self) -> Option<Self> {
        warn!("ServoThreadSafeLayoutElement::next_sibling_element called");
        None
    }

    // Skips non-element nodes
    fn first_element_child(&self) -> Option<Self> {
        warn!("ServoThreadSafeLayoutElement::first_element_child called");
        None
    }

    fn is_html_slot_element(&self) -> bool {
        self.element.is_html_slot_element()
    }

    fn is_html_element_in_html_document(&self) -> bool {
        debug!("ServoThreadSafeLayoutElement::is_html_element_in_html_document called");
        true
    }

    #[inline]
    fn has_local_name(&self, name: &LocalName) -> bool {
        self.element.local_name() == name
    }

    #[inline]
    fn has_namespace(&self, ns: &Namespace) -> bool {
        self.element.namespace() == ns
    }

    #[inline]
    fn is_same_type(&self, other: &Self) -> bool {
        self.element.local_name() == other.element.local_name() &&
            self.element.namespace() == other.element.namespace()
    }

    fn match_pseudo_element(
        &self,
        _pseudo: &PseudoElement,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&style::Namespace>,
        local_name: &style::LocalName,
        operation: &AttrSelectorOperation<&AtomString>,
    ) -> bool {
        match *ns {
            NamespaceConstraint::Specific(ns) => self
                .get_attr_enum(ns, local_name)
                .map_or(false, |value| value.eval_selector(operation)),
            NamespaceConstraint::Any => self
                .element
                .element
                .get_attr_vals_for_layout(local_name)
                .iter()
                .any(|v| v.eval_selector(operation)),
        }
    }

    fn match_non_ts_pseudo_class(
        &self,
        _: &NonTSPseudoClass,
        _: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        // NB: This could maybe be implemented
        warn!("ServoThreadSafeLayoutElement::match_non_ts_pseudo_class called");
        false
    }

    fn is_link(&self) -> bool {
        warn!("ServoThreadSafeLayoutElement::is_link called");
        false
    }

    fn has_id(&self, _id: &AtomIdent, _case_sensitivity: CaseSensitivity) -> bool {
        debug!("ServoThreadSafeLayoutElement::has_id called");
        false
    }

    #[inline]
    fn is_part(&self, _name: &AtomIdent) -> bool {
        debug!("ServoThreadSafeLayoutElement::is_part called");
        false
    }

    fn imported_part(&self, _: &AtomIdent) -> Option<AtomIdent> {
        debug!("ServoThreadSafeLayoutElement::imported_part called");
        None
    }

    fn has_class(&self, _name: &AtomIdent, _case_sensitivity: CaseSensitivity) -> bool {
        debug!("ServoThreadSafeLayoutElement::has_class called");
        false
    }

    fn is_empty(&self) -> bool {
        warn!("ServoThreadSafeLayoutElement::is_empty called");
        false
    }

    fn is_root(&self) -> bool {
        warn!("ServoThreadSafeLayoutElement::is_root called");
        false
    }

    fn apply_selector_flags(&self, flags: ElementSelectorFlags) {
        // Handle flags that apply to the element.
        let self_flags = flags.for_self();
        if !self_flags.is_empty() {
            self.element.element.insert_selector_flags(flags);
        }

        // Handle flags that apply to the parent.
        let parent_flags = flags.for_parent();
        if !parent_flags.is_empty() {
            if let Some(p) = self.element.parent_element() {
                p.element.insert_selector_flags(flags);
            }
        }
    }

    fn add_element_unique_hashes(&self, filter: &mut BloomFilter) -> bool {
        each_relevant_element_hash(self.element, |hash| {
            filter.insert_hash(hash & BLOOM_HASH_MASK)
        });
        true
    }

    fn has_custom_state(&self, _name: &AtomIdent) -> bool {
        false
    }
}
