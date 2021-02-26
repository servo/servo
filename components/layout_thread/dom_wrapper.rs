/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A safe wrapper for DOM nodes that prevents layout from mutating the DOM, from letting DOM nodes
//! escape, and from generally doing anything that it isn't supposed to. This is accomplished via
//! a simple whitelist of allowed operations, along with some lifetime magic to prevent nodes from
//! escaping.
//!
//! As a security wrapper is only as good as its whitelist, be careful when adding operations to
//! this list. The cardinal rules are:
//!
//! 1. Layout is not allowed to mutate the DOM.
//!
//! 2. Layout is not allowed to see anything with `LayoutDom` in the name, because it could hang
//!    onto these objects and cause use-after-free.
//!
//! When implementing wrapper functions, be careful that you do not touch the borrow flags, or you
//! will race and cause spurious thread failure. (Note that I do not believe these races are
//! exploitable, but they'll result in brokenness nonetheless.)
//!
//! Rules of the road for this file:
//!
//! * Do not call any methods on DOM nodes without checking to see whether they use borrow flags.
//!
//!   o Instead of `get_attr()`, use `.get_attr_val_for_layout()`.
//!
//!   o Instead of `html_element_in_html_document()`, use
//!     `html_element_in_html_document_for_layout()`.

#![allow(unsafe_code)]

use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use gfx_traits::ByteIndex;
use html5ever::{LocalName, Namespace};
use layout::data::LayoutData;
use layout::wrapper::GetStyleAndLayoutData;
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use net_traits::image::base::{Image, ImageMetadata};
use range::Range;
use script::layout_exports::NodeFlags;
use script::layout_exports::ShadowRoot;
use script::layout_exports::{
    CharacterDataTypeId, DocumentFragmentTypeId, ElementTypeId, HTMLElementTypeId, NodeTypeId,
    TextTypeId,
};
use script::layout_exports::{Document, Element, Node, Text};
use script::layout_exports::{LayoutCharacterDataHelpers, LayoutDocumentHelpers};
use script::layout_exports::{
    LayoutDom, LayoutElementHelpers, LayoutNodeHelpers, LayoutShadowRootHelpers,
};
use script_layout_interface::wrapper_traits::{
    DangerousThreadSafeLayoutNode, GetStyleAndOpaqueLayoutData, LayoutNode,
};
use script_layout_interface::wrapper_traits::{
    PseudoElementType, ThreadSafeLayoutElement, ThreadSafeLayoutNode,
};
use script_layout_interface::{
    HTMLCanvasData, HTMLMediaData, LayoutNodeType, StyleAndOpaqueLayoutData,
};
use script_layout_interface::{SVGSVGData, StyleData, TrustedNodeAddress};
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::matching::VisitedHandlingMode;
use selectors::matching::{ElementSelectorFlags, MatchingContext, QuirksMode};
use selectors::sink::Push;
use servo_arc::{Arc, ArcBorrow};
use servo_atoms::Atom;
use servo_url::ServoUrl;
use std::borrow::Cow;
use std::fmt;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::atomic::Ordering;
use std::sync::Arc as StdArc;
use style::animation::AnimationSetKey;
use style::applicable_declarations::ApplicableDeclarationBlock;
use style::attr::AttrValue;
use style::context::SharedStyleContext;
use style::data::ElementData;
use style::dom::{DomChildren, LayoutIterator, NodeInfo, OpaqueNode};
use style::dom::{TDocument, TElement, TNode, TShadowRoot};
use style::element_state::*;
use style::font_metrics::ServoMetricsProvider;
use style::media_queries::Device;
use style::properties::{ComputedValues, PropertyDeclarationBlock};
use style::selector_parser::{extended_filtering, PseudoElement, SelectorImpl};
use style::selector_parser::{AttrValue as SelectorAttrValue, Lang, NonTSPseudoClass};
use style::shared_lock::{
    Locked as StyleLocked, SharedRwLock as StyleSharedRwLock, SharedRwLockReadGuard,
};
use style::str::is_whitespace;
use style::stylist::CascadeData;
use style::values::{AtomIdent, AtomString};
use style::CaseSensitivityExt;

#[derive(Clone, Copy)]
pub struct ServoLayoutNode<'dom> {
    /// The wrapped node.
    node: LayoutDom<'dom, Node>,
}

impl<'ln> Debug for ServoLayoutNode<'ln> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(el) = self.as_element() {
            el.fmt(f)
        } else {
            if self.is_text_node() {
                write!(f, "<text node> ({:#x})", self.opaque().0)
            } else {
                write!(f, "<non-text node> ({:#x})", self.opaque().0)
            }
        }
    }
}

impl<'a> PartialEq for ServoLayoutNode<'a> {
    #[inline]
    fn eq(&self, other: &ServoLayoutNode) -> bool {
        self.node == other.node
    }
}

impl<'ln> ServoLayoutNode<'ln> {
    fn from_layout_js(n: LayoutDom<'ln, Node>) -> Self {
        ServoLayoutNode { node: n }
    }

    pub unsafe fn new(address: &TrustedNodeAddress) -> Self {
        ServoLayoutNode::from_layout_js(LayoutDom::from_trusted_node_address(*address))
    }

    fn script_type_id(&self) -> NodeTypeId {
        self.node.type_id_for_layout()
    }
}

impl<'ln> NodeInfo for ServoLayoutNode<'ln> {
    fn is_element(&self) -> bool {
        self.node.is_element_for_layout()
    }

    fn is_text_node(&self) -> bool {
        self.script_type_id() ==
            NodeTypeId::CharacterData(CharacterDataTypeId::Text(TextTypeId::Text))
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct ServoShadowRoot<'dom> {
    /// The wrapped shadow root.
    shadow_root: LayoutDom<'dom, ShadowRoot>,
}

impl<'lr> Debug for ServoShadowRoot<'lr> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_node().fmt(f)
    }
}

impl<'lr> TShadowRoot for ServoShadowRoot<'lr> {
    type ConcreteNode = ServoLayoutNode<'lr>;

    fn as_node(&self) -> Self::ConcreteNode {
        ServoLayoutNode::from_layout_js(self.shadow_root.upcast())
    }

    fn host(&self) -> ServoLayoutElement<'lr> {
        ServoLayoutElement::from_layout_js(self.shadow_root.get_host_for_layout())
    }

    fn style_data<'a>(&self) -> Option<&'a CascadeData>
    where
        Self: 'a,
    {
        Some(&self.shadow_root.get_style_data_for_layout())
    }
}

impl<'lr> ServoShadowRoot<'lr> {
    fn from_layout_js(shadow_root: LayoutDom<'lr, ShadowRoot>) -> Self {
        ServoShadowRoot { shadow_root }
    }

    pub unsafe fn flush_stylesheets(
        &self,
        device: &Device,
        quirks_mode: QuirksMode,
        guard: &SharedRwLockReadGuard,
    ) {
        self.shadow_root
            .flush_stylesheets::<ServoLayoutElement>(device, quirks_mode, guard)
    }
}

impl<'ln> TNode for ServoLayoutNode<'ln> {
    type ConcreteDocument = ServoLayoutDocument<'ln>;
    type ConcreteElement = ServoLayoutElement<'ln>;
    type ConcreteShadowRoot = ServoShadowRoot<'ln>;

    fn parent_node(&self) -> Option<Self> {
        self.node
            .composed_parent_node_ref()
            .map(Self::from_layout_js)
    }

    fn first_child(&self) -> Option<Self> {
        self.node.first_child_ref().map(Self::from_layout_js)
    }

    fn last_child(&self) -> Option<Self> {
        self.node.last_child_ref().map(Self::from_layout_js)
    }

    fn prev_sibling(&self) -> Option<Self> {
        self.node.prev_sibling_ref().map(Self::from_layout_js)
    }

    fn next_sibling(&self) -> Option<Self> {
        self.node.next_sibling_ref().map(Self::from_layout_js)
    }

    fn owner_doc(&self) -> Self::ConcreteDocument {
        ServoLayoutDocument::from_layout_js(self.node.owner_doc_for_layout())
    }

    fn traversal_parent(&self) -> Option<ServoLayoutElement<'ln>> {
        let parent = self.parent_node()?;
        if let Some(shadow) = parent.as_shadow_root() {
            return Some(shadow.host());
        };
        parent.as_element()
    }

    fn opaque(&self) -> OpaqueNode {
        self.get_jsmanaged().opaque()
    }

    fn debug_id(self) -> usize {
        self.opaque().0
    }

    fn as_element(&self) -> Option<ServoLayoutElement<'ln>> {
        as_element(self.node)
    }

    fn as_document(&self) -> Option<ServoLayoutDocument<'ln>> {
        self.node
            .downcast()
            .map(ServoLayoutDocument::from_layout_js)
    }

    fn as_shadow_root(&self) -> Option<ServoShadowRoot<'ln>> {
        self.node.downcast().map(ServoShadowRoot::from_layout_js)
    }

    fn is_in_document(&self) -> bool {
        unsafe { self.node.get_flag(NodeFlags::IS_IN_DOC) }
    }
}

impl<'ln> LayoutNode<'ln> for ServoLayoutNode<'ln> {
    type ConcreteThreadSafeLayoutNode = ServoThreadSafeLayoutNode<'ln>;

    fn to_threadsafe(&self) -> Self::ConcreteThreadSafeLayoutNode {
        ServoThreadSafeLayoutNode::new(*self)
    }

    fn type_id(&self) -> LayoutNodeType {
        self.script_type_id().into()
    }

    unsafe fn initialize_data(&self) {
        if self.get_style_and_layout_data().is_none() {
            let opaque = StyleAndOpaqueLayoutData::new(
                StyleData::new(),
                AtomicRefCell::new(LayoutData::new()),
            );
            self.init_style_and_opaque_layout_data(opaque);
        };
    }

    unsafe fn init_style_and_opaque_layout_data(&self, data: Box<StyleAndOpaqueLayoutData>) {
        self.get_jsmanaged().init_style_and_opaque_layout_data(data);
    }

    unsafe fn take_style_and_opaque_layout_data(&self) -> Box<StyleAndOpaqueLayoutData> {
        self.get_jsmanaged().take_style_and_opaque_layout_data()
    }

    fn is_connected(&self) -> bool {
        unsafe { self.node.get_flag(NodeFlags::IS_CONNECTED) }
    }
}

impl<'dom> GetStyleAndOpaqueLayoutData<'dom> for ServoLayoutNode<'dom> {
    fn get_style_and_opaque_layout_data(self) -> Option<&'dom StyleAndOpaqueLayoutData> {
        self.get_jsmanaged().get_style_and_opaque_layout_data()
    }
}

impl<'dom> GetStyleAndOpaqueLayoutData<'dom> for ServoLayoutElement<'dom> {
    fn get_style_and_opaque_layout_data(self) -> Option<&'dom StyleAndOpaqueLayoutData> {
        self.as_node().get_style_and_opaque_layout_data()
    }
}

impl<'dom> GetStyleAndOpaqueLayoutData<'dom> for ServoThreadSafeLayoutNode<'dom> {
    fn get_style_and_opaque_layout_data(self) -> Option<&'dom StyleAndOpaqueLayoutData> {
        self.node.get_style_and_opaque_layout_data()
    }
}

impl<'dom> GetStyleAndOpaqueLayoutData<'dom> for ServoThreadSafeLayoutElement<'dom> {
    fn get_style_and_opaque_layout_data(self) -> Option<&'dom StyleAndOpaqueLayoutData> {
        self.element.as_node().get_style_and_opaque_layout_data()
    }
}

impl<'dom> ServoLayoutNode<'dom> {
    /// Returns the interior of this node as a `LayoutDom`.
    pub fn get_jsmanaged(self) -> LayoutDom<'dom, Node> {
        self.node
    }
}

// A wrapper around documents that ensures ayout can only ever access safe properties.
#[derive(Clone, Copy)]
pub struct ServoLayoutDocument<'dom> {
    document: LayoutDom<'dom, Document>,
}

impl<'ld> TDocument for ServoLayoutDocument<'ld> {
    type ConcreteNode = ServoLayoutNode<'ld>;

    fn as_node(&self) -> Self::ConcreteNode {
        ServoLayoutNode::from_layout_js(self.document.upcast())
    }

    fn quirks_mode(&self) -> QuirksMode {
        self.document.quirks_mode()
    }

    fn is_html_document(&self) -> bool {
        self.document.is_html_document_for_layout()
    }

    fn shared_lock(&self) -> &StyleSharedRwLock {
        self.document.style_shared_lock()
    }
}

impl<'ld> ServoLayoutDocument<'ld> {
    pub fn root_element(&self) -> Option<ServoLayoutElement<'ld>> {
        self.as_node()
            .dom_children()
            .flat_map(|n| n.as_element())
            .next()
    }

    pub fn needs_paint_from_layout(&self) {
        unsafe { self.document.needs_paint_from_layout() }
    }

    pub fn will_paint(&self) {
        unsafe { self.document.will_paint() }
    }

    pub fn style_shared_lock(&self) -> &StyleSharedRwLock {
        self.document.style_shared_lock()
    }

    pub fn shadow_roots(&self) -> Vec<ServoShadowRoot> {
        unsafe {
            self.document
                .shadow_roots()
                .iter()
                .map(|sr| {
                    debug_assert!(sr.upcast::<Node>().get_flag(NodeFlags::IS_CONNECTED));
                    ServoShadowRoot::from_layout_js(*sr)
                })
                .collect()
        }
    }

    pub fn flush_shadow_roots_stylesheets(
        &self,
        device: &Device,
        quirks_mode: QuirksMode,
        guard: &SharedRwLockReadGuard,
    ) {
        unsafe {
            if !self.document.shadow_roots_styles_changed() {
                return;
            }
            self.document.flush_shadow_roots_stylesheets();
            for shadow_root in self.shadow_roots() {
                shadow_root.flush_stylesheets(device, quirks_mode, guard);
            }
        }
    }

    pub fn from_layout_js(doc: LayoutDom<'ld, Document>) -> Self {
        ServoLayoutDocument { document: doc }
    }
}

/// A wrapper around elements that ensures layout can only ever access safe properties.
#[derive(Clone, Copy)]
pub struct ServoLayoutElement<'dom> {
    element: LayoutDom<'dom, Element>,
}

impl<'le> fmt::Debug for ServoLayoutElement<'le> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}", self.element.local_name())?;
        if let Some(id) = self.id() {
            write!(f, " id={}", id)?;
        }
        write!(f, "> ({:#x})", self.as_node().opaque().0)
    }
}

impl<'le> TElement for ServoLayoutElement<'le> {
    type ConcreteNode = ServoLayoutNode<'le>;
    type TraversalChildrenIterator = DomChildren<Self::ConcreteNode>;

    type FontMetricsProvider = ServoMetricsProvider;

    fn as_node(&self) -> ServoLayoutNode<'le> {
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
        self.element.is_html_element()
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
    fn has_attr(&self, namespace: &style::Namespace, attr: &style::LocalName) -> bool {
        self.get_attr(&**namespace, &**attr).is_some()
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
        if let Some(ref classes) = self.element.get_classes_for_layout() {
            for class in *classes {
                callback(AtomIdent::cast(class))
            }
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
        if self.get_style_and_layout_data().is_some() {
            drop(self.as_node().take_style_and_opaque_layout_data());
        }
    }

    unsafe fn ensure_data(&self) -> AtomicRefMut<ElementData> {
        self.as_node().initialize_data();
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

    unsafe fn set_selector_flags(&self, flags: ElementSelectorFlags) {
        self.element.insert_selector_flags(flags);
    }

    fn has_selector_flags(&self, flags: ElementSelectorFlags) -> bool {
        self.element.has_selector_flags(flags)
    }

    fn has_animations(&self, context: &SharedStyleContext) -> bool {
        // This is not used for pseudo elements currently so we can pass None.
        return self.has_css_animations(context, /* pseudo_element = */ None) ||
            self.has_css_transitions(context, /* pseudo_element = */ None);
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
        extended_filtering(&element_lang, &*value)
    }

    fn is_html_document_body_element(&self) -> bool {
        // This is only used for the "tables inherit from body" quirk, which we
        // don't implement.
        //
        // FIXME(emilio): We should be able to give the right answer though!
        false
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
    fn shadow_root(&self) -> Option<ServoShadowRoot<'le>> {
        self.element
            .get_shadow_root_for_layout()
            .map(ServoShadowRoot::from_layout_js)
    }

    /// The shadow root which roots the subtree this element is contained in.
    fn containing_shadow(&self) -> Option<ServoShadowRoot<'le>> {
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
}

impl<'le> PartialEq for ServoLayoutElement<'le> {
    fn eq(&self, other: &Self) -> bool {
        self.as_node() == other.as_node()
    }
}

impl<'le> Hash for ServoLayoutElement<'le> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.element.hash(state);
    }
}

impl<'le> Eq for ServoLayoutElement<'le> {}

impl<'le> ServoLayoutElement<'le> {
    fn from_layout_js(el: LayoutDom<'le, Element>) -> Self {
        ServoLayoutElement { element: el }
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
        self.get_style_and_opaque_layout_data()
            .map(|data| &data.style_data)
    }

    pub unsafe fn unset_snapshot_flags(&self) {
        self.as_node()
            .node
            .set_flag(NodeFlags::HAS_SNAPSHOT | NodeFlags::HANDLED_SNAPSHOT, false);
    }

    pub unsafe fn set_has_snapshot(&self) {
        self.as_node().node.set_flag(NodeFlags::HAS_SNAPSHOT, true);
    }
}

fn as_element<'dom>(node: LayoutDom<'dom, Node>) -> Option<ServoLayoutElement<'dom>> {
    node.downcast().map(ServoLayoutElement::from_layout_js)
}

impl<'le> ::selectors::Element for ServoLayoutElement<'le> {
    type Impl = SelectorImpl;

    fn opaque(&self) -> ::selectors::OpaqueElement {
        ::selectors::OpaqueElement::new(unsafe { &*(self.as_node().opaque().0 as *const ()) })
    }

    fn parent_element(&self) -> Option<ServoLayoutElement<'le>> {
        self.element
            .upcast()
            .composed_parent_node_ref()
            .and_then(as_element)
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

    fn prev_sibling_element(&self) -> Option<ServoLayoutElement<'le>> {
        let mut node = self.as_node();
        while let Some(sibling) = node.prev_sibling() {
            if let Some(element) = sibling.as_element() {
                return Some(element);
            }
            node = sibling;
        }
        None
    }

    fn next_sibling_element(&self) -> Option<ServoLayoutElement<'le>> {
        let mut node = self.as_node();
        while let Some(sibling) = node.next_sibling() {
            if let Some(element) = sibling.as_element() {
                return Some(element);
            }
            node = sibling;
        }
        None
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&style::Namespace>,
        local_name: &style::LocalName,
        operation: &AttrSelectorOperation<&AtomString>,
    ) -> bool {
        match *ns {
            NamespaceConstraint::Specific(ref ns) => self
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
        match self.as_node().parent_node() {
            None => false,
            Some(node) => match node.script_type_id() {
                NodeTypeId::Document(_) => true,
                _ => false,
            },
        }
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

    fn match_non_ts_pseudo_class<F>(
        &self,
        pseudo_class: &NonTSPseudoClass,
        _: &mut MatchingContext<Self::Impl>,
        _: &mut F,
    ) -> bool
    where
        F: FnMut(&Self, ElementSelectorFlags),
    {
        match *pseudo_class {
            // https://github.com/servo/servo/issues/8718
            NonTSPseudoClass::Link | NonTSPseudoClass::AnyLink => self.is_link(),
            NonTSPseudoClass::Visited => false,

            NonTSPseudoClass::Lang(ref lang) => self.match_element_lang(None, &*lang),

            NonTSPseudoClass::ServoNonZeroBorder => {
                match self
                    .element
                    .get_attr_for_layout(&ns!(), &local_name!("border"))
                {
                    None | Some(&AttrValue::UInt(_, 0)) => false,
                    _ => true,
                }
            },
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
        if !self.element.is_html_element() {
            return false;
        }

        self.as_node().owner_doc().is_html_document()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ServoThreadSafeLayoutNode<'ln> {
    /// The wrapped node.
    node: ServoLayoutNode<'ln>,

    /// The pseudo-element type, with (optionally)
    /// a specified display value to override the stylesheet.
    pseudo: PseudoElementType,
}

impl<'a> PartialEq for ServoThreadSafeLayoutNode<'a> {
    #[inline]
    fn eq(&self, other: &ServoThreadSafeLayoutNode<'a>) -> bool {
        self.node == other.node
    }
}

impl<'ln> DangerousThreadSafeLayoutNode<'ln> for ServoThreadSafeLayoutNode<'ln> {
    unsafe fn dangerous_first_child(&self) -> Option<Self> {
        self.get_jsmanaged()
            .first_child_ref()
            .map(ServoLayoutNode::from_layout_js)
            .map(Self::new)
    }
    unsafe fn dangerous_next_sibling(&self) -> Option<Self> {
        self.get_jsmanaged()
            .next_sibling_ref()
            .map(ServoLayoutNode::from_layout_js)
            .map(Self::new)
    }
}

impl<'ln> ServoThreadSafeLayoutNode<'ln> {
    /// Creates a new `ServoThreadSafeLayoutNode` from the given `ServoLayoutNode`.
    pub fn new(node: ServoLayoutNode<'ln>) -> Self {
        ServoThreadSafeLayoutNode {
            node: node.clone(),
            pseudo: PseudoElementType::Normal,
        }
    }

    /// Returns the interior of this node as a `LayoutDom`. This is highly unsafe for layout to
    /// call and as such is marked `unsafe`.
    unsafe fn get_jsmanaged(&self) -> LayoutDom<'ln, Node> {
        self.node.get_jsmanaged()
    }
}

impl<'ln> NodeInfo for ServoThreadSafeLayoutNode<'ln> {
    fn is_element(&self) -> bool {
        self.node.is_element()
    }

    fn is_text_node(&self) -> bool {
        self.node.is_text_node()
    }
}

impl<'ln> ThreadSafeLayoutNode<'ln> for ServoThreadSafeLayoutNode<'ln> {
    type ConcreteNode = ServoLayoutNode<'ln>;
    type ConcreteThreadSafeLayoutElement = ServoThreadSafeLayoutElement<'ln>;
    type ConcreteElement = ServoLayoutElement<'ln>;
    type ChildrenIterator = ThreadSafeLayoutNodeChildrenIterator<Self>;

    fn opaque(&self) -> OpaqueNode {
        unsafe { self.get_jsmanaged().opaque() }
    }

    fn type_id(&self) -> Option<LayoutNodeType> {
        if self.pseudo == PseudoElementType::Normal {
            Some(self.node.type_id())
        } else {
            None
        }
    }

    fn parent_style(&self) -> Arc<ComputedValues> {
        let parent = self.node.parent_node().unwrap().as_element().unwrap();
        let parent_data = parent.borrow_data().unwrap();
        parent_data.styles.primary().clone()
    }

    fn debug_id(self) -> usize {
        self.node.debug_id()
    }

    fn children(&self) -> LayoutIterator<Self::ChildrenIterator> {
        if let Some(shadow) = self.node.as_element().and_then(|e| e.shadow_root()) {
            return LayoutIterator(ThreadSafeLayoutNodeChildrenIterator::new(
                shadow.as_node().to_threadsafe(),
            ));
        }
        LayoutIterator(ThreadSafeLayoutNodeChildrenIterator::new(*self))
    }

    fn as_element(&self) -> Option<ServoThreadSafeLayoutElement<'ln>> {
        self.node
            .as_element()
            .map(|el| ServoThreadSafeLayoutElement {
                element: el,
                pseudo: self.pseudo,
            })
    }

    fn get_style_and_opaque_layout_data(self) -> Option<&'ln StyleAndOpaqueLayoutData> {
        self.node.get_style_and_opaque_layout_data()
    }

    fn is_ignorable_whitespace(&self, context: &SharedStyleContext) -> bool {
        unsafe {
            let text: LayoutDom<Text> = match self.get_jsmanaged().downcast() {
                Some(text) => text,
                None => return false,
            };

            if !is_whitespace(text.upcast().data_for_layout()) {
                return false;
            }

            // NB: See the rules for `white-space` here:
            //
            //    http://www.w3.org/TR/CSS21/text.html#propdef-white-space
            //
            // If you implement other values for this property, you will almost certainly
            // want to update this check.
            !self
                .style(context)
                .get_inherited_text()
                .white_space
                .preserve_newlines()
        }
    }

    unsafe fn unsafe_get(self) -> Self::ConcreteNode {
        self.node
    }

    fn node_text_content(self) -> Cow<'ln, str> {
        unsafe { self.get_jsmanaged().text_content() }
    }

    fn selection(&self) -> Option<Range<ByteIndex>> {
        let this = unsafe { self.get_jsmanaged() };

        this.selection().map(|range| {
            Range::new(
                ByteIndex(range.start as isize),
                ByteIndex(range.len() as isize),
            )
        })
    }

    fn image_url(&self) -> Option<ServoUrl> {
        let this = unsafe { self.get_jsmanaged() };
        this.image_url()
    }

    fn image_density(&self) -> Option<f64> {
        let this = unsafe { self.get_jsmanaged() };
        this.image_density()
    }

    fn image_data(&self) -> Option<(Option<StdArc<Image>>, Option<ImageMetadata>)> {
        let this = unsafe { self.get_jsmanaged() };
        this.image_data()
    }

    fn canvas_data(&self) -> Option<HTMLCanvasData> {
        let this = unsafe { self.get_jsmanaged() };
        this.canvas_data()
    }

    fn media_data(&self) -> Option<HTMLMediaData> {
        let this = unsafe { self.get_jsmanaged() };
        this.media_data()
    }

    fn svg_data(&self) -> Option<SVGSVGData> {
        let this = unsafe { self.get_jsmanaged() };
        this.svg_data()
    }

    // Can return None if the iframe has no nested browsing context
    fn iframe_browsing_context_id(&self) -> Option<BrowsingContextId> {
        let this = unsafe { self.get_jsmanaged() };
        this.iframe_browsing_context_id()
    }

    // Can return None if the iframe has no nested browsing context
    fn iframe_pipeline_id(&self) -> Option<PipelineId> {
        let this = unsafe { self.get_jsmanaged() };
        this.iframe_pipeline_id()
    }

    fn get_colspan(&self) -> u32 {
        unsafe {
            self.get_jsmanaged()
                .downcast::<Element>()
                .unwrap()
                .get_colspan()
        }
    }

    fn get_rowspan(&self) -> u32 {
        unsafe {
            self.get_jsmanaged()
                .downcast::<Element>()
                .unwrap()
                .get_rowspan()
        }
    }
}

pub struct ThreadSafeLayoutNodeChildrenIterator<ConcreteNode> {
    current_node: Option<ConcreteNode>,
    parent_node: ConcreteNode,
}

impl<'dom, ConcreteNode> ThreadSafeLayoutNodeChildrenIterator<ConcreteNode>
where
    ConcreteNode: DangerousThreadSafeLayoutNode<'dom>,
{
    pub fn new(parent: ConcreteNode) -> Self {
        let first_child: Option<ConcreteNode> = match parent.get_pseudo_element_type() {
            PseudoElementType::Normal => parent
                .get_before_pseudo()
                .or_else(|| parent.get_details_summary_pseudo())
                .or_else(|| unsafe { parent.dangerous_first_child() }),
            PseudoElementType::DetailsContent | PseudoElementType::DetailsSummary => unsafe {
                parent.dangerous_first_child()
            },
            _ => None,
        };
        ThreadSafeLayoutNodeChildrenIterator {
            current_node: first_child,
            parent_node: parent,
        }
    }
}

impl<'dom, ConcreteNode> Iterator for ThreadSafeLayoutNodeChildrenIterator<ConcreteNode>
where
    ConcreteNode: DangerousThreadSafeLayoutNode<'dom>,
{
    type Item = ConcreteNode;
    fn next(&mut self) -> Option<ConcreteNode> {
        use selectors::Element;
        match self.parent_node.get_pseudo_element_type() {
            PseudoElementType::Before | PseudoElementType::After => None,

            PseudoElementType::DetailsSummary => {
                let mut current_node = self.current_node.clone();
                loop {
                    let next_node = if let Some(ref node) = current_node {
                        if let Some(element) = node.as_element() {
                            if element.has_local_name(&local_name!("summary")) &&
                                element.has_namespace(&ns!(html))
                            {
                                self.current_node = None;
                                return Some(node.clone());
                            }
                        }
                        unsafe { node.dangerous_next_sibling() }
                    } else {
                        self.current_node = None;
                        return None;
                    };
                    current_node = next_node;
                }
            },

            PseudoElementType::DetailsContent => {
                let node = self.current_node.clone();
                let node = node.and_then(|node| {
                    if node.is_element() &&
                        node.as_element()
                            .unwrap()
                            .has_local_name(&local_name!("summary")) &&
                        node.as_element().unwrap().has_namespace(&ns!(html))
                    {
                        unsafe { node.dangerous_next_sibling() }
                    } else {
                        Some(node)
                    }
                });
                self.current_node = node.and_then(|node| unsafe { node.dangerous_next_sibling() });
                node
            },

            PseudoElementType::Normal => {
                let node = self.current_node.clone();
                if let Some(ref node) = node {
                    self.current_node = match node.get_pseudo_element_type() {
                        PseudoElementType::Before => self
                            .parent_node
                            .get_details_summary_pseudo()
                            .or_else(|| unsafe { self.parent_node.dangerous_first_child() })
                            .or_else(|| self.parent_node.get_after_pseudo()),
                        PseudoElementType::Normal => unsafe { node.dangerous_next_sibling() }
                            .or_else(|| self.parent_node.get_after_pseudo()),
                        PseudoElementType::DetailsSummary => {
                            self.parent_node.get_details_content_pseudo()
                        },
                        PseudoElementType::DetailsContent => self.parent_node.get_after_pseudo(),
                        PseudoElementType::After => None,
                    };
                }
                node
            },
        }
    }
}

/// A wrapper around elements that ensures layout can only
/// ever access safe properties and cannot race on elements.
#[derive(Clone, Copy, Debug)]
pub struct ServoThreadSafeLayoutElement<'le> {
    element: ServoLayoutElement<'le>,

    /// The pseudo-element type, with (optionally)
    /// a specified display value to override the stylesheet.
    pseudo: PseudoElementType,
}

impl<'le> ThreadSafeLayoutElement<'le> for ServoThreadSafeLayoutElement<'le> {
    type ConcreteThreadSafeLayoutNode = ServoThreadSafeLayoutNode<'le>;
    type ConcreteElement = ServoLayoutElement<'le>;

    fn as_node(&self) -> ServoThreadSafeLayoutNode<'le> {
        ServoThreadSafeLayoutNode {
            node: self.element.as_node(),
            pseudo: self.pseudo.clone(),
        }
    }

    fn get_pseudo_element_type(&self) -> PseudoElementType {
        self.pseudo
    }

    fn with_pseudo(&self, pseudo: PseudoElementType) -> Self {
        ServoThreadSafeLayoutElement {
            element: self.element.clone(),
            pseudo,
        }
    }

    fn type_id(&self) -> Option<LayoutNodeType> {
        self.as_node().type_id()
    }

    unsafe fn unsafe_get(self) -> ServoLayoutElement<'le> {
        self.element
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
/// not for inheritance (styles are inherited appropiately).
impl<'le> ::selectors::Element for ServoThreadSafeLayoutElement<'le> {
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
            NamespaceConstraint::Specific(ref ns) => self
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

    fn match_non_ts_pseudo_class<F>(
        &self,
        _: &NonTSPseudoClass,
        _: &mut MatchingContext<Self::Impl>,
        _: &mut F,
    ) -> bool
    where
        F: FnMut(&Self, ElementSelectorFlags),
    {
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
}
