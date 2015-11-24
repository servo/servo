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
//! will race and cause spurious task failure. (Note that I do not believe these races are
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

use data::{LayoutDataFlags, LayoutDataWrapper, PrivateLayoutData};
use gfx::display_list::OpaqueNode;
use gfx::text::glyph::CharIndex;
use incremental::RestyleDamage;
use msg::constellation_msg::PipelineId;
use opaque_node::OpaqueNodeMethods;
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
use script::dom::node::{HAS_CHANGED, HAS_DIRTY_DESCENDANTS, IS_DIRTY};
use script::dom::node::{LayoutNodeHelpers, Node, SharedLayoutData};
use script::dom::text::Text;
use script::layout_interface::TrustedNodeAddress;
use selectors::matching::DeclarationBlock;
use selectors::parser::{AttrSelector, NamespaceConstraint};
use selectors::states::*;
use smallvec::VecLike;
use std::borrow::ToOwned;
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;
use string_cache::{Atom, Namespace};
use style::computed_values::content::ContentItem;
use style::computed_values::{content, display};
use style::node::TElementAttributes;
use style::properties::ComputedValues;
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock};
use style::restyle_hints::{ElementSnapshot, RESTYLE_DESCENDANTS, RESTYLE_LATER_SIBLINGS, RESTYLE_SELF, RestyleHint};
use url::Url;
use util::str::{is_whitespace, search_index};

/// A wrapper so that layout can access only the methods that it should have access to. Layout must
/// only ever see these and must never see instances of `LayoutJS`.

pub trait LayoutNode<'ln> : Sized + Copy + Clone {
    type ConcreteLayoutElement: LayoutElement<'ln>;
    type ConcreteLayoutDocument: LayoutDocument<'ln>;

    /// Returns the type ID of this node.
    fn type_id(&self) -> NodeTypeId;

    fn is_element(&self) -> bool;

    fn dump(self);

    fn traverse_preorder(self) -> LayoutTreeIterator<'ln, Self>;

    /// Returns an iterator over this node's children.
    fn children(self) -> LayoutNodeChildrenIterator<'ln, Self>;

    fn rev_children(self) -> LayoutNodeReverseChildrenIterator<'ln, Self>;

    /// Converts self into an `OpaqueNode`.
    fn opaque(&self) -> OpaqueNode;

    /// Resets layout data and styles for the node.
    ///
    /// FIXME(pcwalton): Do this as part of fragment building instead of in a traversal.
    fn initialize_layout_data(self);

    /// While doing a reflow, the node at the root has no parent, as far as we're
    /// concerned. This method returns `None` at the reflow root.
    fn layout_parent_node(self, reflow_root: OpaqueNode) -> Option<Self>;

    fn debug_id(self) -> usize;

    fn as_element(&self) -> Option<Self::ConcreteLayoutElement>;

    fn as_document(&self) -> Option<Self::ConcreteLayoutDocument>;

    fn children_count(&self) -> u32;

    fn has_changed(&self) -> bool;

    unsafe fn set_changed(&self, value: bool);

    fn is_dirty(&self) -> bool;

    unsafe fn set_dirty(&self, value: bool);

    fn has_dirty_descendants(&self) -> bool;

    unsafe fn set_dirty_descendants(&self, value: bool);

    fn dirty_self(&self) {
        unsafe {
            self.set_dirty(true);
            self.set_dirty_descendants(true);
        }
    }

    fn dirty_descendants(&self) {
        for ref child in self.children() {
            child.dirty_self();
            child.dirty_descendants();
        }
    }

    /// Borrows the layout data without checks.
    #[inline(always)]
    unsafe fn borrow_layout_data_unchecked(&self) -> *const Option<LayoutDataWrapper>;

    /// Borrows the layout data immutably. Fails on a conflicting borrow.
    #[inline(always)]
    fn borrow_layout_data(&self) -> Ref<Option<LayoutDataWrapper>>;

    /// Borrows the layout data mutably. Fails on a conflicting borrow.
    #[inline(always)]
    fn mutate_layout_data(&self) -> RefMut<Option<LayoutDataWrapper>>;

    fn parent_node(&self) -> Option<Self>;

    fn first_child(&self) -> Option<Self>;

    fn last_child(&self) -> Option<Self>;

    fn prev_sibling(&self) -> Option<Self>;

    fn next_sibling(&self) -> Option<Self>;
}

pub trait LayoutDocument<'ld> : Sized + Copy + Clone {
    type ConcreteLayoutNode: LayoutNode<'ld>;
    type ConcreteLayoutElement: LayoutElement<'ld>;

    fn as_node(&self) -> Self::ConcreteLayoutNode;

    fn root_node(&self) -> Option<Self::ConcreteLayoutNode>;

    fn drain_modified_elements(&self) -> Vec<(Self::ConcreteLayoutElement, ElementSnapshot)>;
}

pub trait LayoutElement<'le> : Sized + Copy + Clone + ::selectors::Element + TElementAttributes {
    type ConcreteLayoutNode: LayoutNode<'le>;
    type ConcreteLayoutDocument: LayoutDocument<'le>;

    fn as_node(&self) -> Self::ConcreteLayoutNode;

    fn style_attribute(&self) -> &'le Option<PropertyDeclarationBlock>;

    fn get_state(&self) -> ElementState;

    /// Properly marks nodes as dirty in response to restyle hints.
    fn note_restyle_hint(&self, mut hint: RestyleHint) {
        // Bail early if there's no restyling to do.
        if hint.is_empty() {
            return;
        }

        // If the restyle hint is non-empty, we need to restyle either this element
        // or one of its siblings. Mark our ancestor chain as having dirty descendants.
        let node = self.as_node();
        let mut curr = node;
        while let Some(parent) = curr.parent_node() {
            if parent.has_dirty_descendants() { break }
            unsafe { parent.set_dirty_descendants(true); }
            curr = parent;
        }

        // Process hints.
        if hint.contains(RESTYLE_SELF) {
            node.dirty_self();

            // FIXME(bholley, #8438): We currently need to RESTYLE_DESCENDANTS in the
            // RESTYLE_SELF case in order to make sure "inherit" style structs propagate
            // properly. See the explanation in the github issue.
            hint.insert(RESTYLE_DESCENDANTS);
        }
        if hint.contains(RESTYLE_DESCENDANTS) {
            unsafe { node.set_dirty_descendants(true); }
            node.dirty_descendants();
        }
        if hint.contains(RESTYLE_LATER_SIBLINGS) {
            let mut next = ::selectors::Element::next_sibling_element(self);
            while let Some(sib) = next {
                let sib_node = sib.as_node();
                sib_node.dirty_self();
                sib_node.dirty_descendants();
                next = ::selectors::Element::next_sibling_element(&sib);
            }
        }
    }
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

impl<'ln> LayoutNode<'ln> for ServoLayoutNode<'ln> {
    type ConcreteLayoutElement = ServoLayoutElement<'ln>;
    type ConcreteLayoutDocument = ServoLayoutDocument<'ln>;

    fn type_id(&self) -> NodeTypeId {
        unsafe {
            self.node.type_id_for_layout()
        }
    }

    fn is_element(&self) -> bool {
        unsafe {
            self.node.is_element_for_layout()
        }
    }

    fn dump(self) {
        self.dump_indent(0);
    }

    fn traverse_preorder(self) -> LayoutTreeIterator<'ln, Self> {
        LayoutTreeIterator::new(self)
    }

    fn children(self) -> LayoutNodeChildrenIterator<'ln, Self> {
        LayoutNodeChildrenIterator {
            current: self.first_child(),
            phantom: PhantomData,
        }
    }

    fn rev_children(self) -> LayoutNodeReverseChildrenIterator<'ln, Self> {
        LayoutNodeReverseChildrenIterator {
            current: self.last_child(),
            phantom: PhantomData,
        }
    }

    fn opaque(&self) -> OpaqueNode {
        OpaqueNodeMethods::from_jsmanaged(unsafe { self.get_jsmanaged() })
    }

    fn initialize_layout_data(self) {
        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref {
            None => {
                *layout_data_ref = Some(LayoutDataWrapper {
                    shared_data: SharedLayoutData { style: None },
                    data: box PrivateLayoutData::new(),
                });
            }
            Some(_) => {}
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
        self.node.downcast().map(|document| ServoLayoutDocument::from_layout_js(document))
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

    unsafe fn borrow_layout_data_unchecked(&self) -> *const Option<LayoutDataWrapper> {
        mem::transmute(self.get_jsmanaged().layout_data_unchecked())
    }

    fn borrow_layout_data(&self) -> Ref<Option<LayoutDataWrapper>> {
        unsafe {
            mem::transmute(self.get_jsmanaged().layout_data())
        }
    }

    fn mutate_layout_data(&self) -> RefMut<Option<LayoutDataWrapper>> {
        unsafe {
            mem::transmute(self.get_jsmanaged().layout_data_mut())
        }
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
        let layout_data_ref = self.borrow_layout_data();
        match *layout_data_ref {
            None => 0,
            Some(ref layout_data) => layout_data.data.flow_construction_result.debug_id()
        }
    }

    /// Returns the interior of this node as a `LayoutJS`. This is highly unsafe for layout to
    /// call and as such is marked `unsafe`.
    unsafe fn get_jsmanaged(&self) -> &LayoutJS<Node> {
        &self.node
    }
}

pub struct LayoutNodeChildrenIterator<'a, ConcreteLayoutNode> where ConcreteLayoutNode: LayoutNode<'a> {
    current: Option<ConcreteLayoutNode>,
    // Satisfy the compiler about the unused lifetime.
    phantom: PhantomData<&'a ()>,
}

impl<'a, ConcreteLayoutNode> Iterator for LayoutNodeChildrenIterator<'a, ConcreteLayoutNode>
                                      where ConcreteLayoutNode: LayoutNode<'a> {
    type Item = ConcreteLayoutNode;
    fn next(&mut self) -> Option<ConcreteLayoutNode> {
        let node = self.current;
        self.current = node.and_then(|node| node.next_sibling());
        node
    }
}

pub struct LayoutNodeReverseChildrenIterator<'a, ConcreteLayoutNode> where ConcreteLayoutNode: LayoutNode<'a> {
    current: Option<ConcreteLayoutNode>,
    // Satisfy the compiler about the unused lifetime.
    phantom: PhantomData<&'a ()>,
}

impl<'a, ConcreteLayoutNode> Iterator for LayoutNodeReverseChildrenIterator<'a, ConcreteLayoutNode>
                                      where ConcreteLayoutNode: LayoutNode<'a> {
    type Item = ConcreteLayoutNode;
    fn next(&mut self) -> Option<ConcreteLayoutNode> {
        let node = self.current;
        self.current = node.and_then(|node| node.prev_sibling());
        node
    }
}

pub struct LayoutTreeIterator<'a, ConcreteLayoutNode> where ConcreteLayoutNode: LayoutNode<'a> {
    stack: Vec<ConcreteLayoutNode>,
    // Satisfy the compiler about the unused lifetime.
    phantom: PhantomData<&'a ()>,
}

impl<'a, ConcreteLayoutNode> LayoutTreeIterator<'a, ConcreteLayoutNode> where ConcreteLayoutNode: LayoutNode<'a> {
    fn new(root: ConcreteLayoutNode) -> LayoutTreeIterator<'a, ConcreteLayoutNode> {
        let mut stack = vec!();
        stack.push(root);
        LayoutTreeIterator {
            stack: stack,
            phantom: PhantomData,
        }
    }
}

impl<'a, ConcreteLayoutNode> Iterator for LayoutTreeIterator<'a, ConcreteLayoutNode>
                                      where ConcreteLayoutNode: LayoutNode<'a> {
    type Item = ConcreteLayoutNode;
    fn next(&mut self) -> Option<ConcreteLayoutNode> {
        let ret = self.stack.pop();
        ret.map(|node| self.stack.extend(node.rev_children()));
        ret
    }
}

// A wrapper around documents that ensures ayout can only ever access safe properties.
#[derive(Copy, Clone)]
pub struct ServoLayoutDocument<'ld> {
    document: LayoutJS<Document>,
    chain: PhantomData<&'ld ()>,
}

impl<'ld> LayoutDocument<'ld> for ServoLayoutDocument<'ld> {
    type ConcreteLayoutNode = ServoLayoutNode<'ld>;
    type ConcreteLayoutElement = ServoLayoutElement<'ld>;

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

impl<'le> LayoutElement<'le> for ServoLayoutElement<'le> {
    type ConcreteLayoutNode = ServoLayoutNode<'le>;
    type ConcreteLayoutDocument = ServoLayoutDocument<'le>;

    fn as_node(&self) -> ServoLayoutNode<'le> {
        ServoLayoutNode::from_layout_js(self.element.upcast())
    }

    fn style_attribute(&self) -> &'le Option<PropertyDeclarationBlock> {
        unsafe {
            &*self.element.style_attribute()
        }
    }

    fn get_state(&self) -> ElementState {
        self.element.get_state_for_layout()
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
    node.downcast().map(|element| ServoLayoutElement::from_layout_js(element))
}

macro_rules! state_getter {
    ($(
        $(#[$Flag_attr: meta])*
        state $css: expr => $variant: ident / $method: ident /
        $flag: ident = $value: expr,
    )+) => {
        $( fn $method(&self) -> bool { self.element.get_state_for_layout().contains($flag) } )+
    }
}

impl<'le> ::selectors::Element for ServoLayoutElement<'le> {
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
            Some(node) => node.type_id() == NodeTypeId::Document,
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

    fn is_link(&self) -> bool {
        // FIXME: This is HTML only.
        let node = self.as_node();
        match node.type_id() {
            // https://html.spec.whatwg.org/multipage/#selector-link
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) => {
                unsafe {
                    (*self.element.unsafe_get()).get_attr_val_for_layout(&ns!(""), &atom!("href")).is_some()
                }
            }
            _ => false,
        }
    }

    #[inline]
    fn is_unvisited_link(&self) -> bool {
        self.is_link()
    }

    #[inline]
    fn is_visited_link(&self) -> bool {
        false
    }

    state_pseudo_classes!(state_getter);

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
            match self.element.get_classes_for_layout() {
                None => {}
                Some(ref classes) => {
                    for class in *classes {
                        callback(class)
                    }
                }
            }
        }
    }

    #[inline]
    fn has_servo_nonzero_border(&self) -> bool {
        unsafe {
            match (*self.element.unsafe_get()).get_attr_for_layout(&ns!(""), &atom!("border")) {
                None | Some(&AttrValue::UInt(_, 0)) => false,
                _ => true,
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
}

impl<'le> TElementAttributes for ServoLayoutElement<'le> {
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

#[derive(Copy, PartialEq, Clone)]
pub enum PseudoElementType<T> {
    Normal,
    Before(T),
    After(T),
}

impl<T> PseudoElementType<T> {
    pub fn is_before(&self) -> bool {
        match *self {
            PseudoElementType::Before(_) => true,
            _ => false,
        }
    }

    pub fn strip(&self) -> PseudoElementType<()> {
        match *self {
            PseudoElementType::Normal => PseudoElementType::Normal,
            PseudoElementType::Before(_) => PseudoElementType::Before(()),
            PseudoElementType::After(_) => PseudoElementType::After(()),
        }
    }
}

/// A thread-safe version of `LayoutNode`, used during flow construction. This type of layout
/// node does not allow any parents or siblings of nodes to be accessed, to avoid races.

pub trait ThreadSafeLayoutNode<'ln> : Clone + Copy + Sized {
    type ConcreteThreadSafeLayoutElement: ThreadSafeLayoutElement<'ln>;

    /// Converts self into an `OpaqueNode`.
    fn opaque(&self) -> OpaqueNode;

    /// Returns the type ID of this node.
    /// Returns `None` if this is a pseudo-element; otherwise, returns `Some`.
    fn type_id(&self) -> Option<NodeTypeId>;

    fn debug_id(self) -> usize;

    fn flow_debug_id(self) -> usize;

    /// Returns an iterator over this node's children.
    fn children(&self) -> ThreadSafeLayoutNodeChildrenIterator<'ln, Self>;

    /// If this is an element, accesses the element data. Fails if this is not an element node.
    #[inline]
    fn as_element(&self) -> Self::ConcreteThreadSafeLayoutElement;

    #[inline]
    fn get_pseudo_element_type(&self) -> PseudoElementType<display::T>;

    #[inline]
    fn get_before_pseudo(&self) -> Option<Self>;

    #[inline]
    fn get_after_pseudo(&self) -> Option<Self>;

    /// Borrows the layout data immutably. Fails on a conflicting borrow.
    ///
    /// TODO(pcwalton): Make this private. It will let us avoid borrow flag checks in some cases.
    #[inline(always)]
    fn borrow_layout_data(&self) -> Ref<Option<LayoutDataWrapper>>;

    /// Borrows the layout data mutably. Fails on a conflicting borrow.
    ///
    /// TODO(pcwalton): Make this private. It will let us avoid borrow flag checks in some cases.
    #[inline(always)]
    fn mutate_layout_data(&self) -> RefMut<Option<LayoutDataWrapper>>;

    /// Returns the style results for the given node. If CSS selector matching
    /// has not yet been performed, fails.
    #[inline]
    fn style(&self) -> Ref<Arc<ComputedValues>> {
        Ref::map(self.borrow_layout_data(), |layout_data_ref| {
            let layout_data = layout_data_ref.as_ref().expect("no layout data");
            let style = match self.get_pseudo_element_type() {
                PseudoElementType::Before(_) => &layout_data.data.before_style,
                PseudoElementType::After(_) => &layout_data.data.after_style,
                PseudoElementType::Normal => &layout_data.shared_data.style,
            };
            style.as_ref().unwrap()
        })
    }

    /// Removes the style from this node.
    fn unstyle(self) {
        let mut layout_data_ref = self.mutate_layout_data();
        let layout_data = layout_data_ref.as_mut().expect("no layout data");

        let style =
            match self.get_pseudo_element_type() {
                PseudoElementType::Before(_) => &mut layout_data.data.before_style,
                PseudoElementType::After (_) => &mut layout_data.data.after_style,
                PseudoElementType::Normal    => &mut layout_data.shared_data.style,
            };

        *style = None;
    }

    fn is_ignorable_whitespace(&self) -> bool;

    /// Get the description of how to account for recent style changes.
    /// This is a simple bitfield and fine to copy by value.
    fn restyle_damage(self) -> RestyleDamage {
        let layout_data_ref = self.borrow_layout_data();
        layout_data_ref.as_ref().unwrap().data.restyle_damage
    }

    /// Set the restyle damage field.
    fn set_restyle_damage(self, damage: RestyleDamage) {
        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref {
            Some(ref mut layout_data) => layout_data.data.restyle_damage = damage,
            _ => panic!("no layout data for this node"),
        }
    }

    /// Returns the layout data flags for this node.
    fn flags(self) -> LayoutDataFlags;

    /// Adds the given flags to this node.
    fn insert_flags(self, new_flags: LayoutDataFlags) {
        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref {
            Some(ref mut layout_data) => layout_data.data.flags.insert(new_flags),
            _ => panic!("no layout data for this node"),
        }
    }

    /// Removes the given flags from this node.
    fn remove_flags(self, flags: LayoutDataFlags) {
        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref {
            Some(ref mut layout_data) => layout_data.data.flags.remove(flags),
            _ => panic!("no layout data for this node"),
        }
    }

    /// Returns true if this node contributes content. This is used in the implementation of
    /// `empty_cells` per CSS 2.1 § 17.6.1.1.
    fn is_content(&self) -> bool {
        match self.type_id() {
            Some(NodeTypeId::Element(..)) | Some(NodeTypeId::CharacterData(CharacterDataTypeId::Text(..))) => true,
            _ => false
        }
    }

    /// If this is a text node, generated content, or a form element, copies out
    /// its content. Otherwise, panics.
    ///
    /// FIXME(pcwalton): This might have too much copying and/or allocation. Profile this.
    fn text_content(&self) -> TextContent;

    /// If the insertion point is within this node, returns it. Otherwise, returns `None`.
    fn insertion_point(&self) -> Option<CharIndex>;

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

// These can violate the thread-safety and therefore are not public.
trait DangerousThreadSafeLayoutNode<'ln> : ThreadSafeLayoutNode<'ln> {
    unsafe fn dangerous_first_child(&self) -> Option<Self>;
    unsafe fn dangerous_next_sibling(&self) -> Option<Self>;
}

pub trait ThreadSafeLayoutElement<'le> {
    type ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode<'le>;

    #[inline]
    fn get_attr(&self, namespace: &Namespace, name: &Atom) -> Option<&'le str>;
}

#[derive(Copy, Clone)]
pub struct ServoThreadSafeLayoutNode<'ln> {
    /// The wrapped node.
    node: ServoLayoutNode<'ln>,

    pseudo: PseudoElementType<display::T>,
}

impl<'ln> DangerousThreadSafeLayoutNode<'ln> for ServoThreadSafeLayoutNode<'ln> {
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

    /// Creates a new `ServoThreadSafeLayoutNode` for the same `LayoutNode`
    /// with a different pseudo-element type.
    fn with_pseudo(&self, pseudo: PseudoElementType<display::T>) -> ServoThreadSafeLayoutNode<'ln> {
        ServoThreadSafeLayoutNode {
            node: self.node.clone(),
            pseudo: pseudo,
        }
    }

    /// Returns the interior of this node as a `LayoutJS`. This is highly unsafe for layout to
    /// call and as such is marked `unsafe`.
    unsafe fn get_jsmanaged(&self) -> &LayoutJS<Node> {
        self.node.get_jsmanaged()
    }

    /// Borrows the layout data without checking.
    #[inline(always)]
    fn borrow_layout_data_unchecked(&self) -> *const Option<LayoutDataWrapper> {
        unsafe {
            self.node.borrow_layout_data_unchecked()
        }
    }
}

impl<'ln> ThreadSafeLayoutNode<'ln> for ServoThreadSafeLayoutNode<'ln> {
    type ConcreteThreadSafeLayoutElement = ServoThreadSafeLayoutElement<'ln>;

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

    fn children(&self) -> ThreadSafeLayoutNodeChildrenIterator<'ln, Self> {
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

    fn get_before_pseudo(&self) -> Option<ServoThreadSafeLayoutNode<'ln>> {
        let layout_data_ref = self.borrow_layout_data();
        let node_layout_data_wrapper = layout_data_ref.as_ref().unwrap();
        node_layout_data_wrapper.data.before_style.as_ref().map(|style| {
            self.with_pseudo(PseudoElementType::Before(style.get_box().display))
        })
    }

    fn get_after_pseudo(&self) -> Option<ServoThreadSafeLayoutNode<'ln>> {
        let layout_data_ref = self.borrow_layout_data();
        let node_layout_data_wrapper = layout_data_ref.as_ref().unwrap();
        node_layout_data_wrapper.data.after_style.as_ref().map(|style| {
            self.with_pseudo(PseudoElementType::After(style.get_box().display))
        })
    }

    fn borrow_layout_data(&self) -> Ref<Option<LayoutDataWrapper>> {
        self.node.borrow_layout_data()
    }

    fn mutate_layout_data(&self) -> RefMut<Option<LayoutDataWrapper>> {
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

    fn flags(self) -> LayoutDataFlags {
        unsafe {
            match *self.borrow_layout_data_unchecked() {
                None => panic!(),
                Some(ref layout_data) => layout_data.data.flags,
            }
        }
    }

    fn text_content(&self) -> TextContent {
        if self.pseudo != PseudoElementType::Normal {
            let layout_data_ref = self.borrow_layout_data();
            let data = &layout_data_ref.as_ref().unwrap().data;

            let style = if self.pseudo.is_before() {
                &data.before_style
            } else {
                &data.after_style
            };
            return match style.as_ref().unwrap().get_box().content {
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

    fn insertion_point(&self) -> Option<CharIndex> {
        let this = unsafe {
            self.get_jsmanaged()
        };

        if let Some(area) = this.downcast::<HTMLTextAreaElement>() {
            let insertion_point = unsafe { area.get_absolute_insertion_point_for_layout() };
            let text = unsafe { area.get_value_for_layout() };
            return Some(CharIndex(search_index(insertion_point, text.char_indices())));
        }
        if let Some(input) = this.downcast::<HTMLInputElement>() {
            let insertion_point_index = unsafe { input.get_insertion_point_index_for_layout() };
            if let Some(insertion_point_index) = insertion_point_index {
                return Some(CharIndex(insertion_point_index));
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

pub struct ThreadSafeLayoutNodeChildrenIterator<'ln, ConcreteNode: ThreadSafeLayoutNode<'ln>> {
    current_node: Option<ConcreteNode>,
    parent_node: ConcreteNode,
    // Satisfy the compiler about the unused lifetime.
    phantom: PhantomData<&'ln ()>,
}

impl<'ln, ConcreteNode> ThreadSafeLayoutNodeChildrenIterator<'ln, ConcreteNode>
                        where ConcreteNode: DangerousThreadSafeLayoutNode<'ln> {
    fn new(parent: ConcreteNode) -> Self {
        let first_child: Option<ConcreteNode> = match parent.get_pseudo_element_type() {
            PseudoElementType::Normal => {
                parent.get_before_pseudo().or_else(|| {
                    unsafe { parent.dangerous_first_child() }
                })
            },
            _ => None,
        };
        ThreadSafeLayoutNodeChildrenIterator {
            current_node: first_child,
            parent_node: parent,
            phantom: PhantomData,
        }
    }
}

impl<'ln, ConcreteNode> Iterator for ThreadSafeLayoutNodeChildrenIterator<'ln, ConcreteNode>
                                 where ConcreteNode: DangerousThreadSafeLayoutNode<'ln> {
    type Item = ConcreteNode;
    fn next(&mut self) -> Option<ConcreteNode> {
        let node = self.current_node.clone();

        if let Some(ref node) = node {
            self.current_node = match node.get_pseudo_element_type() {
                PseudoElementType::Before(_) => {
                    match unsafe { self.parent_node.dangerous_first_child() } {
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
                PseudoElementType::After(_) => {
                    None
                },
            };
        }

        node
    }
}

/// A wrapper around elements that ensures layout can only ever access safe properties and cannot
/// race on elements.
pub struct ServoThreadSafeLayoutElement<'le> {
    element: &'le Element,
}

impl<'le> ThreadSafeLayoutElement<'le> for ServoThreadSafeLayoutElement<'le> {
    type ConcreteThreadSafeLayoutNode = ServoThreadSafeLayoutNode<'le>;

    fn get_attr(&self, namespace: &Namespace, name: &Atom) -> Option<&'le str> {
        unsafe {
            self.element.get_attr_val_for_layout(namespace, name)
        }
    }
}

/// Opaque type stored in type-unsafe work queues for parallel layout.
/// Must be transmutable to and from LayoutNode.
pub type UnsafeLayoutNode = (usize, usize);

pub fn layout_node_to_unsafe_layout_node(node: &ServoLayoutNode) -> UnsafeLayoutNode {
    unsafe {
        let ptr: usize = mem::transmute_copy(node);
        (ptr, 0)
    }
}

pub unsafe fn layout_node_from_unsafe_layout_node(node: &UnsafeLayoutNode) -> ServoLayoutNode {
    let (node, _) = *node;
    mem::transmute(node)
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
