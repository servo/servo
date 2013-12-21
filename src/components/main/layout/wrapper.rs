/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A safe wrapper for DOM nodes that prevents layout from mutating the DOM, from letting DOM nodes
//! escape, and from generally doing anything that it isn't supposed to. This is accomplished via
//! a simple whitelist of allowed operations.
//!
//! As a security wrapper is only as good as its whitelist, be careful when adding operations to
//! this list. The cardinal rules are:
//!
//! (1) Layout is not allowed to mutate the DOM.
//!
//! (2) Layout is not allowed to see anything with `Abstract` in the name, because it could hang
//!     onto these objects and cause use-after-free.

use extra::url::Url;
use script::dom::element::{Element, HTMLAreaElementTypeId, HTMLAnchorElementTypeId};
use script::dom::element::{HTMLLinkElementTypeId};
use script::dom::htmliframeelement::HTMLIFrameElement;
use script::dom::htmlimageelement::HTMLImageElement;
use script::dom::namespace::Namespace;
use script::dom::node::{AbstractNode, DocumentNodeTypeId, ElementNodeTypeId, Node, NodeTypeId};
use script::dom::text::Text;
use servo_msg::constellation_msg::{PipelineId, SubpageId};
use std::cast;
use style::{PropertyDeclarationBlock, TElement, TNode};

/// A wrapper so that layout can access only the methods that it should have access to. Layout must
/// only ever see these and must never see instances of `AbstractNode`.
#[deriving(Clone, Eq)]
pub struct LayoutNode<'a> {
    /// The wrapped node.
    priv node: AbstractNode,

    /// Being chained to a value prevents `LayoutNode`s from escaping.
    priv chain: &'a (),
}

impl<'ln> LayoutNode<'ln> {
    /// Creates a new layout node, scoped to the given closure.
    pub unsafe fn with_layout_node<R>(node: AbstractNode, f: <'a> |LayoutNode<'a>| -> R) -> R {
        let heavy_iron_ball = ();
        f(LayoutNode {
            node: node,
            chain: &heavy_iron_ball,
        })
    }

    /// Creates a new layout node with the same lifetime as this layout node.
    unsafe fn new_with_this_lifetime(&self, node: AbstractNode) -> LayoutNode<'ln> {
        LayoutNode {
            node: node,
            chain: self.chain,
        }
    }

    /// Returns the interior of this node as a `Node`. This is highly unsafe for layout to call
    /// and as such is marked `unsafe`.
    pub unsafe fn get<'a>(&'a self) -> &'a Node {
        cast::transmute(self.node.node())
    }

    /// Returns the first child of this node.
    pub fn first_child(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.first_child().map(|node| self.new_with_this_lifetime(node))
        }
    }

    /// Returns the first child of this node.
    pub fn last_child(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.last_child().map(|node| self.new_with_this_lifetime(node))
        }
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

    /// Returns the type ID of this node. Fails if this node is borrowed mutably.
    pub fn type_id(&self) -> NodeTypeId {
        self.node.type_id()
    }

    /// If this is an image element, returns its URL. If this is not an image element, fails.
    ///
    /// FIXME(pcwalton): Don't copy URLs.
    pub fn image_url(&self) -> Option<Url> {
        unsafe {
            self.with_image_element(|image_element| {
                image_element.image.as_ref().map(|url| (*url).clone())
            })
        }
    }

    /// Downcasts this node to an image element and calls the given closure.
    ///
    /// FIXME(pcwalton): RAII.
    unsafe fn with_image_element<R>(self, f: |&HTMLImageElement| -> R) -> R {
        if !self.node.is_image_element() {
            fail!(~"node is not an image element");
        }
        self.node.transmute(f)
    }

    /// If this node is an iframe element, returns its pipeline and subpage IDs. If this node is
    /// not an iframe element, fails.
    pub fn iframe_pipeline_and_subpage_ids(&self) -> (PipelineId, SubpageId) {
        unsafe {
            self.with_iframe_element(|iframe_element| {
                let size = iframe_element.size.unwrap();
                (size.pipeline_id, size.subpage_id)
            })
        }
    }

    /// Downcasts this node to an iframe element and calls the given closure.
    ///
    /// FIXME(pcwalton): RAII.
    unsafe fn with_iframe_element<R>(self, f: |&HTMLIFrameElement| -> R) -> R {
        if !self.node.is_iframe_element() {
            fail!(~"node is not an iframe element");
        }
        self.node.transmute(f)
    }

    /// Returns true if this node is a text node or false otherwise.
    #[inline]
    pub fn is_text(self) -> bool {
        self.node.is_text()
    }

    /// Returns true if this node consists entirely of ignorable whitespace and false otherwise.
    /// Ignorable whitespace is defined as whitespace that would be removed per CSS 2.1 § 16.6.1.
    pub fn is_ignorable_whitespace(&self) -> bool {
        unsafe {
            self.is_text() && self.with_text(|text| text.element.data.is_whitespace())
        }
    }

    /// If this is a text node, copies out the text. If this is not a text node, fails.
    ///
    /// FIXME(pcwalton): Don't copy text. Atomically reference count instead.
    pub fn text(&self) -> ~str {
        unsafe {
            self.with_text(|text| text.element.data.to_str())
        }
    }

    /// Downcasts this node to a text node and calls the given closure.
    ///
    /// FIXME(pcwalton): RAII.
    unsafe fn with_text<R>(self, f: |&Text| -> R) -> R {
        self.node.with_imm_text(f)
    }

    /// Dumps this node tree, for debugging.
    pub fn dump(&self) {
        self.node.dump()
    }

    /// Returns a string that describes this node, for debugging.
    pub fn debug_str(&self) -> ~str {
        self.node.debug_str()
    }

    /// Traverses the tree in postorder.
    ///
    /// TODO(pcwalton): Offer a parallel version with a compatible API.
    pub fn traverse_postorder<T:PostorderNodeTraversal>(self, traversal: &T) -> bool {
        if traversal.should_prune(self) {
            return true
        }

        let mut opt_kid = self.first_child();
        loop {
            match opt_kid {
                None => break,
                Some(kid) => {
                    if !kid.traverse_postorder(traversal) {
                        return false
                    }
                    opt_kid = kid.next_sibling()
                }
            }
        }

        traversal.process(self)
    }

    /// Traverses the tree in postorder.
    ///
    /// TODO(pcwalton): Offer a parallel version with a compatible API.
    pub fn traverse_postorder_mut<T:PostorderNodeMutTraversal>(mut self, traversal: &mut T)
                                  -> bool {
        if traversal.should_prune(self) {
            return true
        }

        let mut opt_kid = self.first_child();
        loop {
            match opt_kid {
                None => break,
                Some(kid) => {
                    if !kid.traverse_postorder_mut(traversal) {
                        return false
                    }
                    opt_kid = kid.next_sibling()
                }
            }
        }

        traversal.process(self)
    }
}

impl<'ln> TNode<LayoutElement<'ln>> for LayoutNode<'ln> {
    fn parent_node(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.node().parent_node.map(|node| self.new_with_this_lifetime(node))
        }
    }

    fn prev_sibling(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.node().prev_sibling.map(|node| self.new_with_this_lifetime(node))
        }
    }
    
    fn next_sibling(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.node.node().next_sibling.map(|node| self.new_with_this_lifetime(node))
        }
    }

    fn is_element(&self) -> bool {
        match self.node.type_id() {
            ElementNodeTypeId(..) => true,
            _ => false
        }
    }

    fn is_document(&self) -> bool {
        match self.node.type_id() {
            DocumentNodeTypeId(..) => true,
            _ => false
        }
    }

    /// If this is an element, accesses the element data. Fails if this is not an element node.
    #[inline]
    fn with_element<R>(&self, f: |&LayoutElement<'ln>| -> R) -> R {
        self.node.with_imm_element(|element| {
            // FIXME(pcwalton): Workaround until Rust gets multiple lifetime parameters on
            // implementations.
            unsafe {
                f(&LayoutElement {
                    element: cast::transmute_region(element),
                })
            }
        })
    }
}

pub struct LayoutNodeChildrenIterator<'a> {
    priv current_node: Option<LayoutNode<'a>>,
}

impl<'a> Iterator<LayoutNode<'a>> for LayoutNodeChildrenIterator<'a> {
    fn next(&mut self) -> Option<LayoutNode<'a>> {
        let node = self.current_node;
        self.current_node = self.current_node.and_then(|node| {
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

/// A bottom-up, parallelizable traversal.
pub trait PostorderNodeTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process<'a>(&'a self, node: LayoutNode<'a>) -> bool;

    /// Returns true if this node should be pruned. If this returns true, we skip the operation
    /// entirely and do not process any descendant nodes. This is called *before* child nodes are
    /// visited. The default implementation never prunes any nodes.
    fn should_prune<'a>(&'a self, _node: LayoutNode<'a>) -> bool {
        false
    }
}

/// A bottom-up, parallelizable traversal.
pub trait PostorderNodeMutTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process<'a>(&'a mut self, node: LayoutNode<'a>) -> bool;

    /// Returns true if this node should be pruned. If this returns true, we skip the operation
    /// entirely and do not process any descendant nodes. This is called *before* child nodes are
    /// visited. The default implementation never prunes any nodes.
    fn should_prune<'a>(&'a self, _node: LayoutNode<'a>) -> bool {
        false
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
    fn get_local_name<'a>(&'a self) -> &'a str {
        self.element.tag_name.as_slice()
    }

    fn get_namespace_url<'a>(&'a self) -> &'a str {
        self.element.namespace.to_str().unwrap_or("")
    }

    fn get_attr(&self, ns_url: Option<~str>, name: &str) -> Option<~str> {
        let namespace = Namespace::from_str(ns_url);
        self.element.get_attr(namespace, name)
    }

    fn get_link(&self) -> Option<~str> {
        // FIXME: This is HTML only.
        match self.element.node.type_id {
            // http://www.whatwg.org/specs/web-apps/current-work/multipage/selectors.html#
            // selector-link
            ElementNodeTypeId(HTMLAnchorElementTypeId) |
            ElementNodeTypeId(HTMLAreaElementTypeId) |
            ElementNodeTypeId(HTMLLinkElementTypeId) => self.get_attr(None, "href"),
            _ => None,
        }
    }
}

