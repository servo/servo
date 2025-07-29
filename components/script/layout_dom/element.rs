/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::hash::Hash;
use std::sync::atomic::Ordering;
use std::{fmt, slice};

use atomic_refcell::{AtomicRef, AtomicRefMut};
use embedder_traits::UntrustedNodeAddress;
use html5ever::{LocalName, Namespace, local_name, ns};
use js::jsapi::JSObject;
use layout_api::wrapper_traits::{LayoutNode, ThreadSafeLayoutElement, ThreadSafeLayoutNode};
use layout_api::{LayoutDamage, LayoutNodeType, StyleData};
use selectors::Element as _;
use selectors::attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint};
use selectors::bloom::{BLOOM_HASH_MASK, BloomFilter};
use selectors::matching::{ElementSelectorFlags, MatchingContext, VisitedHandlingMode};
use selectors::sink::Push;
use servo_arc::{Arc, ArcBorrow};
use style::CaseSensitivityExt;
use style::animation::AnimationSetKey;
use style::applicable_declarations::ApplicableDeclarationBlock;
use style::attr::AttrValue;
use style::bloom::each_relevant_element_hash;
use style::context::SharedStyleContext;
use style::data::ElementData;
use style::dom::{DomChildren, LayoutIterator, TDocument, TElement, TNode, TShadowRoot};
use style::properties::{ComputedValues, PropertyDeclarationBlock};
use style::selector_parser::{
    AttrValue as SelectorAttrValue, Lang, NonTSPseudoClass, PseudoElement, RestyleDamage,
    SelectorImpl, extended_filtering,
};
use style::shared_lock::Locked as StyleLocked;
use style::stylesheets::scope_rule::ImplicitScopeRoot;
use style::values::computed::{Display, Image};
use style::values::specified::align::AlignFlags;
use style::values::specified::box_::{DisplayInside, DisplayOutside};
use style::values::{AtomIdent, AtomString};
use stylo_atoms::Atom;
use stylo_dom::ElementState;

use crate::dom::attr::AttrHelpersForLayout;
use crate::dom::bindings::inheritance::{
    Castable, CharacterDataTypeId, DocumentFragmentTypeId, ElementTypeId, HTMLElementTypeId,
    NodeTypeId, TextTypeId,
};
use crate::dom::bindings::root::LayoutDom;
use crate::dom::characterdata::LayoutCharacterDataHelpers;
use crate::dom::element::{Element, LayoutElementHelpers};
use crate::dom::htmlslotelement::HTMLSlotElement;
use crate::dom::node::{LayoutNodeHelpers, Node, NodeFlags};
use crate::layout_dom::{ServoLayoutNode, ServoShadowRoot, ServoThreadSafeLayoutNode};

/// A wrapper around elements that ensures layout can only ever access safe properties.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct ServoLayoutElement<'dom> {
    /// The wrapped private DOM Element.
    element: LayoutDom<'dom, Element>,
}

impl fmt::Debug for ServoLayoutElement<'_> {
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

    /// Unset the snapshot flags on the underlying DOM object for this element.
    ///
    /// # Safety
    ///
    /// This function accesses and modifies the underlying DOM object and should
    /// not be used by more than a single thread at once.
    pub unsafe fn unset_snapshot_flags(&self) {
        unsafe {
            self.as_node()
                .node
                .set_flag(NodeFlags::HAS_SNAPSHOT | NodeFlags::HANDLED_SNAPSHOT, false);
        }
    }

    /// Unset the snapshot flags on the underlying DOM object for this element.
    ///
    /// # Safety
    ///
    /// This function accesses and modifies the underlying DOM object and should
    /// not be used by more than a single thread at once.
    pub unsafe fn set_has_snapshot(&self) {
        unsafe {
            self.as_node().node.set_flag(NodeFlags::HAS_SNAPSHOT, true);
        }
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

pub enum DOMDescendantIterator<E>
where
    E: TElement,
{
    /// Iterating over the children of a node, including children of a potential
    /// [ShadowRoot](crate::dom::shadow_root::ShadowRoot)
    Children(DomChildren<E::ConcreteNode>),
    /// Iterating over the content's of a [`<slot>`](HTMLSlotElement) element.
    Slottables { slot: E, index: usize },
}

impl<E> Iterator for DOMDescendantIterator<E>
where
    E: TElement,
{
    type Item = E::ConcreteNode;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Children(children) => children.next(),
            Self::Slottables { slot, index } => {
                let slottables = slot.slotted_nodes();
                let slot = slottables.get(*index)?;
                *index += 1;
                Some(*slot)
            },
        }
    }
}

impl<'dom> style::dom::TElement for ServoLayoutElement<'dom> {
    type ConcreteNode = ServoLayoutNode<'dom>;
    type TraversalChildrenIterator = DOMDescendantIterator<Self>;

    fn as_node(&self) -> ServoLayoutNode<'dom> {
        ServoLayoutNode::from_layout_js(self.element.upcast())
    }

    fn traversal_children(&self) -> LayoutIterator<Self::TraversalChildrenIterator> {
        let iterator = if self.slotted_nodes().is_empty() {
            let children = if let Some(shadow_root) = self.shadow_root() {
                shadow_root.as_node().dom_children()
            } else {
                self.as_node().dom_children()
            };
            DOMDescendantIterator::Children(children)
        } else {
            DOMDescendantIterator::Slottables {
                slot: *self,
                index: 0,
            }
        };

        LayoutIterator(iterator)
    }

    fn traversal_parent(&self) -> Option<Self> {
        self.as_node().traversal_parent()
    }

    fn inheritance_parent(&self) -> Option<Self> {
        if self.is_pseudo_element() {
            // The inheritance parent of an implemented pseudo-element should be the
            // originating element, except if `is_element_backed()` is true, then it should
            // be the flat tree parent. Note `is_element_backed()` differs from the CSS term.
            // At the current time, `is_element_backed()` is always false in Servo.
            //
            // FIXME: handle the cases of element-backed pseudo-elements.
            return self.pseudo_element_originating_element();
        }

        // FIXME: By default the inheritance parent would be the Self::parent_element
        //        but probably we should use the flattened tree parent.
        self.parent_element()
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
        self.element
            .get_attr_for_layout(&ns!(), &local_name!("part"))
            .is_some()
    }

    fn exports_any_part(&self) -> bool {
        self.element
            .get_attr_for_layout(&ns!(), &local_name!("exportparts"))
            .is_some()
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

    fn each_part<F>(&self, mut callback: F)
    where
        F: FnMut(&AtomIdent),
    {
        if let Some(parts) = self.element.get_parts_for_layout() {
            for part in parts {
                callback(AtomIdent::cast(part))
            }
        }
    }

    fn each_exported_part<F>(&self, name: &AtomIdent, callback: F)
    where
        F: FnMut(&AtomIdent),
    {
        let Some(exported_parts) = self
            .element
            .get_attr_for_layout(&ns!(), &local_name!("exportparts"))
        else {
            return;
        };
        exported_parts
            .as_shadow_parts()
            .for_each_exported_part(AtomIdent::cast(name), callback);
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
        unsafe {
            self.as_node()
                .node
                .set_flag(NodeFlags::HANDLED_SNAPSHOT, true);
        }
    }

    unsafe fn set_dirty_descendants(&self) {
        debug_assert!(self.as_node().is_connected());
        unsafe {
            self.as_node()
                .node
                .set_flag(NodeFlags::HAS_DIRTY_DESCENDANTS, true)
        }
    }

    unsafe fn unset_dirty_descendants(&self) {
        unsafe {
            self.as_node()
                .node
                .set_flag(NodeFlags::HAS_DIRTY_DESCENDANTS, false)
        }
    }

    /// Whether this element should match user and content rules.
    /// We would like to match rules from the same tree in all cases and optimize computation.
    /// UA Widget is an exception since we could have a pseudo element selector inside it.
    #[inline]
    fn matches_user_and_content_rules(&self) -> bool {
        !self.as_node().node.is_in_ua_widget()
    }

    /// Returns the pseudo-element implemented by this element, if any. In other words,
    /// the element will match the specified pseudo element throughout the style computation.
    #[inline]
    fn implemented_pseudo_element(&self) -> Option<PseudoElement> {
        self.as_node().node.implemented_pseudo_element()
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
        unsafe { self.as_node().get_jsmanaged().clear_style_and_layout_data() }
    }

    unsafe fn ensure_data(&self) -> AtomicRefMut<ElementData> {
        unsafe {
            self.as_node().get_jsmanaged().initialize_style_data();
        };
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

    /// Returns the implicit scope root for given sheet index and host.
    fn implicit_scope_for_sheet_in_shadow_root(
        opaque_host: ::selectors::OpaqueElement,
        sheet_index: usize,
    ) -> Option<ImplicitScopeRoot> {
        // As long as this "unopaqued" element does not escape this function, we're not leaking
        // potentially-mutable elements from opaque elements.
        let host = unsafe {
            let ptr = opaque_host.as_const_ptr::<JSObject>();
            let untrusted_address = UntrustedNodeAddress::from_id(ptr as usize);
            let node = Node::from_untrusted_node_address(untrusted_address);
            let trusted_address = node.to_trusted_node_address();
            let servo_layout_node = ServoLayoutNode::new(&trusted_address);
            servo_layout_node.as_element().unwrap()
        };
        host.shadow_root()?.implicit_scope_for_sheet(sheet_index)
    }

    fn slotted_nodes(&self) -> &[Self::ConcreteNode] {
        let Some(slot_element) = self.element.unsafe_get().downcast::<HTMLSlotElement>() else {
            return &[];
        };
        let assigned_nodes = slot_element.assigned_nodes();

        // SAFETY:
        // Self::ConcreteNode (aka ServoLayoutNode) and Slottable are guaranteed to have the same
        // layout and alignment as ptr::NonNull<T>. Lifetimes are not an issue because the
        // slottables are being kept alive by the slot element.
        unsafe {
            slice::from_raw_parts(
                assigned_nodes.as_ptr() as *const Self::ConcreteNode,
                assigned_nodes.len(),
            )
        }
    }

    fn compute_layout_damage(old: &ComputedValues, new: &ComputedValues) -> RestyleDamage {
        let box_tree_needs_rebuild = || {
            let old_box = old.get_box();
            let new_box = new.get_box();

            if old_box.display != new_box.display ||
                old_box.float != new_box.float ||
                old_box.position != new_box.position
            {
                return true;
            }

            if old.get_font() != new.get_font() {
                return true;
            }

            // NOTE: This should be kept in sync with the checks in `impl
            // StyleExt::establishes_block_formatting_context` for `ComputedValues` in
            // `components/layout/style_ext.rs`.
            if new_box.display.outside() == DisplayOutside::Block &&
                new_box.display.inside() == DisplayInside::Flow
            {
                let alignment_establishes_new_block_formatting_context =
                    |style: &ComputedValues| {
                        style.get_position().align_content.0.primary() != AlignFlags::NORMAL
                    };

                let old_column = old.get_column();
                let new_column = new.get_column();
                if old_box.overflow_x.is_scrollable() != new_box.overflow_x.is_scrollable() ||
                    old_column.is_multicol() != new_column.is_multicol() ||
                    old_column.column_span != new_column.column_span ||
                    alignment_establishes_new_block_formatting_context(old) !=
                        alignment_establishes_new_block_formatting_context(new)
                {
                    return true;
                }
            }

            if old_box.display.is_list_item() {
                let old_list = old.get_list();
                let new_list = new.get_list();
                if old_list.list_style_position != new_list.list_style_position ||
                    old_list.list_style_image != new_list.list_style_image ||
                    (new_list.list_style_image == Image::None &&
                        old_list.list_style_type != new_list.list_style_type)
                {
                    return true;
                }
            }

            if new.is_pseudo_style() && old.get_counters().content != new.get_counters().content {
                return true;
            }

            false
        };

        let text_shaping_needs_recollect = || {
            if old.clone_direction() != new.clone_direction() ||
                old.clone_unicode_bidi() != new.clone_unicode_bidi()
            {
                return true;
            }

            let old_text = old.get_inherited_text().clone();
            let new_text = new.get_inherited_text().clone();
            if old_text.white_space_collapse != new_text.white_space_collapse ||
                old_text.text_transform != new_text.text_transform ||
                old_text.word_break != new_text.word_break ||
                old_text.overflow_wrap != new_text.overflow_wrap ||
                old_text.letter_spacing != new_text.letter_spacing ||
                old_text.word_spacing != new_text.word_spacing ||
                old_text.text_rendering != new_text.text_rendering
            {
                return true;
            }

            false
        };

        if box_tree_needs_rebuild() {
            RestyleDamage::from_bits_retain(LayoutDamage::REBUILD_BOX.bits())
        } else if text_shaping_needs_recollect() {
            RestyleDamage::from_bits_retain(LayoutDamage::RECOLLECT_BOX_TREE_CHILDREN.bits())
        } else {
            // This element needs to be laid out again, but does not have any damage to
            // its box. In the future, we will distinguish between types of damage to the
            // fragment as well.
            RestyleDamage::RELAYOUT
        }
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

    #[inline]
    fn is_pseudo_element(&self) -> bool {
        self.implemented_pseudo_element().is_some()
    }

    #[inline]
    fn pseudo_element_originating_element(&self) -> Option<Self> {
        debug_assert!(self.is_pseudo_element());
        debug_assert!(!self.matches_user_and_content_rules());
        self.containing_shadow_host()
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
                .is_some_and(|value| value.eval_selector(operation)),
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
                .contains(NonTSPseudoClass::ReadWrite.state_flag()),

            NonTSPseudoClass::Active |
            NonTSPseudoClass::Autofill |
            NonTSPseudoClass::Checked |
            NonTSPseudoClass::Default |
            NonTSPseudoClass::Defined |
            NonTSPseudoClass::Disabled |
            NonTSPseudoClass::Enabled |
            NonTSPseudoClass::Focus |
            NonTSPseudoClass::FocusVisible |
            NonTSPseudoClass::FocusWithin |
            NonTSPseudoClass::Fullscreen |
            NonTSPseudoClass::Hover |
            NonTSPseudoClass::InRange |
            NonTSPseudoClass::Indeterminate |
            NonTSPseudoClass::Invalid |
            NonTSPseudoClass::Modal |
            NonTSPseudoClass::MozMeterOptimum |
            NonTSPseudoClass::MozMeterSubOptimum |
            NonTSPseudoClass::MozMeterSubSubOptimum |
            NonTSPseudoClass::Optional |
            NonTSPseudoClass::OutOfRange |
            NonTSPseudoClass::PlaceholderShown |
            NonTSPseudoClass::PopoverOpen |
            NonTSPseudoClass::ReadWrite |
            NonTSPseudoClass::Required |
            NonTSPseudoClass::Target |
            NonTSPseudoClass::UserInvalid |
            NonTSPseudoClass::UserValid |
            NonTSPseudoClass::Valid => self
                .element
                .get_state_for_layout()
                .contains(pseudo_class.state_flag()),
        }
    }

    fn match_pseudo_element(
        &self,
        pseudo: &PseudoElement,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        self.implemented_pseudo_element() == Some(*pseudo)
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
                .is_some_and(|atom| case_sensitivity.eq_atom(atom, id))
        }
    }

    #[inline]
    fn is_part(&self, name: &AtomIdent) -> bool {
        self.element.has_class_or_part_for_layout(
            name,
            &local_name!("part"),
            CaseSensitivity::CaseSensitive,
        )
    }

    fn imported_part(&self, name: &AtomIdent) -> Option<AtomIdent> {
        self.element
            .get_attr_for_layout(&ns!(), &local_name!("exportparts"))?
            .as_shadow_parts()
            .imported_part(name)
            .map(|import| AtomIdent::new(import.clone()))
    }

    #[inline]
    fn has_class(&self, name: &AtomIdent, case_sensitivity: CaseSensitivity) -> bool {
        self.element
            .has_class_or_part_for_layout(name, &local_name!("class"), case_sensitivity)
    }

    fn is_html_slot_element(&self) -> bool {
        self.element.is::<HTMLSlotElement>()
    }

    #[allow(unsafe_code)]
    fn assigned_slot(&self) -> Option<Self> {
        self.as_node().assigned_slot()
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

    /// The pseudo-element type for this element, or `None` if it is the non-pseudo
    /// version of the element.
    pub(super) pseudo: Option<PseudoElement>,
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

    fn pseudo_element(&self) -> Option<PseudoElement> {
        self.pseudo
    }

    fn with_pseudo(&self, pseudo_element_type: PseudoElement) -> Option<Self> {
        if pseudo_element_type.is_eager() &&
            self.style_data()
                .styles
                .pseudos
                .get(&pseudo_element_type)
                .is_none()
        {
            return None;
        }

        if pseudo_element_type == PseudoElement::DetailsSummary &&
            (!self.has_local_name(&local_name!("details")) || !self.has_namespace(&ns!(html)))
        {
            return None;
        }

        if pseudo_element_type == PseudoElement::DetailsContent &&
            (!self.has_local_name(&local_name!("details")) ||
                !self.has_namespace(&ns!(html)) ||
                self.get_attr(&ns!(), &local_name!("open")).is_none())
        {
            return None;
        }

        Some(ServoThreadSafeLayoutElement {
            element: self.element,
            pseudo: Some(pseudo_element_type),
        })
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

    fn is_root(&self) -> bool {
        self.element.is_root()
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
impl ::selectors::Element for ServoThreadSafeLayoutElement<'_> {
    type Impl = SelectorImpl;

    fn opaque(&self) -> ::selectors::OpaqueElement {
        ::selectors::OpaqueElement::new(unsafe { &*(self.as_node().opaque().0 as *const ()) })
    }

    fn parent_element(&self) -> Option<Self> {
        warn!("ServoThreadSafeLayoutElement::parent_element called");
        None
    }

    #[inline]
    fn parent_node_is_shadow_root(&self) -> bool {
        self.element.parent_node_is_shadow_root()
    }

    #[inline]
    fn containing_shadow_host(&self) -> Option<Self> {
        self.element
            .containing_shadow_host()
            .and_then(|element| element.as_node().to_threadsafe().as_element())
    }

    #[inline]
    fn is_pseudo_element(&self) -> bool {
        self.element.is_pseudo_element()
    }

    #[inline]
    fn pseudo_element_originating_element(&self) -> Option<Self> {
        self.element
            .pseudo_element_originating_element()
            .and_then(|element| element.as_node().to_threadsafe().as_element())
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
        self.element.is_html_element_in_html_document()
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

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&style::Namespace>,
        local_name: &style::LocalName,
        operation: &AttrSelectorOperation<&AtomString>,
    ) -> bool {
        match *ns {
            NamespaceConstraint::Specific(ns) => self
                .get_attr_enum(ns, local_name)
                .is_some_and(|value| value.eval_selector(operation)),
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

    fn match_pseudo_element(
        &self,
        pseudo: &PseudoElement,
        context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        self.element.match_pseudo_element(pseudo, context)
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
