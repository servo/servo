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

use CaseSensitivityExt;
use app_units::Au;
use applicable_declarations::ApplicableDeclarationBlock;
use atomic_refcell::{AtomicRefCell, AtomicRef, AtomicRefMut};
use context::{QuirksMode, SharedStyleContext, PostAnimationTasks, UpdateAnimationsTasks};
use data::ElementData;
use dom::{LayoutIterator, NodeInfo, OpaqueNode, TElement, TDocument, TNode};
use element_state::{ElementState, DocumentState};
use font_metrics::{FontMetrics, FontMetricsProvider, FontMetricsQueryResult};
use gecko::data::PerDocumentStyleData;
use gecko::global_style_data::GLOBAL_STYLE_DATA;
use gecko::selector_parser::{SelectorImpl, NonTSPseudoClass, PseudoElement};
use gecko::snapshot_helpers;
use gecko_bindings::bindings;
use gecko_bindings::bindings::{Gecko_ConstructStyleChildrenIterator, Gecko_DestroyStyleChildrenIterator};
use gecko_bindings::bindings::{Gecko_ElementState, Gecko_GetDocumentLWTheme};
use gecko_bindings::bindings::{Gecko_GetLastChild, Gecko_GetNextStyleChild};
use gecko_bindings::bindings::{Gecko_IsRootElement, Gecko_MatchesElement};
use gecko_bindings::bindings::{Gecko_SetNodeFlags, Gecko_UnsetNodeFlags};
use gecko_bindings::bindings::Gecko_ClassOrClassList;
use gecko_bindings::bindings::Gecko_ElementHasAnimations;
use gecko_bindings::bindings::Gecko_ElementHasCSSAnimations;
use gecko_bindings::bindings::Gecko_ElementHasCSSTransitions;
use gecko_bindings::bindings::Gecko_GetActiveLinkAttrDeclarationBlock;
use gecko_bindings::bindings::Gecko_GetAnimationRule;
use gecko_bindings::bindings::Gecko_GetExtraContentStyleDeclarations;
use gecko_bindings::bindings::Gecko_GetHTMLPresentationAttrDeclarationBlock;
use gecko_bindings::bindings::Gecko_GetStyleAttrDeclarationBlock;
use gecko_bindings::bindings::Gecko_GetUnvisitedLinkAttrDeclarationBlock;
use gecko_bindings::bindings::Gecko_GetVisitedLinkAttrDeclarationBlock;
use gecko_bindings::bindings::Gecko_IsSignificantChild;
use gecko_bindings::bindings::Gecko_MatchLang;
use gecko_bindings::bindings::Gecko_UnsetDirtyStyleAttr;
use gecko_bindings::bindings::Gecko_UpdateAnimations;
use gecko_bindings::structs;
use gecko_bindings::structs::{RawGeckoElement, RawGeckoNode, RawGeckoXBLBinding};
use gecko_bindings::structs::{nsAtom, nsIContent, nsINode_BooleanFlag};
use gecko_bindings::structs::ELEMENT_HANDLED_SNAPSHOT;
use gecko_bindings::structs::ELEMENT_HAS_ANIMATION_ONLY_DIRTY_DESCENDANTS_FOR_SERVO;
use gecko_bindings::structs::ELEMENT_HAS_DIRTY_DESCENDANTS_FOR_SERVO;
use gecko_bindings::structs::ELEMENT_HAS_SNAPSHOT;
use gecko_bindings::structs::EffectCompositor_CascadeLevel as CascadeLevel;
use gecko_bindings::structs::NODE_DESCENDANTS_NEED_FRAMES;
use gecko_bindings::structs::NODE_NEEDS_FRAME;
use gecko_bindings::structs::nsChangeHint;
use gecko_bindings::structs::nsIDocument_DocumentTheme as DocumentTheme;
use gecko_bindings::structs::nsRestyleHint;
use gecko_bindings::sugar::ownership::{HasArcFFI, HasSimpleFFI};
use hash::FnvHashMap;
use logical_geometry::WritingMode;
use media_queries::Device;
use properties::{ComputedValues, LonghandId};
use properties::{Importance, PropertyDeclaration, PropertyDeclarationBlock};
use properties::animated_properties::{AnimationValue, AnimationValueMap};
use properties::animated_properties::TransitionProperty;
use properties::style_structs::Font;
use rule_tree::CascadeLevel as ServoCascadeLevel;
use selector_parser::{AttrValue, Direction, PseudoClassStringArg};
use selectors::{Element, OpaqueElement};
use selectors::attr::{AttrSelectorOperation, AttrSelectorOperator, CaseSensitivity, NamespaceConstraint};
use selectors::matching::{ElementSelectorFlags, MatchingContext};
use selectors::matching::VisitedHandlingMode;
use selectors::sink::Push;
use servo_arc::{Arc, ArcBorrow, RawOffsetArc};
use shared_lock::Locked;
use std::cell::RefCell;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::DerefMut;
use std::ptr;
use string_cache::{Atom, Namespace, WeakAtom, WeakNamespace};
use stylist::Stylist;

/// A simple wrapper over `nsIDocument`.
#[derive(Clone, Copy)]
pub struct GeckoDocument<'ld>(pub &'ld structs::nsIDocument);

impl<'ld> TDocument for GeckoDocument<'ld> {
    type ConcreteNode = GeckoNode<'ld>;

    #[inline]
    fn as_node(&self) -> Self::ConcreteNode {
        GeckoNode(&self.0._base)
    }

    #[inline]
    fn is_html_document(&self) -> bool {
        self.0.mType == structs::root::nsIDocument_Type::eHTML
    }

    #[inline]
    fn quirks_mode(&self) -> QuirksMode {
        self.0.mCompatMode.into()
    }

    fn elements_with_id(&self, id: &Atom) -> Result<&[GeckoElement<'ld>], ()> {
        unsafe {
            let array = bindings::Gecko_GetElementsWithId(self.0, id.as_ptr());
            if array.is_null() {
                return Ok(&[]);
            }

            let elements: &[*mut RawGeckoElement] = &**array;

            // NOTE(emilio): We rely on the in-memory representation of
            // GeckoElement<'ld> and *mut RawGeckoElement being the same.
            #[allow(dead_code)]
            unsafe fn static_assert() {
                mem::transmute::<*mut RawGeckoElement, GeckoElement<'static>>(0xbadc0de as *mut _);
            }

            Ok(mem::transmute(elements))
        }
    }
}

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

impl<'ln> PartialEq for GeckoNode<'ln> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 as *const _ == other.0 as *const _
    }
}

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
    #[inline]
    fn is_document(&self) -> bool {
        // This is a DOM constant that isn't going to change.
        const DOCUMENT_NODE: u16 = 9;
        self.node_info().mInner.mNodeType == DOCUMENT_NODE
    }

    #[inline]
    fn from_content(content: &'ln nsIContent) -> Self {
        GeckoNode(&content._base)
    }

    #[inline]
    fn flags(&self) -> u32 {
        (self.0)._base._base_1.mFlags
    }

    #[inline]
    fn node_info(&self) -> &structs::NodeInfo {
        debug_assert!(!self.0.mNodeInfo.mRawPtr.is_null());
        unsafe { &*self.0.mNodeInfo.mRawPtr }
    }

    // These live in different locations depending on processor architecture.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn bool_flags(&self) -> u32 {
        (self.0)._base._base_1.mBoolFlags
    }

    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn bool_flags(&self) -> u32 {
        (self.0).mBoolFlags
    }

    #[inline]
    fn get_bool_flag(&self, flag: nsINode_BooleanFlag) -> bool {
        self.bool_flags() & (1u32 << flag as u32) != 0
    }

    /// WARNING: This logic is duplicated in Gecko's FlattenedTreeParentIsParent.
    /// Make sure to mirror any modifications in both places.
    #[inline]
    fn flattened_tree_parent_is_parent(&self) -> bool {
        use gecko_bindings::structs::*;
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

        if let Some(parent) = parent_el {
            if parent.has_shadow_root() || parent.get_xbl_binding().is_some() {
                return false;
            }
        }

        true
    }

    #[inline]
    fn flattened_tree_parent(&self) -> Option<Self> {
        // TODO(emilio): Measure and consider not doing this fast-path and take
        // always the common path, it's only a function call and from profiles
        // it seems that keeping this fast path makes the compiler not inline
        // `flattened_tree_parent`.
        if self.flattened_tree_parent_is_parent() {
            debug_assert_eq!(
                unsafe { bindings::Gecko_GetFlattenedTreeParentNode(self.0).map(GeckoNode) },
                self.parent_node(),
                "Fast path stopped holding!"
            );

            return self.parent_node();
        }

        // NOTE(emilio): If this call is too expensive, we could manually
        // inline more aggressively.
        unsafe { bindings::Gecko_GetFlattenedTreeParentNode(self.0).map(GeckoNode) }
    }

    #[inline]
    fn contains_non_whitespace_content(&self) -> bool {
        unsafe { Gecko_IsSignificantChild(self.0, true, false) }
    }
}

impl<'ln> NodeInfo for GeckoNode<'ln> {
    #[inline]
    fn is_element(&self) -> bool {
        self.get_bool_flag(nsINode_BooleanFlag::NodeIsElement)
    }

    fn is_text_node(&self) -> bool {
        // This is a DOM constant that isn't going to change.
        const TEXT_NODE: u16 = 3;
        self.node_info().mInner.mNodeType == TEXT_NODE
    }
}

impl<'ln> TNode for GeckoNode<'ln> {
    type ConcreteDocument = GeckoDocument<'ln>;
    type ConcreteElement = GeckoElement<'ln>;

    #[inline]
    fn parent_node(&self) -> Option<Self> {
        unsafe { self.0.mParent.as_ref().map(GeckoNode) }
    }

    #[inline]
    fn first_child(&self) -> Option<Self> {
        unsafe { self.0.mFirstChild.as_ref().map(GeckoNode::from_content) }
    }

    #[inline]
    fn last_child(&self) -> Option<Self> {
        unsafe { Gecko_GetLastChild(self.0).map(GeckoNode) }
    }

    #[inline]
    fn prev_sibling(&self) -> Option<Self> {
        unsafe { self.0.mPreviousSibling.as_ref().map(GeckoNode::from_content) }
    }

    #[inline]
    fn next_sibling(&self) -> Option<Self> {
        unsafe { self.0.mNextSibling.as_ref().map(GeckoNode::from_content) }
    }

    #[inline]
    fn owner_doc(&self) -> Self::ConcreteDocument {
        debug_assert!(!self.node_info().mDocument.is_null());
        GeckoDocument(unsafe { &*self.node_info().mDocument })
    }

    #[inline]
    fn is_in_document(&self) -> bool {
        self.get_bool_flag(nsINode_BooleanFlag::IsInDocument)
    }

    fn traversal_parent(&self) -> Option<GeckoElement<'ln>> {
        self.flattened_tree_parent().and_then(|n| n.as_element())
    }

    #[inline]
    fn opaque(&self) -> OpaqueNode {
        let ptr: usize = self.0 as *const _ as usize;
        OpaqueNode(ptr)
    }

    fn debug_id(self) -> usize {
        unimplemented!()
    }

    #[inline]
    fn as_element(&self) -> Option<GeckoElement<'ln>> {
        if self.is_element() {
            unsafe { Some(GeckoElement(&*(self.0 as *const _ as *const RawGeckoElement))) }
        } else {
            None
        }
    }

    #[inline]
    fn as_document(&self) -> Option<Self::ConcreteDocument> {
        if self.is_document() {
            Some(self.owner_doc())
        } else {
            None
        }
    }
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
    GeckoIterator(structs::StyleChildrenIterator),
}

impl<'a> Drop for GeckoChildrenIterator<'a> {
    fn drop(&mut self) {
        if let GeckoChildrenIterator::GeckoIterator(ref mut it) = *self {
            unsafe {
                Gecko_DestroyStyleChildrenIterator(it);
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
                // We do this unsafe lengthening of the lifetime here because
                // structs::StyleChildrenIterator is actually StyleChildrenIterator<'a>,
                // however we can't express this easily with bindgen, and it would
                // introduce functions with two input lifetimes into bindgen,
                // which would be out of scope for elision.
                Gecko_GetNextStyleChild(&mut * (it as *mut _)).map(GeckoNode)
            }
        }
    }
}

/// A Simple wrapper over a non-null Gecko `nsXBLBinding` pointer.
#[derive(Clone, Copy)]
pub struct GeckoXBLBinding<'lb>(pub &'lb RawGeckoXBLBinding);

impl<'lb> GeckoXBLBinding<'lb> {
    #[inline]
    fn base_binding(&self) -> Option<Self> {
        unsafe { self.0.mNextBinding.mRawPtr.as_ref().map(GeckoXBLBinding) }
    }

    #[inline]
    fn anon_content(&self) -> *const nsIContent {
        self.0.mContent.raw::<nsIContent>()
    }

    #[inline]
    fn inherits_style(&self) -> bool {
        unsafe { bindings::Gecko_XBLBinding_InheritsStyle(self.0) }
    }

    // This duplicates the logic in Gecko's
    // nsBindingManager::GetBindingWithContent.
    fn get_binding_with_content(&self) -> Option<Self> {
        let mut binding = *self;
        loop {
            if !binding.anon_content().is_null() {
                return Some(binding);
            }
            binding = binding.base_binding()?;
        }
    }

    fn each_xbl_stylist<F>(&self, f: &mut F)
    where
        F: FnMut(AtomicRef<'lb, Stylist>),
    {
        if let Some(base) = self.base_binding() {
            base.each_xbl_stylist(f);
        }

        let raw_data = unsafe {
            bindings::Gecko_XBLBinding_GetRawServoStyleSet(self.0)
        };

        if let Some(raw_data) = raw_data {
            let data = PerDocumentStyleData::from_ffi(&*raw_data).borrow();
            f(AtomicRef::map(data, |d| &d.stylist));
        }
    }
}

/// A simple wrapper over a non-null Gecko `Element` pointer.
#[derive(Clone, Copy)]
pub struct GeckoElement<'le>(pub &'le RawGeckoElement);

impl<'le> fmt::Debug for GeckoElement<'le> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}", self.get_local_name())?;
        if let Some(id) = self.get_id() {
            write!(f, " id={}", id)?;
        }

        let mut first = true;
        let mut any = false;
        self.each_class(|c| {
            if first {
                first = false;
                any = true;
                let _ = f.write_str(" class=\"");
            } else {
                let _ = f.write_str(" ");
            }
            let _ = write!(f, "{}", c);
        });

        if any {
            f.write_str("\"")?;
        }

        write!(f, "> ({:#x})", self.as_node().opaque().0)
    }
}

impl<'le> GeckoElement<'le> {
    #[inline]
    fn may_have_anonymous_children(&self) -> bool {
        self.as_node().get_bool_flag(nsINode_BooleanFlag::ElementMayHaveAnonymousChildren)
    }

    #[inline]
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

    unsafe fn unset_flags(&self, flags: u32) {
        Gecko_UnsetNodeFlags(self.as_node().0, flags)
    }

    /// Returns true if this element has descendants for lazy frame construction.
    pub fn descendants_need_frames(&self) -> bool {
        self.flags() & (NODE_DESCENDANTS_NEED_FRAMES as u32) != 0
    }

    /// Returns true if this element needs lazy frame construction.
    pub fn needs_frame(&self) -> bool {
        self.flags() & (NODE_NEEDS_FRAME as u32) != 0
    }

    /// Returns true if this element has a shadow root.
    fn has_shadow_root(&self) -> bool {
        self.get_extended_slots()
            .map_or(false, |slots| !slots.mShadowRoot.mRawPtr.is_null())
    }

    /// Returns a reference to the DOM slots for this Element, if they exist.
    fn get_dom_slots(&self) -> Option<&structs::FragmentOrElement_nsDOMSlots> {
        let slots = self.as_node().0.mSlots as *const structs::FragmentOrElement_nsDOMSlots;
        unsafe { slots.as_ref() }
    }

    /// Returns a reference to the extended DOM slots for this Element.
    fn get_extended_slots(
        &self,
    ) -> Option<&structs::FragmentOrElement_nsExtendedDOMSlots> {
        self.get_dom_slots().and_then(|s| unsafe {
            (s._base.mExtendedSlots.mPtr as *const structs::FragmentOrElement_nsExtendedDOMSlots).as_ref()
        })
    }

    #[inline]
    fn may_be_in_binding_manager(&self) -> bool {
        self.flags() & (structs::NODE_MAY_BE_IN_BINDING_MNGR as u32) != 0
    }

    #[inline]
    fn get_xbl_binding(&self) -> Option<GeckoXBLBinding<'le>> {
        if !self.may_be_in_binding_manager() {
            return None;
        }

        unsafe { bindings::Gecko_GetXBLBinding(self.0).map(GeckoXBLBinding) }
    }

    #[inline]
    fn get_xbl_binding_with_content(&self) -> Option<GeckoXBLBinding<'le>> {
        self.get_xbl_binding()
            .and_then(|b| b.get_binding_with_content())
    }

    #[inline]
    fn has_xbl_binding_with_content(&self) -> bool {
        !self.get_xbl_binding_with_content().is_none()
    }

    /// This and has_xbl_binding_parent duplicate the logic in Gecko's virtual
    /// nsINode::GetBindingParent function, which only has two implementations:
    /// one for XUL elements, and one for other elements.  We just hard code in
    /// our knowledge of those two implementations here.
    fn get_xbl_binding_parent(&self) -> Option<Self> {
        if self.is_xul_element() {
            // FIXME(heycam): Having trouble with bindgen on nsXULElement,
            // where the binding parent is stored in a member variable
            // rather than in slots.  So just get it through FFI for now.
            unsafe {
                bindings::Gecko_GetBindingParent(self.0).map(GeckoElement)
            }
        } else {
            let binding_parent = unsafe {
                self.get_non_xul_xbl_binding_parent_raw_content().as_ref()
            }.map(GeckoNode::from_content).and_then(|n| n.as_element());

            debug_assert!(binding_parent == unsafe {
                bindings::Gecko_GetBindingParent(self.0).map(GeckoElement)
            });
            binding_parent
        }
    }

    fn get_non_xul_xbl_binding_parent_raw_content(&self) -> *mut nsIContent {
        debug_assert!(!self.is_xul_element());
        self.get_extended_slots()
            .map_or(ptr::null_mut(), |slots| slots._base.mBindingParent)
    }

    fn has_xbl_binding_parent(&self) -> bool {
        if self.is_xul_element() {
            // FIXME(heycam): Having trouble with bindgen on nsXULElement,
            // where the binding parent is stored in a member variable
            // rather than in slots.  So just get it through FFI for now.
            unsafe { bindings::Gecko_GetBindingParent(self.0).is_some() }
        } else {
            !self.get_non_xul_xbl_binding_parent_raw_content().is_null()
        }
    }

    #[inline]
    fn namespace_id(&self) -> i32 {
        self.as_node().node_info().mInner.mNamespaceID
    }

    #[inline]
    fn is_xul_element(&self) -> bool {
        self.namespace_id() == (structs::root::kNameSpaceID_XUL as i32)
    }

    /// Sets the specified element data, return any existing data.
    ///
    /// Like `ensure_data`, only safe to call with exclusive access to the
    /// element.
    pub unsafe fn set_data(&self, replace_data: Option<ElementData>) -> Option<ElementData> {
        match (self.get_data(), replace_data) {
            (Some(old), Some(replace_data)) => {
                Some(mem::replace(old.borrow_mut().deref_mut(), replace_data))
            }
            (Some(old), None) => {
                let old_data = mem::replace(old.borrow_mut().deref_mut(), ElementData::default());
                self.0.mServoData.set(ptr::null_mut());
                Some(old_data)
            }
            (None, Some(replace_data)) => {
                let ptr = Box::into_raw(Box::new(AtomicRefCell::new(replace_data)));
                self.0.mServoData.set(ptr);
                None
            }
            (None, None) => None,
        }
    }

    #[inline]
    fn has_id(&self) -> bool {
        self.as_node().get_bool_flag(nsINode_BooleanFlag::ElementHasID)
    }

    #[inline]
    fn get_state_internal(&self) -> u64 {
        if !self.as_node().get_bool_flag(nsINode_BooleanFlag::ElementHasLockedStyleStates) {
            return self.0.mState.mStates;
        }
        unsafe { Gecko_ElementState(self.0) }
    }

    #[inline]
    fn document_state(&self) -> DocumentState {
        DocumentState::from_bits_truncate(
            self.as_node().owner_doc().0.mDocumentState.mStates
        )
    }

    #[inline]
    fn may_have_class(&self) -> bool {
        self.as_node()
            .get_bool_flag(nsINode_BooleanFlag::ElementMayHaveClass)
    }

    #[inline]
    fn has_properties(&self) -> bool {
        use gecko_bindings::structs::NODE_HAS_PROPERTIES;

        (self.flags() & NODE_HAS_PROPERTIES as u32) != 0
    }

    #[inline]
    fn get_before_or_after_pseudo(&self, is_before: bool) -> Option<Self> {
        if !self.has_properties() {
            return None;
        }

        unsafe { bindings::Gecko_GetBeforeOrAfterPseudo(self.0, is_before).map(GeckoElement) }
    }

    #[inline]
    fn may_have_style_attribute(&self) -> bool {
        self.as_node()
            .get_bool_flag(nsINode_BooleanFlag::ElementMayHaveStyle)
    }

    #[inline]
    fn get_document_theme(&self) -> DocumentTheme {
        let node = self.as_node();
        unsafe { Gecko_GetDocumentLWTheme(node.owner_doc().0) }
    }

    /// Only safe to call on the main thread, with exclusive access to the element and
    /// its ancestors.
    /// This function is also called after display property changed for SMIL animation.
    ///
    /// Also this function schedules style flush.
    unsafe fn maybe_restyle<'a>(
        &self,
        data: &'a mut ElementData,
        animation_only: bool,
    ) -> bool {
        if !data.has_styles() {
            return false;
        }

        // Propagate the bit up the chain.
        if animation_only {
            bindings::Gecko_NoteAnimationOnlyDirtyElement(self.0);
        } else {
            bindings::Gecko_NoteDirtyElement(self.0);
        }

        // Ensure and return the RestyleData.
        true
    }

    /// Set restyle and change hints to the element data.
    pub fn note_explicit_hints(
        &self,
        restyle_hint: nsRestyleHint,
        change_hint: nsChangeHint,
    ) {
        use gecko::restyle_damage::GeckoRestyleDamage;
        use invalidation::element::restyle_hints::RestyleHint;

        let damage = GeckoRestyleDamage::new(change_hint);
        debug!("note_explicit_hints: {:?}, restyle_hint={:?}, change_hint={:?}",
               self, restyle_hint, change_hint);

        let restyle_hint: RestyleHint = restyle_hint.into();
        debug_assert!(!(restyle_hint.has_animation_hint() &&
                        restyle_hint.has_non_animation_hint()),
                      "Animation restyle hints should not appear with non-animation restyle hints");

        let mut maybe_data = self.mutate_data();
        let should_restyle = maybe_data.as_mut().map_or(false, |d| unsafe {
            self.maybe_restyle(d, restyle_hint.has_animation_hint())
        });
        if should_restyle {
            maybe_data
                .as_mut()
                .unwrap()
                .hint
                .insert(restyle_hint.into());
            maybe_data.as_mut().unwrap().damage |= damage;
        } else {
            debug!("(Element not styled, discarding hints)");
        }
    }

    /// This logic is duplicated in Gecko's nsIContent::IsRootOfAnonymousSubtree.
    #[inline]
    fn is_root_of_anonymous_subtree(&self) -> bool {
        use gecko_bindings::structs::NODE_IS_ANONYMOUS_ROOT;
        self.flags() & (NODE_IS_ANONYMOUS_ROOT as u32) != 0
    }

    /// This logic is duplicated in Gecko's nsIContent::IsRootOfNativeAnonymousSubtree.
    #[inline]
    fn is_root_of_native_anonymous_subtree(&self) -> bool {
        use gecko_bindings::structs::NODE_IS_NATIVE_ANONYMOUS_ROOT;
        return self.flags() & (NODE_IS_NATIVE_ANONYMOUS_ROOT as u32) != 0
    }

    /// This logic is duplicated in Gecko's nsINode::IsInNativeAnonymousSubtree.
    #[inline]
    fn is_in_native_anonymous_subtree(&self) -> bool {
        use gecko_bindings::structs::NODE_IS_IN_NATIVE_ANONYMOUS_SUBTREE;
        self.flags() & (NODE_IS_IN_NATIVE_ANONYMOUS_SUBTREE as u32) != 0
    }

    /// This logic is duplicate in Gecko's nsIContent::IsInShadowTree().
    #[inline]
    fn is_in_shadow_tree(&self) -> bool {
        use gecko_bindings::structs::NODE_IS_IN_SHADOW_TREE;
        self.flags() & (NODE_IS_IN_SHADOW_TREE as u32) != 0
    }

    /// This logic is duplicated in Gecko's nsIContent::IsInAnonymousSubtree.
    #[inline]
    fn is_in_anonymous_subtree(&self) -> bool {
        self.is_in_native_anonymous_subtree() ||
        (!self.is_in_shadow_tree() && self.has_xbl_binding_parent())
    }
}

/// Converts flags from the layout used by rust-selectors to the layout used
/// by Gecko. We could align these and then do this without conditionals, but
/// it's probably not worth the trouble.
fn selector_flags_to_node_flags(flags: ElementSelectorFlags) -> u32 {
    use gecko_bindings::structs::*;
    let mut gecko_flags = 0u32;
    if flags.contains(ElementSelectorFlags::HAS_SLOW_SELECTOR) {
        gecko_flags |= NODE_HAS_SLOW_SELECTOR as u32;
    }
    if flags.contains(ElementSelectorFlags::HAS_SLOW_SELECTOR_LATER_SIBLINGS) {
        gecko_flags |= NODE_HAS_SLOW_SELECTOR_LATER_SIBLINGS as u32;
    }
    if flags.contains(ElementSelectorFlags::HAS_EDGE_CHILD_SELECTOR) {
        gecko_flags |= NODE_HAS_EDGE_CHILD_SELECTOR as u32;
    }
    if flags.contains(ElementSelectorFlags::HAS_EMPTY_SELECTOR) {
        gecko_flags |= NODE_HAS_EMPTY_SELECTOR as u32;
    }

    gecko_flags
}

fn get_animation_rule(
    element: &GeckoElement,
    cascade_level: CascadeLevel,
) -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
    use gecko_bindings::sugar::ownership::HasSimpleFFI;
    // Also, we should try to reuse the PDB, to avoid creating extra rule nodes.
    let mut animation_values = AnimationValueMap::default();
    if unsafe { Gecko_GetAnimationRule(element.0,
                                       cascade_level,
                                       AnimationValueMap::as_ffi_mut(&mut animation_values)) } {
        let shared_lock = &GLOBAL_STYLE_DATA.shared_lock;
        Some(Arc::new(shared_lock.wrap(
            PropertyDeclarationBlock::from_animation_value_map(&animation_values))))
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
        let sizes = unsafe { Gecko_GetBaseSize(font_name.as_ptr()) };
        cache.push((font_name.clone(), sizes));
        sizes.size_for_generic(font_family)
    }

    fn query(
        &self,
        font: &Font,
        font_size: Au,
        wm: WritingMode,
        in_media_query: bool,
        device: &Device,
    ) -> FontMetricsQueryResult {
        use gecko_bindings::bindings::Gecko_GetFontMetrics;
        let gecko_metrics = unsafe {
            Gecko_GetFontMetrics(
                device.pres_context(),
                wm.is_vertical() && !wm.is_sideways(),
                font.gecko(),
                font_size.0,
                // we don't use the user font set in a media query
                !in_media_query,
            )
        };
        let metrics = FontMetrics {
            x_height: Au(gecko_metrics.mXSize),
            zero_advance_measure: Au(gecko_metrics.mChSize),
        };
        FontMetricsQueryResult::Available(metrics)
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
            _ => unreachable!("Unknown generic ID"),
        })
    }
}

impl<'le> TElement for GeckoElement<'le> {
    type ConcreteNode = GeckoNode<'le>;
    type FontMetricsProvider = GeckoFontMetricsProvider;
    type TraversalChildrenIterator = GeckoChildrenIterator<'le>;

    fn inheritance_parent(&self) -> Option<Self> {
        if self.is_native_anonymous() {
            self.closest_non_native_anonymous_ancestor()
        } else {
            self.as_node()
                .flattened_tree_parent()
                .and_then(|n| n.as_element())
        }
    }

    fn traversal_children(&self) -> LayoutIterator<GeckoChildrenIterator<'le>> {
        // This condition is similar to the check that
        // StyleChildrenIterator::IsNeeded does, except that it might return
        // true if we used to (but no longer) have anonymous content from
        // ::before/::after, XBL bindings, or nsIAnonymousContentCreators.
        if self.is_in_anonymous_subtree() ||
           self.has_xbl_binding_with_content() ||
           self.is_in_shadow_tree() ||
           self.may_have_anonymous_children() {
            unsafe {
                let mut iter: structs::StyleChildrenIterator = ::std::mem::zeroed();
                Gecko_ConstructStyleChildrenIterator(self.0, &mut iter);
                return LayoutIterator(GeckoChildrenIterator::GeckoIterator(iter));
            }
        }

        LayoutIterator(GeckoChildrenIterator::Current(self.as_node().first_child()))
    }

    fn before_pseudo_element(&self) -> Option<Self> {
        self.get_before_or_after_pseudo(/* is_before = */ true)
    }

    fn after_pseudo_element(&self) -> Option<Self> {
        self.get_before_or_after_pseudo(/* is_before = */ false)
    }

    /// Ensure this accurately represents the rules that an element may ever
    /// match, even in the native anonymous content case.
    fn style_scope(&self) -> Self::ConcreteNode {
        if self.implemented_pseudo_element().is_some() {
            return self.closest_non_native_anonymous_ancestor().unwrap().style_scope();
        }

        if self.is_in_native_anonymous_subtree() {
            return self.as_node().owner_doc().as_node();
        }

        if self.get_xbl_binding().is_some() {
            return self.as_node();
        }

        if let Some(parent) = self.get_xbl_binding_parent() {
            return parent.as_node();
        }

        self.as_node().owner_doc().as_node()
    }


    #[inline]
    fn is_html_element(&self) -> bool {
        self.namespace_id() == (structs::root::kNameSpaceID_XHTML as i32)
    }

    /// Return the list of slotted nodes of this node.
    #[inline]
    fn slotted_nodes(&self) -> &[Self::ConcreteNode] {
        if !self.is_html_slot_element() || !self.is_in_shadow_tree() {
            return &[];
        }

        let slot: &structs::HTMLSlotElement = unsafe {
            mem::transmute(self.0)
        };

        if cfg!(debug_assertions) {
            let base: &RawGeckoElement = &slot._base._base._base._base;
            assert_eq!(base as *const _, self.0 as *const _, "Bad cast");
        }

        let assigned_nodes: &[structs::RefPtr<structs::nsINode>] =
            &*slot.mAssignedNodes;

        debug_assert_eq!(
            mem::size_of::<structs::RefPtr<structs::nsINode>>(),
            mem::size_of::<Self::ConcreteNode>(),
            "Bad cast!"
        );

        unsafe { mem::transmute(assigned_nodes) }
    }

    /// Execute `f` for each anonymous content child element (apart from
    /// ::before and ::after) whose originating element is `self`.
    fn each_anonymous_content_child<F>(&self, mut f: F)
    where
        F: FnMut(Self),
    {
        let array: *mut structs::nsTArray<*mut nsIContent> =
            unsafe { bindings::Gecko_GetAnonymousContentForElement(self.0) };

        if array.is_null() {
            return;
        }

        for content in unsafe { &**array } {
            let node = GeckoNode::from_content(unsafe { &**content });
            let element = match node.as_element() {
                Some(e) => e,
                None => continue,
            };

            f(element);
        }

        unsafe { bindings::Gecko_DestroyAnonymousContentList(array) };
    }

    fn closest_non_native_anonymous_ancestor(&self) -> Option<Self> {
        debug_assert!(self.is_native_anonymous());
        let mut parent = self.traversal_parent()?;

        loop {
            if !parent.is_native_anonymous() {
                return Some(parent);
            }

            parent = parent.traversal_parent()?;
        }
    }

    #[inline]
    fn as_node(&self) -> Self::ConcreteNode {
        unsafe { GeckoNode(&*(self.0 as *const _ as *const RawGeckoNode)) }
    }

    fn owner_doc_matches_for_testing(&self, device: &Device) -> bool {
        self.as_node().owner_doc().0 as *const structs::nsIDocument ==
            device.pres_context().mDocument.raw::<structs::nsIDocument>()
    }

    fn style_attribute(&self) -> Option<ArcBorrow<Locked<PropertyDeclarationBlock>>> {
        if !self.may_have_style_attribute() {
            return None;
        }

        let declarations = unsafe { Gecko_GetStyleAttrDeclarationBlock(self.0) };
        let declarations: Option<&RawOffsetArc<Locked<PropertyDeclarationBlock>>> =
            declarations.and_then(|s| s.as_arc_opt());
        declarations.map(|s| s.borrow_arc())
    }

    fn unset_dirty_style_attribute(&self) {
        if !self.may_have_style_attribute() {
            return;
        }

        unsafe { Gecko_UnsetDirtyStyleAttr(self.0) };
    }

    fn get_smil_override(&self) -> Option<ArcBorrow<Locked<PropertyDeclarationBlock>>> {
        unsafe {
            let slots = self.get_extended_slots()?;

            let base_declaration: &structs::DeclarationBlock =
                slots.mSMILOverrideStyleDeclaration.mRawPtr.as_ref()?;

            assert_eq!(base_declaration.mType, structs::StyleBackendType_Servo);
            let declaration: &structs::ServoDeclarationBlock =
                mem::transmute(base_declaration);

            debug_assert_eq!(
                &declaration._base as *const structs::DeclarationBlock,
                base_declaration as *const structs::DeclarationBlock
            );

            let raw: &structs::RawServoDeclarationBlock = declaration.mRaw.mRawPtr.as_ref()?;

            Some(Locked::<PropertyDeclarationBlock>::as_arc(
                &*(&raw as *const &structs::RawServoDeclarationBlock)
            ).borrow_arc())
        }
    }

    fn get_animation_rule_by_cascade(
        &self,
        cascade_level: ServoCascadeLevel,
    ) -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        match cascade_level {
            ServoCascadeLevel::Animations => self.get_animation_rule(),
            ServoCascadeLevel::Transitions => self.get_transition_rule(),
            _ => panic!("Unsupported cascade level for getting the animation rule")
        }
    }

    fn get_animation_rule(
        &self,
    ) -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        get_animation_rule(self, CascadeLevel::Animations)
    }

    fn get_transition_rule(
        &self,
    ) -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        get_animation_rule(self, CascadeLevel::Transitions)
    }

    #[inline]
    fn get_state(&self) -> ElementState {
        ElementState::from_bits_truncate(self.get_state_internal())
    }

    #[inline]
    fn has_attr(&self, namespace: &Namespace, attr: &Atom) -> bool {
        unsafe {
            bindings::Gecko_HasAttr(self.0, namespace.0.as_ptr(), attr.as_ptr())
        }
    }

    fn get_id(&self) -> Option<Atom> {
        if !self.has_id() {
            return None;
        }

        let ptr = unsafe {
            bindings::Gecko_AtomAttrValue(self.0, atom!("id").as_ptr())
        };

        if ptr.is_null() {
            None
        } else {
            Some(Atom::from(ptr))
        }
    }

    fn each_class<F>(&self, callback: F)
    where
        F: FnMut(&Atom),
    {
        if !self.may_have_class() {
            return;
        }

        snapshot_helpers::each_class(self.0, callback, Gecko_ClassOrClassList)
    }

    #[inline]
    fn has_snapshot(&self) -> bool {
        self.flags() & (ELEMENT_HAS_SNAPSHOT as u32) != 0
    }

    #[inline]
    fn handled_snapshot(&self) -> bool {
        self.flags() & (ELEMENT_HANDLED_SNAPSHOT as u32) != 0
    }

    unsafe fn set_handled_snapshot(&self) {
        debug_assert!(self.get_data().is_some());
        self.set_flags(ELEMENT_HANDLED_SNAPSHOT as u32)
    }

    #[inline]
    fn has_dirty_descendants(&self) -> bool {
        self.flags() & (ELEMENT_HAS_DIRTY_DESCENDANTS_FOR_SERVO as u32) != 0
    }

    unsafe fn set_dirty_descendants(&self) {
        debug_assert!(self.get_data().is_some());
        debug!("Setting dirty descendants: {:?}", self);
        self.set_flags(ELEMENT_HAS_DIRTY_DESCENDANTS_FOR_SERVO as u32)
    }

    unsafe fn unset_dirty_descendants(&self) {
        self.unset_flags(ELEMENT_HAS_DIRTY_DESCENDANTS_FOR_SERVO as u32)
    }

    #[inline]
    fn has_animation_only_dirty_descendants(&self) -> bool {
        self.flags() & (ELEMENT_HAS_ANIMATION_ONLY_DIRTY_DESCENDANTS_FOR_SERVO as u32) != 0
    }

    unsafe fn set_animation_only_dirty_descendants(&self) {
        self.set_flags(ELEMENT_HAS_ANIMATION_ONLY_DIRTY_DESCENDANTS_FOR_SERVO as u32)
    }

    unsafe fn unset_animation_only_dirty_descendants(&self) {
        self.unset_flags(ELEMENT_HAS_ANIMATION_ONLY_DIRTY_DESCENDANTS_FOR_SERVO as u32)
    }

    unsafe fn clear_descendant_bits(&self) {
        self.unset_flags(ELEMENT_HAS_DIRTY_DESCENDANTS_FOR_SERVO as u32 |
                         ELEMENT_HAS_ANIMATION_ONLY_DIRTY_DESCENDANTS_FOR_SERVO as u32 |
                         NODE_DESCENDANTS_NEED_FRAMES as u32)
    }

    #[inline]
    unsafe fn clear_dirty_bits(&self) {
        self.unset_flags(ELEMENT_HAS_DIRTY_DESCENDANTS_FOR_SERVO as u32 |
                         ELEMENT_HAS_ANIMATION_ONLY_DIRTY_DESCENDANTS_FOR_SERVO as u32 |
                         NODE_DESCENDANTS_NEED_FRAMES as u32 |
                         NODE_NEEDS_FRAME as u32)
    }

    fn is_visited_link(&self) -> bool {
        self.get_state().intersects(ElementState::IN_VISITED_STATE)
    }

    #[inline]
    fn is_native_anonymous(&self) -> bool {
        use gecko_bindings::structs::NODE_IS_NATIVE_ANONYMOUS;
        self.flags() & (NODE_IS_NATIVE_ANONYMOUS as u32) != 0
    }

    #[inline]
    fn matches_user_and_author_rules(&self) -> bool {
        !self.is_in_native_anonymous_subtree()
    }

    fn implemented_pseudo_element(&self) -> Option<PseudoElement> {
        if !self.is_native_anonymous() {
            return None;
        }

        if !self.has_properties() {
            return None;
        }

        let pseudo_type =
            unsafe { bindings::Gecko_GetImplementedPseudo(self.0) };
        PseudoElement::from_pseudo_type(pseudo_type)
    }

    fn store_children_to_process(&self, _: isize) {
        // This is only used for bottom-up traversal, and is thus a no-op for Gecko.
    }

    fn did_process_child(&self) -> isize {
        panic!("Atomic child count not implemented in Gecko");
    }

    #[inline(always)]
    fn get_data(&self) -> Option<&AtomicRefCell<ElementData>> {
        unsafe { self.0.mServoData.get().as_ref() }
    }

    unsafe fn ensure_data(&self) -> AtomicRefMut<ElementData> {
        if self.get_data().is_none() {
            debug!("Creating ElementData for {:?}", self);
            let ptr = Box::into_raw(Box::new(AtomicRefCell::new(ElementData::default())));
            self.0.mServoData.set(ptr);
        }
        self.mutate_data().unwrap()
    }

    unsafe fn clear_data(&self) {
        let ptr = self.0.mServoData.get();
        self.unset_flags(ELEMENT_HAS_SNAPSHOT as u32 |
                         ELEMENT_HANDLED_SNAPSHOT as u32 |
                         structs::Element_kAllServoDescendantBits |
                         NODE_NEEDS_FRAME as u32);
        if !ptr.is_null() {
            debug!("Dropping ElementData for {:?}", self);
            let data = Box::from_raw(self.0.mServoData.get());
            self.0.mServoData.set(ptr::null_mut());

            // Perform a mutable borrow of the data in debug builds. This
            // serves as an assertion that there are no outstanding borrows
            // when we destroy the data.
            debug_assert!({ let _ = data.borrow_mut(); true });
        }
    }

    #[inline]
    fn skip_item_display_fixup(&self) -> bool {
        debug_assert!(
            self.implemented_pseudo_element().is_none(),
            "Just don't call me if I'm a pseudo, you should know the answer already"
        );
        self.is_root_of_native_anonymous_subtree()
    }

    unsafe fn set_selector_flags(&self, flags: ElementSelectorFlags) {
        debug_assert!(!flags.is_empty());
        self.set_flags(selector_flags_to_node_flags(flags));
    }

    fn has_selector_flags(&self, flags: ElementSelectorFlags) -> bool {
        let node_flags = selector_flags_to_node_flags(flags);
        (self.flags() & node_flags) == node_flags
    }

    #[inline]
    fn may_have_animations(&self) -> bool {
        if let Some(pseudo) = self.implemented_pseudo_element() {
            if !pseudo.is_before_or_after() {
                return false;
            }
            return self.parent_element()
                       .map_or(false, |p| {
                           p.as_node()
                            .get_bool_flag(nsINode_BooleanFlag::ElementHasAnimations)
            });
        }
        self.as_node().get_bool_flag(nsINode_BooleanFlag::ElementHasAnimations)
    }

    /// Process various tasks that are a result of animation-only restyle.
    fn process_post_animation(&self,
                              tasks: PostAnimationTasks) {
        use gecko_bindings::structs::nsChangeHint_nsChangeHint_Empty;
        use gecko_bindings::structs::nsRestyleHint_eRestyle_Subtree;

        debug_assert!(!tasks.is_empty(), "Should be involved a task");

        // If display style was changed from none to other, we need to resolve
        // the descendants in the display:none subtree. Instead of resolving
        // those styles in animation-only restyle, we defer it to a subsequent
        // normal restyle.
        if tasks.intersects(PostAnimationTasks::DISPLAY_CHANGED_FROM_NONE_FOR_SMIL) {
            debug_assert!(self.implemented_pseudo_element()
                              .map_or(true, |p| !p.is_before_or_after()),
                          "display property animation shouldn't run on pseudo elements \
                           since it's only for SMIL");
            self.note_explicit_hints(nsRestyleHint_eRestyle_Subtree,
                                     nsChangeHint_nsChangeHint_Empty);
        }
    }

    /// Update various animation-related state on a given (pseudo-)element as
    /// results of normal restyle.
    fn update_animations(&self,
                         before_change_style: Option<Arc<ComputedValues>>,
                         tasks: UpdateAnimationsTasks) {
        // We have to update animations even if the element has no computed
        // style since it means the element is in a display:none subtree, we
        // should destroy all CSS animations in display:none subtree.
        let computed_data = self.borrow_data();
        let computed_values =
            computed_data.as_ref().map(|d| d.styles.primary());
        let before_change_values =
            before_change_style.as_ref().map(|x| &**x);
        let computed_values_opt = computed_values.as_ref().map(|x| &***x);
        unsafe {
            Gecko_UpdateAnimations(self.0,
                                   before_change_values,
                                   computed_values_opt,
                                   tasks.bits());
        }
    }

    fn has_animations(&self) -> bool {
        self.may_have_animations() && unsafe { Gecko_ElementHasAnimations(self.0) }
    }

    fn has_css_animations(&self) -> bool {
        self.may_have_animations() && unsafe { Gecko_ElementHasCSSAnimations(self.0) }
    }

    fn has_css_transitions(&self) -> bool {
        self.may_have_animations() && unsafe { Gecko_ElementHasCSSTransitions(self.0) }
    }

    fn each_xbl_stylist<'a, F>(&self, mut f: F) -> bool
    where
        'le: 'a,
        F: FnMut(AtomicRef<'a, Stylist>),
    {
        // Walk the binding scope chain, starting with the binding attached to
        // our content, up till we run out of scopes or we get cut off.
        //
        // If we are a NAC pseudo-element, we want to get rules from our
        // rule_hash_target, that is, our originating element.
        let mut current = Some(self.rule_hash_target());

        while let Some(element) = current {
            if let Some(binding) = element.get_xbl_binding() {
                binding.each_xbl_stylist(&mut f);

                // If we're not looking at our original element, allow the
                // binding to cut off style inheritance.
                if element != *self {
                    if !binding.inherits_style() {
                        // Go no further; we're not inheriting style from
                        // anything above here.
                        break;
                    }
                }
            }

            if element.is_root_of_native_anonymous_subtree() {
                // Deliberately cut off style inheritance here.
                break;
            }

            current = element.get_xbl_binding_parent();
        }

        // If current has something, this means we cut off inheritance at some
        // point in the loop.
        current.is_some()
    }

    fn xbl_binding_anonymous_content(&self) -> Option<GeckoNode<'le>> {
        self.get_xbl_binding_with_content()
            .map(|b| unsafe { b.anon_content().as_ref() }.unwrap())
            .map(GeckoNode::from_content)
    }

    fn get_css_transitions_info(
        &self,
    ) -> FnvHashMap<LonghandId, Arc<AnimationValue>> {
        use gecko_bindings::bindings::Gecko_ElementTransitions_EndValueAt;
        use gecko_bindings::bindings::Gecko_ElementTransitions_Length;

        let collection_length =
            unsafe { Gecko_ElementTransitions_Length(self.0) } as usize;
        let mut map = FnvHashMap::with_capacity_and_hasher(
            collection_length,
            Default::default()
        );

        for i in 0..collection_length {
            let raw_end_value = unsafe {
                 Gecko_ElementTransitions_EndValueAt(self.0, i)
            };

            let end_value = AnimationValue::arc_from_borrowed(&raw_end_value)
                .expect("AnimationValue not found in ElementTransitions");

            let property = end_value.id();
            map.insert(property, end_value.clone_arc());
        }
        map
    }

    fn might_need_transitions_update(
        &self,
        old_values: Option<&ComputedValues>,
        new_values: &ComputedValues,
    ) -> bool {
        use properties::longhands::display::computed_value::T as Display;

        let old_values = match old_values {
            Some(v) => v,
            None => return false,
        };

        let new_box_style = new_values.get_box();
        let transition_not_running = !self.has_css_transitions() &&
                                     new_box_style.transition_property_count() == 1 &&
                                     new_box_style.transition_combined_duration_at(0) <= 0.0f32;
        let new_display_style = new_box_style.clone_display();
        let old_display_style = old_values.get_box().clone_display();

        new_box_style.transition_property_count() > 0 &&
        !transition_not_running &&
        (new_display_style != Display::None &&
         old_display_style != Display::None)
    }

    // Detect if there are any changes that require us to update transitions.
    // This is used as a more thoroughgoing check than the, cheaper
    // might_need_transitions_update check.
    //
    // The following logic shadows the logic used on the Gecko side
    // (nsTransitionManager::DoUpdateTransitions) where we actually perform the
    // update.
    //
    // https://drafts.csswg.org/css-transitions/#starting
    fn needs_transitions_update(
        &self,
        before_change_style: &ComputedValues,
        after_change_style: &ComputedValues
    ) -> bool {
        use gecko_bindings::structs::nsCSSPropertyID;
        use properties::LonghandIdSet;

        debug_assert!(self.might_need_transitions_update(Some(before_change_style),
                                                         after_change_style),
                      "We should only call needs_transitions_update if \
                       might_need_transitions_update returns true");

        let after_change_box_style = after_change_style.get_box();
        let transitions_count = after_change_box_style.transition_property_count();
        let existing_transitions = self.get_css_transitions_info();

        // Check if this property is none, custom or unknown.
        let is_none_or_custom_property = |property: nsCSSPropertyID| -> bool {
            return property == nsCSSPropertyID::eCSSPropertyExtra_no_properties ||
                   property == nsCSSPropertyID::eCSSPropertyExtra_variable ||
                   property == nsCSSPropertyID::eCSSProperty_UNKNOWN;
        };

        let mut transitions_to_keep = LonghandIdSet::new();

        for i in 0..transitions_count {
            let property = after_change_box_style.transition_nscsspropertyid_at(i);
            let combined_duration = after_change_box_style.transition_combined_duration_at(i);

            // We don't need to update transition for none/custom properties.
            if is_none_or_custom_property(property) {
                continue;
            }

            let transition_property: TransitionProperty = property.into();

            let mut property_check_helper = |property: &LonghandId| -> bool {
                transitions_to_keep.insert(*property);
                self.needs_transitions_update_per_property(
                    property,
                    combined_duration,
                    before_change_style,
                    after_change_style,
                    &existing_transitions
                )
            };

            match transition_property {
                TransitionProperty::All => {
                    if TransitionProperty::any(property_check_helper) {
                        return true;
                    }
                },
                TransitionProperty::Unsupported(..) => {},
                TransitionProperty::Shorthand(ref shorthand) => {
                    if shorthand.longhands().iter().any(property_check_helper) {
                        return true;
                    }
                },
                TransitionProperty::Longhand(ref longhand_id) => {
                    if property_check_helper(longhand_id) {
                        return true;
                    }
                },
            }
        }

        // Check if we have to cancel the running transition because this is not
        // a matching transition-property value.
        existing_transitions.keys().any(|property| {
            !transitions_to_keep.contains(*property)
        })
    }

    fn needs_transitions_update_per_property(
        &self,
        longhand_id: &LonghandId,
        combined_duration: f32,
        before_change_style: &ComputedValues,
        after_change_style: &ComputedValues,
        existing_transitions: &FnvHashMap<LonghandId, Arc<AnimationValue>>,
    ) -> bool {
        use values::animated::{Animate, Procedure};

        // If there is an existing transition, update only if the end value
        // differs.
        //
        // If the end value has not changed, we should leave the currently
        // running transition as-is since we don't want to interrupt its timing
        // function.
        if let Some(ref existing) = existing_transitions.get(longhand_id) {
            let after_value =
                AnimationValue::from_computed_values(
                    longhand_id,
                    after_change_style
                ).unwrap();

            return ***existing != after_value
        }

        let from = AnimationValue::from_computed_values(
            &longhand_id,
            before_change_style,
        );
        let to = AnimationValue::from_computed_values(
            &longhand_id,
            after_change_style,
        );

        debug_assert_eq!(to.is_some(), from.is_some());

        combined_duration > 0.0f32 &&
        from != to &&
        from.unwrap().animate(
            to.as_ref().unwrap(),
            Procedure::Interpolate { progress: 0.5 }
        ).is_ok()
    }

    #[inline]
    fn lang_attr(&self) -> Option<AttrValue> {
        let ptr = unsafe { bindings::Gecko_LangValue(self.0) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { Atom::from_addrefed(ptr) })
        }
    }

    fn match_element_lang(
        &self,
        override_lang: Option<Option<AttrValue>>,
        value: &PseudoClassStringArg
    ) -> bool {
        // Gecko supports :lang() from CSS Selectors 3, which only accepts a
        // single language tag, and which performs simple dash-prefix matching
        // on it.
        debug_assert!(value.len() > 0 && value[value.len() - 1] == 0,
                      "expected value to be null terminated");
        let override_lang_ptr = match &override_lang {
            &Some(Some(ref atom)) => atom.as_ptr(),
            _ => ptr::null_mut(),
        };
        unsafe {
            Gecko_MatchLang(self.0, override_lang_ptr, override_lang.is_some(), value.as_ptr())
        }
    }

    fn is_html_document_body_element(&self) -> bool {
        if self.get_local_name() != &*local_name!("body") {
            return false;
        }

        if !self.is_html_element() {
            return false;
        }

        unsafe { bindings::Gecko_IsDocumentBody(self.0) }
    }

    fn synthesize_presentational_hints_for_legacy_attributes<V>(
        &self,
        visited_handling: VisitedHandlingMode,
        hints: &mut V
    )
    where
        V: Push<ApplicableDeclarationBlock>,
    {
        use properties::longhands::_x_lang::SpecifiedValue as SpecifiedLang;
        use properties::longhands::_x_text_zoom::SpecifiedValue as SpecifiedZoom;
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
            static ref SVG_TEXT_DISABLE_ZOOM_RULE: ApplicableDeclarationBlock = {
                let global_style_data = &*GLOBAL_STYLE_DATA;
                let pdb = PropertyDeclarationBlock::with_one(
                    PropertyDeclaration::XTextZoom(SpecifiedZoom(false)),
                    Importance::Normal
                );
                let arc = Arc::new(global_style_data.shared_lock.wrap(pdb));
                ApplicableDeclarationBlock::from_declarations(arc, ServoCascadeLevel::PresHints)
            };
        };

        let ns = self.namespace_id();
        // <th> elements get a default MozCenterOrInherit which may get overridden
        if ns == structs::kNameSpaceID_XHTML as i32 {
            if self.get_local_name().as_ptr() == atom!("th").as_ptr() {
                hints.push(TH_RULE.clone());
            } else if self.get_local_name().as_ptr() == atom!("table").as_ptr() &&
                      self.as_node().owner_doc().quirks_mode() == QuirksMode::Quirks {
                hints.push(TABLE_COLOR_RULE.clone());
            }
        }
        if ns == structs::kNameSpaceID_SVG as i32 {
            if self.get_local_name().as_ptr() == atom!("text").as_ptr() {
                hints.push(SVG_TEXT_DISABLE_ZOOM_RULE.clone());
            }
        }
        let declarations = unsafe { Gecko_GetHTMLPresentationAttrDeclarationBlock(self.0) };
        let declarations: Option<&RawOffsetArc<Locked<PropertyDeclarationBlock>>> =
            declarations.and_then(|s| s.as_arc_opt());
        if let Some(decl) = declarations {
            hints.push(
                ApplicableDeclarationBlock::from_declarations(decl.clone_arc(), ServoCascadeLevel::PresHints)
            );
        }
        let declarations = unsafe { Gecko_GetExtraContentStyleDeclarations(self.0) };
        let declarations: Option<&RawOffsetArc<Locked<PropertyDeclarationBlock>>> =
            declarations.and_then(|s| s.as_arc_opt());
        if let Some(decl) = declarations {
            hints.push(
                ApplicableDeclarationBlock::from_declarations(decl.clone_arc(), ServoCascadeLevel::PresHints)
            );
        }

        // Support for link, vlink, and alink presentation hints on <body>
        if self.is_link() {
            // Unvisited vs. visited styles are computed up-front based on the
            // visited mode (not the element's actual state).
            let declarations = match visited_handling {
                VisitedHandlingMode::AllLinksVisitedAndUnvisited => {
                    unreachable!("We should never try to selector match with \
                                 AllLinksVisitedAndUnvisited");
                },
                VisitedHandlingMode::AllLinksUnvisited => unsafe {
                    Gecko_GetUnvisitedLinkAttrDeclarationBlock(self.0)
                },
                VisitedHandlingMode::RelevantLinkVisited => unsafe {
                    Gecko_GetVisitedLinkAttrDeclarationBlock(self.0)
                },
            };
            let declarations: Option<&RawOffsetArc<Locked<PropertyDeclarationBlock>>> =
                declarations.and_then(|s| s.as_arc_opt());
            if let Some(decl) = declarations {
                hints.push(
                    ApplicableDeclarationBlock::from_declarations(decl.clone_arc(), ServoCascadeLevel::PresHints)
                );
            }

            let active = self.get_state().intersects(NonTSPseudoClass::Active.state_flag());
            if active {
                let declarations = unsafe { Gecko_GetActiveLinkAttrDeclarationBlock(self.0) };
                let declarations: Option<&RawOffsetArc<Locked<PropertyDeclarationBlock>>> =
                    declarations.and_then(|s| s.as_arc_opt());
                if let Some(decl) = declarations {
                    hints.push(
                        ApplicableDeclarationBlock::from_declarations(decl.clone_arc(), ServoCascadeLevel::PresHints)
                    );
                }
            }
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
        if ns == structs::kNameSpaceID_MathML as i32 {
            if self.get_local_name().as_ptr() == atom!("math").as_ptr() {
                hints.push(MATHML_LANG_RULE.clone());
            }
        }
    }
}

impl<'le> PartialEq for GeckoElement<'le> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 as *const _ == other.0 as *const _
    }
}

impl<'le> Eq for GeckoElement<'le> {}

impl<'le> Hash for GeckoElement<'le> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0 as *const RawGeckoElement).hash(state);
    }
}

impl<'le> ::selectors::Element for GeckoElement<'le> {
    type Impl = SelectorImpl;

    #[inline]
    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self.0)
    }

    #[inline]
    fn parent_element(&self) -> Option<Self> {
        // FIXME(emilio): This will need to jump across if the parent node is a
        // shadow root to get the shadow host.
        let parent_node = self.as_node().parent_node();
        parent_node.and_then(|n| n.as_element())
    }

    #[inline]
    fn pseudo_element_originating_element(&self) -> Option<Self> {
        debug_assert!(self.implemented_pseudo_element().is_some());
        self.closest_non_native_anonymous_ancestor()
    }

    #[inline]
    fn assigned_slot(&self) -> Option<Self> {
        let slot = self.get_extended_slots()?._base.mAssignedSlot.mRawPtr;

        unsafe {
            Some(GeckoElement(&slot.as_ref()?._base._base._base._base))
        }
    }

    #[inline]
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

    #[inline]
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

    #[inline]
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

    #[inline]
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

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&Namespace>,
        local_name: &Atom,
        operation: &AttrSelectorOperation<&Atom>
    ) -> bool {
        unsafe {
            match *operation {
                AttrSelectorOperation::Exists => {
                    bindings::Gecko_HasAttr(self.0,
                                            ns.atom_or_null(),
                                            local_name.as_ptr())
                }
                AttrSelectorOperation::WithValue { operator, case_sensitivity, expected_value } => {
                    let ignore_case = match case_sensitivity {
                        CaseSensitivity::CaseSensitive => false,
                        CaseSensitivity::AsciiCaseInsensitive => true,
                    };
                    // FIXME: case sensitivity for operators other than Equal
                    match operator {
                        AttrSelectorOperator::Equal => bindings::Gecko_AttrEquals(
                            self.0,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                            ignore_case
                        ),
                        AttrSelectorOperator::Includes => bindings::Gecko_AttrIncludes(
                            self.0,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                            ignore_case,
                        ),
                        AttrSelectorOperator::DashMatch => bindings::Gecko_AttrDashEquals(
                            self.0,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                            ignore_case,
                        ),
                        AttrSelectorOperator::Prefix => bindings::Gecko_AttrHasPrefix(
                            self.0,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                            ignore_case,
                        ),
                        AttrSelectorOperator::Suffix => bindings::Gecko_AttrHasSuffix(
                            self.0,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                            ignore_case,
                        ),
                        AttrSelectorOperator::Substring => bindings::Gecko_AttrHasSubstring(
                            self.0,
                            ns.atom_or_null(),
                            local_name.as_ptr(),
                            expected_value.as_ptr(),
                            ignore_case,
                        ),
                    }
                }
            }
        }
    }

    #[inline]
    fn is_root(&self) -> bool {
        let parent_node = match self.as_node().parent_node() {
            Some(parent_node) => parent_node,
            None => return false,
        };

        if !parent_node.is_document() {
            return false;
        }

        unsafe {
            Gecko_IsRootElement(self.0)
        }
    }

    fn is_empty(&self) -> bool {
        !self.as_node().dom_children().any(|child| unsafe {
            Gecko_IsSignificantChild(child.0, true, true)
        })
    }

    #[inline]
    fn get_local_name(&self) -> &WeakAtom {
        unsafe {
            WeakAtom::new(self.as_node().node_info().mInner.mName)
        }
    }

    #[inline]
    fn get_namespace(&self) -> &WeakNamespace {
        unsafe {
            let namespace_manager = structs::nsContentUtils_sNameSpaceManager;
            WeakNamespace::new((*namespace_manager).mURIArray[self.namespace_id() as usize].mRawPtr)
        }
    }

    fn match_non_ts_pseudo_class<F>(
        &self,
        pseudo_class: &NonTSPseudoClass,
        context: &mut MatchingContext<Self::Impl>,
        flags_setter: &mut F,
    ) -> bool
    where
        F: FnMut(&Self, ElementSelectorFlags),
    {
        use selectors::matching::*;
        match *pseudo_class {
            NonTSPseudoClass::Focus |
            NonTSPseudoClass::Enabled |
            NonTSPseudoClass::Disabled |
            NonTSPseudoClass::Checked |
            NonTSPseudoClass::Fullscreen |
            NonTSPseudoClass::MozFullScreen |
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
            NonTSPseudoClass::MozMeterSubSubOptimum |
            NonTSPseudoClass::MozHasDirAttr |
            NonTSPseudoClass::MozDirAttrLTR |
            NonTSPseudoClass::MozDirAttrRTL |
            NonTSPseudoClass::MozDirAttrLikeAuto |
            NonTSPseudoClass::MozAutofill |
            NonTSPseudoClass::Active |
            NonTSPseudoClass::Hover |
            NonTSPseudoClass::MozAutofillPreview => {
                self.get_state().intersects(pseudo_class.state_flag())
            },
            NonTSPseudoClass::AnyLink => self.is_link(),
            NonTSPseudoClass::Link => {
                self.is_link() && context.visited_handling().matches_unvisited()
            }
            NonTSPseudoClass::Visited => {
                self.is_link() && context.visited_handling().matches_visited()
            }
            NonTSPseudoClass::MozFirstNode => {
                flags_setter(self, ElementSelectorFlags::HAS_EDGE_CHILD_SELECTOR);
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
                flags_setter(self, ElementSelectorFlags::HAS_EDGE_CHILD_SELECTOR);
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
                flags_setter(self, ElementSelectorFlags::HAS_EMPTY_SELECTOR);
                if self.as_node().dom_children().any(|c| c.contains_non_whitespace_content()) {
                    return false
                }
                true
            }
            NonTSPseudoClass::MozTableBorderNonzero |
            NonTSPseudoClass::MozBrowserFrame |
            NonTSPseudoClass::MozNativeAnonymous |
            NonTSPseudoClass::MozUseShadowTreeRoot => unsafe {
                Gecko_MatchesElement(pseudo_class.to_gecko_pseudoclasstype().unwrap(), self.0)
            },
            NonTSPseudoClass::MozIsHTML => {
                self.is_html_element_in_html_document()
            }
            NonTSPseudoClass::MozLWTheme => {
                self.get_document_theme() != DocumentTheme::Doc_Theme_None
            }
            NonTSPseudoClass::MozLWThemeBrightText => {
                self.get_document_theme() == DocumentTheme::Doc_Theme_Bright
            }
            NonTSPseudoClass::MozLWThemeDarkText => {
                self.get_document_theme() == DocumentTheme::Doc_Theme_Dark
            }
            NonTSPseudoClass::MozWindowInactive => {
                let state_bit = DocumentState::NS_DOCUMENT_STATE_WINDOW_INACTIVE;
                if context.extra_data.document_state.intersects(state_bit) {
                    return !context.in_negation();
                }

                self.document_state().contains(state_bit)
            }
            NonTSPseudoClass::MozPlaceholder => false,
            NonTSPseudoClass::MozAny(ref sels) => {
                context.nest(|context| {
                    sels.iter().any(|s| {
                        matches_complex_selector(s.iter(), self, context, flags_setter)
                    })
                })
            }
            NonTSPseudoClass::Lang(ref lang_arg) => {
                self.match_element_lang(None, lang_arg)
            }
            NonTSPseudoClass::MozLocaleDir(ref dir) => {
                let state_bit = DocumentState::NS_DOCUMENT_STATE_RTL_LOCALE;
                if context.extra_data.document_state.intersects(state_bit) {
                    // NOTE(emilio): We could still return false for
                    // Direction::Other(..), but we don't bother.
                    return !context.in_negation();
                }

                let doc_is_rtl = self.document_state().contains(state_bit);

                match **dir {
                    Direction::Ltr => !doc_is_rtl,
                    Direction::Rtl => doc_is_rtl,
                    Direction::Other(..) => false,
                }
            }
            NonTSPseudoClass::Dir(ref dir) => {
                match **dir {
                    Direction::Ltr => self.get_state().intersects(ElementState::IN_LTR_STATE),
                    Direction::Rtl => self.get_state().intersects(ElementState::IN_RTL_STATE),
                    Direction::Other(..) => false,
                }
            }
        }
    }

    fn match_pseudo_element(
        &self,
        pseudo_element: &PseudoElement,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        // TODO(emilio): I believe we could assert we are a pseudo-element and
        // match the proper pseudo-element, given how we rulehash the stuff
        // based on the pseudo.
        match self.implemented_pseudo_element() {
            Some(ref pseudo) => *pseudo == pseudo_element.canonical(),
            None => false,
        }
    }

    #[inline]
    fn is_link(&self) -> bool {
        self.get_state().intersects(NonTSPseudoClass::AnyLink.state_flag())
    }

    #[inline]
    fn has_id(&self, id: &Atom, case_sensitivity: CaseSensitivity) -> bool {
        if !self.has_id() {
            return false
        }

        unsafe {
            let ptr = bindings::Gecko_AtomAttrValue(self.0, atom!("id").as_ptr());

            if ptr.is_null() {
                false
            } else {
                case_sensitivity.eq_atom(WeakAtom::new(ptr), id)
            }
        }
    }

    #[inline(always)]
    fn has_class(&self, name: &Atom, case_sensitivity: CaseSensitivity) -> bool {
        if !self.may_have_class() {
            return false;
        }

        snapshot_helpers::has_class(
            self.0,
            name,
            case_sensitivity,
            Gecko_ClassOrClassList,
        )
    }

    #[inline]
    fn is_html_element_in_html_document(&self) -> bool {
        self.is_html_element() &&
        self.as_node().owner_doc().is_html_document()
    }

    #[inline]
    fn ignores_nth_child_selectors(&self) -> bool {
        self.is_root_of_anonymous_subtree()
    }

    #[inline]
    fn blocks_ancestor_combinators(&self) -> bool {
        if !self.is_root_of_anonymous_subtree() {
            return false
        }

        match self.parent_element() {
            Some(e) => {
                // If this element is the shadow root of an use-element shadow
                // tree, according to the spec, we should not match rules
                // cross the shadow DOM boundary.
                e.get_local_name() == &*local_name!("use") &&
                e.get_namespace() == &*ns!("http://www.w3.org/2000/svg")
            },
            None => false,
        }
    }
}

/// A few helpers to help with attribute selectors and snapshotting.
pub trait NamespaceConstraintHelpers {
    /// Returns the namespace of the selector, or null otherwise.
    fn atom_or_null(&self) -> *mut nsAtom;
}

impl<'a> NamespaceConstraintHelpers for NamespaceConstraint<&'a Namespace> {
    fn atom_or_null(&self) -> *mut nsAtom {
        match *self {
            NamespaceConstraint::Any => ptr::null_mut(),
            NamespaceConstraint::Specific(ref ns) => ns.0.as_ptr(),
        }
    }
}
