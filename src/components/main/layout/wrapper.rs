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
//! exploitable, but they'll result in brokenness nonetheless.) In general, you must not use the
//! `Cast` functions; use explicit checks and `transmute_copy` instead. You must also not use
//! `.get()`; instead, use `.unsafe_get()`.

use extra::url::Url;
use script::dom::bindings::codegen::InheritTypes::{ElementDerived, HTMLIFrameElementDerived};
use script::dom::bindings::codegen::InheritTypes::{HTMLImageElementDerived, TextDerived};
use script::dom::bindings::js::JS;
use script::dom::element::{Element, HTMLAreaElementTypeId, HTMLAnchorElementTypeId};
use script::dom::element::{HTMLLinkElementTypeId};
use script::dom::htmliframeelement::HTMLIFrameElement;
use script::dom::htmlimageelement::HTMLImageElement;
use script::dom::node::{DocumentNodeTypeId, ElementNodeTypeId, Node, NodeTypeId, NodeHelpers};
use script::dom::text::Text;
use servo_msg::constellation_msg::{PipelineId, SubpageId};
use servo_util::namespace;
use servo_util::namespace::Namespace;
use std::cast;
use std::cell::{Ref, RefMut};
use style::{PropertyDeclarationBlock, TElement, TNode, AttrSelector, SpecificNamespace};
use style::{AnyNamespace};

use layout::util::LayoutDataWrapper;

/// Allows some convenience methods on generic layout nodes.
pub trait TLayoutNode {
    /// Creates a new layout node with the same lifetime as this layout node.
    unsafe fn new_with_this_lifetime(&self, node: &JS<Node>) -> Self;

    /// Returns the type ID of this node. Fails if this node is borrowed mutably.
    fn type_id(&self) -> NodeTypeId;

    /// Returns the interior of this node as a `JS`. This is highly unsafe for layout to
    /// call and as such is marked `unsafe`.
    unsafe fn get_jsmanaged<'a>(&'a self) -> &'a JS<Node>;

    /// Returns the interior of this node as a `Node`. This is highly unsafe for layout to call
    /// and as such is marked `unsafe`.
    unsafe fn get<'a>(&'a self) -> &'a Node {
        cast::transmute::<*mut Node,&'a Node>(self.get_jsmanaged().unsafe_get())
    }

    fn node_is_element(&self) -> bool {
        match self.type_id() {
            ElementNodeTypeId(..) => true,
            _ => false
        }
    }

    fn node_is_document(&self) -> bool {
        match self.type_id() {
            DocumentNodeTypeId(..) => true,
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
            (*image_element.unsafe_get()).extra.image.as_ref().map(|url| (*url).clone())
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
            let size = (*iframe_element.unsafe_get()).size.unwrap();
            (size.pipeline_id, size.subpage_id)
        }
    }

    /// If this is a text node, copies out the text. If this is not a text node, fails.
    ///
    /// FIXME(pcwalton): Don't copy text. Atomically reference count instead.
    fn text(&self) -> ~str {
        unsafe {
            if !self.get().is_text() {
                fail!("not text!")
            }
            let text: JS<Text> = self.get_jsmanaged().transmute_copy();
            (*text.unsafe_get()).characterdata.data.to_str()
        }
    }

    /// Returns the first child of this node.
    fn first_child(&self) -> Option<Self> {
        unsafe {
            self.get().first_child_ref().map(|node| self.new_with_this_lifetime(node))
        }
    }

    /// Dumps this node tree, for debugging.
    fn dump(&self) {
        // TODO(pcwalton): Reimplement this in a way that's safe for layout to call.
    }
}

/// A wrapper so that layout can access only the methods that it should have access to. Layout must
/// only ever see these and must never see instances of `JS`.
#[deriving(Clone, Eq)]
pub struct LayoutNode<'a> {
    /// The wrapped node.
    priv node: JS<Node>,

    /// Being chained to a value prevents `LayoutNode`s from escaping.
    priv chain: &'a (),
}

impl<'ln> TLayoutNode for LayoutNode<'ln> {
    unsafe fn new_with_this_lifetime(&self, node: &JS<Node>) -> LayoutNode<'ln> {
        LayoutNode {
            node: node.transmute_copy(),
            chain: self.chain,
        }
    }
    fn type_id(&self) -> NodeTypeId {
        self.node.type_id()
    }
    unsafe fn get_jsmanaged<'a>(&'a self) -> &'a JS<Node> {
        &self.node
    }
}

impl<'ln> LayoutNode<'ln> {
    /// Creates a new layout node, scoped to the given closure.
    pub unsafe fn with_layout_node<R>(node: JS<Node>, f: <'a> |LayoutNode<'a>| -> R) -> R {
        let heavy_iron_ball = ();
        f(LayoutNode {
            node: node,
            chain: &heavy_iron_ball,
        })
    }

    /// Iterates over this node and all its descendants, in preorder.
    ///
    /// FIXME(pcwalton): Terribly inefficient. We should use parallelism.
    pub fn traverse_preorder(&self) -> LayoutTreeIterator<'ln> {
        let mut nodes = ~[];
        gather_layout_nodes(self, &mut nodes, false);
        LayoutTreeIterator::new(nodes)
    }

    /// Returns an iterator over this node's children.
    pub fn children(&self) -> LayoutNodeChildrenIterator<'ln> {
        LayoutNodeChildrenIterator {
            current_node: self.first_child(),
        }
    }
}

impl<'ln> TNode<LayoutElement<'ln>> for LayoutNode<'ln> {
    fn parent_node(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.get().parent_node_ref().map(|node| self.new_with_this_lifetime(node))
        }
    }

    fn prev_sibling(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.get().prev_sibling_ref().map(|node| self.new_with_this_lifetime(node))
        }
    }

    fn next_sibling(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.get().next_sibling_ref().map(|node| self.new_with_this_lifetime(node))
        }
    }

    /// If this is an element, accesses the element data. Fails if this is not an element node.
    #[inline]
    fn with_element<R>(&self, f: |&LayoutElement<'ln>| -> R) -> R {
        unsafe {
            if !self.node.is_element() {
                fail!("not an element!")
            }
            let elem: JS<Element> = self.node.transmute_copy();
            let element = elem.get();
            f(&LayoutElement {
                element: cast::transmute_region(element),
            })
        }
    }

    fn is_element(&self) -> bool {
        self.node_is_element()
    }

    fn is_document(&self) -> bool {
        self.node_is_document()
    }

    fn match_attr(&self, attr: &AttrSelector, test: |&str| -> bool) -> bool {
        self.with_element(|element| {
            let name = if element.element.html_element_in_html_document() {
                attr.lower_name.as_slice()
            } else {
                attr.name.as_slice()
            };
            match attr.namespace {
                SpecificNamespace(ref ns) => {
                    element.get_attr(ns, name)
                           .map_default(false, |attr| test(attr))
                },
                // FIXME: https://github.com/mozilla/servo/issues/1558
                AnyNamespace => false,
            }
        })
    }
}

pub struct LayoutNodeChildrenIterator<'a> {
    priv current_node: Option<LayoutNode<'a>>,
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
    priv nodes: ~[LayoutNode<'a>],
    priv index: uint,
}

impl<'a> LayoutTreeIterator<'a> {
    fn new(nodes: ~[LayoutNode<'a>]) -> LayoutTreeIterator<'a> {
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
fn gather_layout_nodes<'a>(cur: &LayoutNode<'a>, refs: &mut ~[LayoutNode<'a>], postorder: bool) {
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
    priv element: &'le Element,
}

impl<'le> LayoutElement<'le> {
    pub fn style_attribute(&self) -> &'le Option<PropertyDeclarationBlock> {
        &self.element.style_attribute
    }
}

impl<'le> TElement for LayoutElement<'le> {
    #[inline]
    fn get_local_name<'a>(&'a self) -> &'a str {
        self.element.tag_name.as_slice()
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
        match self.element.node.type_id {
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
        self.element.node.get_hover_state()
    }
}

/// A thread-safe version of `LayoutNode`, used during flow construction. This type of layout
/// node does not allow any parents or siblings of nodes to be accessed, to avoid races.
pub struct ThreadSafeLayoutNode<'ln> {
    /// The wrapped node.
    priv node: JS<Node>,

    /// Being chained to a value prevents `ThreadSafeLayoutNode`s from escaping.
    priv chain: &'ln (),
}

impl<'ln> TLayoutNode for ThreadSafeLayoutNode<'ln> {
    /// Creates a new layout node with the same lifetime as this layout node.
    unsafe fn new_with_this_lifetime(&self, node: &JS<Node>) -> ThreadSafeLayoutNode<'ln> {
        ThreadSafeLayoutNode {
            node: node.transmute_copy(),
            chain: self.chain,
        }
    }
    fn type_id(&self) -> NodeTypeId {
        self.node.type_id()
    }
    unsafe fn get_jsmanaged<'a>(&'a self) -> &'a JS<Node> {
        &self.node
    }
}

impl<'ln> Clone for ThreadSafeLayoutNode<'ln> {
    fn clone(&self) -> ThreadSafeLayoutNode<'ln> {
        ThreadSafeLayoutNode {
            node: self.node.clone(),
            chain: self.chain,
        }
    }
}

impl<'ln> ThreadSafeLayoutNode<'ln> {
    /// Creates a new `ThreadSafeLayoutNode` from the given `LayoutNode`.
    pub fn new<'a>(node: &LayoutNode<'a>) -> ThreadSafeLayoutNode<'a> {
        ThreadSafeLayoutNode {
            node: node.node.clone(),
            chain: node.chain,
        }
    }

    /// Returns the next sibling of this node. Unsafe and private because this can lead to races.
    unsafe fn next_sibling(&self) -> Option<ThreadSafeLayoutNode<'ln>> {
        self.node.get().next_sibling_ref().map(|node| self.new_with_this_lifetime(node))
    }

    /// Returns an iterator over this node's children.
    pub fn children(&self) -> ThreadSafeLayoutNodeChildrenIterator<'ln> {
        ThreadSafeLayoutNodeChildrenIterator {
            current_node: self.first_child(),
        }
    }

    /// If this is an element, accesses the element data. Fails if this is not an element node.
    #[inline]
    pub fn with_element<R>(&self, f: |&ThreadSafeLayoutElement| -> R) -> R {
        unsafe {
            if !self.node.is_element() {
                fail!("not an element!")
            }
            let elem: JS<Element> = self.node.transmute_copy();
            let element = elem.unsafe_get();
            // FIXME(pcwalton): Workaround until Rust gets multiple lifetime parameters on
            // implementations.
            f(&ThreadSafeLayoutElement {
                element: cast::transmute::<*mut Element,&mut Element>(element),
            })
        }
    }

    /// Borrows the layout data immutably. Fails on a conflicting borrow.
    #[inline(always)]
    pub fn borrow_layout_data<'a>(&'a self) -> Ref<'a,Option<LayoutDataWrapper>> {
        unsafe {
            cast::transmute(self.get().layout_data.borrow())
        }
    }

    /// Borrows the layout data mutably. Fails on a conflicting borrow.
    #[inline(always)]
    pub fn mutate_layout_data<'a>(&'a self) -> RefMut<'a,Option<LayoutDataWrapper>> {
        unsafe {
            cast::transmute(self.get().layout_data.borrow_mut())
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
}

pub struct ThreadSafeLayoutNodeChildrenIterator<'a> {
    priv current_node: Option<ThreadSafeLayoutNode<'a>>,
}

impl<'a> Iterator<ThreadSafeLayoutNode<'a>> for ThreadSafeLayoutNodeChildrenIterator<'a> {
    fn next(&mut self) -> Option<ThreadSafeLayoutNode<'a>> {
        let node = self.current_node.clone();
        self.current_node = self.current_node.clone().and_then(|node| {
            unsafe {
                node.next_sibling()
            }
        });
        node
    }
}

/// A wrapper around elements that ensures layout can only ever access safe properties and cannot
/// race on elements.
pub struct ThreadSafeLayoutElement<'le> {
    priv element: &'le Element,
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
/// Must be transmutable to and from LayoutNode/ThreadsafeLayoutNode/PaddedUnsafeFlow.
pub type UnsafeLayoutNode = (uint, uint, uint);

pub fn layout_node_to_unsafe_layout_node(node: &LayoutNode) -> UnsafeLayoutNode {
    unsafe {
        cast::transmute_copy(node)
    }
}

