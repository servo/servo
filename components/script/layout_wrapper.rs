/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

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
//! 2. Layout is not allowed to see anything with `LayoutJS` in the name, because it could hang
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

use atomic_refcell::{AtomicRef, AtomicRefCell};
use dom::bindings::inheritance::{CharacterDataTypeId, ElementTypeId};
use dom::bindings::inheritance::{HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::LayoutJS;
use dom::characterdata::LayoutCharacterDataHelpers;
use dom::document::{Document, LayoutDocumentHelpers, PendingRestyle};
use dom::element::{Element, LayoutElementHelpers, RawLayoutElementHelpers};
use dom::node::{CAN_BE_FRAGMENTED, DIRTY_ON_VIEWPORT_SIZE_CHANGE, HAS_DIRTY_DESCENDANTS, IS_IN_DOC};
use dom::node::{HANDLED_SNAPSHOT, HAS_SNAPSHOT};
use dom::node::{LayoutNodeHelpers, Node};
use dom::text::Text;
use gfx_traits::ByteIndex;
use html5ever::{LocalName, Namespace};
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use range::Range;
use script_layout_interface::{HTMLCanvasData, LayoutNodeType, SVGSVGData, TrustedNodeAddress};
use script_layout_interface::{OpaqueStyleAndLayoutData, StyleData};
use script_layout_interface::wrapper_traits::{DangerousThreadSafeLayoutNode, GetLayoutData, LayoutNode};
use script_layout_interface::wrapper_traits::{PseudoElementType, ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use selectors::attr::{AttrSelectorOperation, NamespaceConstraint};
use selectors::matching::{ElementSelectorFlags, MatchingContext, RelevantLinkStatus, VisitedHandlingMode};
use servo_atoms::Atom;
use servo_url::ServoUrl;
use std::fmt;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::transmute;
use std::sync::atomic::Ordering;
use style;
use style::attr::AttrValue;
use style::computed_values::display;
use style::context::{QuirksMode, SharedStyleContext};
use style::data::ElementData;
use style::dom::{DescendantsBit, DirtyDescendants, LayoutIterator, NodeInfo, OpaqueNode};
use style::dom::{PresentationalHintsSynthesizer, TElement, TNode, UnsafeNode};
use style::element_state::*;
use style::font_metrics::ServoMetricsProvider;
use style::properties::{ComputedValues, PropertyDeclarationBlock};
use style::selector_parser::{AttrValue as SelectorAttrValue, NonTSPseudoClass, PseudoClassStringArg};
use style::selector_parser::{PseudoElement, SelectorImpl, extended_filtering};
use style::shared_lock::{SharedRwLock as StyleSharedRwLock, Locked as StyleLocked};
use style::sink::Push;
use style::str::is_whitespace;
use style::stylearc::Arc;
use style::stylist::ApplicableDeclarationBlock;

#[derive(Copy, Clone)]
pub struct ServoLayoutNode<'a> {
    /// The wrapped node.
    node: LayoutJS<Node>,

    /// Being chained to a PhantomData prevents `LayoutNode`s from escaping.
    chain: PhantomData<&'a ()>,
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
    fn from_layout_js(n: LayoutJS<Node>) -> ServoLayoutNode<'ln> {
        ServoLayoutNode {
            node: n,
            chain: PhantomData,
        }
    }

    pub unsafe fn new(address: &TrustedNodeAddress) -> ServoLayoutNode {
        ServoLayoutNode::from_layout_js(LayoutJS::from_trusted_node_address(*address))
    }

    /// Creates a new layout node with the same lifetime as this layout node.
    pub unsafe fn new_with_this_lifetime(&self, node: &LayoutJS<Node>) -> ServoLayoutNode<'ln> {
        ServoLayoutNode {
            node: *node,
            chain: self.chain,
        }
    }

    fn script_type_id(&self) -> NodeTypeId {
        unsafe {
            self.node.type_id_for_layout()
        }
    }

    pub fn as_document(&self) -> Option<ServoLayoutDocument<'ln>> {
        self.node.downcast().map(ServoLayoutDocument::from_layout_js)
    }
}

impl<'ln> NodeInfo for ServoLayoutNode<'ln> {
    fn is_element(&self) -> bool {
        unsafe {
            self.node.is_element_for_layout()
        }
    }

    fn is_text_node(&self) -> bool {
        self.script_type_id() == NodeTypeId::CharacterData(CharacterDataTypeId::Text)
    }
}

impl<'ln> TNode for ServoLayoutNode<'ln> {
    type ConcreteElement = ServoLayoutElement<'ln>;
    type ConcreteChildrenIterator = ServoChildrenIterator<'ln>;

    fn to_unsafe(&self) -> UnsafeNode {
        unsafe {
            (self.node.unsafe_get() as usize, 0)
        }
    }

    unsafe fn from_unsafe(n: &UnsafeNode) -> Self {
        let (node, _) = *n;
        transmute(node)
    }

    fn children(self) -> LayoutIterator<ServoChildrenIterator<'ln>> {
        LayoutIterator(ServoChildrenIterator {
            current: self.first_child(),
        })
    }

    fn opaque(&self) -> OpaqueNode {
        unsafe { self.get_jsmanaged().opaque() }
    }

    fn debug_id(self) -> usize {
        self.opaque().0
    }

    fn as_element(&self) -> Option<ServoLayoutElement<'ln>> {
        as_element(self.node)
    }

    fn needs_dirty_on_viewport_size_changed(&self) -> bool {
        unsafe { self.node.get_flag(DIRTY_ON_VIEWPORT_SIZE_CHANGE) }
    }

    unsafe fn set_dirty_on_viewport_size_changed(&self) {
        self.node.set_flag(DIRTY_ON_VIEWPORT_SIZE_CHANGE, true);
    }

    fn can_be_fragmented(&self) -> bool {
        unsafe { self.node.get_flag(CAN_BE_FRAGMENTED) }
    }

    unsafe fn set_can_be_fragmented(&self, value: bool) {
        self.node.set_flag(CAN_BE_FRAGMENTED, value)
    }

    fn parent_node(&self) -> Option<ServoLayoutNode<'ln>> {
        unsafe {
            self.node.parent_node_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn is_in_doc(&self) -> bool {
        unsafe { (*self.node.unsafe_get()).is_in_doc() }
    }
}

pub struct ServoChildrenIterator<'a> {
    current: Option<ServoLayoutNode<'a>>,
}

impl<'a> Iterator for ServoChildrenIterator<'a> {
    type Item = ServoLayoutNode<'a>;
    fn next(&mut self) -> Option<ServoLayoutNode<'a>> {
        let node = self.current;
        self.current = node.and_then(|node| node.next_sibling());
        node
    }
}

impl<'ln> LayoutNode for ServoLayoutNode<'ln> {
    type ConcreteThreadSafeLayoutNode = ServoThreadSafeLayoutNode<'ln>;

    fn to_threadsafe(&self) -> Self::ConcreteThreadSafeLayoutNode {
        ServoThreadSafeLayoutNode::new(self)
    }

    fn type_id(&self) -> LayoutNodeType {
        self.script_type_id().into()
    }

    unsafe fn init_style_and_layout_data(&self, data: OpaqueStyleAndLayoutData) {
        self.get_jsmanaged().init_style_and_layout_data(data);
    }

    unsafe fn take_style_and_layout_data(&self) -> OpaqueStyleAndLayoutData {
        self.get_jsmanaged().take_style_and_layout_data()
    }

    fn first_child(&self) -> Option<ServoLayoutNode<'ln>> {
        unsafe {
            self.node.first_child_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn last_child(&self) -> Option<ServoLayoutNode<'ln>> {
        unsafe {
            self.node.last_child_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn prev_sibling(&self) -> Option<ServoLayoutNode<'ln>> {
        unsafe {
            self.node.prev_sibling_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn next_sibling(&self) -> Option<ServoLayoutNode<'ln>> {
        unsafe {
            self.node.next_sibling_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }
}

impl<'ln> GetLayoutData for ServoLayoutNode<'ln> {
    fn get_style_and_layout_data(&self) -> Option<OpaqueStyleAndLayoutData> {
        unsafe {
            self.get_jsmanaged().get_style_and_layout_data()
        }
    }
}

impl<'le> GetLayoutData for ServoLayoutElement<'le> {
    fn get_style_and_layout_data(&self) -> Option<OpaqueStyleAndLayoutData> {
        self.as_node().get_style_and_layout_data()
    }
}

impl<'ln> GetLayoutData for ServoThreadSafeLayoutNode<'ln> {
    fn get_style_and_layout_data(&self) -> Option<OpaqueStyleAndLayoutData> {
        self.node.get_style_and_layout_data()
    }
}

impl<'le> GetLayoutData for ServoThreadSafeLayoutElement<'le> {
    fn get_style_and_layout_data(&self) -> Option<OpaqueStyleAndLayoutData> {
        self.element.as_node().get_style_and_layout_data()
    }
}

impl<'ln> ServoLayoutNode<'ln> {
    /// Returns the interior of this node as a `LayoutJS`. This is highly unsafe for layout to
    /// call and as such is marked `unsafe`.
    pub unsafe fn get_jsmanaged(&self) -> &LayoutJS<Node> {
        &self.node
    }
}

// A wrapper around documents that ensures ayout can only ever access safe properties.
#[derive(Copy, Clone)]
pub struct ServoLayoutDocument<'ld> {
    document: LayoutJS<Document>,
    chain: PhantomData<&'ld ()>,
}

impl<'ld> ServoLayoutDocument<'ld> {
    fn as_node(&self) -> ServoLayoutNode<'ld> {
        ServoLayoutNode::from_layout_js(self.document.upcast())
    }

    pub fn root_node(&self) -> Option<ServoLayoutNode<'ld>> {
        self.as_node().children().find(ServoLayoutNode::is_element)
    }

    pub fn drain_pending_restyles(&self) -> Vec<(ServoLayoutElement<'ld>, PendingRestyle)> {
        let elements =  unsafe { self.document.drain_pending_restyles() };
        elements.into_iter().map(|(el, snapshot)| (ServoLayoutElement::from_layout_js(el), snapshot)).collect()
    }

    pub fn needs_paint_from_layout(&self) {
        unsafe { self.document.needs_paint_from_layout(); }
    }

    pub fn will_paint(&self) {
        unsafe { self.document.will_paint(); }
    }

    pub fn quirks_mode(&self) -> QuirksMode {
        unsafe { self.document.quirks_mode() }
    }

    pub fn style_shared_lock(&self) -> &StyleSharedRwLock {
        unsafe { self.document.style_shared_lock() }
    }

    pub fn from_layout_js(doc: LayoutJS<Document>) -> ServoLayoutDocument<'ld> {
        ServoLayoutDocument {
            document: doc,
            chain: PhantomData,
        }
    }
}

/// A wrapper around elements that ensures layout can only ever access safe properties.
#[derive(Copy, Clone)]
pub struct ServoLayoutElement<'le> {
    element: LayoutJS<Element>,
    chain: PhantomData<&'le ()>,
}

impl<'le> fmt::Debug for ServoLayoutElement<'le> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "<{}", self.element.local_name()));
        if let &Some(ref id) = unsafe { &*self.element.id_attribute() } {
            try!(write!(f, " id={}", id));
        }
        write!(f, "> ({:#x})", self.as_node().opaque().0)
    }
}

impl<'le> PresentationalHintsSynthesizer for ServoLayoutElement<'le> {
    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self,
                                                                _visited_handling: VisitedHandlingMode,
                                                                hints: &mut V)
        where V: Push<ApplicableDeclarationBlock>
    {
        unsafe {
            self.element.synthesize_presentational_hints_for_legacy_attributes(hints);
        }
    }
}

impl<'le> TElement for ServoLayoutElement<'le> {
    type ConcreteNode = ServoLayoutNode<'le>;

    type FontMetricsProvider = ServoMetricsProvider;

    fn as_node(&self) -> ServoLayoutNode<'le> {
        ServoLayoutNode::from_layout_js(self.element.upcast())
    }

    fn style_attribute(&self) -> Option<&Arc<StyleLocked<PropertyDeclarationBlock>>> {
        unsafe {
            (*self.element.style_attribute()).as_ref()
        }
    }

    fn get_state(&self) -> ElementState {
        self.element.get_state_for_layout()
    }

    #[inline]
    fn has_attr(&self, namespace: &Namespace, attr: &LocalName) -> bool {
        self.get_attr(namespace, attr).is_some()
    }

    #[inline]
    fn attr_equals(&self, namespace: &Namespace, attr: &LocalName, val: &Atom) -> bool {
        self.get_attr(namespace, attr).map_or(false, |x| x == val)
    }

    #[inline(always)]
    fn each_class<F>(&self, mut callback: F) where F: FnMut(&Atom) {
        unsafe {
            if let Some(ref classes) = self.element.get_classes_for_layout() {
                for class in *classes {
                    callback(class)
                }
            }
        }
    }

    #[inline]
    fn existing_style_for_restyle_damage<'a>(&'a self,
                                             current_cv: &'a ComputedValues,
                                             _pseudo_element: Option<&PseudoElement>)
                                             -> Option<&'a ComputedValues> {
        Some(current_cv)
    }

    fn has_dirty_descendants(&self) -> bool {
        unsafe { self.as_node().node.get_flag(HAS_DIRTY_DESCENDANTS) }
    }

    fn has_snapshot(&self) -> bool {
        unsafe { self.as_node().node.get_flag(HAS_SNAPSHOT) }
    }

    fn handled_snapshot(&self) -> bool {
        unsafe { self.as_node().node.get_flag(HANDLED_SNAPSHOT) }
    }

    unsafe fn set_handled_snapshot(&self) {
        self.as_node().node.set_flag(HANDLED_SNAPSHOT, true);
    }

    unsafe fn note_descendants<B: DescendantsBit<Self>>(&self) {
        debug_assert!(self.get_data().is_some());
        style::dom::raw_note_descendants::<Self, B>(*self);
    }

    unsafe fn set_dirty_descendants(&self) {
        debug_assert!(self.as_node().node.get_flag(IS_IN_DOC));
        self.as_node().node.set_flag(HAS_DIRTY_DESCENDANTS, true)
    }

    unsafe fn unset_dirty_descendants(&self) {
        self.as_node().node.set_flag(HAS_DIRTY_DESCENDANTS, false)
    }

    fn store_children_to_process(&self, n: isize) {
        let data = self.get_style_data().unwrap();
        data.parallel.children_to_process.store(n, Ordering::Relaxed);
    }

    fn did_process_child(&self) -> isize {
        let data = self.get_style_data().unwrap();
        let old_value = data.parallel.children_to_process.fetch_sub(1, Ordering::Relaxed);
        debug_assert!(old_value >= 1);
        old_value - 1
    }

    fn get_data(&self) -> Option<&AtomicRefCell<ElementData>> {
        unsafe {
            self.get_style_and_layout_data().map(|d| {
                &(*d.ptr.get()).element_data
            })
        }
    }

    fn skip_root_and_item_based_display_fixup(&self) -> bool {
        false
    }

    unsafe fn set_selector_flags(&self, flags: ElementSelectorFlags) {
        self.element.insert_selector_flags(flags);
    }

    fn has_selector_flags(&self, flags: ElementSelectorFlags) -> bool {
        self.element.has_selector_flags(flags)
    }

    fn has_animations(&self) -> bool {
        // We use this function not only for Gecko but also for Servo to know if this element has
        // animations, so we maybe try to get the important rules of this element. This is used for
        // off-main thread animations, but we don't support it on Servo, so return false directly.
        false
    }

    fn has_css_animations(&self) -> bool {
        unreachable!("this should be only called on gecko");
    }

    fn has_css_transitions(&self) -> bool {
        unreachable!("this should be only called on gecko");
    }

    #[inline]
    fn lang_attr(&self) -> Option<SelectorAttrValue> {
        self.get_attr(&ns!(xml), &local_name!("lang"))
            .or_else(|| self.get_attr(&ns!(), &local_name!("lang")))
            .map(|v| String::from(v as &str))
    }

    fn match_element_lang(&self,
                          override_lang: Option<Option<SelectorAttrValue>>,
                          value: &PseudoClassStringArg)
                          -> bool
    {
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
            Some(None) => String::new(),
            None => self.element.get_lang_for_layout(),
        };
        extended_filtering(&element_lang, &*value)
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
    fn from_layout_js(el: LayoutJS<Element>) -> ServoLayoutElement<'le> {
        ServoLayoutElement {
            element: el,
            chain: PhantomData,
        }
    }

    #[inline]
    fn get_attr_enum(&self, namespace: &Namespace, name: &LocalName) -> Option<&AttrValue> {
        unsafe {
            (*self.element.unsafe_get()).get_attr_for_layout(namespace, name)
        }
    }

    #[inline]
    fn get_attr(&self, namespace: &Namespace, name: &LocalName) -> Option<&str> {
        unsafe {
            (*self.element.unsafe_get()).get_attr_val_for_layout(namespace, name)
        }
    }

    fn get_style_data(&self) -> Option<&StyleData> {
        unsafe {
            self.get_style_and_layout_data().map(|d| &*d.ptr.get())
        }
    }

    pub unsafe fn unset_snapshot_flags(&self) {
        self.as_node().node.set_flag(HAS_SNAPSHOT | HANDLED_SNAPSHOT, false);
    }

    pub unsafe fn set_has_snapshot(&self) {
        self.as_node().node.set_flag(HAS_SNAPSHOT, true);
    }

    // FIXME(bholley): This should be merged with TElement::note_descendants,
    // but that requires re-testing and possibly fixing the broken callers given
    // the FIXME below, which I don't have time to do right now.
    //
    // FIXME(emilio): We'd also need to relax the invariant in note_descendants
    // re. the data already available I think.
    pub unsafe fn note_dirty_descendant(&self) {
        use ::selectors::Element;

        let mut current = Some(*self);
        while let Some(el) = current {
            // FIXME(bholley): Ideally we'd have the invariant that any element
            // with has_dirty_descendants also has the bit set on all its
            // ancestors.  However, there are currently some corner-cases where
            // we get that wrong.  I have in-flight patches to fix all this
            // stuff up, so we just always propagate this bit for now.
            el.set_dirty_descendants();
            current = el.parent_element();
        }

        debug_assert!(self.descendants_bit_is_propagated::<DirtyDescendants>());
    }
}

fn as_element<'le>(node: LayoutJS<Node>) -> Option<ServoLayoutElement<'le>> {
    node.downcast().map(ServoLayoutElement::from_layout_js)
}

impl<'le> ::selectors::Element for ServoLayoutElement<'le> {
    type Impl = SelectorImpl;

    fn parent_element(&self) -> Option<ServoLayoutElement<'le>> {
        unsafe {
            self.element.upcast().parent_node_ref().and_then(as_element)
        }
    }

    fn first_child_element(&self) -> Option<ServoLayoutElement<'le>> {
        self.as_node().children().filter_map(|n| n.as_element()).next()
    }

    fn last_child_element(&self) -> Option<ServoLayoutElement<'le>> {
        self.as_node().rev_children().filter_map(|n| n.as_element()).next()
    }

    fn prev_sibling_element(&self) -> Option<ServoLayoutElement<'le>> {
        let mut node = self.as_node();
        while let Some(sibling) = node.prev_sibling() {
            if let Some(element) = sibling.as_element() {
                return Some(element)
            }
            node = sibling;
        }
        None
    }

    fn next_sibling_element(&self) -> Option<ServoLayoutElement<'le>> {
        let mut node = self.as_node();
        while let Some(sibling) = node.next_sibling() {
            if let Some(element) = sibling.as_element() {
                return Some(element)
            }
            node = sibling;
        }
        None
    }

    fn attr_matches(&self,
                    ns: &NamespaceConstraint<&Namespace>,
                    local_name: &LocalName,
                    operation: &AttrSelectorOperation<&String>)
                    -> bool {
        match *ns {
            NamespaceConstraint::Specific(ref ns) => {
                self.get_attr_enum(ns, local_name)
                    .map_or(false, |value| value.eval_selector(operation))
            }
            NamespaceConstraint::Any => {
                let values = unsafe {
                    (*self.element.unsafe_get()).get_attr_vals_for_layout(local_name)
                };
                values.iter().any(|value| value.eval_selector(operation))
            }
        }
    }

    fn is_root(&self) -> bool {
        match self.as_node().parent_node() {
            None => false,
            Some(node) => {
                match node.script_type_id() {
                    NodeTypeId::Document(_) => true,
                    _ => false
                }
            },
        }
    }

    fn is_empty(&self) -> bool {
        self.as_node().children().all(|node| match node.script_type_id() {
            NodeTypeId::Element(..) => false,
            NodeTypeId::CharacterData(CharacterDataTypeId::Text) => unsafe {
                node.node.downcast().unwrap().data_for_layout().is_empty()
            },
            _ => true
        })
    }

    #[inline]
    fn get_local_name(&self) -> &LocalName {
        self.element.local_name()
    }

    #[inline]
    fn get_namespace(&self) -> &Namespace {
        self.element.namespace()
    }

    fn match_pseudo_element(&self,
                            _pseudo: &PseudoElement,
                            _context: &mut MatchingContext)
                            -> bool
    {
        false
    }

    fn match_non_ts_pseudo_class<F>(&self,
                                    pseudo_class: &NonTSPseudoClass,
                                    _: &mut MatchingContext,
                                    _: &RelevantLinkStatus,
                                    _: &mut F)
                                    -> bool
        where F: FnMut(&Self, ElementSelectorFlags),
    {
        match *pseudo_class {
            // https://github.com/servo/servo/issues/8718
            NonTSPseudoClass::Link |
            NonTSPseudoClass::AnyLink => self.is_link(),
            NonTSPseudoClass::Visited => false,

            NonTSPseudoClass::Lang(ref lang) => self.match_element_lang(None, &*lang),

            NonTSPseudoClass::ServoNonZeroBorder => unsafe {
                match (*self.element.unsafe_get()).get_attr_for_layout(&ns!(), &local_name!("border")) {
                    None | Some(&AttrValue::UInt(_, 0)) => false,
                    _ => true,
                }
            },
            NonTSPseudoClass::ServoCaseSensitiveTypeAttr(ref expected_value) => {
                self.get_attr_enum(&ns!(), &local_name!("type"))
                    .map_or(false, |attr| attr == expected_value)
            }
            NonTSPseudoClass::ReadOnly =>
                !self.element.get_state_for_layout().contains(pseudo_class.state_flag()),

            NonTSPseudoClass::Active |
            NonTSPseudoClass::Focus |
            NonTSPseudoClass::Fullscreen |
            NonTSPseudoClass::Hover |
            NonTSPseudoClass::Enabled |
            NonTSPseudoClass::Disabled |
            NonTSPseudoClass::Checked |
            NonTSPseudoClass::Indeterminate |
            NonTSPseudoClass::ReadWrite |
            NonTSPseudoClass::PlaceholderShown |
            NonTSPseudoClass::Target =>
                self.element.get_state_for_layout().contains(pseudo_class.state_flag())
        }
    }

    #[inline]
    fn is_link(&self) -> bool {
        unsafe {
            match self.as_node().script_type_id() {
                // https://html.spec.whatwg.org/multipage/#selector-link
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) |
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement)) |
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) =>
                    (*self.element.unsafe_get()).get_attr_val_for_layout(&ns!(), &local_name!("href")).is_some(),
                _ => false,
            }
        }
    }

    #[inline]
    fn get_id(&self) -> Option<Atom> {
        unsafe {
            (*self.element.id_attribute()).clone()
        }
    }

    #[inline]
    fn has_class(&self, name: &Atom) -> bool {
        unsafe {
            self.element.has_class_for_layout(name)
        }
    }

    fn is_html_element_in_html_document(&self) -> bool {
        unsafe {
            self.element.html_element_in_html_document_for_layout()
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ServoThreadSafeLayoutNode<'ln> {
    /// The wrapped node.
    node: ServoLayoutNode<'ln>,

    /// The pseudo-element type, with (optionally)
    /// a specified display value to override the stylesheet.
    pseudo: PseudoElementType<Option<display::T>>,
}

impl<'a> PartialEq for ServoThreadSafeLayoutNode<'a> {
    #[inline]
    fn eq(&self, other: &ServoThreadSafeLayoutNode<'a>) -> bool {
        self.node == other.node
    }
}

impl<'ln> DangerousThreadSafeLayoutNode for ServoThreadSafeLayoutNode<'ln> {
    unsafe fn dangerous_first_child(&self) -> Option<Self> {
            self.get_jsmanaged().first_child_ref()
                .map(|node| self.new_with_this_lifetime(&node))
    }
    unsafe fn dangerous_next_sibling(&self) -> Option<Self> {
            self.get_jsmanaged().next_sibling_ref()
                .map(|node| self.new_with_this_lifetime(&node))
    }
}

impl<'ln> ServoThreadSafeLayoutNode<'ln> {
    /// Creates a new layout node with the same lifetime as this layout node.
    pub unsafe fn new_with_this_lifetime(&self, node: &LayoutJS<Node>) -> ServoThreadSafeLayoutNode<'ln> {
        ServoThreadSafeLayoutNode {
            node: self.node.new_with_this_lifetime(node),
            pseudo: PseudoElementType::Normal,
        }
    }

    /// Creates a new `ServoThreadSafeLayoutNode` from the given `ServoLayoutNode`.
    pub fn new<'a>(node: &ServoLayoutNode<'a>) -> ServoThreadSafeLayoutNode<'a> {
        ServoThreadSafeLayoutNode {
            node: node.clone(),
            pseudo: PseudoElementType::Normal,
        }
    }

    /// Returns the interior of this node as a `LayoutJS`. This is highly unsafe for layout to
    /// call and as such is marked `unsafe`.
    unsafe fn get_jsmanaged(&self) -> &LayoutJS<Node> {
        self.node.get_jsmanaged()
    }
}

// NB: The implementation here is a bit tricky because elements implementing
// pseudos are supposed to return false for is_element().
impl<'ln> NodeInfo for ServoThreadSafeLayoutNode<'ln> {
    fn is_element(&self) -> bool {
        self.pseudo == PseudoElementType::Normal && self.node.is_element()
    }

    fn is_text_node(&self) -> bool {
        self.node.is_text_node()
    }

    fn needs_layout(&self) -> bool {
        self.node.is_text_node() || self.node.is_element()
    }
}

impl<'ln> ThreadSafeLayoutNode for ServoThreadSafeLayoutNode<'ln> {
    type ConcreteNode = ServoLayoutNode<'ln>;
    type ConcreteThreadSafeLayoutElement = ServoThreadSafeLayoutElement<'ln>;
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

    #[inline]
    fn type_id_without_excluding_pseudo_elements(&self) -> LayoutNodeType {
        self.node.type_id()
    }

    fn parent_style(&self) -> Arc<ComputedValues> {
        let parent = self.node.parent_node().unwrap().as_element().unwrap();
        let parent_data = parent.get_data().unwrap().borrow();
        parent_data.styles().primary.values().clone()
    }

    fn debug_id(self) -> usize {
        self.node.debug_id()
    }

    fn children(&self) -> LayoutIterator<Self::ChildrenIterator> {
        LayoutIterator(ThreadSafeLayoutNodeChildrenIterator::new(*self))
    }

    fn as_element(&self) -> Option<ServoThreadSafeLayoutElement<'ln>> {
        self.node.as_element().map(|el| ServoThreadSafeLayoutElement {
            element: el,
            pseudo: self.pseudo,
        })
    }

    fn get_style_and_layout_data(&self) -> Option<OpaqueStyleAndLayoutData> {
        self.node.get_style_and_layout_data()
    }

    fn is_ignorable_whitespace(&self, context: &SharedStyleContext) -> bool {
        unsafe {
            let text: LayoutJS<Text> = match self.get_jsmanaged().downcast() {
                Some(text) => text,
                None => return false
            };

            if !is_whitespace(text.upcast().data_for_layout()) {
                return false
            }

            // NB: See the rules for `white-space` here:
            //
            //    http://www.w3.org/TR/CSS21/text.html#propdef-white-space
            //
            // If you implement other values for this property, you will almost certainly
            // want to update this check.
            !self.style(context).get_inheritedtext().white_space.preserve_newlines()
        }
    }

    unsafe fn unsafe_get(self) -> Self::ConcreteNode {
        self.node
    }

    fn can_be_fragmented(&self) -> bool {
        self.node.can_be_fragmented()
    }

    fn node_text_content(&self) -> String {
        let this = unsafe { self.get_jsmanaged() };
        return this.text_content();
    }

    fn selection(&self) -> Option<Range<ByteIndex>> {
        let this = unsafe { self.get_jsmanaged() };

        this.selection().map(|range| {
            Range::new(ByteIndex(range.start as isize),
                       ByteIndex(range.len() as isize))
        })
    }

    fn image_url(&self) -> Option<ServoUrl> {
        let this = unsafe { self.get_jsmanaged() };
        this.image_url()
    }

    fn canvas_data(&self) -> Option<HTMLCanvasData> {
        let this = unsafe { self.get_jsmanaged() };
        this.canvas_data()
    }

    fn svg_data(&self) -> Option<SVGSVGData> {
        let this = unsafe { self.get_jsmanaged() };
        this.svg_data()
    }

    fn iframe_browsing_context_id(&self) -> BrowsingContextId {
        let this = unsafe { self.get_jsmanaged() };
        this.iframe_browsing_context_id()
    }

    fn iframe_pipeline_id(&self) -> PipelineId {
        let this = unsafe { self.get_jsmanaged() };
        this.iframe_pipeline_id()
    }

    fn get_colspan(&self) -> u32 {
        unsafe {
            self.get_jsmanaged().downcast::<Element>().unwrap().get_colspan()
        }
    }

    fn get_rowspan(&self) -> u32 {
        unsafe {
            self.get_jsmanaged().downcast::<Element>().unwrap().get_rowspan()
        }
    }
}

pub struct ThreadSafeLayoutNodeChildrenIterator<ConcreteNode: ThreadSafeLayoutNode> {
    current_node: Option<ConcreteNode>,
    parent_node: ConcreteNode,
}

impl<ConcreteNode> ThreadSafeLayoutNodeChildrenIterator<ConcreteNode>
                   where ConcreteNode: DangerousThreadSafeLayoutNode {
    pub fn new(parent: ConcreteNode) -> Self {
        let first_child: Option<ConcreteNode> = match parent.get_pseudo_element_type() {
            PseudoElementType::Normal => {
                parent.get_before_pseudo().or_else(|| parent.get_details_summary_pseudo()).or_else(|| {
                    unsafe { parent.dangerous_first_child() }
                })
            },
            PseudoElementType::DetailsContent(_) | PseudoElementType::DetailsSummary(_) => {
                unsafe { parent.dangerous_first_child() }
            },
            _ => None,
        };
        ThreadSafeLayoutNodeChildrenIterator {
            current_node: first_child,
            parent_node: parent,
        }
    }
}

impl<ConcreteNode> Iterator for ThreadSafeLayoutNodeChildrenIterator<ConcreteNode>
                            where ConcreteNode: DangerousThreadSafeLayoutNode {
    type Item = ConcreteNode;
    fn next(&mut self) -> Option<ConcreteNode> {
        use ::selectors::Element;
        match self.parent_node.get_pseudo_element_type() {
            PseudoElementType::Before(_) | PseudoElementType::After(_) => None,

            PseudoElementType::DetailsSummary(_) => {
                let mut current_node = self.current_node.clone();
                loop {
                    let next_node = if let Some(ref node) = current_node {
                        if let Some(element) = node.as_element() {
                            if element.get_local_name() == &local_name!("summary") &&
                               element.get_namespace() == &ns!(html) {
                                self.current_node = None;
                                return Some(node.clone());
                            }
                        }
                        unsafe { node.dangerous_next_sibling() }
                    } else {
                        self.current_node = None;
                        return None
                    };
                    current_node = next_node;
                }
            }

            PseudoElementType::DetailsContent(_) => {
                let node = self.current_node.clone();
                let node = node.and_then(|node| {
                    if node.is_element() &&
                       node.as_element().unwrap().get_local_name() == &local_name!("summary") &&
                       node.as_element().unwrap().get_namespace() == &ns!(html) {
                        unsafe { node.dangerous_next_sibling() }
                    } else {
                        Some(node)
                    }
                });
                self.current_node = node.and_then(|node| unsafe { node.dangerous_next_sibling() });
                node
            }

            PseudoElementType::Normal => {
                let node = self.current_node.clone();
                if let Some(ref node) = node {
                    self.current_node = match node.get_pseudo_element_type() {
                        PseudoElementType::Before(_) => {
                            self.parent_node.get_details_summary_pseudo()
                                .or_else(|| unsafe { self.parent_node.dangerous_first_child() })
                                .or_else(|| self.parent_node.get_after_pseudo())
                        },
                        PseudoElementType::Normal => {
                            unsafe { node.dangerous_next_sibling() }.or_else(|| self.parent_node.get_after_pseudo())
                        },
                        PseudoElementType::DetailsSummary(_) => self.parent_node.get_details_content_pseudo(),
                        PseudoElementType::DetailsContent(_) => self.parent_node.get_after_pseudo(),
                        PseudoElementType::After(_) => {
                            None
                        },
                    };
                }
                node
            }

        }
    }
}

/// A wrapper around elements that ensures layout can only
/// ever access safe properties and cannot race on elements.
#[derive(Copy, Clone, Debug)]
pub struct ServoThreadSafeLayoutElement<'le> {
    element: ServoLayoutElement<'le>,

    /// The pseudo-element type, with (optionally)
    /// a specified display value to override the stylesheet.
    pseudo: PseudoElementType<Option<display::T>>,
}

impl<'le> ThreadSafeLayoutElement for ServoThreadSafeLayoutElement<'le> {
    type ConcreteThreadSafeLayoutNode = ServoThreadSafeLayoutNode<'le>;

    fn as_node(&self) -> ServoThreadSafeLayoutNode<'le> {
        ServoThreadSafeLayoutNode {
            node: self.element.as_node(),
            pseudo: self.pseudo.clone(),
        }
    }

    fn get_pseudo_element_type(&self) -> PseudoElementType<Option<display::T>> {
        self.pseudo
    }

    fn with_pseudo(&self,
                   pseudo: PseudoElementType<Option<display::T>>) -> Self {
        ServoThreadSafeLayoutElement {
            element: self.element.clone(),
            pseudo: pseudo,
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
        self.element.get_data()
            .expect("Unstyled layout node?")
            .borrow()
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

    fn parent_element(&self) -> Option<Self> {
        warn!("ServoThreadSafeLayoutElement::parent_element called");
        None
    }

    fn first_child_element(&self) -> Option<Self> {
        warn!("ServoThreadSafeLayoutElement::first_child_element called");
        None
    }

    // Skips non-element nodes
    fn last_child_element(&self) -> Option<Self> {
        warn!("ServoThreadSafeLayoutElement::last_child_element called");
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

    fn is_html_element_in_html_document(&self) -> bool {
        debug!("ServoThreadSafeLayoutElement::is_html_element_in_html_document called");
        true
    }

    #[inline]
    fn get_local_name(&self) -> &LocalName {
        self.element.get_local_name()
    }

    #[inline]
    fn get_namespace(&self) -> &Namespace {
        self.element.get_namespace()
    }

    fn match_pseudo_element(&self,
                            _pseudo: &PseudoElement,
                            _context: &mut MatchingContext)
                            -> bool
    {
        false
    }

    fn attr_matches(&self,
                    ns: &NamespaceConstraint<&Namespace>,
                    local_name: &LocalName,
                    operation: &AttrSelectorOperation<&String>)
                    -> bool {
        match *ns {
            NamespaceConstraint::Specific(ref ns) => {
                self.get_attr_enum(ns, local_name)
                    .map_or(false, |value| value.eval_selector(operation))
            }
            NamespaceConstraint::Any => {
                let values = unsafe {
                    (*self.element.element.unsafe_get()).get_attr_vals_for_layout(local_name)
                };
                values.iter().any(|v| v.eval_selector(operation))
            }
        }
    }

    fn match_non_ts_pseudo_class<F>(&self,
                                    _: &NonTSPseudoClass,
                                    _: &mut MatchingContext,
                                    _: &RelevantLinkStatus,
                                    _: &mut F)
                                    -> bool
        where F: FnMut(&Self, ElementSelectorFlags),
    {
        // NB: This could maybe be implemented
        warn!("ServoThreadSafeLayoutElement::match_non_ts_pseudo_class called");
        false
    }

    fn is_link(&self) -> bool {
        warn!("ServoThreadSafeLayoutElement::is_link called");
        false
    }

    fn get_id(&self) -> Option<Atom> {
        debug!("ServoThreadSafeLayoutElement::get_id called");
        None
    }

    fn has_class(&self, _name: &Atom) -> bool {
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

impl<'le> PresentationalHintsSynthesizer for ServoThreadSafeLayoutElement<'le> {
    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self,
                                                                _visited_handling: VisitedHandlingMode,
                                                                _hints: &mut V)
        where V: Push<ApplicableDeclarationBlock> {}
}
