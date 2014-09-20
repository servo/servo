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
//! 2. Layout is not allowed to see anything with `JS` in the name, because it could hang
//!    onto these objects and cause use-after-free.
//!
//! When implementing wrapper functions, be careful that you do not touch the borrow flags, or you
//! will race and cause spurious task failure. (Note that I do not believe these races are
//! exploitable, but they'll result in brokenness nonetheless.)
//!
//! Rules of the road for this file:
//!
//! * In general, you must not use the `Cast` functions; use explicit checks and `transmute_copy`
//!   instead.
//!
//! * You must also not use `.get()`; instead, use `.unsafe_get()`.
//!
//! * Do not call any methods on DOM nodes without checking to see whether they use borrow flags.
//!
//!   o Instead of `get_attr()`, use `.get_attr_val_for_layout()`.
//!
//!   o Instead of `html_element_in_html_document()`, use
//!     `html_element_in_html_document_for_layout()`.

use css::node_style::StyledNode;
use util::{LayoutDataAccess, LayoutDataWrapper, PrivateLayoutData};

use script::dom::bindings::codegen::InheritTypes::{HTMLIFrameElementDerived};
use script::dom::bindings::codegen::InheritTypes::{HTMLImageElementDerived, TextDerived};
use script::dom::bindings::js::JS;
use script::dom::element::{Element, HTMLAreaElementTypeId, HTMLAnchorElementTypeId};
use script::dom::element::{HTMLLinkElementTypeId, LayoutElementHelpers, RawLayoutElementHelpers};
use script::dom::htmliframeelement::HTMLIFrameElement;
use script::dom::htmlimageelement::{HTMLImageElement, LayoutHTMLImageElementHelpers};
use script::dom::node::{DocumentNodeTypeId, ElementNodeTypeId, Node, NodeTypeId};
use script::dom::node::{LayoutNodeHelpers, RawLayoutNodeHelpers, SharedLayoutData, TextNodeTypeId};
use script::dom::text::Text;
use script::layout_interface::LayoutChan;
use servo_msg::constellation_msg::{PipelineId, SubpageId};
use servo_util::atom::Atom;
use servo_util::namespace::Namespace;
use servo_util::namespace;
use servo_util::str::is_whitespace;
use std::cell::{RefCell, Ref, RefMut};
use std::kinds::marker::ContravariantLifetime;
use std::mem;
use style::computed_values::{content, display, white_space};
use style::{AnyNamespace, AttrSelector, PropertyDeclarationBlock, SpecificNamespace, TElement};
use style::{TNode};
use url::Url;

/// Allows some convenience methods on generic layout nodes.
pub trait TLayoutNode {
    /// Creates a new layout node with the same lifetime as this layout node.
    unsafe fn new_with_this_lifetime(&self, node: &JS<Node>) -> Self;

    /// Returns the type ID of this node. Fails if this node is borrowed mutably. Returns `None`
    /// if this is a pseudo-element; otherwise, returns `Some`.
    fn type_id(&self) -> Option<NodeTypeId>;

    /// Returns the interior of this node as a `JS`. This is highly unsafe for layout to
    /// call and as such is marked `unsafe`.
    unsafe fn get_jsmanaged<'a>(&'a self) -> &'a JS<Node>;

    /// Returns the interior of this node as a `Node`. This is highly unsafe for layout to call
    /// and as such is marked `unsafe`.
    unsafe fn get<'a>(&'a self) -> &'a Node {
        &*self.get_jsmanaged().unsafe_get()
    }

    fn node_is_element(&self) -> bool {
        match self.type_id() {
            Some(ElementNodeTypeId(..)) => true,
            _ => false
        }
    }

    fn node_is_document(&self) -> bool {
        match self.type_id() {
            Some(DocumentNodeTypeId(..)) => true,
            _ => false
        }
    }

    /// If this is an image element, returns its URL. If this is not an image element, fails.
    ///
    /// FIXME(pcwalton): Don't copy URLs.
    fn image_url(&self) -> Option<Url> {
        unsafe {
            if !self.get().is_htmlimageelement() {
                fail!("not an image!")
            }
            let image_element: JS<HTMLImageElement> = self.get_jsmanaged().transmute_copy();
            image_element.image().as_ref().map(|url| (*url).clone())
        }
    }

    /// If this node is an iframe element, returns its pipeline and subpage IDs. If this node is
    /// not an iframe element, fails.
    fn iframe_pipeline_and_subpage_ids(&self) -> (PipelineId, SubpageId) {
        unsafe {
            if !self.get().is_htmliframeelement() {
                fail!("not an iframe element!")
            }
            let iframe_element: JS<HTMLIFrameElement> = self.get_jsmanaged().transmute_copy();
            let size = (*iframe_element.unsafe_get()).size.deref().get().unwrap();
            (size.pipeline_id, size.subpage_id)
        }
    }

    /// If this is a text node, copies out the text. If this is not a text node, fails.
    ///
    /// FIXME(pcwalton): Don't copy text. Atomically reference count instead.
    fn text(&self) -> String;

    /// Returns the first child of this node.
    fn first_child(&self) -> Option<Self>;

    /// Dumps this node tree, for debugging.
    fn dump(&self) {
        // TODO(pcwalton): Reimplement this in a way that's safe for layout to call.
    }
}

/// A wrapper so that layout can access only the methods that it should have access to. Layout must
/// only ever see these and must never see instances of `JS`.
pub struct LayoutNode<'a> {
    /// The wrapped node.
    node: JS<Node>,

    /// Being chained to a ContravariantLifetime prevents `LayoutNode`s from escaping.
    pub chain: ContravariantLifetime<'a>,
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
    unsafe fn new_with_this_lifetime(&self, node: &JS<Node>) -> LayoutNode<'ln> {
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

    unsafe fn get_jsmanaged<'a>(&'a self) -> &'a JS<Node> {
        &self.node
    }

    fn first_child(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.get_jsmanaged().first_child_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn text(&self) -> String {
        unsafe {
            if !self.get().is_text() {
                fail!("not text!")
            }
            let text: JS<Text> = self.get_jsmanaged().transmute_copy();
            (*text.unsafe_get()).characterdata.data.deref().borrow().clone()
        }
    }
}

impl<'ln> LayoutNode<'ln> {
    /// Creates a new layout node, scoped to the given closure.
    pub unsafe fn with_layout_node<R>(node: JS<Node>, f: <'a> |LayoutNode<'a>| -> R) -> R {
        f(LayoutNode {
            node: node,
            chain: ContravariantLifetime,
        })
    }

    /// Iterates over this node and all its descendants, in preorder.
    ///
    /// FIXME(pcwalton): Terribly inefficient. We should use parallelism.
    pub fn traverse_preorder(&self) -> LayoutTreeIterator<'ln> {
        let mut nodes = vec!();
        gather_layout_nodes(self, &mut nodes, false);
        LayoutTreeIterator::new(nodes)
    }

    /// Returns an iterator over this node's children.
    pub fn children(&self) -> LayoutNodeChildrenIterator<'ln> {
        LayoutNodeChildrenIterator {
            current_node: self.first_child(),
        }
    }

    pub unsafe fn get_jsmanaged<'a>(&'a self) -> &'a JS<Node> {
        &self.node
    }

    /// Resets layout data and styles for the node.
    ///
    /// FIXME(pcwalton): Do this as part of fragment building instead of in a traversal.
    pub fn initialize_layout_data(&self, chan: LayoutChan) {
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
}

impl<'ln> TNode<LayoutElement<'ln>> for LayoutNode<'ln> {
    fn parent_node(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.parent_node_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn tnode_first_child(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.first_child_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn prev_sibling(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.prev_sibling_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn next_sibling(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.next_sibling_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    /// If this is an element, accesses the element data. Fails if this is not an element node.
    #[inline]
    fn as_element(&self) -> LayoutElement<'ln> {
        unsafe {
            assert!(self.node.is_element_for_layout());
            let elem: JS<Element> = self.node.transmute_copy();
            let element = &*elem.unsafe_get();
            LayoutElement {
                element: mem::transmute(element),
            }
        }
    }

    fn is_element(&self) -> bool {
        self.node_is_element()
    }

    fn is_document(&self) -> bool {
        self.node_is_document()
    }

    fn match_attr(&self, attr: &AttrSelector, test: |&str| -> bool) -> bool {
        assert!(self.is_element())
        let name = if self.is_html_element_in_html_document() {
            attr.lower_name.as_slice()
        } else {
            attr.name.as_slice()
        };
        match attr.namespace {
            SpecificNamespace(ref ns) => {
                let element = self.as_element();
                element.get_attr(ns, name)
                        .map_or(false, |attr| test(attr))
            },
            // FIXME: https://github.com/mozilla/servo/issues/1558
            AnyNamespace => false,
        }
    }

    fn is_html_element_in_html_document(&self) -> bool {
        unsafe {
            self.is_element() && {
                let element: JS<Element> = self.node.transmute_copy();
                element.html_element_in_html_document_for_layout()
            }
        }
    }
}

pub struct LayoutNodeChildrenIterator<'a> {
    current_node: Option<LayoutNode<'a>>,
}

impl<'a> Iterator<LayoutNode<'a>> for LayoutNodeChildrenIterator<'a> {
    fn next(&mut self) -> Option<LayoutNode<'a>> {
        let node = self.current_node.clone();
        self.current_node = node.clone().and_then(|node| {
            node.next_sibling()
        });
        node
    }
}

// FIXME: Do this without precomputing a vector of refs.
// Easy for preorder; harder for postorder.
//
// FIXME(pcwalton): Parallelism! Eventually this should just be nuked.
pub struct LayoutTreeIterator<'a> {
    nodes: Vec<LayoutNode<'a>>,
    index: uint,
}

impl<'a> LayoutTreeIterator<'a> {
    fn new(nodes: Vec<LayoutNode<'a>>) -> LayoutTreeIterator<'a> {
        LayoutTreeIterator {
            nodes: nodes,
            index: 0,
        }
    }
}

impl<'a> Iterator<LayoutNode<'a>> for LayoutTreeIterator<'a> {
    fn next(&mut self) -> Option<LayoutNode<'a>> {
        if self.index >= self.nodes.len() {
            None
        } else {
            let v = self.nodes[self.index].clone();
            self.index += 1;
            Some(v)
        }
    }
}

/// FIXME(pcwalton): This is super inefficient.
fn gather_layout_nodes<'a>(cur: &LayoutNode<'a>, refs: &mut Vec<LayoutNode<'a>>, postorder: bool) {
    if !postorder {
        refs.push(cur.clone());
    }
    for kid in cur.children() {
        gather_layout_nodes(&kid, refs, postorder)
    }
    if postorder {
        refs.push(cur.clone());
    }
}

/// A wrapper around elements that ensures layout can only ever access safe properties.
pub struct LayoutElement<'le> {
    element: &'le Element,
}

impl<'le> LayoutElement<'le> {
    pub fn style_attribute(&self) -> &'le Option<PropertyDeclarationBlock> {
        let style: &Option<PropertyDeclarationBlock> = unsafe {
            let style: &RefCell<Option<PropertyDeclarationBlock>> = self.element.style_attribute.deref();
            // cast to the direct reference to T placed on the head of RefCell<T>
            mem::transmute(style)
        };
        style
    }
}

impl<'le> TElement for LayoutElement<'le> {
    #[inline]
    fn get_local_name<'a>(&'a self) -> &'a Atom {
        &self.element.local_name
    }

    #[inline]
    fn get_namespace<'a>(&'a self) -> &'a Namespace {
        &self.element.namespace
    }

    #[inline]
    fn get_attr(&self, namespace: &Namespace, name: &str) -> Option<&'static str> {
        unsafe { self.element.get_attr_val_for_layout(namespace, name) }
    }

    fn get_link(&self) -> Option<&'static str> {
        // FIXME: This is HTML only.
        match self.element.node.type_id_for_layout() {
            // http://www.whatwg.org/specs/web-apps/current-work/multipage/selectors.html#
            // selector-link
            ElementNodeTypeId(HTMLAnchorElementTypeId) |
            ElementNodeTypeId(HTMLAreaElementTypeId) |
            ElementNodeTypeId(HTMLLinkElementTypeId) => {
                unsafe { self.element.get_attr_val_for_layout(&namespace::Null, "href") }
            }
            _ => None,
        }
    }

    fn get_hover_state(&self) -> bool {
        unsafe {
            self.element.node.get_hover_state_for_layout()
        }
    }

    #[inline]
    fn get_id(&self) -> Option<Atom> {
        unsafe { self.element.get_attr_atom_for_layout(&namespace::Null, "id") }
    }

    fn get_disabled_state(&self) -> bool {
        unsafe {
            self.element.node.get_disabled_state_for_layout()
        }
    }

    fn get_enabled_state(&self) -> bool {
        unsafe {
            self.element.node.get_enabled_state_for_layout()
        }
    }

    fn has_class(&self, name: &str) -> bool {
        unsafe {
            self.element.has_class_for_layout(name)
        }
    }
}

fn get_content(content_list: &content::T) -> String {
    match *content_list {
        content::Content(ref value) => {
            let iter = &mut value.clone().into_iter().peekable();
            match iter.next() {
                Some(content::StringContent(content)) => content,
                _ => "".to_string(),
            }
        }
        _ => "".to_string(),
    }
}

#[deriving(PartialEq, Clone)]
pub enum PseudoElementType {
    Normal,
    Before,
    After,
    BeforeBlock,
    AfterBlock,
}

/// A thread-safe version of `LayoutNode`, used during flow construction. This type of layout
/// node does not allow any parents or siblings of nodes to be accessed, to avoid races.
#[deriving(Clone)]
pub struct ThreadSafeLayoutNode<'ln> {
    /// The wrapped node.
    node: LayoutNode<'ln>,

    pseudo: PseudoElementType,
}

impl<'ln> TLayoutNode for ThreadSafeLayoutNode<'ln> {
    /// Creates a new layout node with the same lifetime as this layout node.
    unsafe fn new_with_this_lifetime(&self, node: &JS<Node>) -> ThreadSafeLayoutNode<'ln> {
        ThreadSafeLayoutNode {
            node: LayoutNode {
                node: node.transmute_copy(),
                chain: self.node.chain,
            },
            pseudo: Normal,
        }
    }

    /// Returns `None` if this is a pseudo-element.
    fn type_id(&self) -> Option<NodeTypeId> {
        if self.pseudo != Normal {
            return None
        }

        self.node.type_id()
    }

    unsafe fn get_jsmanaged<'a>(&'a self) -> &'a JS<Node> {
        self.node.get_jsmanaged()
    }

    unsafe fn get<'a>(&'a self) -> &'a Node { // this change.
        mem::transmute::<*mut Node,&'a Node>(self.get_jsmanaged().unsafe_get())
    }

    fn first_child(&self) -> Option<ThreadSafeLayoutNode<'ln>> {
        if self.pseudo != Normal {
            return None
        }

        if self.has_before_pseudo() {
            if self.is_block(Before) && self.pseudo == Normal {
                let pseudo_before_node = self.with_pseudo(BeforeBlock);
                return Some(pseudo_before_node)
            } else if self.pseudo == Normal || self.pseudo == BeforeBlock {
                let pseudo_before_node = self.with_pseudo(Before);
                return Some(pseudo_before_node)
            }
        }

        unsafe {
            self.get_jsmanaged().first_child_ref().map(|node| self.new_with_this_lifetime(&node))
        }
    }

    fn text(&self) -> String {
        if self.pseudo != Normal {
            let layout_data_ref = self.borrow_layout_data();
            let node_layout_data_wrapper = layout_data_ref.as_ref().unwrap();

            if self.pseudo == Before || self.pseudo == BeforeBlock {
                let before_style = node_layout_data_wrapper.data.before_style.as_ref().unwrap();
                return get_content(&before_style.get_box().content)
            } else {
                let after_style = node_layout_data_wrapper.data.after_style.as_ref().unwrap();
                return get_content(&after_style.get_box().content)
            }
        }

        unsafe {
            if !self.get().is_text() {
                fail!("not text!")
            }
            let text: JS<Text> = self.get_jsmanaged().transmute_copy();
            (*text.unsafe_get()).characterdata.data.deref().borrow().clone()
        }
    }
}

impl<'ln> ThreadSafeLayoutNode<'ln> {
    /// Creates a new `ThreadSafeLayoutNode` from the given `LayoutNode`.
    pub fn new<'a>(node: &LayoutNode<'a>) -> ThreadSafeLayoutNode<'a> {
        ThreadSafeLayoutNode {
            node: node.clone(),
            pseudo: Normal,
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

    /// Returns the next sibling of this node. Unsafe and private because this can lead to races.
    unsafe fn next_sibling(&self) -> Option<ThreadSafeLayoutNode<'ln>> {
        if self.pseudo == Before || self.pseudo == BeforeBlock {
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
    pub fn as_element(&self) -> ThreadSafeLayoutElement {
        unsafe {
            assert!(self.get_jsmanaged().is_element_for_layout());
            let elem: JS<Element> = self.get_jsmanaged().transmute_copy();
            let element = elem.unsafe_get();
            // FIXME(pcwalton): Workaround until Rust gets multiple lifetime parameters on
            // implementations.
            ThreadSafeLayoutElement {
                element: &mut *element,
            }
        }
    }

    pub fn get_pseudo_element_type(&self) ->  PseudoElementType {
        self.pseudo
    }

    pub fn is_block(&self, kind: PseudoElementType) -> bool {
        let mut layout_data_ref = self.mutate_layout_data();
        let node_layout_data_wrapper = layout_data_ref.as_mut().unwrap();

        let display = match kind {
            Before | BeforeBlock => {
                let before_style = node_layout_data_wrapper.data.before_style.as_ref().unwrap();
                before_style.get_box().display
            }
            After | AfterBlock => {
                let after_style = node_layout_data_wrapper.data.after_style.as_ref().unwrap();
                after_style.get_box().display
            }
            Normal => {
                let after_style = node_layout_data_wrapper.shared_data.style.as_ref().unwrap();
                after_style.get_box().display
            }
        };

        display == display::block
    }

    pub fn has_before_pseudo(&self) -> bool {
        let layout_data_wrapper = self.borrow_layout_data();
        let layout_data_wrapper_ref = layout_data_wrapper.as_ref().unwrap();
        layout_data_wrapper_ref.data.before_style.is_some()
    }

    pub fn has_after_pseudo(&self) -> bool {
        let layout_data_wrapper = self.borrow_layout_data();
        let layout_data_wrapper_ref = layout_data_wrapper.as_ref().unwrap();
        layout_data_wrapper_ref.data.after_style.is_some()
    }

    /// Borrows the layout data immutably. Fails on a conflicting borrow.
    #[inline(always)]
    pub fn borrow_layout_data<'a>(&'a self) -> Ref<'a,Option<LayoutDataWrapper>> {
        unsafe {
            mem::transmute(self.get().layout_data.deref().borrow())
        }
    }

    /// Borrows the layout data mutably. Fails on a conflicting borrow.
    #[inline(always)]
    pub fn mutate_layout_data<'a>(&'a self) -> RefMut<'a,Option<LayoutDataWrapper>> {
        unsafe {
            mem::transmute(self.get().layout_data.deref().borrow_mut())
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
        match self.type_id() {
            Some(TextNodeTypeId) => {
                unsafe {
                    let text: JS<Text> = self.get_jsmanaged().transmute_copy();
                    if !is_whitespace((*text.unsafe_get()).characterdata.data.deref().borrow().as_slice()) {
                        return false
                    }

                    // NB: See the rules for `white-space` here:
                    //
                    //    http://www.w3.org/TR/CSS21/text.html#propdef-white-space
                    //
                    // If you implement other values for this property, you will almost certainly
                    // want to update this check.
                    match self.style().get_inheritedtext().white_space {
                        white_space::normal => true,
                        _ => false,
                    }
                }
            }
            _ => false
        }
    }
}

pub struct ThreadSafeLayoutNodeChildrenIterator<'a> {
    current_node: Option<ThreadSafeLayoutNode<'a>>,
    parent_node: Option<ThreadSafeLayoutNode<'a>>,
}

impl<'a> Iterator<ThreadSafeLayoutNode<'a>> for ThreadSafeLayoutNodeChildrenIterator<'a> {
    fn next(&mut self) -> Option<ThreadSafeLayoutNode<'a>> {
        let node = self.current_node.clone();

        match node {
            Some(ref node) => {
                if node.pseudo == After || node.pseudo == AfterBlock {
                    return None
                }

                match self.parent_node {
                    Some(ref parent_node) => {
                        if parent_node.pseudo == Normal {
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
                            let pseudo_after_node = if parent_node.is_block(After) && parent_node.pseudo == Normal {
                                let pseudo_after_node = parent_node.with_pseudo(AfterBlock);
                                Some(pseudo_after_node)
                            } else if parent_node.pseudo == Normal {
                                let pseudo_after_node = parent_node.with_pseudo(After);
                                Some(pseudo_after_node)
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
    pub fn get_attr(&self, namespace: &Namespace, name: &str) -> Option<&'static str> {
        unsafe { self.element.get_attr_val_for_layout(namespace, name) }
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
pub type UnsafeLayoutNode = (uint, uint);

pub fn layout_node_to_unsafe_layout_node(node: &LayoutNode) -> UnsafeLayoutNode {
    unsafe {
        let ptr: uint = mem::transmute_copy(node);
        (ptr, 0)
    }
}

// FIXME(#3044): This should be updated to use a real lifetime instead of
// faking one.
pub unsafe fn layout_node_from_unsafe_layout_node(node: &UnsafeLayoutNode) -> LayoutNode<'static> {
    let (node, _) = *node;
    mem::transmute(node)
}

