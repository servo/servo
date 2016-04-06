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

use core::nonzero::NonZero;
use data::{LayoutDataFlags, PrivateLayoutData};
use gfx::display_list::OpaqueNode;
use gfx::text::glyph::CharIndex;
use incremental::RestyleDamage;
use msg::constellation_msg::PipelineId;
use opaque_node::OpaqueNodeMethods;
use range::{Range, RangeIndex};
use script::dom::attr::AttrValue;
use script::dom::bindings::inheritance::{Castable, CharacterDataTypeId, ElementTypeId};
use script::dom::bindings::inheritance::{HTMLElementTypeId, NodeTypeId};
use script::dom::bindings::js::LayoutJS;
use script::dom::characterdata::LayoutCharacterDataHelpers;
use script::dom::document::{Document, LayoutDocumentHelpers};
use script::dom::element::{Element, LayoutElementHelpers, RawLayoutElementHelpers};
use script::dom::htmlcanvaselement::{LayoutHTMLCanvasElementHelpers, HTMLCanvasData};
use script::dom::htmliframeelement::HTMLIFrameElement;
use script::dom::htmlimageelement::LayoutHTMLImageElementHelpers;
use script::dom::htmlinputelement::{HTMLInputElement, LayoutHTMLInputElementHelpers};
use script::dom::htmltextareaelement::{HTMLTextAreaElement, LayoutHTMLTextAreaElementHelpers};
use script::dom::node::{CAN_BE_FRAGMENTED, HAS_CHANGED, HAS_DIRTY_DESCENDANTS, IS_DIRTY};
use script::dom::node::{LayoutNodeHelpers, Node, OpaqueStyleAndLayoutData};
use script::dom::text::Text;
use script::layout_interface::TrustedNodeAddress;
use selectors::matching::{DeclarationBlock, ElementFlags};
use selectors::parser::{AttrSelector, NamespaceConstraint};
use smallvec::VecLike;
use std::borrow::ToOwned;
use std::cell::{Ref, RefCell, RefMut};
use std::marker::PhantomData;
use std::mem::{transmute, transmute_copy};
use std::sync::Arc;
use string_cache::{Atom, Namespace};
use style::computed_values::content::ContentItem;
use style::computed_values::{content, display};
use style::dom::{TDocument, TElement, TNode, UnsafeNode};
use style::element_state::*;
use style::properties::{ComputedValues, ServoComputedValues};
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock};
use style::restyle_hints::ElementSnapshot;
use style::selector_impl::{NonTSPseudoClass, PseudoElement, ServoSelectorImpl};
use style::servo::PrivateStyleData;
use url::Url;
use util::str::{is_whitespace, search_index};

pub type NonOpaqueStyleAndLayoutData = *mut RefCell<PrivateLayoutData>;

/// A wrapper so that layout can access only the methods that it should have access to. Layout must
/// only ever see these and must never see instances of `LayoutJS`.

pub trait LayoutNode : TNode {
    type ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode;
    fn to_threadsafe(&self) -> Self::ConcreteThreadSafeLayoutNode;

    /// Returns the type ID of this node.
    fn type_id(&self) -> NodeTypeId;

    /// Similar to borrow_data*, but returns the full PrivateLayoutData rather
    /// than only the PrivateStyleData.
    unsafe fn borrow_layout_data_unchecked(&self) -> Option<*const PrivateLayoutData>;
    fn borrow_layout_data(&self) -> Option<Ref<PrivateLayoutData>>;
    fn mutate_layout_data(&self) -> Option<RefMut<PrivateLayoutData>>;
}

#[derive(Copy, Clone)]
pub struct ServoLayoutNode<'a> {
    /// The wrapped node.
    node: LayoutJS<Node>,

    /// Being chained to a PhantomData prevents `LayoutNode`s from escaping.
    chain: PhantomData<&'a ()>,
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
}

impl<'ln> TNode for ServoLayoutNode<'ln> {
    type ConcreteComputedValues = ServoComputedValues;
    type ConcreteElement = ServoLayoutElement<'ln>;
    type ConcreteDocument = ServoLayoutDocument<'ln>;
    type ConcreteRestyleDamage = RestyleDamage;

    fn to_unsafe(&self) -> UnsafeNode {
        unsafe {
            let ptr: usize = transmute_copy(self);
            (ptr, 0)
        }
    }

    unsafe fn from_unsafe(n: &UnsafeNode) -> Self {
        let (node, _) = *n;
        transmute(node)
    }

    fn is_text_node(&self) -> bool {
        self.type_id() == NodeTypeId::CharacterData(CharacterDataTypeId::Text)
    }

    fn is_element(&self) -> bool {
        unsafe {
            self.node.is_element_for_layout()
        }
    }

    fn dump(self) {
        self.dump_indent(0);
    }

    fn opaque(&self) -> OpaqueNode {
        OpaqueNodeMethods::from_jsmanaged(unsafe { self.get_jsmanaged() })
    }

    fn initialize_data(self) {
        let has_data = unsafe { self.borrow_data_unchecked().is_some() };
        if !has_data {
            let ptr: NonOpaqueStyleAndLayoutData =
                Box::into_raw(box RefCell::new(PrivateLayoutData::new()));
            let opaque = OpaqueStyleAndLayoutData {
                ptr: unsafe { NonZero::new(ptr as *mut ()) }
            };
            unsafe {
                self.node.init_style_and_layout_data(opaque);
            }
        }
    }

    fn layout_parent_node(self, reflow_root: OpaqueNode) -> Option<ServoLayoutNode<'ln>> {
        if self.opaque() == reflow_root {
            None
        } else {
            self.parent_node()
        }
    }

    fn debug_id(self) -> usize {
        self.opaque().to_untrusted_node_address().0 as usize
    }

    fn children_count(&self) -> u32 {
        unsafe { self.node.children_count() }
    }

    fn as_element(&self) -> Option<ServoLayoutElement<'ln>> {
        as_element(self.node)
    }

    fn as_document(&self) -> Option<ServoLayoutDocument<'ln>> {
        self.node.downcast().map(ServoLayoutDocument::from_layout_js)
    }

    fn has_changed(&self) -> bool {
        unsafe { self.node.get_flag(HAS_CHANGED) }
    }

    unsafe fn set_changed(&self, value: bool) {
        self.node.set_flag(HAS_CHANGED, value)
    }

    fn is_dirty(&self) -> bool {
        unsafe { self.node.get_flag(IS_DIRTY) }
    }

    unsafe fn set_dirty(&self, value: bool) {
        self.node.set_flag(IS_DIRTY, value)
    }

    fn has_dirty_descendants(&self) -> bool {
        unsafe { self.node.get_flag(HAS_DIRTY_DESCENDANTS) }
    }

    unsafe fn set_dirty_descendants(&self, value: bool) {
        self.node.set_flag(HAS_DIRTY_DESCENDANTS, value)
    }

    fn can_be_fragmented(&self) -> bool {
        unsafe { self.node.get_flag(CAN_BE_FRAGMENTED) }
    }

    unsafe fn set_can_be_fragmented(&self, value: bool) {
        self.node.set_flag(CAN_BE_FRAGMENTED, value)
    }

    unsafe fn borrow_data_unchecked(&self) -> Option<*const PrivateStyleData> {
        self.borrow_layout_data_unchecked().map(|d| &(*d).style_data as *const PrivateStyleData)
    }

    fn borrow_data(&self) -> Option<Ref<PrivateStyleData>> {
        unsafe { self.borrow_layout_data().map(|d| transmute(d)) }
    }

    fn mutate_data(&self) -> Option<RefMut<PrivateStyleData>> {
        unsafe { self.mutate_layout_data().map(|d| transmute(d)) }
    }

    fn restyle_damage(self) -> RestyleDamage {
        self.borrow_layout_data().unwrap().restyle_damage
    }

    fn set_restyle_damage(self, damage: RestyleDamage) {
        self.mutate_layout_data().unwrap().restyle_damage = damage;
    }

    fn parent_node(&self) -> Option<ServoLayoutNode<'ln>> {
        unsafe {
            self.node.parent_node_ref().map(|node| self.new_with_this_lifetime(&node))
        }
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

impl<'ln> LayoutNode for ServoLayoutNode<'ln> {
    type ConcreteThreadSafeLayoutNode = ServoThreadSafeLayoutNode<'ln>;

    fn to_threadsafe(&self) -> Self::ConcreteThreadSafeLayoutNode {
        ServoThreadSafeLayoutNode::new(self)
    }

    fn type_id(&self) -> NodeTypeId {
        unsafe {
            self.node.type_id_for_layout()
        }
    }

    unsafe fn borrow_layout_data_unchecked(&self) -> Option<*const PrivateLayoutData> {
        self.get_jsmanaged().get_style_and_layout_data().map(|opaque| {
            let container = *opaque.ptr as NonOpaqueStyleAndLayoutData;
            &(*(*container).as_unsafe_cell().get()) as *const PrivateLayoutData
        })
    }

    fn borrow_layout_data(&self) -> Option<Ref<PrivateLayoutData>> {
        unsafe {
            self.get_jsmanaged().get_style_and_layout_data().map(|opaque| {
                let container = *opaque.ptr as NonOpaqueStyleAndLayoutData;
                (*container).borrow()
            })
        }
    }

    fn mutate_layout_data(&self) -> Option<RefMut<PrivateLayoutData>> {
        unsafe {
            self.get_jsmanaged().get_style_and_layout_data().map(|opaque| {
                let container = *opaque.ptr as NonOpaqueStyleAndLayoutData;
                (*container).borrow_mut()
            })
        }
    }
}


impl<'ln> ServoLayoutNode<'ln> {
    fn dump_indent(self, indent: u32) {
        let mut s = String::new();
        for _ in 0..indent {
            s.push_str("  ");
        }

        s.push_str(&self.debug_str());
        println!("{}", s);

        for kid in self.children() {
            kid.dump_indent(indent + 1);
        }
    }

    fn debug_str(self) -> String {
        format!("{:?}: changed={} dirty={} dirty_descendants={}",
                self.type_id(), self.has_changed(), self.is_dirty(), self.has_dirty_descendants())
    }

    pub fn flow_debug_id(self) -> usize {
        self.borrow_layout_data().map_or(0, |d| d.flow_construction_result.debug_id())
    }

    /// Returns the interior of this node as a `LayoutJS`. This is highly unsafe for layout to
    /// call and as such is marked `unsafe`.
    unsafe fn get_jsmanaged(&self) -> &LayoutJS<Node> {
        &self.node
    }
}

// A wrapper around documents that ensures ayout can only ever access safe properties.
#[derive(Copy, Clone)]
pub struct ServoLayoutDocument<'ld> {
    document: LayoutJS<Document>,
    chain: PhantomData<&'ld ()>,
}

impl<'ld> TDocument for ServoLayoutDocument<'ld> {
    type ConcreteNode = ServoLayoutNode<'ld>;
    type ConcreteElement = ServoLayoutElement<'ld>;

    fn as_node(&self) -> ServoLayoutNode<'ld> {
        ServoLayoutNode::from_layout_js(self.document.upcast())
    }

    fn root_node(&self) -> Option<ServoLayoutNode<'ld>> {
        self.as_node().children().find(ServoLayoutNode::is_element)
    }

    fn drain_modified_elements(&self) -> Vec<(ServoLayoutElement<'ld>, ElementSnapshot)> {
        let elements =  unsafe { self.document.drain_modified_elements() };
        elements.into_iter().map(|(el, snapshot)| (ServoLayoutElement::from_layout_js(el), snapshot)).collect()
    }
}

impl<'ld> ServoLayoutDocument<'ld> {
    fn from_layout_js(doc: LayoutJS<Document>) -> ServoLayoutDocument<'ld> {
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

impl<'le> TElement for ServoLayoutElement<'le> {
    type ConcreteNode = ServoLayoutNode<'le>;
    type ConcreteDocument = ServoLayoutDocument<'le>;

    fn as_node(&self) -> ServoLayoutNode<'le> {
        ServoLayoutNode::from_layout_js(self.element.upcast())
    }

    fn style_attribute(&self) -> &Option<PropertyDeclarationBlock> {
        unsafe {
            &*self.element.style_attribute()
        }
    }

    fn get_state(&self) -> ElementState {
        self.element.get_state_for_layout()
    }

    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, hints: &mut V)
        where V: VecLike<DeclarationBlock<Vec<PropertyDeclaration>>>
    {
        unsafe {
            self.element.synthesize_presentational_hints_for_legacy_attributes(hints);
        }
    }

    #[inline]
    fn get_attr<'a>(&'a self, namespace: &Namespace, name: &Atom) -> Option<&'a str> {
        unsafe {
            (*self.element.unsafe_get()).get_attr_val_for_layout(namespace, name)
        }
    }

    #[inline]
    fn get_attrs<'a>(&'a self, name: &Atom) -> Vec<&'a str> {
        unsafe {
            (*self.element.unsafe_get()).get_attr_vals_for_layout(name)
        }
    }
}


impl<'le> ServoLayoutElement<'le> {
    fn from_layout_js(el: LayoutJS<Element>) -> ServoLayoutElement<'le> {
        ServoLayoutElement {
            element: el,
            chain: PhantomData,
        }
    }
}

fn as_element<'le>(node: LayoutJS<Node>) -> Option<ServoLayoutElement<'le>> {
    node.downcast().map(ServoLayoutElement::from_layout_js)
}

impl<'le> ::selectors::Element for ServoLayoutElement<'le> {
    type Impl = ServoSelectorImpl;

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

    fn is_root(&self) -> bool {
        match self.as_node().parent_node() {
            None => false,
            Some(node) => {
                match node.type_id() {
                    NodeTypeId::Document(_) => true,
                    _ => false
                }
            },
        }
    }

    fn is_empty(&self) -> bool {
        self.as_node().children().all(|node| match node.type_id() {
            NodeTypeId::Element(..) => false,
            NodeTypeId::CharacterData(CharacterDataTypeId::Text) => unsafe {
                node.node.downcast().unwrap().data_for_layout().is_empty()
            },
            _ => true
        })
    }

    #[inline]
    fn get_local_name(&self) -> &Atom {
        self.element.local_name()
    }

    #[inline]
    fn get_namespace(&self) -> &Namespace {
        self.element.namespace()
    }

    fn match_non_ts_pseudo_class(&self, pseudo_class: NonTSPseudoClass) -> bool {
        match pseudo_class {
            // https://github.com/servo/servo/issues/8718
            NonTSPseudoClass::Link |
            NonTSPseudoClass::AnyLink => unsafe {
                match self.as_node().type_id() {
                    // https://html.spec.whatwg.org/multipage/#selector-link
                    NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) |
                    NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement)) |
                    NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) =>
                        (*self.element.unsafe_get()).get_attr_val_for_layout(&ns!(), &atom!("href")).is_some(),
                    _ => false,
                }
            },
            NonTSPseudoClass::Visited => false,

            NonTSPseudoClass::ServoNonZeroBorder => unsafe {
                match (*self.element.unsafe_get()).get_attr_for_layout(&ns!(), &atom!("border")) {
                    None | Some(&AttrValue::UInt(_, 0)) => false,
                    _ => true,
                }
            },

            NonTSPseudoClass::Active |
            NonTSPseudoClass::Focus |
            NonTSPseudoClass::Hover |
            NonTSPseudoClass::Enabled |
            NonTSPseudoClass::Disabled |
            NonTSPseudoClass::Checked |
            NonTSPseudoClass::Indeterminate =>
                self.element.get_state_for_layout().contains(pseudo_class.state_flag())
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

    fn match_attr<F>(&self, attr: &AttrSelector, test: F) -> bool where F: Fn(&str) -> bool {
        let name = if self.is_html_element_in_html_document() {
            &attr.lower_name
        } else {
            &attr.name
        };
        match attr.namespace {
            NamespaceConstraint::Specific(ref ns) => {
                self.get_attr(ns, name).map_or(false, |attr| test(attr))
            },
            NamespaceConstraint::Any => {
                self.get_attrs(name).iter().any(|attr| test(*attr))
            }
        }
    }

    fn is_html_element_in_html_document(&self) -> bool {
        unsafe {
            self.element.html_element_in_html_document_for_layout()
        }
    }

    fn insert_flags(&self, flags: ElementFlags) {
        self.element.insert_atomic_flags(flags);
    }
}

#[derive(Copy, PartialEq, Clone)]
pub enum PseudoElementType<T> {
    Normal,
    Before(T),
    After(T),
    DetailsSummary(T),
    DetailsContent(T),
}

impl<T> PseudoElementType<T> {
    pub fn is_before(&self) -> bool {
        match *self {
            PseudoElementType::Before(_) => true,
            _ => false,
        }
    }

    pub fn is_before_or_after(&self) -> bool {
        match *self {
            PseudoElementType::Before(_) | PseudoElementType::After(_) => true,
            _ => false,
        }
    }

    pub fn strip(&self) -> PseudoElementType<()> {
        match *self {
            PseudoElementType::Normal => PseudoElementType::Normal,
            PseudoElementType::Before(_) => PseudoElementType::Before(()),
            PseudoElementType::After(_) => PseudoElementType::After(()),
            PseudoElementType::DetailsSummary(_) => PseudoElementType::DetailsSummary(()),
            PseudoElementType::DetailsContent(_) => PseudoElementType::DetailsContent(()),
        }
    }
}

/// A thread-safe version of `LayoutNode`, used during flow construction. This type of layout
/// node does not allow any parents or siblings of nodes to be accessed, to avoid races.

pub trait ThreadSafeLayoutNode : Clone + Copy + Sized + PartialEq {
    type ConcreteThreadSafeLayoutElement: ThreadSafeLayoutElement<ConcreteThreadSafeLayoutNode = Self>;
    type ChildrenIterator: Iterator<Item = Self> + Sized;

    /// Creates a new `ThreadSafeLayoutNode` for the same `LayoutNode`
    /// with a different pseudo-element type.
    fn with_pseudo(&self, pseudo: PseudoElementType<display::T>) -> Self;

    /// Converts self into an `OpaqueNode`.
    fn opaque(&self) -> OpaqueNode;

    /// Returns the type ID of this node.
    /// Returns `None` if this is a pseudo-element; otherwise, returns `Some`.
    fn type_id(&self) -> Option<NodeTypeId>;

    fn debug_id(self) -> usize;

    fn flow_debug_id(self) -> usize;

    /// Returns an iterator over this node's children.
    fn children(&self) -> Self::ChildrenIterator;

    #[inline]
    fn is_element(&self) -> bool { if let Some(NodeTypeId::Element(_)) = self.type_id() { true } else { false } }

    /// If this is an element, accesses the element data. Fails if this is not an element node.
    #[inline]
    fn as_element(&self) -> Self::ConcreteThreadSafeLayoutElement;

    #[inline]
    fn get_pseudo_element_type(&self) -> PseudoElementType<display::T>;

    #[inline]
    fn get_before_pseudo(&self) -> Option<Self> {
        self.borrow_layout_data().unwrap()
            .style_data.per_pseudo
            .get(&PseudoElement::Before)
            .map(|style| {
                self.with_pseudo(PseudoElementType::Before(style.get_box().display))
            })
    }

    #[inline]
    fn get_after_pseudo(&self) -> Option<Self> {
        self.borrow_layout_data().unwrap()
            .style_data.per_pseudo
            .get(&PseudoElement::After)
            .map(|style| {
                self.with_pseudo(PseudoElementType::After(style.get_box().display))
            })
    }

    // TODO(emilio): Since the ::-details-* pseudos are internal, just affecting one element, and
    // only changing `display` property when the element `open` attribute changes, this should be
    // eligible for not being cascaded eagerly, reading the display property from layout instead.
    #[inline]
    fn get_details_summary_pseudo(&self) -> Option<Self> {
        if self.is_element() &&
           self.as_element().get_local_name() == &atom!("details") &&
           self.as_element().get_namespace() == &ns!(html) {
            self.borrow_layout_data().unwrap()
                .style_data.per_pseudo
                .get(&PseudoElement::DetailsSummary)
                .map(|style| {
                    self.with_pseudo(PseudoElementType::DetailsSummary(style.get_box().display))
                })
        } else {
            None
        }
    }

    #[inline]
    fn get_details_content_pseudo(&self) -> Option<Self> {
        if self.is_element() &&
           self.as_element().get_local_name() == &atom!("details") &&
           self.as_element().get_namespace() == &ns!(html) {
            self.borrow_layout_data().unwrap()
                .style_data.per_pseudo
                .get(&PseudoElement::DetailsContent)
                .map(|style| {
                    self.with_pseudo(PseudoElementType::DetailsContent(style.get_box().display))
                })
        } else {
            None
        }
    }

    /// Borrows the layout data immutably. Fails on a conflicting borrow.
    ///
    /// TODO(pcwalton): Make this private. It will let us avoid borrow flag checks in some cases.
    #[inline(always)]
    fn borrow_layout_data(&self) -> Option<Ref<PrivateLayoutData>>;

    /// Borrows the layout data mutably. Fails on a conflicting borrow.
    ///
    /// TODO(pcwalton): Make this private. It will let us avoid borrow flag checks in some cases.
    #[inline(always)]
    fn mutate_layout_data(&self) -> Option<RefMut<PrivateLayoutData>>;

    /// Returns the style results for the given node. If CSS selector matching
    /// has not yet been performed, fails.
    ///
    /// Unlike the version on TNode, this handles pseudo-elements.
    #[inline]
    fn style(&self) -> Ref<Arc<ServoComputedValues>> {
        Ref::map(self.borrow_layout_data().unwrap(), |data| {
            let style = match self.get_pseudo_element_type() {
                PseudoElementType::Before(_) => data.style_data.per_pseudo.get(&PseudoElement::Before),
                PseudoElementType::After(_) => data.style_data.per_pseudo.get(&PseudoElement::After),
                PseudoElementType::DetailsSummary(_) => data.style_data.per_pseudo.get(&PseudoElement::DetailsSummary),
                PseudoElementType::DetailsContent(_) => data.style_data.per_pseudo.get(&PseudoElement::DetailsContent),
                PseudoElementType::Normal => data.style_data.style.as_ref(),
            };
            style.unwrap()
        })
    }

    #[inline]
    fn selected_style(&self) -> Ref<Arc<ServoComputedValues>> {
        Ref::map(self.borrow_layout_data().unwrap(), |data| {
            data.style_data.per_pseudo.get(&PseudoElement::Selection).unwrap_or(data.style_data.style.as_ref().unwrap())
        })
    }

    /// Removes the style from this node.
    ///
    /// Unlike the version on TNode, this handles pseudo-elements.
    fn unstyle(self) {
        let mut data = self.mutate_layout_data().unwrap();

        match self.get_pseudo_element_type() {
            PseudoElementType::Before(_) => {
                data.style_data.per_pseudo.remove(&PseudoElement::Before);
            }
            PseudoElementType::After(_) => {
                data.style_data.per_pseudo.remove(&PseudoElement::After);
            }
            PseudoElementType::DetailsSummary(_) => {
                data.style_data.per_pseudo.remove(&PseudoElement::DetailsSummary);
            }
            PseudoElementType::DetailsContent(_) => {
                data.style_data.per_pseudo.remove(&PseudoElement::DetailsContent);
            }

            PseudoElementType::Normal => {
                data.style_data.style = None;
            }
        };
    }

    fn is_ignorable_whitespace(&self) -> bool;

    fn restyle_damage(self) -> RestyleDamage;

    fn set_restyle_damage(self, damage: RestyleDamage);

    /// Returns the layout data flags for this node.
    fn flags(self) -> LayoutDataFlags;

    /// Adds the given flags to this node.
    fn insert_flags(self, new_flags: LayoutDataFlags) {
        self.mutate_layout_data().unwrap().flags.insert(new_flags);
    }

    /// Removes the given flags from this node.
    fn remove_flags(self, flags: LayoutDataFlags) {
        self.mutate_layout_data().unwrap().flags.remove(flags);
    }

    /// Returns true if this node contributes content. This is used in the implementation of
    /// `empty_cells` per CSS 2.1 § 17.6.1.1.
    fn is_content(&self) -> bool {
        match self.type_id() {
            Some(NodeTypeId::Element(..)) | Some(NodeTypeId::CharacterData(CharacterDataTypeId::Text)) => true,
            _ => false
        }
    }

    fn can_be_fragmented(&self) -> bool;

    /// If this is a text node, generated content, or a form element, copies out
    /// its content. Otherwise, panics.
    ///
    /// FIXME(pcwalton): This might have too much copying and/or allocation. Profile this.
    fn text_content(&self) -> TextContent;

    /// If the insertion point is within this node, returns it. Otherwise, returns `None`.
    fn selection(&self) -> Option<Range<CharIndex>>;

    /// If this is an image element, returns its URL. If this is not an image element, fails.
    ///
    /// FIXME(pcwalton): Don't copy URLs.
    fn image_url(&self) -> Option<Url>;

    fn canvas_data(&self) -> Option<HTMLCanvasData>;

    /// If this node is an iframe element, returns its pipeline ID. If this node is
    /// not an iframe element, fails.
    fn iframe_pipeline_id(&self) -> PipelineId;

    fn get_colspan(&self) -> u32;
}

// This trait is only public so that it can be implemented by the gecko wrapper.
// It can be used to violate thread-safety, so don't use it elsewhere in layout!
pub trait DangerousThreadSafeLayoutNode : ThreadSafeLayoutNode {
    unsafe fn dangerous_first_child(&self) -> Option<Self>;
    unsafe fn dangerous_next_sibling(&self) -> Option<Self>;
}

pub trait ThreadSafeLayoutElement: Clone + Copy + Sized {
    type ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode<ConcreteThreadSafeLayoutElement = Self>;

    #[inline]
    fn get_attr<'a>(&'a self, namespace: &Namespace, name: &Atom) -> Option<&'a str>;

    #[inline]
    fn get_local_name(&self) -> &Atom;

    #[inline]
    fn get_namespace(&self) -> &Namespace;
}

#[derive(Copy, Clone)]
pub struct ServoThreadSafeLayoutNode<'ln> {
    /// The wrapped node.
    node: ServoLayoutNode<'ln>,

    pseudo: PseudoElementType<display::T>,
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

impl<'ln> ThreadSafeLayoutNode for ServoThreadSafeLayoutNode<'ln> {
    type ConcreteThreadSafeLayoutElement = ServoThreadSafeLayoutElement<'ln>;
    type ChildrenIterator = ThreadSafeLayoutNodeChildrenIterator<Self>;

    fn with_pseudo(&self, pseudo: PseudoElementType<display::T>) -> ServoThreadSafeLayoutNode<'ln> {
        ServoThreadSafeLayoutNode {
            node: self.node.clone(),
            pseudo: pseudo,
        }
    }

    fn opaque(&self) -> OpaqueNode {
        OpaqueNodeMethods::from_jsmanaged(unsafe { self.get_jsmanaged() })
    }

    fn type_id(&self) -> Option<NodeTypeId> {
        if self.pseudo != PseudoElementType::Normal {
            return None
        }

        Some(self.node.type_id())
    }

    fn debug_id(self) -> usize {
        self.node.debug_id()
    }

    fn flow_debug_id(self) -> usize {
        self.node.flow_debug_id()
    }

    fn children(&self) -> Self::ChildrenIterator {
        ThreadSafeLayoutNodeChildrenIterator::new(*self)
    }

    fn as_element(&self) -> ServoThreadSafeLayoutElement<'ln> {
        unsafe {
            let element = match self.get_jsmanaged().downcast() {
                Some(e) => e.unsafe_get(),
                None => panic!("not an element")
            };
            // FIXME(pcwalton): Workaround until Rust gets multiple lifetime parameters on
            // implementations.
            ServoThreadSafeLayoutElement {
                element: &*element,
            }
        }
    }

    fn get_pseudo_element_type(&self) -> PseudoElementType<display::T> {
        self.pseudo
    }

    fn borrow_layout_data(&self) -> Option<Ref<PrivateLayoutData>> {
        self.node.borrow_layout_data()
    }

    fn mutate_layout_data(&self) -> Option<RefMut<PrivateLayoutData>> {
        self.node.mutate_layout_data()
    }

    fn is_ignorable_whitespace(&self) -> bool {
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
            !self.style().get_inheritedtext().white_space.preserve_newlines()
        }
    }

    fn restyle_damage(self) -> RestyleDamage {
        self.node.restyle_damage()
    }

    fn set_restyle_damage(self, damage: RestyleDamage) {
        self.node.set_restyle_damage(damage)
    }

    fn flags(self) -> LayoutDataFlags {
        unsafe {
            (*self.node.borrow_layout_data_unchecked().unwrap()).flags
        }
    }

    fn can_be_fragmented(&self) -> bool {
        self.node.can_be_fragmented()
    }

    fn text_content(&self) -> TextContent {
        if self.pseudo.is_before_or_after() {
            let data = &self.borrow_layout_data().unwrap().style_data;

            let style = if self.pseudo.is_before() {
                data.per_pseudo.get(&PseudoElement::Before).unwrap()
            } else {
                data.per_pseudo.get(&PseudoElement::After).unwrap()
            };

            return match style.as_ref().get_counters().content {
                content::T::Content(ref value) if !value.is_empty() => {
                    TextContent::GeneratedContent((*value).clone())
                }
                _ => TextContent::GeneratedContent(vec![]),
            };
        }

        let this = unsafe { self.get_jsmanaged() };
        if let Some(text) = this.downcast::<Text>() {
            let data = unsafe {
                text.upcast().data_for_layout().to_owned()
            };
            return TextContent::Text(data);
        }
        if let Some(input) = this.downcast::<HTMLInputElement>() {
            let data = unsafe { input.get_value_for_layout() };
            return TextContent::Text(data);
        }
        if let Some(area) = this.downcast::<HTMLTextAreaElement>() {
            let data = unsafe { area.get_value_for_layout() };
            return TextContent::Text(data);
        }

        panic!("not text!")
    }

    fn selection(&self) -> Option<Range<CharIndex>> {
        let this = unsafe {
            self.get_jsmanaged()
        };

        if let Some(area) = this.downcast::<HTMLTextAreaElement>() {
            if let Some(selection) = unsafe { area.get_absolute_selection_for_layout() } {
                let text = unsafe { area.get_value_for_layout() };
                let begin_byte = selection.begin();
                let begin = search_index(begin_byte, text.char_indices());
                let length = search_index(selection.length(), text[begin_byte..].char_indices());
                return Some(Range::new(CharIndex(begin), CharIndex(length)));
            }
        }
        if let Some(input) = this.downcast::<HTMLInputElement>() {
            if let Some(selection) = unsafe { input.get_selection_for_layout() } {
                return Some(Range::new(CharIndex(selection.begin()),
                                       CharIndex(selection.length())));
            }
        }
        None
    }

    fn image_url(&self) -> Option<Url> {
        unsafe {
            self.get_jsmanaged().downcast()
                .expect("not an image!")
                .image_url()
        }
    }

    fn canvas_data(&self) -> Option<HTMLCanvasData> {
        unsafe {
            let canvas_element = self.get_jsmanaged().downcast();
            canvas_element.map(|canvas| canvas.data())
        }
    }

    fn iframe_pipeline_id(&self) -> PipelineId {
        use script::dom::htmliframeelement::HTMLIFrameElementLayoutMethods;
        unsafe {
            let iframe_element = self.get_jsmanaged().downcast::<HTMLIFrameElement>()
                .expect("not an iframe element!");
            iframe_element.pipeline_id().unwrap()
        }
    }

    fn get_colspan(&self) -> u32 {
        unsafe {
            self.get_jsmanaged().downcast::<Element>().unwrap().get_colspan()
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
        match self.parent_node.get_pseudo_element_type() {
            PseudoElementType::Before(_) | PseudoElementType::After(_) => None,

            PseudoElementType::DetailsSummary(_) => {
                let mut current_node = self.current_node.clone();
                loop {
                    let next_node = if let Some(ref node) = current_node {
                        if node.is_element() &&
                                node.as_element().get_local_name() == &atom!("summary") &&
                                node.as_element().get_namespace() == &ns!(html) {
                            self.current_node = None;
                            return Some(node.clone());
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
                       node.as_element().get_local_name() == &atom!("summary") &&
                       node.as_element().get_namespace() == &ns!(html) {
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
                            let first = self.parent_node.get_details_summary_pseudo().or_else(|| unsafe {
                                self.parent_node.dangerous_first_child()
                            });
                            match first {
                                Some(first) => Some(first),
                                None => self.parent_node.get_after_pseudo(),
                            }
                        },
                        PseudoElementType::Normal => {
                            match unsafe { node.dangerous_next_sibling() } {
                                Some(next) => Some(next),
                                None => self.parent_node.get_after_pseudo(),
                            }
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

/// A wrapper around elements that ensures layout can only ever access safe properties and cannot
/// race on elements.
#[derive(Copy, Clone)]
pub struct ServoThreadSafeLayoutElement<'le> {
    element: &'le Element,
}

impl<'le> ThreadSafeLayoutElement for ServoThreadSafeLayoutElement<'le> {
    type ConcreteThreadSafeLayoutNode = ServoThreadSafeLayoutNode<'le>;

    fn get_attr<'a>(&'a self, namespace: &Namespace, name: &Atom) -> Option<&'a str> {
        unsafe {
            self.element.get_attr_val_for_layout(namespace, name)
        }
    }

    #[inline]
    fn get_local_name(&self) -> &Atom {
        self.element.local_name()
    }

    #[inline]
    fn get_namespace(&self) -> &Namespace {
        self.element.namespace()
    }
}

pub enum TextContent {
    Text(String),
    GeneratedContent(Vec<ContentItem>),
}

impl TextContent {
    pub fn is_empty(&self) -> bool {
        match *self {
            TextContent::Text(_) => false,
            TextContent::GeneratedContent(ref content) => content.is_empty(),
        }
    }
}
