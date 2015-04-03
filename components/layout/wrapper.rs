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
//! * You must not use `.get()`; instead, use `.unsafe_get()`.
//!
//! * Do not call any methods on DOM nodes without checking to see whether they use borrow flags.
//!
//!   o Instead of `get_attr()`, use `.get_attr_val_for_layout()`.
//!
//!   o Instead of `html_element_in_html_document()`, use
//!     `html_element_in_html_document_for_layout()`.

#![allow(unsafe_code)]

use canvas::canvas_msg::CanvasMsg;
use context::SharedLayoutContext;
use css::node_style::StyledNode;
use incremental::RestyleDamage;
use data::{LayoutDataAccess, LayoutDataFlags, LayoutDataWrapper, PrivateLayoutData};
use opaque_node::OpaqueNodeMethods;

use cssparser::RGBA;
use gfx::display_list::OpaqueNode;
use script::dom::bindings::codegen::InheritTypes::{CharacterDataCast, ElementCast};
use script::dom::bindings::codegen::InheritTypes::{HTMLIFrameElementCast, HTMLCanvasElementCast};
use script::dom::bindings::codegen::InheritTypes::{HTMLImageElementCast, HTMLInputElementCast};
use script::dom::bindings::codegen::InheritTypes::{HTMLTextAreaElementCast, NodeCast, TextCast};
use script::dom::bindings::js::LayoutJS;
use script::dom::characterdata::LayoutCharacterDataHelpers;
use script::dom::element::{Element, ElementTypeId};
use script::dom::element::{LayoutElementHelpers, RawLayoutElementHelpers};
use script::dom::htmlelement::HTMLElementTypeId;
use script::dom::htmlcanvaselement::{HTMLCanvasElement, LayoutHTMLCanvasElementHelpers};
use script::dom::htmliframeelement::HTMLIFrameElement;
use script::dom::htmlimageelement::LayoutHTMLImageElementHelpers;
use script::dom::htmlinputelement::{HTMLInputElement, LayoutHTMLInputElementHelpers};
use script::dom::htmltextareaelement::{HTMLTextAreaElement, LayoutHTMLTextAreaElementHelpers};
use script::dom::node::{Node, NodeTypeId};
use script::dom::node::{LayoutNodeHelpers, RawLayoutNodeHelpers, SharedLayoutData};
use script::dom::node::{HAS_CHANGED, IS_DIRTY, HAS_DIRTY_SIBLINGS, HAS_DIRTY_DESCENDANTS};
use script::dom::text::Text;
use script::layout_interface::LayoutChan;
use msg::constellation_msg::{PipelineId, SubpageId};
use util::str::{LengthOrPercentageOrAuto, is_whitespace};
use std::borrow::ToOwned;
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;
use std::mem;
use std::sync::mpsc::Sender;
use string_cache::{Atom, Namespace};
use style::computed_values::content::ContentItem;
use style::computed_values::{content, display, white_space};
use selectors::parser::{NamespaceConstraint, AttrSelector};
use style::legacy::{IntegerAttribute, LengthAttribute, SimpleColorAttribute};
use style::legacy::{UnsignedIntegerAttribute};
use style::node::{TElement, TElementAttributes, TNode};
use style::properties::PropertyDeclarationBlock;
use url::Url;

/// Allows some convenience methods on generic layout nodes.
pub trait TLayoutNode {
    /// Creates a new layout node with the same lifetime as this layout node.
    unsafe fn new_with_this_lifetime(&self, node: &LayoutJS<Node>) -> Self;

    /// Returns the type ID of this node. Fails if this node is borrowed mutably. Returns `None`
    /// if this is a pseudo-element; otherwise, returns `Some`.
    fn type_id(&self) -> Option<NodeTypeId>;

    /// Returns the interior of this node as a `LayoutJS`. This is highly unsafe for layout to
    /// call and as such is marked `unsafe`.
    unsafe fn get_jsmanaged<'a>(&'a self) -> &'a LayoutJS<Node>;

    /// Returns the interior of this node as a `Node`. This is highly unsafe for layout to call
    /// and as such is marked `unsafe`.
    unsafe fn get<'a>(&'a self) -> &'a Node {
        &*self.get_jsmanaged().unsafe_get()
    }

    fn node_is_element(&self) -> bool {
        match self.type_id() {
            Some(NodeTypeId::Element(..)) => true,
            _ => false
        }
    }

    fn node_is_document(&self) -> bool {
        match self.type_id() {
            Some(NodeTypeId::Document(..)) => true,
            _ => false
        }
    }

    /// If this is an image element, returns its URL. If this is not an image element, fails.
    ///
    /// FIXME(pcwalton): Don't copy URLs.
    fn image_url(&self) -> Option<Url> {
        unsafe {
            match HTMLImageElementCast::to_layout_js(self.get_jsmanaged()) {
                Some(elem) => elem.image().as_ref().map(|url| (*url).clone()),
                None => panic!("not an image!")
            }
        }
    }

    fn get_renderer(&self) -> Option<Sender<CanvasMsg>> {
        unsafe {
            let canvas_element: Option<LayoutJS<HTMLCanvasElement>> = HTMLCanvasElementCast::to_layout_js(self.get_jsmanaged());
            canvas_element.and_then(|elem| elem.get_renderer())
        }
    }

    fn get_canvas_width(&self) -> u32 {
        unsafe {
            let canvas_element: Option<LayoutJS<HTMLCanvasElement>> = HTMLCanvasElementCast::to_layout_js(self.get_jsmanaged());
            canvas_element.unwrap().get_canvas_width()
        }
    }

    fn get_canvas_height(&self) -> u32 {
        unsafe {
            let canvas_element: Option<LayoutJS<HTMLCanvasElement>> = HTMLCanvasElementCast::to_layout_js(self.get_jsmanaged());
            canvas_element.unwrap().get_canvas_height()
        }
    }

    /// If this node is an iframe element, returns its pipeline and subpage IDs. If this node is
    /// not an iframe element, fails.
    fn iframe_pipeline_and_subpage_ids(&self) -> (PipelineId, SubpageId) {
        unsafe {
            let iframe_element: LayoutJS<HTMLIFrameElement> =
                match HTMLIFrameElementCast::to_layout_js(self.get_jsmanaged()) {
                    Some(elem) => elem,
                    None => panic!("not an iframe element!")
                };
            ((*iframe_element.unsafe_get()).containing_page_pipeline_id().unwrap(),
             (*iframe_element.unsafe_get()).subpage_id().unwrap())
        }
    }

    /// If this is a text node or generated content, copies out its content. If this is not a text
    /// node, fails.
    ///
    /// FIXME(pcwalton): This might have too much copying and/or allocation. Profile this.
    fn text_content(&self) -> Vec<ContentItem>;

    /// Returns the first child of this node.
    fn first_child(&self) -> Option<Self>;
}

/// A wrapper so that layout can access only the methods that it should have access to. Layout must
/// only ever see these and must never see instances of `LayoutJS`.
#[derive(Copy)]
pub struct LayoutNode<'a> {
    /// The wrapped node.
    node: LayoutJS<Node>,

    /// Being chained to a PhantomData prevents `LayoutNode`s from escaping.
    pub chain: PhantomData<&'a ()>,
}

impl<'ln> Clone for LayoutNode<'ln> {
    fn clone(&self) -> LayoutNode<'ln> {
        LayoutNode {
            node: self.node.clone(),
            chain: self.chain,
        }
    }
}

impl<'a> PartialEq for LayoutNode<'a> {
    #[inline]
    fn eq(&self, other: &LayoutNode) -> bool {
        self.node == other.node
    }
}

impl<'ln> TLayoutNode for LayoutNode<'ln> {
    unsafe fn new_with_this_lifetime(&self, node: &LayoutJS<Node>) -> LayoutNode<'ln> {
        LayoutNode {
            node: node.transmute_copy(),
            chain: self.chain,
        }
    }

    fn type_id(&self) -> Option<NodeTypeId> {
        unsafe {
            Some(self.node.type_id_for_layout())
        }
    }

    unsafe fn get_jsmanaged<'a>(&'a self) -> &'a LayoutJS<Node> {
        &self.node
    }

    fn first_child(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.get_jsmanaged().first_child_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn text_content(&self) -> Vec<ContentItem> {
        unsafe {
            let text: Option<LayoutJS<Text>> = TextCast::to_layout_js(self.get_jsmanaged());
            if let Some(text) = text {
                return vec![
                    ContentItem::String(
                        CharacterDataCast::from_layout_js(&text).data_for_layout().to_owned())
                ];
            }
            let input: Option<LayoutJS<HTMLInputElement>> =
                HTMLInputElementCast::to_layout_js(self.get_jsmanaged());
            if let Some(input) = input {
                return vec![ContentItem::String(input.get_value_for_layout())];
            }
            let area: Option<LayoutJS<HTMLTextAreaElement>> =
                HTMLTextAreaElementCast::to_layout_js(self.get_jsmanaged());
            if let Some(area) = area {
                return vec![ContentItem::String(area.get_value_for_layout())];
            }

            panic!("not text!")
        }
    }
}

impl<'ln> LayoutNode<'ln> {
    pub fn dump(self) {
        self.dump_indent(0);
    }

    fn dump_indent(self, indent: u32) {
        let mut s = String::new();
        for _ in range(0, indent) {
            s.push_str("  ");
        }

        s.push_str(self.debug_str().as_slice());
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

    pub fn traverse_preorder(self) -> LayoutTreeIterator<'ln> {
        LayoutTreeIterator::new(self)
    }

    fn last_child(self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.get_jsmanaged().last_child_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    /// Returns an iterator over this node's children.
    pub fn children(self) -> LayoutNodeChildrenIterator<'ln> {
        // FIXME(zwarich): Remove this when UFCS lands and there is a better way
        // of disambiguating methods.
        fn first_child<T: TLayoutNode>(this: T) -> Option<T> {
            this.first_child()
        }

        LayoutNodeChildrenIterator {
            current: first_child(self),
        }
    }

    pub fn rev_children(self) -> LayoutNodeReverseChildrenIterator<'ln> {
        LayoutNodeReverseChildrenIterator {
            current: self.last_child()
        }

    }

    pub unsafe fn get_jsmanaged<'a>(&'a self) -> &'a LayoutJS<Node> {
        &self.node
    }

    /// Resets layout data and styles for the node.
    ///
    /// FIXME(pcwalton): Do this as part of fragment building instead of in a traversal.
    pub fn initialize_layout_data(self, chan: LayoutChan) {
        let mut layout_data_ref = self.mutate_layout_data();
        match *layout_data_ref {
            None => {
                *layout_data_ref = Some(LayoutDataWrapper {
                    chan: Some(chan),
                    shared_data: SharedLayoutData { style: None },
                    data: box PrivateLayoutData::new(),
                });
            }
            Some(_) => {}
        }
    }

    pub fn has_children(self) -> bool {
        self.first_child().is_some()
    }

    /// While doing a reflow, the node at the root has no parent, as far as we're
    /// concerned. This method returns `None` at the reflow root.
    pub fn layout_parent_node(self, shared: &SharedLayoutContext) -> Option<LayoutNode<'ln>> {
        match shared.reflow_root {
            None => panic!("layout_parent_node(): This layout has no access to the DOM!"),
            Some(reflow_root) => {
                let opaque_node: OpaqueNode = OpaqueNodeMethods::from_layout_node(&self);
                if opaque_node == reflow_root {
                    None
                } else {
                    self.parent_node()
                }
            }
        }
    }

    pub fn debug_id(self) -> usize {
        let opaque: OpaqueNode = OpaqueNodeMethods::from_layout_node(&self);
        opaque.to_untrusted_node_address().0 as usize
    }
}

impl<'ln> TNode<'ln> for LayoutNode<'ln> {
    type Element = LayoutElement<'ln>;

    fn parent_node(self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.parent_node_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn first_child(self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.first_child_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn last_child(self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.last_child_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn prev_sibling(self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.prev_sibling_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn next_sibling(self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.next_sibling_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    /// If this is an element, accesses the element data. Fails if this is not an element node.
    #[inline]
    fn as_element(self) -> LayoutElement<'ln> {
        unsafe {
            let elem: LayoutJS<Element> = match ElementCast::to_layout_js(&self.node) {
                Some(elem) => elem,
                None => panic!("not an element")
            };

            LayoutElement {
                element: &*elem.unsafe_get(),
            }
        }
    }

    fn is_element(self) -> bool {
        self.node_is_element()
    }

    fn is_document(self) -> bool {
        self.node_is_document()
    }

    fn match_attr<F>(self, attr: &AttrSelector, test: F) -> bool where F: Fn(&str) -> bool {
        assert!(self.is_element());
        let name = if self.is_html_element_in_html_document() {
            &attr.lower_name
        } else {
            &attr.name
        };
        match attr.namespace {
            NamespaceConstraint::Specific(ref ns) => {
                let element = self.as_element();
                element.get_attr(ns, name).map_or(false, |attr| test(attr))
            },
            NamespaceConstraint::Any => {
                let element = self.as_element();
                element.get_attrs(name).iter().any(|attr| test(*attr))
            }
        }
    }

    fn is_html_element_in_html_document(self) -> bool {
        unsafe {
            match ElementCast::to_layout_js(&self.node) {
                Some(elem) => elem.html_element_in_html_document_for_layout(),
                None => false
            }
        }
    }

    fn has_changed(self) -> bool {
        unsafe { self.node.get_flag(HAS_CHANGED) }
    }

    unsafe fn set_changed(self, value: bool) {
        self.node.set_flag(HAS_CHANGED, value)
    }

    fn is_dirty(self) -> bool {
        unsafe { self.node.get_flag(IS_DIRTY) }
    }

    unsafe fn set_dirty(self, value: bool) {
        self.node.set_flag(IS_DIRTY, value)
    }

    fn has_dirty_siblings(self) -> bool {
        unsafe { self.node.get_flag(HAS_DIRTY_SIBLINGS) }
    }

    unsafe fn set_dirty_siblings(self, value: bool) {
        self.node.set_flag(HAS_DIRTY_SIBLINGS, value);
    }

    fn has_dirty_descendants(self) -> bool {
        unsafe { self.node.get_flag(HAS_DIRTY_DESCENDANTS) }
    }

    unsafe fn set_dirty_descendants(self, value: bool) {
        self.node.set_flag(HAS_DIRTY_DESCENDANTS, value)
    }
}

pub struct LayoutNodeChildrenIterator<'a> {
    current: Option<LayoutNode<'a>>,
}

impl<'a> Iterator for LayoutNodeChildrenIterator<'a> {
    type Item = LayoutNode<'a>;
    fn next(&mut self) -> Option<LayoutNode<'a>> {
        let node = self.current;
        self.current = node.and_then(|node| node.next_sibling());
        node
    }
}

pub struct LayoutNodeReverseChildrenIterator<'a> {
    current: Option<LayoutNode<'a>>,
}

impl<'a> Iterator for LayoutNodeReverseChildrenIterator<'a> {
    type Item = LayoutNode<'a>;
    fn next(&mut self) -> Option<LayoutNode<'a>> {
        let node = self.current;
        self.current = node.and_then(|node| node.prev_sibling());
        node
    }
}

pub struct LayoutTreeIterator<'a> {
    stack: Vec<LayoutNode<'a>>,
}

impl<'a> LayoutTreeIterator<'a> {
    fn new(root: LayoutNode<'a>) -> LayoutTreeIterator<'a> {
        let mut stack = vec!();
        stack.push(root);
        LayoutTreeIterator {
            stack: stack
        }
    }
}

impl<'a> Iterator for LayoutTreeIterator<'a> {
    type Item = LayoutNode<'a>;
    fn next(&mut self) -> Option<LayoutNode<'a>> {
        let ret = self.stack.pop();
        ret.map(|node| self.stack.extend(node.rev_children()));
        ret
    }
}

/// A wrapper around elements that ensures layout can only ever access safe properties.
#[derive(Copy)]
pub struct LayoutElement<'le> {
    element: &'le Element,
}

impl<'le> LayoutElement<'le> {
    pub fn style_attribute(&self) -> &'le Option<PropertyDeclarationBlock> {
        let style: &Option<PropertyDeclarationBlock> = unsafe {
            &*self.element.style_attribute().borrow_for_layout()
        };
        style
    }
}

impl<'le> TElement<'le> for LayoutElement<'le> {
    #[inline]
    fn get_local_name(self) -> &'le Atom {
        self.element.local_name()
    }

    #[inline]
    fn get_namespace(self) -> &'le Namespace {
        self.element.namespace()
    }

    #[inline]
    fn get_attr(self, namespace: &Namespace, name: &Atom) -> Option<&'le str> {
        unsafe { self.element.get_attr_val_for_layout(namespace, name) }
    }

    #[inline]
    fn get_attrs(self, name: &Atom) -> Vec<&'le str> {
        unsafe {
            self.element.get_attr_vals_for_layout(name)
        }
    }

    fn get_link(self) -> Option<&'le str> {
        // FIXME: This is HTML only.
        let node: &Node = NodeCast::from_actual(self.element);
        match node.type_id_for_layout() {
            // https://html.spec.whatwg.org/multipage/#selector-link
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAnchorElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLAreaElement)) |
            NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLLinkElement)) => {
                unsafe {
                    self.element.get_attr_val_for_layout(&ns!(""), &atom!("href"))
                }
            }
            _ => None,
        }
    }

    #[inline]
    fn get_hover_state(self) -> bool {
        unsafe {
            let node: &Node = NodeCast::from_actual(self.element);
            node.get_hover_state_for_layout()
        }
    }

    #[inline]
    fn get_focus_state(self) -> bool {
        unsafe {
            let node: &Node = NodeCast::from_actual(self.element);
            node.get_focus_state_for_layout()
        }
    }

    #[inline]
    fn get_id(self) -> Option<Atom> {
        unsafe {
            self.element.get_attr_atom_for_layout(&ns!(""), &atom!("id"))
        }
    }

    #[inline]
    fn get_disabled_state(self) -> bool {
        unsafe {
            let node: &Node = NodeCast::from_actual(self.element);
            node.get_disabled_state_for_layout()
        }
    }

    #[inline]
    fn get_enabled_state(self) -> bool {
        unsafe {
            let node: &Node = NodeCast::from_actual(self.element);
            node.get_enabled_state_for_layout()
        }
    }

    #[inline]
    fn get_checked_state(self) -> bool {
        unsafe {
            self.element.get_checked_state_for_layout()
        }
    }

    #[inline]
    fn get_indeterminate_state(self) -> bool {
        unsafe {
            self.element.get_indeterminate_state_for_layout()
        }
    }

    #[inline]
    fn has_class(self, name: &Atom) -> bool {
        unsafe {
            self.element.has_class_for_layout(name)
        }
    }

    #[inline(always)]
    fn each_class<F>(self, mut callback: F) where F: FnMut(&Atom) {
        unsafe {
            match self.element.get_classes_for_layout() {
                None => {}
                Some(ref classes) => {
                    for class in classes.iter() {
                        callback(class)
                    }
                }
            }
        }
    }

    #[inline]
    fn has_nonzero_border(self) -> bool {
        unsafe {
            match self.element.get_unsigned_integer_attribute_for_layout(
                    UnsignedIntegerAttribute::Border) {
                None | Some(0) => false,
                _ => true,
            }
        }
    }
}

impl<'le> TElementAttributes for LayoutElement<'le> {
    fn get_length_attribute(self, length_attribute: LengthAttribute) -> LengthOrPercentageOrAuto {
        unsafe {
            self.element.get_length_attribute_for_layout(length_attribute)
        }
    }

    fn get_integer_attribute(self, integer_attribute: IntegerAttribute) -> Option<i32> {
        unsafe {
            self.element.get_integer_attribute_for_layout(integer_attribute)
        }
    }

    fn get_unsigned_integer_attribute(self, attribute: UnsignedIntegerAttribute) -> Option<u32> {
        unsafe {
            self.element.get_unsigned_integer_attribute_for_layout(attribute)
        }
    }

    fn get_simple_color_attribute(self, attribute: SimpleColorAttribute) -> Option<RGBA> {
        unsafe {
            self.element.get_simple_color_attribute_for_layout(attribute)
        }
    }
}

fn get_content(content_list: &content::T) -> Vec<ContentItem> {
    match *content_list {
        content::T::Content(ref value) if !value.is_empty() => (*value).clone(),
        _ => vec![],
    }
}

#[derive(Copy, PartialEq, Clone)]
pub enum PseudoElementType {
    Normal,
    Before(display::T),
    After(display::T),
}

impl PseudoElementType {
    pub fn is_before(&self) -> bool {
        match *self {
            PseudoElementType::Before(_) => true,
            _ => false,
        }
    }

    pub fn is_after(&self) -> bool {
        match *self {
            PseudoElementType::After(_) => true,
            _ => false,
        }
    }
}

/// A thread-safe version of `LayoutNode`, used during flow construction. This type of layout
/// node does not allow any parents or siblings of nodes to be accessed, to avoid races.
#[derive(Copy, Clone)]
pub struct ThreadSafeLayoutNode<'ln> {
    /// The wrapped node.
    node: LayoutNode<'ln>,

    pseudo: PseudoElementType,
}

impl<'ln> TLayoutNode for ThreadSafeLayoutNode<'ln> {
    /// Creates a new layout node with the same lifetime as this layout node.
    unsafe fn new_with_this_lifetime(&self, node: &LayoutJS<Node>) -> ThreadSafeLayoutNode<'ln> {
        ThreadSafeLayoutNode {
            node: LayoutNode {
                node: node.transmute_copy(),
                chain: self.node.chain,
            },
            pseudo: PseudoElementType::Normal,
        }
    }

    /// Returns `None` if this is a pseudo-element.
    fn type_id(&self) -> Option<NodeTypeId> {
        if self.pseudo != PseudoElementType::Normal {
            return None
        }

        self.node.type_id()
    }

    unsafe fn get_jsmanaged<'a>(&'a self) -> &'a LayoutJS<Node> {
        self.node.get_jsmanaged()
    }

    unsafe fn get<'a>(&'a self) -> &'a Node { // this change.
        &*self.get_jsmanaged().unsafe_get()
    }

    fn first_child(&self) -> Option<ThreadSafeLayoutNode<'ln>> {
        if self.pseudo != PseudoElementType::Normal {
            return None
        }

        if self.has_before_pseudo() {
            // FIXME(pcwalton): This logic looks weird. Is it right?
            match self.pseudo {
                PseudoElementType::Normal => {
                    let pseudo_before_node = self.with_pseudo(PseudoElementType::Before(self.get_before_display()));
                    return Some(pseudo_before_node)
                }
                PseudoElementType::Before(display::T::inline) => {}
                PseudoElementType::Before(_) => {
                    let pseudo_before_node = self.with_pseudo(PseudoElementType::Before(display::T::inline));
                    return Some(pseudo_before_node)
                }
                _ => {}
            }
        }

        unsafe {
            self.get_jsmanaged().first_child_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn text_content(&self) -> Vec<ContentItem> {
        if self.pseudo != PseudoElementType::Normal {
            let layout_data_ref = self.borrow_layout_data();
            let node_layout_data_wrapper = layout_data_ref.as_ref().unwrap();

            if self.pseudo.is_before() {
                let before_style = node_layout_data_wrapper.data.before_style.as_ref().unwrap();
                return get_content(&before_style.get_box().content)
            } else {
                let after_style = node_layout_data_wrapper.data.after_style.as_ref().unwrap();
                return get_content(&after_style.get_box().content)
            }
        }
        self.node.text_content()
    }
}

impl<'ln> ThreadSafeLayoutNode<'ln> {
    /// Creates a new `ThreadSafeLayoutNode` from the given `LayoutNode`.
    pub fn new<'a>(node: &LayoutNode<'a>) -> ThreadSafeLayoutNode<'a> {
        ThreadSafeLayoutNode {
            node: node.clone(),
            pseudo: PseudoElementType::Normal,
        }
    }

    /// Creates a new `ThreadSafeLayoutNode` for the same `LayoutNode`
    /// with a different pseudo-element type.
    fn with_pseudo(&self, pseudo: PseudoElementType) -> ThreadSafeLayoutNode<'ln> {
        ThreadSafeLayoutNode {
            node: self.node.clone(),
            pseudo: pseudo,
        }
    }

    pub fn debug_id(self) -> usize {
        self.node.debug_id()
    }

    pub fn flow_debug_id(self) -> usize {
        self.node.flow_debug_id()
    }

    /// Returns the next sibling of this node. Unsafe and private because this can lead to races.
    unsafe fn next_sibling(&self) -> Option<ThreadSafeLayoutNode<'ln>> {
        if self.pseudo.is_before() {
            return self.get_jsmanaged().first_child_ref().map(|node| self.new_with_this_lifetime(&node))
        }

        self.get_jsmanaged().next_sibling_ref().map(|node| self.new_with_this_lifetime(&node))
    }

    /// Returns an iterator over this node's children.
    pub fn children(&self) -> ThreadSafeLayoutNodeChildrenIterator<'ln> {
        ThreadSafeLayoutNodeChildrenIterator {
            current_node: self.first_child(),
            parent_node: Some(self.clone()),
        }
    }

    /// If this is an element, accesses the element data. Fails if this is not an element node.
    #[inline]
    pub fn as_element(&self) -> ThreadSafeLayoutElement<'ln> {
        unsafe {
            let element = match ElementCast::to_layout_js(self.get_jsmanaged()) {
                Some(e) => e.unsafe_get(),
                None => panic!("not an element")
            };
            // FIXME(pcwalton): Workaround until Rust gets multiple lifetime parameters on
            // implementations.
            ThreadSafeLayoutElement {
                element: &*element,
            }
        }
    }

    #[inline]
    pub fn get_pseudo_element_type(&self) -> PseudoElementType {
        self.pseudo
    }

    #[inline]
    pub fn get_normal_display(&self) -> display::T {
        let mut layout_data_ref = self.mutate_layout_data();
        let node_layout_data_wrapper = layout_data_ref.as_mut().unwrap();
        let style = node_layout_data_wrapper.shared_data.style.as_ref().unwrap();
        style.get_box().display
    }

    #[inline]
    pub fn get_before_display(&self) -> display::T {
        let mut layout_data_ref = self.mutate_layout_data();
        let node_layout_data_wrapper = layout_data_ref.as_mut().unwrap();
        let style = node_layout_data_wrapper.data.before_style.as_ref().unwrap();
        style.get_box().display
    }

    #[inline]
    pub fn get_after_display(&self) -> display::T {
        let mut layout_data_ref = self.mutate_layout_data();
        let node_layout_data_wrapper = layout_data_ref.as_mut().unwrap();
        let style = node_layout_data_wrapper.data.after_style.as_ref().unwrap();
        style.get_box().display
    }

    #[inline]
    pub fn has_before_pseudo(&self) -> bool {
        let layout_data_wrapper = self.borrow_layout_data();
        let layout_data_wrapper_ref = layout_data_wrapper.as_ref().unwrap();
        layout_data_wrapper_ref.data.before_style.is_some()
    }

    #[inline]
    pub fn has_after_pseudo(&self) -> bool {
        let layout_data_wrapper = self.borrow_layout_data();
        let layout_data_wrapper_ref = layout_data_wrapper.as_ref().unwrap();
        layout_data_wrapper_ref.data.after_style.is_some()
    }

    /// Borrows the layout data without checking. Fails on a conflicting borrow.
    #[inline(always)]
    fn borrow_layout_data_unchecked<'a>(&'a self) -> *const Option<LayoutDataWrapper> {
        unsafe {
            mem::transmute(self.get().layout_data_unchecked())
        }
    }

    /// Borrows the layout data immutably. Fails on a conflicting borrow.
    ///
    /// TODO(pcwalton): Make this private. It will let us avoid borrow flag checks in some cases.
    #[inline(always)]
    pub fn borrow_layout_data<'a>(&'a self) -> Ref<'a,Option<LayoutDataWrapper>> {
        unsafe {
            mem::transmute(self.get().layout_data())
        }
    }

    /// Borrows the layout data mutably. Fails on a conflicting borrow.
    ///
    /// TODO(pcwalton): Make this private. It will let us avoid borrow flag checks in some cases.
    #[inline(always)]
    pub fn mutate_layout_data<'a>(&'a self) -> RefMut<'a,Option<LayoutDataWrapper>> {
        unsafe {
            mem::transmute(self.get().layout_data_mut())
        }
    }

    /// Traverses the tree in postorder.
    ///
    /// TODO(pcwalton): Offer a parallel version with a compatible API.
    pub fn traverse_postorder_mut<T:PostorderNodeMutTraversal>(&mut self, traversal: &mut T)
                                  -> bool {
        if traversal.should_prune(self) {
            return true
        }

        let mut opt_kid = self.first_child();
        loop {
            match opt_kid {
                None => break,
                Some(mut kid) => {
                    if !kid.traverse_postorder_mut(traversal) {
                        return false
                    }
                    unsafe {
                        opt_kid = kid.next_sibling()
                    }
                }
            }
        }

        traversal.process(self)
    }

    pub fn is_ignorable_whitespace(&self) -> bool {
        unsafe {
            let text: LayoutJS<Text> = match TextCast::to_layout_js(self.get_jsmanaged()) {
                Some(text) => text,
                None => return false
            };

            if !is_whitespace(CharacterDataCast::from_layout_js(&text).data_for_layout()) {
                return false
            }

            // NB: See the rules for `white-space` here:
            //
            //    http://www.w3.org/TR/CSS21/text.html#propdef-white-space
            //
            // If you implement other values for this property, you will almost certainly
            // want to update this check.
            match self.style().get_inheritedtext().white_space {
                white_space::T::normal => true,
                _ => false,
            }
        }
    }

    pub fn get_input_value(&self) -> String {
        unsafe {
            let input: Option<LayoutJS<HTMLInputElement>> = HTMLInputElementCast::to_layout_js(self.get_jsmanaged());
            match input {
                Some(input) => input.get_value_for_layout(),
                None => panic!("not an input element!")
            }
        }
    }

    pub fn get_input_size(&self) -> u32 {
        unsafe {
            match HTMLInputElementCast::to_layout_js(self.get_jsmanaged()) {
                Some(input) => input.get_size_for_layout(),
                None => panic!("not an input element!")
            }
        }
    }

    pub fn get_unsigned_integer_attribute(self, attribute: UnsignedIntegerAttribute)
                                          -> Option<u32> {
        unsafe {
            let elem: Option<LayoutJS<Element>> = ElementCast::to_layout_js(self.get_jsmanaged());
            match elem {
                Some(element) => {
                    (*element.unsafe_get()).get_unsigned_integer_attribute_for_layout(attribute)
                }
                None => panic!("not an element!")
            }
        }
    }

    /// Get the description of how to account for recent style changes.
    /// This is a simple bitfield and fine to copy by value.
    pub fn restyle_damage(self) -> RestyleDamage {
        let layout_data_ref = self.borrow_layout_data();
        layout_data_ref.as_ref().unwrap().data.restyle_damage
    }

    /// Set the restyle damage field.
    pub fn set_restyle_damage(self, damage: RestyleDamage) {
        let mut layout_data_ref = self.mutate_layout_data();
        match &mut *layout_data_ref {
            &mut Some(ref mut layout_data) => layout_data.data.restyle_damage = damage,
            _ => panic!("no layout data for this node"),
        }
    }

    /// Returns the layout data flags for this node.
    pub fn flags(self) -> LayoutDataFlags {
        unsafe {
            match *self.borrow_layout_data_unchecked() {
                None => panic!(),
                Some(ref layout_data) => layout_data.data.flags,
            }
        }
    }

    /// Adds the given flags to this node.
    pub fn insert_flags(self, new_flags: LayoutDataFlags) {
        let mut layout_data_ref = self.mutate_layout_data();
        match &mut *layout_data_ref {
            &mut Some(ref mut layout_data) => layout_data.data.flags.insert(new_flags),
            _ => panic!("no layout data for this node"),
        }
    }

    /// Removes the given flags from this node.
    pub fn remove_flags(self, flags: LayoutDataFlags) {
        let mut layout_data_ref = self.mutate_layout_data();
        match &mut *layout_data_ref {
            &mut Some(ref mut layout_data) => layout_data.data.flags.remove(flags),
            _ => panic!("no layout data for this node"),
        }
    }

    /// Returns true if this node contributes content. This is used in the implementation of
    /// `empty_cells` per CSS 2.1 § 17.6.1.1.
    pub fn is_content(&self) -> bool {
        match self.type_id() {
            Some(NodeTypeId::Element(..)) | Some(NodeTypeId::Text(..)) => true,
            _ => false
        }
    }
}

pub struct ThreadSafeLayoutNodeChildrenIterator<'a> {
    current_node: Option<ThreadSafeLayoutNode<'a>>,
    parent_node: Option<ThreadSafeLayoutNode<'a>>,
}

impl<'a> Iterator for ThreadSafeLayoutNodeChildrenIterator<'a> {
    type Item = ThreadSafeLayoutNode<'a>;
    fn next(&mut self) -> Option<ThreadSafeLayoutNode<'a>> {
        let node = self.current_node.clone();

        match node {
            Some(ref node) => {
                if node.pseudo.is_after() {
                    return None
                }

                match self.parent_node {
                    Some(ref parent_node) => {
                        if parent_node.pseudo == PseudoElementType::Normal {
                            self.current_node = self.current_node.clone().and_then(|node| {
                                unsafe {
                                    node.next_sibling()
                                }
                            });
                        } else {
                            self.current_node = None;
                        }
                    }
                    None => {}
                }
            }
            None => {
                match self.parent_node {
                    Some(ref parent_node) => {
                        if parent_node.has_after_pseudo() {
                            let pseudo_after_node = if parent_node.pseudo == PseudoElementType::Normal {
                                let pseudo = PseudoElementType::After(parent_node.get_after_display());
                                Some(parent_node.with_pseudo(pseudo))
                            } else {
                                None
                            };
                            self.current_node = pseudo_after_node;
                            return self.current_node.clone()
                        }
                   }
                   None => {}
                }
            }
        }

        node
    }
}

/// A wrapper around elements that ensures layout can only ever access safe properties and cannot
/// race on elements.
pub struct ThreadSafeLayoutElement<'le> {
    element: &'le Element,
}

impl<'le> ThreadSafeLayoutElement<'le> {
    #[inline]
    pub fn get_attr(&self, namespace: &Namespace, name: &Atom) -> Option<&'le str> {
        unsafe {
            self.element.get_attr_val_for_layout(namespace, name)
        }
    }
}

/// A bottom-up, parallelizable traversal.
pub trait PostorderNodeMutTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process<'a>(&'a mut self, node: &ThreadSafeLayoutNode<'a>) -> bool;

    /// Returns true if this node should be pruned. If this returns true, we skip the operation
    /// entirely and do not process any descendant nodes. This is called *before* child nodes are
    /// visited. The default implementation never prunes any nodes.
    fn should_prune<'a>(&'a self, _node: &ThreadSafeLayoutNode<'a>) -> bool {
        false
    }
}

/// Opaque type stored in type-unsafe work queues for parallel layout.
/// Must be transmutable to and from LayoutNode/ThreadSafeLayoutNode.
pub type UnsafeLayoutNode = (usize, usize);

pub fn layout_node_to_unsafe_layout_node(node: &LayoutNode) -> UnsafeLayoutNode {
    unsafe {
        let ptr: usize = mem::transmute_copy(node);
        (ptr, 0)
    }
}

// FIXME(#3044): This should be updated to use a real lifetime instead of
// faking one.
pub unsafe fn layout_node_from_unsafe_layout_node(node: &UnsafeLayoutNode) -> LayoutNode<'static> {
    let (node, _) = *node;
    mem::transmute(node)
}

/// A top-down traversal.
pub trait PreorderDomTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&self, node: LayoutNode);
}

/// A bottom-up traversal, with a optional in-order pass.
pub trait PostorderDomTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&self, node: LayoutNode);
}
