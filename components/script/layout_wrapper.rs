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

use dom::bindings::inheritance::{CharacterDataTypeId, ElementTypeId};
use dom::bindings::inheritance::{HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::LayoutJS;
use dom::characterdata::LayoutCharacterDataHelpers;
use dom::document::{Document, LayoutDocumentHelpers};
use dom::element::{Element, LayoutElementHelpers, RawLayoutElementHelpers};
use dom::node::{CAN_BE_FRAGMENTED, HAS_CHANGED, HAS_DIRTY_DESCENDANTS, IS_DIRTY, DIRTY_ON_VIEWPORT_SIZE_CHANGE};
use dom::node::{Node, LayoutNodeHelpers};
use dom::text::Text;
use gfx_traits::ByteIndex;
use msg::constellation_msg::PipelineId;
use range::Range;
use script_layout_interface::restyle_damage::RestyleDamage;
use script_layout_interface::wrapper_traits::{DangerousThreadSafeLayoutNode, LayoutNode, PseudoElementType};
use script_layout_interface::wrapper_traits::{ThreadSafeLayoutNode, ThreadSafeLayoutElement};
use script_layout_interface::{HTMLCanvasData, LayoutNodeType, TrustedNodeAddress};
use script_layout_interface::{OpaqueStyleAndLayoutData, PartialStyleAndLayoutData};
use selectors::matching::{DeclarationBlock, ElementFlags};
use selectors::parser::{AttrSelector, NamespaceConstraint};
use std::marker::PhantomData;
use std::mem::{transmute, transmute_copy};
use string_cache::{Atom, BorrowedAtom, BorrowedNamespace, Namespace};
use style::attr::AttrValue;
use style::computed_values::display;
use style::context::SharedStyleContext;
use style::data::PrivateStyleData;
use style::dom::{PresentationalHintsSynthetizer, OpaqueNode, TDocument, TElement, TNode, UnsafeNode};
use style::element_state::*;
use style::properties::{PropertyDeclaration, PropertyDeclarationBlock};
use style::refcell::{Ref, RefCell, RefMut};
use style::selector_impl::{ElementSnapshot, NonTSPseudoClass, ServoSelectorImpl};
use style::sink::Push;
use style::str::is_whitespace;
use url::Url;

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

    fn script_type_id(&self) -> NodeTypeId {
        unsafe {
            self.node.type_id_for_layout()
        }
    }
}

impl<'ln> TNode for ServoLayoutNode<'ln> {
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
        self.script_type_id() == NodeTypeId::CharacterData(CharacterDataTypeId::Text)
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
        unsafe { self.get_jsmanaged().opaque() }
    }

    fn layout_parent_node(self, reflow_root: OpaqueNode) -> Option<ServoLayoutNode<'ln>> {
        if self.opaque() == reflow_root {
            None
        } else {
            self.parent_node()
        }
    }

    fn debug_id(self) -> usize {
        self.opaque().0
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

    fn needs_dirty_on_viewport_size_changed(&self) -> bool {
        unsafe { self.node.get_flag(DIRTY_ON_VIEWPORT_SIZE_CHANGE) }
    }

    unsafe fn set_dirty_on_viewport_size_changed(&self) {
        self.node.set_flag(DIRTY_ON_VIEWPORT_SIZE_CHANGE, true);
    }

    fn set_descendants_dirty_on_viewport_size_changed(&self) {
        for ref child in self.children() {
            unsafe {
                child.set_dirty_on_viewport_size_changed();
            }
            child.set_descendants_dirty_on_viewport_size_changed();
        }
    }

    fn can_be_fragmented(&self) -> bool {
        unsafe { self.node.get_flag(CAN_BE_FRAGMENTED) }
    }

    unsafe fn set_can_be_fragmented(&self, value: bool) {
        self.node.set_flag(CAN_BE_FRAGMENTED, value)
    }

    unsafe fn borrow_data_unchecked(&self) -> Option<*const PrivateStyleData> {
        self.get_style_data().map(|d| {
            &(*d.as_unsafe_cell().get()).style_data as *const _
        })
    }

    fn borrow_data(&self) -> Option<Ref<PrivateStyleData>> {
        self.get_style_data().map(|d| {
            Ref::map(d.borrow(), |d| &d.style_data)
        })
    }

    fn mutate_data(&self) -> Option<RefMut<PrivateStyleData>> {
        self.get_style_data().map(|d| {
            RefMut::map(d.borrow_mut(), |d| &mut d.style_data)
        })
    }

    fn restyle_damage(self) -> RestyleDamage {
        self.get_style_data().unwrap().borrow().restyle_damage
    }

    fn set_restyle_damage(self, damage: RestyleDamage) {
        self.get_style_data().unwrap().borrow_mut().restyle_damage = damage;
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

    fn type_id(&self) -> LayoutNodeType {
        self.script_type_id().into()
    }

    fn get_style_data(&self) -> Option<&RefCell<PartialStyleAndLayoutData>> {
        unsafe {
            self.get_jsmanaged().get_style_and_layout_data().map(|d| {
                &**d.ptr
            })
        }
    }

    fn init_style_and_layout_data(&self, data: OpaqueStyleAndLayoutData) {
        unsafe {
            self.get_jsmanaged().init_style_and_layout_data(data);
        }
    }

    fn get_style_and_layout_data(&self) -> Option<OpaqueStyleAndLayoutData> {
        unsafe {
            self.get_jsmanaged().get_style_and_layout_data()
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
                self.script_type_id(), self.has_changed(), self.is_dirty(), self.has_dirty_descendants())
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

impl<'le> PresentationalHintsSynthetizer for ServoLayoutElement<'le> {
    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, hints: &mut V)
        where V: Push<DeclarationBlock<Vec<PropertyDeclaration>>>
    {
        unsafe {
            self.element.synthesize_presentational_hints_for_legacy_attributes(hints);
        }
    }
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

    #[inline]
    fn has_attr(&self, namespace: &Namespace, attr: &Atom) -> bool {
        self.get_attr(namespace, attr).is_some()
    }

    #[inline]
    fn attr_equals(&self, namespace: &Namespace, attr: &Atom, val: &Atom) -> bool {
        self.get_attr(namespace, attr).map_or(false, |x| x == val)
    }
}


impl<'le> ServoLayoutElement<'le> {
    fn from_layout_js(el: LayoutJS<Element>) -> ServoLayoutElement<'le> {
        ServoLayoutElement {
            element: el,
            chain: PhantomData,
        }
    }

    #[inline]
    fn get_attr(&self, namespace: &Namespace, name: &Atom) -> Option<&str> {
        unsafe {
            (*self.element.unsafe_get()).get_attr_val_for_layout(namespace, name)
        }
    }
}

fn as_element<'le>(node: LayoutJS<Node>) -> Option<ServoLayoutElement<'le>> {
    node.downcast().map(ServoLayoutElement::from_layout_js)
}

impl<'le> ::selectors::MatchAttrGeneric for ServoLayoutElement<'le> {
    fn match_attr<F>(&self, attr: &AttrSelector, test: F) -> bool where F: Fn(&str) -> bool {
        use ::selectors::Element;
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
                let attrs = unsafe {
                    (*self.element.unsafe_get()).get_attr_vals_for_layout(name)
                };
                attrs.iter().any(|attr| test(*attr))
            }
        }
    }
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
    fn get_local_name<'a>(&'a self) -> BorrowedAtom<'a> {
        BorrowedAtom(self.element.local_name())
    }

    #[inline]
    fn get_namespace<'a>(&'a self) -> BorrowedNamespace<'a> {
        BorrowedNamespace(self.element.namespace())
    }

    fn match_non_ts_pseudo_class(&self, pseudo_class: NonTSPseudoClass) -> bool {
        match pseudo_class {
            // https://github.com/servo/servo/issues/8718
            NonTSPseudoClass::Link |
            NonTSPseudoClass::AnyLink => unsafe {
                match self.as_node().script_type_id() {
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

            NonTSPseudoClass::ReadOnly =>
                !self.element.get_state_for_layout().contains(pseudo_class.state_flag()),

            NonTSPseudoClass::Active |
            NonTSPseudoClass::Focus |
            NonTSPseudoClass::Hover |
            NonTSPseudoClass::Enabled |
            NonTSPseudoClass::Disabled |
            NonTSPseudoClass::Checked |
            NonTSPseudoClass::Indeterminate |
            NonTSPseudoClass::ReadWrite |
            NonTSPseudoClass::PlaceholderShown =>
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

    fn is_html_element_in_html_document(&self) -> bool {
        unsafe {
            self.element.html_element_in_html_document_for_layout()
        }
    }

    fn insert_flags(&self, flags: ElementFlags) {
        self.element.insert_atomic_flags(flags);
    }
}

#[derive(Copy, Clone)]
pub struct ServoThreadSafeLayoutNode<'ln> {
    /// The wrapped node.
    node: ServoLayoutNode<'ln>,

    /// The pseudo-element type, with (optionally),
    /// an specified display value to override the stylesheet.
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

impl<'ln> ThreadSafeLayoutNode for ServoThreadSafeLayoutNode<'ln> {
    type ConcreteThreadSafeLayoutElement = ServoThreadSafeLayoutElement<'ln>;
    type ChildrenIterator = ThreadSafeLayoutNodeChildrenIterator<Self>;

    fn with_pseudo(&self,
                   pseudo: PseudoElementType<Option<display::T>>) -> ServoThreadSafeLayoutNode<'ln> {
        ServoThreadSafeLayoutNode {
            node: self.node.clone(),
            pseudo: pseudo,
        }
    }

    fn opaque(&self) -> OpaqueNode {
        unsafe { self.get_jsmanaged().opaque() }
    }

    fn type_id(&self) -> Option<LayoutNodeType> {
        if self.pseudo != PseudoElementType::Normal {
            return None
        }

        Some(self.node.type_id())
    }

    #[inline]
    fn type_id_without_excluding_pseudo_elements(&self) -> LayoutNodeType {
        self.node.type_id()
    }

    fn debug_id(self) -> usize {
        self.node.debug_id()
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

    fn get_pseudo_element_type(&self) -> PseudoElementType<Option<display::T>> {
        self.pseudo
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

    fn restyle_damage(self) -> RestyleDamage {
        self.node.restyle_damage()
    }

    fn set_restyle_damage(self, damage: RestyleDamage) {
        self.node.set_restyle_damage(damage)
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

    fn image_url(&self) -> Option<Url> {
        let this = unsafe { self.get_jsmanaged() };
        this.image_url()
    }

    fn canvas_data(&self) -> Option<HTMLCanvasData> {
        let this = unsafe { self.get_jsmanaged() };
        this.canvas_data()
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

    fn get_style_data(&self) -> Option<&RefCell<PartialStyleAndLayoutData>> {
        self.node.get_style_data()
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
                           node.as_element().get_local_name() == atom!("summary") &&
                           node.as_element().get_namespace() == ns!(html) {
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
                       node.as_element().get_local_name() == atom!("summary") &&
                       node.as_element().get_namespace() == ns!(html) {
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

/// A wrapper around elements that ensures layout can only
/// ever access safe properties and cannot race on elements.
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
    fn get_local_name<'a>(&'a self) -> BorrowedAtom<'a> {
        BorrowedAtom(self.element.local_name())
    }

    #[inline]
    fn get_namespace<'a>(&'a self) -> BorrowedNamespace<'a> {
        BorrowedNamespace(self.element.namespace())
    }
}

/// This implementation of `::selectors::Element` is used for implementing lazy
/// pseudo-elements.
///
/// Lazy pseudo-elements in Servo only allows selectors using safe properties,
/// i.e., local_name, attributes, so they can only be used for **private**
/// pseudo-elements (like `::-servo-details-content`).
///
/// Probably a few more of this functions can be implemented (like `has_class`,
/// `each_class`, etc), but they have no use right now.
///
/// Note that the element implementation is needed only for selector matching,
/// not for inheritance (styles are inherited appropiately).
impl<'le> ::selectors::MatchAttrGeneric for ServoThreadSafeLayoutElement<'le> {
    fn match_attr<F>(&self, attr: &AttrSelector, test: F) -> bool
        where F: Fn(&str) -> bool {
        match attr.namespace {
            NamespaceConstraint::Specific(ref ns) => {
                self.get_attr(ns, &attr.name).map_or(false, |attr| test(attr))
            },
            NamespaceConstraint::Any => {
                unsafe {
                    self.element.get_attr_vals_for_layout(&attr.name).iter()
                        .any(|attr| test(*attr))
                }
            }
        }
    }
}
impl<'le> ::selectors::Element for ServoThreadSafeLayoutElement<'le> {
    type Impl = ServoSelectorImpl;

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
    fn get_local_name<'a>(&'a self) -> BorrowedAtom<'a> {
        ThreadSafeLayoutElement::get_local_name(self)
    }

    #[inline]
    fn get_namespace<'a>(&'a self) -> BorrowedNamespace<'a> {
        ThreadSafeLayoutElement::get_namespace(self)
    }

    fn match_non_ts_pseudo_class(&self, _: NonTSPseudoClass) -> bool {
        // NB: This could maybe be implemented
        warn!("ServoThreadSafeLayoutElement::match_non_ts_pseudo_class called");
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

    fn each_class<F>(&self, _callback: F)
        where F: FnMut(&Atom) {
        warn!("ServoThreadSafeLayoutElement::each_class called");
    }
}

impl<'le> PresentationalHintsSynthetizer for ServoThreadSafeLayoutElement<'le> {
    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, _hints: &mut V)
        where V: Push<DeclarationBlock<Vec<PropertyDeclaration>>> {}
}
