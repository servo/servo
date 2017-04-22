/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

//! Wrapper definitions on top of Gecko types in order to be used in the style
//! system.
//!
//! This really follows the Servo pattern in
//! `components/script/layout_wrapper.rs`.
//!
//! This theoretically should live in its own crate, but now it lives in the
//! style system it's kind of pointless in the Stylo case, and only Servo forces
//! the separation between the style system implementation and everything else.

use app_units::Au;
use atomic_refcell::AtomicRefCell;
use context::{SharedStyleContext, UpdateAnimationsTasks};
use data::ElementData;
use dom::{self, AnimationRules, DescendantsBit, LayoutIterator, NodeInfo, TElement, TNode, UnsafeNode};
use dom::{OpaqueNode, PresentationalHintsSynthetizer};
use element_state::ElementState;
use error_reporting::StdoutErrorReporter;
use font_metrics::{FontMetricsProvider, FontMetricsQueryResult};
use gecko::global_style_data::GLOBAL_STYLE_DATA;
use gecko::selector_parser::{SelectorImpl, NonTSPseudoClass, PseudoElement};
use gecko::snapshot_helpers;
use gecko_bindings::bindings;
use gecko_bindings::bindings::{Gecko_DropStyleChildrenIterator, Gecko_MaybeCreateStyleChildrenIterator};
use gecko_bindings::bindings::{Gecko_ElementState, Gecko_GetLastChild, Gecko_GetNextStyleChild};
use gecko_bindings::bindings::{Gecko_IsRootElement, Gecko_MatchesElement, Gecko_Namespace};
use gecko_bindings::bindings::{Gecko_SetNodeFlags, Gecko_UnsetNodeFlags};
use gecko_bindings::bindings::Gecko_ClassOrClassList;
use gecko_bindings::bindings::Gecko_ElementHasAnimations;
use gecko_bindings::bindings::Gecko_ElementHasCSSAnimations;
use gecko_bindings::bindings::Gecko_ElementHasCSSTransitions;
use gecko_bindings::bindings::Gecko_GetAnimationRule;
use gecko_bindings::bindings::Gecko_GetExtraContentStyleDeclarations;
use gecko_bindings::bindings::Gecko_GetHTMLPresentationAttrDeclarationBlock;
use gecko_bindings::bindings::Gecko_GetStyleAttrDeclarationBlock;
use gecko_bindings::bindings::Gecko_GetStyleContext;
use gecko_bindings::bindings::Gecko_IsSignificantChild;
use gecko_bindings::bindings::Gecko_MatchStringArgPseudo;
use gecko_bindings::bindings::Gecko_UpdateAnimations;
use gecko_bindings::structs;
use gecko_bindings::structs::{RawGeckoElement, RawGeckoNode};
use gecko_bindings::structs::{nsIAtom, nsIContent, nsStyleContext};
use gecko_bindings::structs::EffectCompositor_CascadeLevel as CascadeLevel;
use gecko_bindings::structs::NODE_HAS_ANIMATION_ONLY_DIRTY_DESCENDANTS_FOR_SERVO;
use gecko_bindings::structs::NODE_HAS_DIRTY_DESCENDANTS_FOR_SERVO;
use gecko_bindings::structs::NODE_IS_IN_NATIVE_ANONYMOUS_SUBTREE;
use gecko_bindings::structs::NODE_IS_NATIVE_ANONYMOUS;
use gecko_bindings::sugar::ownership::HasArcFFI;
use logical_geometry::WritingMode;
use media_queries::Device;
use parking_lot::RwLock;
use properties::{ComputedValues, parse_style_attribute};
use properties::{Importance, PropertyDeclaration, PropertyDeclarationBlock};
use properties::animated_properties::{AnimationValue, AnimationValueMap, TransitionProperty};
use properties::style_structs::Font;
use rule_tree::CascadeLevel as ServoCascadeLevel;
use selector_parser::{ElementExt, Snapshot};
use selectors::Element;
use selectors::matching::{ElementSelectorFlags, StyleRelations};
use selectors::parser::{AttrSelector, NamespaceConstraint};
use shared_lock::Locked;
use sink::Push;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ptr;
use std::sync::Arc;
use string_cache::{Atom, Namespace, WeakAtom, WeakNamespace};
use stylesheets::UrlExtraData;
use stylist::ApplicableDeclarationBlock;

/// A simple wrapper over a non-null Gecko node (`nsINode`) pointer.
///
/// Important: We don't currently refcount the DOM, because the wrapper lifetime
/// magic guarantees that our LayoutFoo references won't outlive the root, and
/// we don't mutate any of the references on the Gecko side during restyle.
///
/// We could implement refcounting if need be (at a potentially non-trivial
/// performance cost) by implementing Drop and making LayoutFoo non-Copy.
#[derive(Clone, Copy)]
pub struct GeckoNode<'ln>(pub &'ln RawGeckoNode);

impl<'ln> fmt::Debug for GeckoNode<'ln> {
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

impl<'ln> GeckoNode<'ln> {
    fn from_content(content: &'ln nsIContent) -> Self {
        GeckoNode(&content._base)
    }

    fn flags(&self) -> u32 {
        (self.0)._base._base_1.mFlags
    }

    fn node_info(&self) -> &structs::NodeInfo {
        debug_assert!(!self.0.mNodeInfo.mRawPtr.is_null());
        unsafe { &*self.0.mNodeInfo.mRawPtr }
    }

    fn owner_doc(&self) -> &structs::nsIDocument {
        debug_assert!(!self.node_info().mDocument.is_null());
        unsafe { &*self.node_info().mDocument }
    }

    fn first_child(&self) -> Option<GeckoNode<'ln>> {
        unsafe { self.0.mFirstChild.as_ref().map(GeckoNode::from_content) }
    }

    fn last_child(&self) -> Option<GeckoNode<'ln>> {
        unsafe { Gecko_GetLastChild(self.0).map(GeckoNode) }
    }

    fn prev_sibling(&self) -> Option<GeckoNode<'ln>> {
        unsafe { self.0.mPreviousSibling.as_ref().map(GeckoNode::from_content) }
    }

    fn next_sibling(&self) -> Option<GeckoNode<'ln>> {
        unsafe { self.0.mNextSibling.as_ref().map(GeckoNode::from_content) }
    }

    /// Simple iterator over all this node's children.  Unlike `.children()`, this iterator does
    /// not filter out nodes that don't need layout.
    fn dom_children(self) -> GeckoChildrenIterator<'ln> {
        GeckoChildrenIterator::Current(self.first_child())
    }

    /// WARNING: This logic is duplicated in Gecko's FlattenedTreeParentIsParent.
    /// Make sure to mirror any modifications in both places.
    fn flattened_tree_parent_is_parent(&self) -> bool {
        use ::gecko_bindings::structs::*;
        let flags = self.flags();
        if flags & (NODE_MAY_BE_IN_BINDING_MNGR as u32 |
                    NODE_IS_IN_SHADOW_TREE as u32) != 0 {
            return false;
        }

        let parent = unsafe { self.0.mParent.as_ref() }.map(GeckoNode);
        let parent_el = parent.and_then(|p| p.as_element());
        if flags & (NODE_IS_NATIVE_ANONYMOUS_ROOT as u32) != 0 &&
           parent_el.map_or(false, |el| el.is_root())
        {
            return false;
        }

        if parent_el.map_or(false, |el| el.has_shadow_root()) {
            return false;
        }

        true
    }

    fn contains_non_whitespace_content(&self) -> bool {
        unsafe { Gecko_IsSignificantChild(self.0, true, false) }
    }
}

impl<'ln> NodeInfo for GeckoNode<'ln> {
    fn is_element(&self) -> bool {
        use gecko_bindings::structs::nsINode_BooleanFlag;
        self.0.mBoolFlags & (1u32 << nsINode_BooleanFlag::NodeIsElement as u32) != 0
    }

    fn is_text_node(&self) -> bool {
        // This is a DOM constant that isn't going to change.
        const TEXT_NODE: u16 = 3;
        self.node_info().mInner.mNodeType == TEXT_NODE
    }
}

impl<'ln> TNode for GeckoNode<'ln> {
    type ConcreteElement = GeckoElement<'ln>;
    type ConcreteChildrenIterator = GeckoChildrenIterator<'ln>;

    fn to_unsafe(&self) -> UnsafeNode {
        (self.0 as *const _ as usize, 0)
    }

    unsafe fn from_unsafe(n: &UnsafeNode) -> Self {
        GeckoNode(&*(n.0 as *mut RawGeckoNode))
    }

    fn children(self) -> LayoutIterator<GeckoChildrenIterator<'ln>> {
        let maybe_iter = unsafe { Gecko_MaybeCreateStyleChildrenIterator(self.0) };
        if let Some(iter) = maybe_iter.into_owned_opt() {
            LayoutIterator(GeckoChildrenIterator::GeckoIterator(iter))
        } else {
            LayoutIterator(GeckoChildrenIterator::Current(self.first_child()))
        }
    }

    fn opaque(&self) -> OpaqueNode {
        let ptr: usize = self.0 as *const _ as usize;
        OpaqueNode(ptr)
    }

    fn debug_id(self) -> usize {
        unimplemented!()
    }

    fn as_element(&self) -> Option<GeckoElement<'ln>> {
        if self.is_element() {
            unsafe { Some(GeckoElement(&*(self.0 as *const _ as *const RawGeckoElement))) }
        } else {
            None
        }
    }

    fn can_be_fragmented(&self) -> bool {
        // FIXME(SimonSapin): Servo uses this to implement CSS multicol / fragmentation
        // Maybe this isn’t useful for Gecko?
        false
    }

    unsafe fn set_can_be_fragmented(&self, _value: bool) {
        // FIXME(SimonSapin): Servo uses this to implement CSS multicol / fragmentation
        // Maybe this isn’t useful for Gecko?
    }

    fn parent_node(&self) -> Option<Self> {
        let fast_path = self.flattened_tree_parent_is_parent();
        debug_assert!(fast_path == unsafe { bindings::Gecko_FlattenedTreeParentIsParent(self.0) });
        if fast_path {
            unsafe { self.0.mParent.as_ref().map(GeckoNode) }
        } else {
            unsafe { bindings::Gecko_GetParentNode(self.0).map(GeckoNode) }
        }
    }

    fn is_in_doc(&self) -> bool {
        unsafe { bindings::Gecko_IsInDocument(self.0) }
    }

    fn needs_dirty_on_viewport_size_changed(&self) -> bool {
        // Gecko's node doesn't have the DIRTY_ON_VIEWPORT_SIZE_CHANGE flag,
        // so we force them to be dirtied on viewport size change, regardless if
        // they use viewport percentage size or not.
        // TODO(shinglyu): implement this in Gecko: https://github.com/servo/servo/pull/11890
        true
    }

    // TODO(shinglyu): implement this in Gecko: https://github.com/servo/servo/pull/11890
    unsafe fn set_dirty_on_viewport_size_changed(&self) {}
}

/// A wrapper on top of two kind of iterators, depending on the parent being
/// iterated.
///
/// We generally iterate children by traversing the light-tree siblings of the
/// first child like Servo does.
///
/// However, for nodes with anonymous children, we use a custom (heavier-weight)
/// Gecko-implemented iterator.
///
/// FIXME(emilio): If we take into account shadow DOM, we're going to need the
/// flat tree pretty much always. We can try to optimize the case where there's
/// no shadow root sibling, probably.
pub enum GeckoChildrenIterator<'a> {
    /// A simple iterator that tracks the current node being iterated and
    /// replaces it with the next sibling when requested.
    Current(Option<GeckoNode<'a>>),
    /// A Gecko-implemented iterator we need to drop appropriately.
    GeckoIterator(bindings::StyleChildrenIteratorOwned),
}

impl<'a> Drop for GeckoChildrenIterator<'a> {
    fn drop(&mut self) {
        if let GeckoChildrenIterator::GeckoIterator(ref it) = *self {
            unsafe {
                Gecko_DropStyleChildrenIterator(ptr::read(it as *const _));
            }
        }
    }
}

impl<'a> Iterator for GeckoChildrenIterator<'a> {
    type Item = GeckoNode<'a>;
    fn next(&mut self) -> Option<GeckoNode<'a>> {
        match *self {
            GeckoChildrenIterator::Current(curr) => {
                let next = curr.and_then(|node| node.next_sibling());
                *self = GeckoChildrenIterator::Current(next);
                curr
            },
            GeckoChildrenIterator::GeckoIterator(ref mut it) => unsafe {
                Gecko_GetNextStyleChild(it).map(GeckoNode)
            }
        }
    }
}

/// A simple wrapper over a non-null Gecko `Element` pointer.
#[derive(Clone, Copy)]
pub struct GeckoElement<'le>(pub &'le RawGeckoElement);

impl<'le> fmt::Debug for GeckoElement<'le> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "<{}", self.get_local_name()));
        if let Some(id) = self.get_id() {
            try!(write!(f, " id={}", id));
        }
        write!(f, "> ({:#x})", self.as_node().opaque().0)
    }
}

impl<'le> GeckoElement<'le> {
    /// Parse the style attribute of an element.
    pub fn parse_style_attribute(value: &str,
                                 url_data: &UrlExtraData) -> PropertyDeclarationBlock {
        parse_style_attribute(value, url_data, &StdoutErrorReporter)
    }

    fn flags(&self) -> u32 {
        self.raw_node()._base._base_1.mFlags
    }

    fn raw_node(&self) -> &RawGeckoNode {
        &(self.0)._base._base._base
    }

    // FIXME: We can implement this without OOL calls, but we can't easily given
    // GeckoNode is a raw reference.
    //
    // We can use a Cell<T>, but that's a bit of a pain.
    fn set_flags(&self, flags: u32) {
        unsafe { Gecko_SetNodeFlags(self.as_node().0, flags) }
    }

    fn unset_flags(&self, flags: u32) {
        unsafe { Gecko_UnsetNodeFlags(self.as_node().0, flags) }
    }

    /// Returns true if this element has a shadow root.
    fn has_shadow_root(&self) -> bool {
        self.get_dom_slots().map_or(false, |slots| !slots.mShadowRoot.mRawPtr.is_null())
    }

    /// Returns a reference to the DOM slots for this Element, if they exist.
    fn get_dom_slots(&self) -> Option<&structs::FragmentOrElement_nsDOMSlots> {
        let slots = self.as_node().0.mSlots as *const structs::FragmentOrElement_nsDOMSlots;
        unsafe { slots.as_ref() }
    }

    /// Clear the element data for a given element.
    pub fn clear_data(&self) {
        let ptr = self.0.mServoData.get();
        if !ptr.is_null() {
            debug!("Dropping ElementData for {:?}", self);
            let data = unsafe { Box::from_raw(self.0.mServoData.get()) };
            self.0.mServoData.set(ptr::null_mut());

            // Perform a mutable borrow of the data in debug builds. This
            // serves as an assertion that there are no outstanding borrows
            // when we destroy the data.
            debug_assert!({ let _ = data.borrow_mut(); true });
        }
    }

    /// Ensures the element has data, returning the existing data or allocating
    /// it.
    ///
    /// Only safe to call with exclusive access to the element, given otherwise
    /// it could race to allocate and leak.
    pub unsafe fn ensure_data(&self) -> &AtomicRefCell<ElementData> {
        match self.get_data() {
            Some(x) => x,
            None => {
                debug!("Creating ElementData for {:?}", self);
                let ptr = Box::into_raw(Box::new(AtomicRefCell::new(ElementData::new(None))));
                self.0.mServoData.set(ptr);
                unsafe { &* ptr }
            },
        }
    }

    /// Creates a blank snapshot for this element.
    pub fn create_snapshot(&self) -> Snapshot {
        Snapshot::new(*self)
    }
}

/// Converts flags from the layout used by rust-selectors to the layout used
/// by Gecko. We could align these and then do this without conditionals, but
/// it's probably not worth the trouble.
fn selector_flags_to_node_flags(flags: ElementSelectorFlags) -> u32 {
    use gecko_bindings::structs::*;
    use selectors::matching::*;
    let mut gecko_flags = 0u32;
    if flags.contains(HAS_SLOW_SELECTOR) {
        gecko_flags |= NODE_HAS_SLOW_SELECTOR as u32;
    }
    if flags.contains(HAS_SLOW_SELECTOR_LATER_SIBLINGS) {
        gecko_flags |= NODE_HAS_SLOW_SELECTOR_LATER_SIBLINGS as u32;
    }
    if flags.contains(HAS_EDGE_CHILD_SELECTOR) {
        gecko_flags |= NODE_HAS_EDGE_CHILD_SELECTOR as u32;
    }
    if flags.contains(HAS_EMPTY_SELECTOR) {
        gecko_flags |= NODE_HAS_EMPTY_SELECTOR as u32;
    }

    gecko_flags
}

fn get_animation_rule(element: &GeckoElement,
                      pseudo: Option<&PseudoElement>,
                      cascade_level: CascadeLevel)
                      -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
    let atom_ptr = PseudoElement::ns_atom_or_null_from_opt(pseudo);
    let animation_values = Arc::new(RwLock::new(AnimationValueMap::new()));
    if unsafe { Gecko_GetAnimationRule(element.0, atom_ptr, cascade_level,
                                       HasArcFFI::arc_as_borrowed(&animation_values)) } {
        let shared_lock = &GLOBAL_STYLE_DATA.shared_lock;
        Some(Arc::new(shared_lock.wrap(
            PropertyDeclarationBlock::from_animation_value_map(&animation_values.read()))))
    } else {
        None
    }
}

#[derive(Debug)]
/// Gecko font metrics provider
pub struct GeckoFontMetricsProvider {
    /// Cache of base font sizes for each language
    ///
    /// Usually will have 1 element.
    ///
    // This may be slow on pages using more languages, might be worth optimizing
    // by caching lang->group mapping separately and/or using a hashmap on larger
    // loads.
    pub font_size_cache: RefCell<Vec<(Atom, ::gecko_bindings::structs::FontSizePrefs)>>,
}

impl GeckoFontMetricsProvider {
    /// Construct
    pub fn new() -> Self {
        GeckoFontMetricsProvider {
            font_size_cache: RefCell::new(Vec::new()),
        }
    }
}

impl FontMetricsProvider for GeckoFontMetricsProvider {
    fn create_from(_: &SharedStyleContext) -> GeckoFontMetricsProvider {
        GeckoFontMetricsProvider::new()
    }

    fn get_size(&self, font_name: &Atom, font_family: u8) -> Au {
        use gecko_bindings::bindings::Gecko_GetBaseSize;
        let mut cache = self.font_size_cache.borrow_mut();
        if let Some(sizes) = cache.iter().find(|el| el.0 == *font_name) {
            return sizes.1.size_for_generic(font_family);
        }
        let sizes = unsafe {
            Gecko_GetBaseSize(font_name.as_ptr())
        };
        cache.push((font_name.clone(), sizes));
        sizes.size_for_generic(font_family)
    }

    fn query(&self, _font: &Font, _font_size: Au, _wm: WritingMode,
             _in_media_query: bool, _device: &Device) -> FontMetricsQueryResult {
        // Disabled until we can make font metrics thread safe (bug 1356105)
        //
        // use gecko_bindings::bindings::Gecko_GetFontMetrics;
        // let gecko_metrics = unsafe {
        //     Gecko_GetFontMetrics(&*device.pres_context,
        //                          wm.is_vertical() && !wm.is_sideways(),
        //                          font.gecko(),
        //                          font_size.0,
        //                          // we don't use the user font set in a media query
        //                          !in_media_query)
        // };
        // let metrics = FontMetrics {
        //     x_height: Au(gecko_metrics.mXSize),
        //     zero_advance_measure: Au(gecko_metrics.mChSize),
        // };
        // FontMetricsQueryResult::Available(metrics)
        FontMetricsQueryResult::NotAvailable
    }
}

impl structs::FontSizePrefs {
    fn size_for_generic(&self, font_family: u8) -> Au {
        Au(match font_family {
            structs::kPresContext_DefaultVariableFont_ID => self.mDefaultVariableSize,
            structs::kPresContext_DefaultFixedFont_ID => self.mDefaultFixedSize,
            structs::kGenericFont_serif => self.mDefaultSerifSize,
            structs::kGenericFont_sans_serif => self.mDefaultSansSerifSize,
            structs::kGenericFont_monospace => self.mDefaultMonospaceSize,
            structs::kGenericFont_cursive => self.mDefaultCursiveSize,
            structs::kGenericFont_fantasy => self.mDefaultFantasySize,
            x => unreachable!("Unknown generic ID {}", x),
        })
    }
}

impl<'le> TElement for GeckoElement<'le> {
    type ConcreteNode = GeckoNode<'le>;
    type FontMetricsProvider = GeckoFontMetricsProvider;

    fn as_node(&self) -> Self::ConcreteNode {
        unsafe { GeckoNode(&*(self.0 as *const _ as *const RawGeckoNode)) }
    }

    fn style_attribute(&self) -> Option<&Arc<Locked<PropertyDeclarationBlock>>> {
        let declarations = unsafe { Gecko_GetStyleAttrDeclarationBlock(self.0) };
        declarations.map(|s| s.as_arc_opt()).unwrap_or(None)
    }

    fn get_animation_rules(&self, pseudo: Option<&PseudoElement>) -> AnimationRules {
        AnimationRules(self.get_animation_rule(pseudo),
                       self.get_transition_rule(pseudo))
    }

    fn get_animation_rule_by_cascade(&self,
                                     pseudo: Option<&PseudoElement>,
                                     cascade_level: ServoCascadeLevel)
                                     -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        match cascade_level {
            ServoCascadeLevel::Animations => self.get_animation_rule(pseudo),
            ServoCascadeLevel::Transitions => self.get_transition_rule(pseudo),
            _ => panic!("Unsupported cascade level for getting the animation rule")
        }
    }

    fn get_animation_rule(&self, pseudo: Option<&PseudoElement>)
                          -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        get_animation_rule(self, pseudo, CascadeLevel::Animations)
    }

    fn get_transition_rule(&self, pseudo: Option<&PseudoElement>)
                           -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        get_animation_rule(self, pseudo, CascadeLevel::Transitions)
    }

    fn get_state(&self) -> ElementState {
        unsafe {
            ElementState::from_bits_truncate(Gecko_ElementState(self.0))
        }
    }

    #[inline]
    fn has_attr(&self, namespace: &Namespace, attr: &Atom) -> bool {
        unsafe {
            bindings::Gecko_HasAttr(self.0,
                                    namespace.0.as_ptr(),
                                    attr.as_ptr())
        }
    }

    #[inline]
    fn attr_equals(&self, namespace: &Namespace, attr: &Atom, val: &Atom) -> bool {
        unsafe {
            bindings::Gecko_AttrEquals(self.0,
                                       namespace.0.as_ptr(),
                                       attr.as_ptr(),
                                       val.as_ptr(),
                                       /* ignoreCase = */ false)
        }
    }

    fn existing_style_for_restyle_damage<'a>(&'a self,
                                             _existing_values: &'a Arc<ComputedValues>,
                                             pseudo: Option<&PseudoElement>)
                                             -> Option<&'a nsStyleContext> {
        let atom_ptr = PseudoElement::ns_atom_or_null_from_opt(pseudo);
        unsafe {
            let context_ptr = Gecko_GetStyleContext(self.as_node().0, atom_ptr);
            context_ptr.as_ref()
        }
    }

    fn has_dirty_descendants(&self) -> bool {
        self.flags() & (NODE_HAS_DIRTY_DESCENDANTS_FOR_SERVO as u32) != 0
    }

    unsafe fn set_dirty_descendants(&self) {
        debug_assert!(self.get_data().is_some());
        debug!("Setting dirty descendants: {:?}", self);
        self.set_flags(NODE_HAS_DIRTY_DESCENDANTS_FOR_SERVO as u32)
    }

    unsafe fn note_descendants<B: DescendantsBit<Self>>(&self) {
        // FIXME(emilio): We seem to reach this in Gecko's
        // layout/style/test/test_pseudoelement_state.html, while changing the
        // state of an anonymous content element which is styled, but whose
        // parent isn't, presumably because we've cleared the data and haven't
        // reached it yet.
        //
        // Otherwise we should be able to assert this.
        if self.get_data().is_none() {
            return;
        }

        if dom::raw_note_descendants::<Self, B>(*self) {
            bindings::Gecko_SetOwnerDocumentNeedsStyleFlush(self.0);
        }
    }

    unsafe fn unset_dirty_descendants(&self) {
        self.unset_flags(NODE_HAS_DIRTY_DESCENDANTS_FOR_SERVO as u32)
    }

    fn has_animation_only_dirty_descendants(&self) -> bool {
        self.flags() & (NODE_HAS_ANIMATION_ONLY_DIRTY_DESCENDANTS_FOR_SERVO as u32) != 0
    }

    unsafe fn set_animation_only_dirty_descendants(&self) {
        self.set_flags(NODE_HAS_ANIMATION_ONLY_DIRTY_DESCENDANTS_FOR_SERVO as u32)
    }

    unsafe fn unset_animation_only_dirty_descendants(&self) {
        self.unset_flags(NODE_HAS_ANIMATION_ONLY_DIRTY_DESCENDANTS_FOR_SERVO as u32)
    }

    fn is_native_anonymous(&self) -> bool {
        self.flags() & (NODE_IS_NATIVE_ANONYMOUS as u32) != 0
    }

    fn store_children_to_process(&self, _: isize) {
        // This is only used for bottom-up traversal, and is thus a no-op for Gecko.
    }

    fn did_process_child(&self) -> isize {
        panic!("Atomic child count not implemented in Gecko");
    }

    fn get_data(&self) -> Option<&AtomicRefCell<ElementData>> {
        unsafe { self.0.mServoData.get().as_ref() }
    }

    fn skip_root_and_item_based_display_fixup(&self) -> bool {
        // We don't want to fix up display values of native anonymous content.
        // Additionally, we want to skip root-based display fixup for document
        // level native anonymous content subtree roots, since they're not
        // really roots from the style fixup perspective.  Checking that we
        // are NAC handles both cases.
        self.flags() & (NODE_IS_IN_NATIVE_ANONYMOUS_SUBTREE as u32) != 0
    }

    unsafe fn set_selector_flags(&self, flags: ElementSelectorFlags) {
        debug_assert!(!flags.is_empty());
        self.set_flags(selector_flags_to_node_flags(flags));
    }

    fn has_selector_flags(&self, flags: ElementSelectorFlags) -> bool {
        let node_flags = selector_flags_to_node_flags(flags);
        (self.flags() & node_flags) == node_flags
    }

    fn update_animations(&self,
                         pseudo: Option<&PseudoElement>,
                         before_change_style: Option<Arc<ComputedValues>>,
                         tasks: UpdateAnimationsTasks) {
        // We have to update animations even if the element has no computed style
        // since it means the element is in a display:none subtree, we should destroy
        // all CSS animations in display:none subtree.
        let computed_data = self.borrow_data();
        let computed_values =
            computed_data.as_ref().map(|d|
                pseudo.map_or_else(|| d.styles().primary.values(),
                                   |p| d.styles().pseudos.get(p).unwrap().values())
            );
        let computed_values_opt = computed_values.map(|v|
            *HasArcFFI::arc_as_borrowed(v)
        );

        let parent_element = if pseudo.is_some() {
            self.parent_element()
        } else {
            Some(*self)
        };
        let parent_data = parent_element.as_ref().and_then(|e| e.borrow_data());
        let parent_values = parent_data.as_ref().map(|d| d.styles().primary.values());
        let parent_values_opt = parent_values.map(|v|
            *HasArcFFI::arc_as_borrowed(v)
        );

        let atom_ptr = PseudoElement::ns_atom_or_null_from_opt(pseudo);
        let before_change_values = before_change_style.as_ref().map(|v| *HasArcFFI::arc_as_borrowed(v));
        unsafe {
            Gecko_UpdateAnimations(self.0, atom_ptr,
                                   before_change_values,
                                   computed_values_opt,
                                   parent_values_opt,
                                   tasks.bits());
        }
    }

    fn has_animations(&self, pseudo: Option<&PseudoElement>) -> bool {
        let atom_ptr = PseudoElement::ns_atom_or_null_from_opt(pseudo);
        unsafe { Gecko_ElementHasAnimations(self.0, atom_ptr) }
    }

    fn has_css_animations(&self, pseudo: Option<&PseudoElement>) -> bool {
        let atom_ptr = PseudoElement::ns_atom_or_null_from_opt(pseudo);
        unsafe { Gecko_ElementHasCSSAnimations(self.0, atom_ptr) }
    }

    fn has_css_transitions(&self, pseudo: Option<&PseudoElement>) -> bool {
        let atom_ptr = PseudoElement::ns_atom_or_null_from_opt(pseudo);
        unsafe { Gecko_ElementHasCSSTransitions(self.0, atom_ptr) }
    }

    fn get_css_transitions_info(&self,
                                pseudo: Option<&PseudoElement>)
                                -> HashMap<TransitionProperty, Arc<AnimationValue>> {
        use gecko_bindings::bindings::Gecko_ElementTransitions_EndValueAt;
        use gecko_bindings::bindings::Gecko_ElementTransitions_Length;
        use gecko_bindings::bindings::Gecko_ElementTransitions_PropertyAt;

        let atom_ptr = PseudoElement::ns_atom_or_null_from_opt(pseudo);
        let collection_length = unsafe { Gecko_ElementTransitions_Length(self.0, atom_ptr) };
        let mut map = HashMap::with_capacity(collection_length);
        for i in 0..collection_length {
            let (property, raw_end_value) = unsafe {
                (Gecko_ElementTransitions_PropertyAt(self.0, atom_ptr, i as usize).into(),
                 Gecko_ElementTransitions_EndValueAt(self.0, atom_ptr, i as usize))
            };
            let end_value = AnimationValue::arc_from_borrowed(&raw_end_value);
            debug_assert!(end_value.is_some());
            map.insert(property, end_value.unwrap().clone());
        }
        map
    }

    fn might_need_transitions_update(&self,
                                     old_values: &Option<&Arc<ComputedValues>>,
                                     new_values: &Arc<ComputedValues>,
                                     pseudo: Option<&PseudoElement>) -> bool {
        use properties::longhands::display::computed_value as display;

        if old_values.is_none() {
            return false;
        }

        let ref new_box_style = new_values.get_box();
        let transition_not_running = !self.has_css_transitions(pseudo) &&
                                     new_box_style.transition_property_count() == 1 &&
                                     new_box_style.transition_combined_duration_at(0) <= 0.0f32;
        let new_display_style = new_box_style.clone_display();
        let old_display_style = old_values.map(|ref old| old.get_box().clone_display()).unwrap();

        new_box_style.transition_property_count() > 0 &&
        !transition_not_running &&
        (new_display_style != display::T::none &&
         old_display_style != display::T::none)
    }

    // Detect if there are any changes that require us to update transitions. This is used as a
    // more thoroughgoing check than the, cheaper might_need_transitions_update check.
    // The following logic shadows the logic used on the Gecko side
    // (nsTransitionManager::DoUpdateTransitions) where we actually perform the update.
    // https://drafts.csswg.org/css-transitions/#starting
    fn needs_transitions_update(&self,
                                before_change_style: &Arc<ComputedValues>,
                                after_change_style: &Arc<ComputedValues>,
                                pseudo: Option<&PseudoElement>) -> bool {
        use gecko_bindings::structs::nsCSSPropertyID;
        use properties::{PropertyId, animated_properties};
        use std::collections::HashSet;

        debug_assert!(self.might_need_transitions_update(&Some(before_change_style),
                                                         after_change_style,
                                                         pseudo),
                      "We should only call needs_transitions_update if \
                       might_need_transitions_update returns true");

        let ref after_change_box_style = after_change_style.get_box();
        let transitions_count = after_change_box_style.transition_property_count();
        let existing_transitions = self.get_css_transitions_info(pseudo);
        let mut transitions_to_keep = if !existing_transitions.is_empty() &&
                                         (after_change_box_style.transition_nscsspropertyid_at(0) !=
                                              nsCSSPropertyID::eCSSPropertyExtra_all_properties) {
            Some(HashSet::<TransitionProperty>::with_capacity(transitions_count))
        } else {
            None
        };

        // Check if this property is none, custom or unknown.
        let is_none_or_custom_property = |property: nsCSSPropertyID| -> bool {
            return property == nsCSSPropertyID::eCSSPropertyExtra_no_properties ||
                   property == nsCSSPropertyID::eCSSPropertyExtra_variable ||
                   property == nsCSSPropertyID::eCSSProperty_UNKNOWN;
        };

        for i in 0..transitions_count {
            let property = after_change_box_style.transition_nscsspropertyid_at(i);
            let combined_duration = after_change_box_style.transition_combined_duration_at(i);

            // We don't need to update transition for none/custom properties.
            if is_none_or_custom_property(property) {
                continue;
            }

            let mut property_check_helper = |property: TransitionProperty| -> bool {
                if self.needs_transitions_update_per_property(property,
                                                              combined_duration,
                                                              before_change_style,
                                                              after_change_style,
                                                              &existing_transitions) {
                    return true;
                }

                if let Some(set) = transitions_to_keep.as_mut() {
                    set.insert(property);
                }
                false
            };
            if property == nsCSSPropertyID::eCSSPropertyExtra_all_properties {
                if TransitionProperty::any(property_check_helper) {
                    return true;
                }
            } else {
                let is_shorthand = PropertyId::from_nscsspropertyid(property).ok().map_or(false, |p| {
                        p.as_shorthand().is_ok()
                });
                if is_shorthand {
                    let shorthand: TransitionProperty = property.into();
                    if shorthand.longhands().iter().any(|&p| property_check_helper(p)) {
                        return true;
                    }
                } else {
                    if animated_properties::nscsspropertyid_is_animatable(property) &&
                       property_check_helper(property.into()) {
                        return true;
                    }
                }
            }
        }

        // Check if we have to cancel the running transition because this is not a matching
        // transition-property value.
        transitions_to_keep.map_or(false, |set| {
            existing_transitions.keys().any(|property| !set.contains(property))
        })
    }

    fn needs_transitions_update_per_property(&self,
                                             property: TransitionProperty,
                                             combined_duration: f32,
                                             before_change_style: &Arc<ComputedValues>,
                                             after_change_style: &Arc<ComputedValues>,
                                             existing_transitions: &HashMap<TransitionProperty,
                                                                            Arc<AnimationValue>>)
                                             -> bool {
        use properties::animated_properties::AnimatedProperty;

        // We don't allow transitions on properties that are not interpolable.
        if property.is_discrete() {
            return false;
        }

        if existing_transitions.contains_key(&property) {
            // If there is an existing transition, update only if the end value differs.
            // If the end value has not changed, we should leave the currently running
            // transition as-is since we don't want to interrupt its timing function.
            let after_value =
                Arc::new(AnimationValue::from_computed_values(&property, after_change_style));
            return existing_transitions.get(&property).unwrap() != &after_value;
        }

        combined_duration > 0.0f32 &&
        AnimatedProperty::from_transition_property(&property,
                                                   before_change_style,
                                                   after_change_style).does_animate()
    }
}

impl<'le> PartialEq for GeckoElement<'le> {
    fn eq(&self, other: &Self) -> bool {
        self.0 as *const _ == other.0 as *const _
    }
}

impl<'le> Eq for GeckoElement<'le> {}

impl<'le> Hash for GeckoElement<'le> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0 as *const _).hash(state);
    }
}

impl<'le> PresentationalHintsSynthetizer for GeckoElement<'le> {
    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, hints: &mut V)
        where V: Push<ApplicableDeclarationBlock>,
    {
        use properties::longhands::_x_lang::SpecifiedValue as SpecifiedLang;
        use properties::longhands::color::SpecifiedValue as SpecifiedColor;
        use properties::longhands::text_align::SpecifiedValue as SpecifiedTextAlign;
        use values::specified::color::Color;
        lazy_static! {
            static ref TH_RULE: ApplicableDeclarationBlock = {
                let global_style_data = &*GLOBAL_STYLE_DATA;
                let pdb = PropertyDeclarationBlock::with_one(
                    PropertyDeclaration::TextAlign(SpecifiedTextAlign::MozCenterOrInherit),
                    Importance::Normal
                );
                let arc = Arc::new(global_style_data.shared_lock.wrap(pdb));
                ApplicableDeclarationBlock::from_declarations(arc, ServoCascadeLevel::PresHints)
            };
            static ref TABLE_COLOR_RULE: ApplicableDeclarationBlock = {
                let global_style_data = &*GLOBAL_STYLE_DATA;
                let pdb = PropertyDeclarationBlock::with_one(
                    PropertyDeclaration::Color(SpecifiedColor(Color::InheritFromBodyQuirk.into())),
                    Importance::Normal
                );
                let arc = Arc::new(global_style_data.shared_lock.wrap(pdb));
                ApplicableDeclarationBlock::from_declarations(arc, ServoCascadeLevel::PresHints)
            };
            static ref MATHML_LANG_RULE: ApplicableDeclarationBlock = {
                let global_style_data = &*GLOBAL_STYLE_DATA;
                let pdb = PropertyDeclarationBlock::with_one(
                    PropertyDeclaration::XLang(SpecifiedLang(atom!("x-math"))),
                    Importance::Normal
                );
                let arc = Arc::new(global_style_data.shared_lock.wrap(pdb));
                ApplicableDeclarationBlock::from_declarations(arc, ServoCascadeLevel::PresHints)
            };
        };

        let ns = self.get_namespace();
        // <th> elements get a default MozCenterOrInherit which may get overridden
        if ns == &*Namespace(atom!("http://www.w3.org/1999/xhtml")) {
            if self.get_local_name().as_ptr() == atom!("th").as_ptr() {
                hints.push(TH_RULE.clone());
            } else if self.get_local_name().as_ptr() == atom!("table").as_ptr() &&
                      self.as_node().owner_doc().mCompatMode == structs::nsCompatibility::eCompatibility_NavQuirks {
                hints.push(TABLE_COLOR_RULE.clone());
            }
        }
        let declarations = unsafe { Gecko_GetHTMLPresentationAttrDeclarationBlock(self.0) };
        let declarations = declarations.and_then(|s| s.as_arc_opt());
        if let Some(decl) = declarations {
            hints.push(
                ApplicableDeclarationBlock::from_declarations(Clone::clone(decl), ServoCascadeLevel::PresHints)
            );
        }
        let declarations = unsafe { Gecko_GetExtraContentStyleDeclarations(self.0) };
        let declarations = declarations.and_then(|s| s.as_arc_opt());
        if let Some(decl) = declarations {
            hints.push(
                ApplicableDeclarationBlock::from_declarations(Clone::clone(decl), ServoCascadeLevel::PresHints)
            );
        }

        // xml:lang has precedence over lang, which can be
        // set by Gecko_GetHTMLPresentationAttrDeclarationBlock
        //
        // http://www.whatwg.org/specs/web-apps/current-work/multipage/elements.html#language
        let ptr = unsafe {
            bindings::Gecko_GetXMLLangValue(self.0)
        };
        if !ptr.is_null() {
            let global_style_data = &*GLOBAL_STYLE_DATA;

            let pdb = PropertyDeclarationBlock::with_one(
                PropertyDeclaration::XLang(SpecifiedLang(unsafe { Atom::from_addrefed(ptr) })),
                Importance::Normal
            );
            let arc = Arc::new(global_style_data.shared_lock.wrap(pdb));
            hints.push(ApplicableDeclarationBlock::from_declarations(arc, ServoCascadeLevel::PresHints))
        }
        // MathML's default lang has precedence over both `lang` and `xml:lang`
        if ns == &*Namespace(atom!("http://www.w3.org/1998/Math/MathML")) {
            if self.get_local_name().as_ptr() == atom!("math").as_ptr() {
                hints.push(MATHML_LANG_RULE.clone());
            }
        }
    }
}

impl<'le> ::selectors::Element for GeckoElement<'le> {
    fn parent_element(&self) -> Option<Self> {
        let parent_node = self.as_node().parent_node();
        parent_node.and_then(|n| n.as_element())
    }

    fn first_child_element(&self) -> Option<Self> {
        let mut child = self.as_node().first_child();
        while let Some(child_node) = child {
            if let Some(el) = child_node.as_element() {
                return Some(el)
            }
            child = child_node.next_sibling();
        }
        None
    }

    fn last_child_element(&self) -> Option<Self> {
        let mut child = self.as_node().last_child();
        while let Some(child_node) = child {
            if let Some(el) = child_node.as_element() {
                return Some(el)
            }
            child = child_node.prev_sibling();
        }
        None
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        let mut sibling = self.as_node().prev_sibling();
        while let Some(sibling_node) = sibling {
            if let Some(el) = sibling_node.as_element() {
                return Some(el)
            }
            sibling = sibling_node.prev_sibling();
        }
        None
    }

    fn next_sibling_element(&self) -> Option<Self> {
        let mut sibling = self.as_node().next_sibling();
        while let Some(sibling_node) = sibling {
            if let Some(el) = sibling_node.as_element() {
                return Some(el)
            }
            sibling = sibling_node.next_sibling();
        }
        None
    }

    fn is_root(&self) -> bool {
        unsafe {
            Gecko_IsRootElement(self.0)
        }
    }

    fn is_empty(&self) -> bool {
        !self.as_node().dom_children().any(|child| unsafe {
            Gecko_IsSignificantChild(child.0, true, true)
        })
    }

    fn get_local_name(&self) -> &WeakAtom {
        unsafe {
            WeakAtom::new(self.as_node().node_info().mInner.mName.raw())
        }
    }

    fn get_namespace(&self) -> &WeakNamespace {
        unsafe {
            WeakNamespace::new(Gecko_Namespace(self.0))
        }
    }

    fn match_non_ts_pseudo_class<F>(&self,
                                    pseudo_class: &NonTSPseudoClass,
                                    relations: &mut StyleRelations,
                                    flags_setter: &mut F)
                                    -> bool
        where F: FnMut(&Self, ElementSelectorFlags),
    {
        use selectors::matching::*;
        match *pseudo_class {
            NonTSPseudoClass::AnyLink |
            NonTSPseudoClass::Link |
            NonTSPseudoClass::Visited |
            NonTSPseudoClass::Active |
            NonTSPseudoClass::Focus |
            NonTSPseudoClass::Hover |
            NonTSPseudoClass::Enabled |
            NonTSPseudoClass::Disabled |
            NonTSPseudoClass::Checked |
            NonTSPseudoClass::Fullscreen |
            NonTSPseudoClass::Indeterminate |
            NonTSPseudoClass::PlaceholderShown |
            NonTSPseudoClass::Target |
            NonTSPseudoClass::Valid |
            NonTSPseudoClass::Invalid |
            NonTSPseudoClass::MozUIValid |
            NonTSPseudoClass::MozBroken |
            NonTSPseudoClass::MozUserDisabled |
            NonTSPseudoClass::MozSuppressed |
            NonTSPseudoClass::MozLoading |
            NonTSPseudoClass::MozHandlerBlocked |
            NonTSPseudoClass::MozHandlerDisabled |
            NonTSPseudoClass::MozHandlerCrashed |
            NonTSPseudoClass::Required |
            NonTSPseudoClass::Optional |
            NonTSPseudoClass::MozReadOnly |
            NonTSPseudoClass::MozReadWrite |
            NonTSPseudoClass::Unresolved |
            NonTSPseudoClass::FocusWithin |
            NonTSPseudoClass::MozDragOver |
            NonTSPseudoClass::MozDevtoolsHighlighted |
            NonTSPseudoClass::MozStyleeditorTransitioning |
            NonTSPseudoClass::MozFocusRing |
            NonTSPseudoClass::MozHandlerClickToPlay |
            NonTSPseudoClass::MozHandlerVulnerableUpdatable |
            NonTSPseudoClass::MozHandlerVulnerableNoUpdate |
            NonTSPseudoClass::MozMathIncrementScriptLevel |
            NonTSPseudoClass::InRange |
            NonTSPseudoClass::OutOfRange |
            NonTSPseudoClass::Default |
            NonTSPseudoClass::MozSubmitInvalid |
            NonTSPseudoClass::MozUIInvalid |
            NonTSPseudoClass::MozMeterOptimum |
            NonTSPseudoClass::MozMeterSubOptimum |
            NonTSPseudoClass::MozMeterSubSubOptimum => {
                // NB: It's important to use `intersect` instead of `contains`
                // here, to handle `:any-link` correctly.
                self.get_state().intersects(pseudo_class.state_flag())
            },
            NonTSPseudoClass::MozFirstNode => {
                flags_setter(self, HAS_EDGE_CHILD_SELECTOR);
                let mut elem = self.as_node();
                while let Some(prev) = elem.prev_sibling() {
                    if prev.contains_non_whitespace_content() {
                        return false
                    }
                    elem = prev;
                }
                true
            }
            NonTSPseudoClass::MozLastNode => {
                flags_setter(self, HAS_EDGE_CHILD_SELECTOR);
                let mut elem = self.as_node();
                while let Some(next) = elem.next_sibling() {
                    if next.contains_non_whitespace_content() {
                        return false
                    }
                    elem = next;
                }
                true
            }
            NonTSPseudoClass::MozOnlyWhitespace => {
                flags_setter(self, HAS_EMPTY_SELECTOR);
                if self.as_node().dom_children().any(|c| c.contains_non_whitespace_content()) {
                    return false
                }
                true
            }
            NonTSPseudoClass::MozTableBorderNonzero |
            NonTSPseudoClass::MozBrowserFrame => unsafe {
                Gecko_MatchesElement(pseudo_class.to_gecko_pseudoclasstype().unwrap(), self.0)
            },
            NonTSPseudoClass::MozAny(ref sels) => {
                sels.iter().any(|s| {
                    matches_complex_selector(s, self, relations, flags_setter)
                })
            }
            NonTSPseudoClass::MozSystemMetric(ref s) |
            NonTSPseudoClass::MozLocaleDir(ref s) |
            NonTSPseudoClass::MozEmptyExceptChildrenWithLocalname(ref s) |
            NonTSPseudoClass::Dir(ref s) |
            NonTSPseudoClass::Lang(ref s) => {
                unsafe {
                    let mut set_slow_selector = false;
                    let matches = Gecko_MatchStringArgPseudo(self.0,
                                       pseudo_class.to_gecko_pseudoclasstype().unwrap(),
                                       s.as_ptr(), &mut set_slow_selector);
                    if set_slow_selector {
                        flags_setter(self, HAS_SLOW_SELECTOR);
                    }
                    matches
                }
            }
        }
    }

    fn get_id(&self) -> Option<Atom> {
        let ptr = unsafe {
            bindings::Gecko_AtomAttrValue(self.0,
                                          atom!("id").as_ptr())
        };

        if ptr.is_null() {
            None
        } else {
            Some(Atom::from(ptr))
        }
    }

    fn has_class(&self, name: &Atom) -> bool {
        snapshot_helpers::has_class(self.0,
                                    name,
                                    Gecko_ClassOrClassList)
    }

    fn each_class<F>(&self, callback: F)
        where F: FnMut(&Atom)
    {
        snapshot_helpers::each_class(self.0,
                                     callback,
                                     Gecko_ClassOrClassList)
    }

    fn is_html_element_in_html_document(&self) -> bool {
        let node = self.as_node();
        let node_info = node.node_info();
        node_info.mInner.mNamespaceID == (structs::root::kNameSpaceID_XHTML as i32) &&
        node.owner_doc().mType == structs::root::nsIDocument_Type::eHTML
    }
}

/// A few helpers to help with attribute selectors and snapshotting.
pub trait AttrSelectorHelpers {
    /// Returns the namespace of the selector, or null otherwise.
    fn ns_or_null(&self) -> *mut nsIAtom;
    /// Returns the proper selector name depending on whether the requesting
    /// element is an HTML element in an HTML document or not.
    fn select_name(&self, is_html_element_in_html_document: bool) -> *mut nsIAtom;
}

impl AttrSelectorHelpers for AttrSelector<SelectorImpl> {
    fn ns_or_null(&self) -> *mut nsIAtom {
        match self.namespace {
            NamespaceConstraint::Any => ptr::null_mut(),
            NamespaceConstraint::Specific(ref ns) => ns.url.0.as_ptr(),
        }
    }

    fn select_name(&self, is_html_element_in_html_document: bool) -> *mut nsIAtom {
        if is_html_element_in_html_document {
            self.lower_name.as_ptr()
        } else {
            self.name.as_ptr()
        }
    }
}

impl<'le> ::selectors::MatchAttr for GeckoElement<'le> {
    type Impl = SelectorImpl;

    fn match_attr_has(&self, attr: &AttrSelector<Self::Impl>) -> bool {
        unsafe {
            bindings::Gecko_HasAttr(self.0,
                                    attr.ns_or_null(),
                                    attr.select_name(self.is_html_element_in_html_document()))
        }
    }
    fn match_attr_equals(&self, attr: &AttrSelector<Self::Impl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_AttrEquals(self.0,
                                       attr.ns_or_null(),
                                       attr.select_name(self.is_html_element_in_html_document()),
                                       value.as_ptr(),
                                       /* ignoreCase = */ false)
        }
    }
    fn match_attr_equals_ignore_ascii_case(&self, attr: &AttrSelector<Self::Impl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_AttrEquals(self.0,
                                       attr.ns_or_null(),
                                       attr.select_name(self.is_html_element_in_html_document()),
                                       value.as_ptr(),
                                       /* ignoreCase = */ false)
        }
    }
    fn match_attr_includes(&self, attr: &AttrSelector<Self::Impl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_AttrIncludes(self.0,
                                         attr.ns_or_null(),
                                         attr.select_name(self.is_html_element_in_html_document()),
                                         value.as_ptr())
        }
    }
    fn match_attr_dash(&self, attr: &AttrSelector<Self::Impl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_AttrDashEquals(self.0,
                                           attr.ns_or_null(),
                                           attr.select_name(self.is_html_element_in_html_document()),
                                           value.as_ptr())
        }
    }
    fn match_attr_prefix(&self, attr: &AttrSelector<Self::Impl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_AttrHasPrefix(self.0,
                                          attr.ns_or_null(),
                                          attr.select_name(self.is_html_element_in_html_document()),
                                          value.as_ptr())
        }
    }
    fn match_attr_substring(&self, attr: &AttrSelector<Self::Impl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_AttrHasSubstring(self.0,
                                             attr.ns_or_null(),
                                             attr.select_name(self.is_html_element_in_html_document()),
                                             value.as_ptr())
        }
    }
    fn match_attr_suffix(&self, attr: &AttrSelector<Self::Impl>, value: &Atom) -> bool {
        unsafe {
            bindings::Gecko_AttrHasSuffix(self.0,
                                          attr.ns_or_null(),
                                          attr.select_name(self.is_html_element_in_html_document()),
                                          value.as_ptr())
        }
    }
}

impl<'le> ElementExt for GeckoElement<'le> {
    #[inline]
    fn is_link(&self) -> bool {
        self.match_non_ts_pseudo_class(&NonTSPseudoClass::AnyLink,
                                       &mut StyleRelations::empty(),
                                       &mut |_, _| {})
    }

    #[inline]
    fn matches_user_and_author_rules(&self) -> bool {
        self.flags() & (NODE_IS_IN_NATIVE_ANONYMOUS_SUBTREE as u32) == 0
    }
}
