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
use script::dom::node::{AbstractNode, DocumentNodeTypeId, ElementNodeTypeId, Node, NodeTypeId};
use script::dom::text::Text;
use servo_msg::constellation_msg::{PipelineId, SubpageId};
use servo_util::concurrentmap::{ConcurrentHashMap, ConcurrentHashMapIterator};
use servo_util::namespace;
use servo_util::namespace::Namespace;
use std::cast;
use std::cell::{Ref, RefMut};
use style::{PropertyDeclarationBlock, TElement, TNode,
            AttrSelector, SpecificNamespace, AnyNamespace};
use style::{PseudoElement, Before, After};
use layout::util::LayoutDataAccess;

use layout::util::LayoutDataWrapper;

/// Allows some convenience methods on generic layout nodes.
pub trait TLayoutNode {
    /// Creates a new layout node with the same lifetime as this layout node.
    unsafe fn new_with_this_lifetime(&self, node: AbstractNode) -> Self;

    /// Returns the type ID of this node. Fails if this node is borrowed mutably.
    fn type_id(&self) -> NodeTypeId;

    /// Returns the interior of this node as an `AbstractNode`. This is highly unsafe for layout to
    /// call and as such is marked `unsafe`.
    unsafe fn get_abstract(&self) -> AbstractNode;

    /// Returns the interior of this node as a `Node`. This is highly unsafe for layout to call
    /// and as such is marked `unsafe`.
    unsafe fn get<'a>(&'a self) -> &'a Node {
        let node = self.get_abstract();
        cast::transmute(node.node())
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
            self.with_image_element(|image_element| {
                image_element.image.as_ref().map(|url| (*url).clone())
            })
        }
    }

    /// Downcasts this node to an iframe element and calls the given closure.
    ///
    /// FIXME(pcwalton): RAII.
    unsafe fn with_iframe_element<R>(&self, f: |&HTMLIFrameElement| -> R) -> R {
        if !self.get_abstract().is_iframe_element() {
            fail!(~"node is not an iframe element");
        }
        self.get_abstract().transmute(f)
    }

    /// Downcasts this node to an image element and calls the given closure.
    ///
    /// FIXME(pcwalton): RAII.
    unsafe fn with_image_element<R>(&self, f: |&HTMLImageElement| -> R) -> R {
        if !self.get_abstract().is_image_element() {
            fail!(~"node is not an image element");
        }
        self.get_abstract().transmute(f)
    }

    /// If this node is an iframe element, returns its pipeline and subpage IDs. If this node is
    /// not an iframe element, fails.
    fn iframe_pipeline_and_subpage_ids(&self) -> (PipelineId, SubpageId) {
        unsafe {
            self.with_iframe_element(|iframe_element| {
                let size = iframe_element.size.unwrap();
                (size.pipeline_id, size.subpage_id)
            })
        }
    }

    /// If this is a text node, copies out the text. If this is not a text node, fails.
    ///
    /// FIXME(pcwalton): Don't copy text. Atomically reference count instead.
    fn text(&self) -> ~str {
        unsafe {
            self.with_text(|text| text.characterdata.data.to_str())
        }
    }

    /// Downcasts this node to a text node and calls the given closure.
    ///
    /// FIXME(pcwalton): RAII.
    unsafe fn with_text<R>(&self, f: |&Text| -> R) -> R {
        self.get_abstract().with_imm_text(f)
    }

    /// Returns true if this node is a text node or false otherwise.
    #[inline]
    fn is_text(&self) -> bool {
        unsafe { self.get_abstract().is_text() }
    }

    /// Returns the first child of this node.
    fn first_child(&self) -> Option<Self>; 

    /// Dumps this node tree, for debugging.
    fn dump(&self) {
        unsafe {
            self.get_abstract().dump()
        }
    }
}

/// A wrapper so that layout can access only the methods that it should have access to. Layout must
/// only ever see these and must never see instances of `AbstractNode`.
#[deriving(Clone, Eq)]
pub struct LayoutNode<'a> {
    /// The wrapped node.
    priv node: AbstractNode,

    /// Being chained to a value prevents `LayoutNode`s from escaping.
    priv chain: &'a (),
}

impl<'ln> TLayoutNode for LayoutNode<'ln> {
    unsafe fn new_with_this_lifetime(&self, node: AbstractNode) -> LayoutNode<'ln> {
        LayoutNode {
            node: node,
            chain: self.chain,
        }
    }
    fn type_id(&self) -> NodeTypeId {
        self.node.type_id()
    }
    unsafe fn get_abstract(&self) -> AbstractNode {
        self.node
    }
    fn first_child(&self) -> Option<LayoutNode<'ln>> {
        unsafe {
            self.get_abstract().first_child().map(|node| self.new_with_this_lifetime(node))
        }
    }
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

    /// Returns true if this node is a text node or false otherwise.
    #[inline]
    pub fn is_text(self) -> bool {
        self.node.is_text()
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
        let node = self.current_node;
        self.current_node = self.current_node.and_then(|node| {
            node.next_sibling()
        });
        node
    }
}

pub struct LayoutPseudoNode {
    /// The wrapped node.
    node: AbstractNode
}

impl LayoutPseudoNode {
    #[inline(always)]
    pub fn from_layout_pseudo(node: AbstractNode) -> LayoutPseudoNode {
        LayoutPseudoNode {
            node: node,
        }
    }
    #[inline(always)]
    pub fn get_abstract(&mut self) -> AbstractNode {
        self.node
    }
}

impl Drop for LayoutPseudoNode {
    fn drop(&mut self) {
        if self.node.is_element() {
            let _: ~Element = unsafe { cast::transmute(self.node) };
        } else if self.node.is_text() {
            let _: ~Text = unsafe { cast::transmute(self.node) };
        } else {
            fail!("LayoutPseudoNode should be element or text");
        }
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
    priv node: AbstractNode,

    /// Being chained to a value prevents `ThreadSafeLayoutNode`s from escaping.
    priv chain: &'ln (),
}

impl<'ln> TLayoutNode for ThreadSafeLayoutNode<'ln> {
    /// Creates a new layout node with the same lifetime as this layout node.
    unsafe fn new_with_this_lifetime(&self, node: AbstractNode) -> ThreadSafeLayoutNode<'ln> {
        ThreadSafeLayoutNode {
            node: node,
            chain: self.chain,
        }
    }
    fn type_id(&self) -> NodeTypeId {
        self.node.type_id()
    }
    unsafe fn get_abstract(&self) -> AbstractNode {
        self.node
    }
    fn first_child(&self) -> Option<ThreadSafeLayoutNode<'ln>> {
        unsafe {
            if self.is_pseudo_before() {
                if !self.is_first_child() && self.is_pseudo_after() {
                    return self.get_pseudo_before_node().map(|node|{
                        node.mut_node().next_sibling = self.get_pseudo_after_node();
                        self.new_with_this_lifetime(node)
                    })
                }
                return self.get_pseudo_before_node().map(|node| self.new_with_this_lifetime(node))
            }
            if self.is_pseudo_after() {
                if !self.is_first_child() {
                   return self.get_pseudo_after_node().map(|node| self.new_with_this_lifetime(node))
                }
            }
            self.get_abstract().first_child().map(|node| self.new_with_this_lifetime(node))
        }
    }
}

impl<'ln> ThreadSafeLayoutNode<'ln> {
    /// Creates a new `ThreadSafeLayoutNode` from the given `LayoutNode`.
    pub fn new<'a>(node: LayoutNode<'a>) -> ThreadSafeLayoutNode<'a> {
        ThreadSafeLayoutNode {
            node: node.node,
            chain: node.chain,
        }
    }

    pub fn to_pseudo_layout_node<'a>(node: ThreadSafeLayoutNode<'a>) -> LayoutNode<'a> {
        LayoutNode {
            node: node.node,
            chain: node.chain,
        }
    }

    pub unsafe fn parent_node(&self) -> Option<ThreadSafeLayoutNode<'ln>> {
        self.node.node().parent_node.map(|node| self.new_with_this_lifetime(node))
    }

    unsafe fn next_sibling(&self) -> Option<ThreadSafeLayoutNode<'ln>> {
        if self.is_next_after_sibling() {
            return self.get_next_after_sibling_node().map(|node| self.new_with_this_lifetime(node)) 
        } 

        self.node.node().next_sibling.map(|node| self.new_with_this_lifetime(node))
    }

    pub unsafe fn last_child(&self) -> Option<ThreadSafeLayoutNode<'ln>> {
        self.node.node().last_child.map(|node| self.new_with_this_lifetime(node))
    }

    pub fn set_parent_node(&mut self, new_parent_node: &ThreadSafeLayoutNode) {
        self.node.mut_node().parent_node = Some(new_parent_node.node);
    }

    pub fn set_first_child(&mut self, new_first_child: &ThreadSafeLayoutNode) {
        self.node.mut_node().first_child = Some(new_first_child.node);
    }

    pub fn set_last_child(&mut self, new_last_child: &ThreadSafeLayoutNode) {
        self.node.mut_node().last_child = Some(new_last_child.node);
    }

    pub fn set_prev_sibling(&mut self, new_prev_sibling: &ThreadSafeLayoutNode) {
        self.node.mut_node().prev_sibling = Some(new_prev_sibling.node);
    }

    pub fn set_next_sibling(&mut self, new_next_sibling: &ThreadSafeLayoutNode) {
        self.node.mut_node().next_sibling = Some(new_next_sibling.node);
    }

    pub fn is_first_child(&self) -> bool {
        match self.node.node().first_child {
            Some(_) => return true,
            None => return false,
        }
    }

    pub fn is_last_child(&self) -> bool {
        match self.node.node().last_child {
            Some(_) => return true,
            None => return false,
        }
    }

    pub fn is_next_after_sibling(&self) -> bool {
        let mut layout_data_ref = self.mutate_layout_data();
        let node_ldw = layout_data_ref.get().get_mut_ref();
        node_ldw.data.is_next_after_sibling()
    }

    pub fn set_next_after_sibling_node(&self, parent: AbstractNode, child: AbstractNode) {
        let mut layout_data_ref = self.mutate_layout_data();
        let node_ldw = layout_data_ref.get().get_mut_ref();
        node_ldw.data.set_next_after_sibling_node(parent, child);
    }

    pub fn get_next_after_sibling_node(&self) -> Option<AbstractNode> {
        let mut layout_data_ref = self.mutate_layout_data();
        let node_ldw = layout_data_ref.get().get_mut_ref();
        node_ldw.data.get_next_after_sibling_node()
    }

    pub fn get_pseudo_before_node(&self) -> Option<AbstractNode> {
        let mut layout_data_ref = self.mutate_layout_data();
        let node_ldw = layout_data_ref.get().get_mut_ref();
        node_ldw.data.get_pseudo_before_node()
    }

    pub fn get_pseudo_after_node(&self) -> Option<AbstractNode> {
         let mut layout_data_ref = self.mutate_layout_data();
         let node_ldw = layout_data_ref.get().get_mut_ref();
         node_ldw.data.get_pseudo_after_node()
    }

    pub fn set_pseudo_before_node(&self, parent: AbstractNode, child: AbstractNode) {
        let mut layout_data_ref = self.mutate_layout_data();
        let node_ldw = layout_data_ref.get().get_mut_ref();
        node_ldw.data.set_pseudo_before_node(parent, child);
    }

    pub fn set_pseudo_after_node(&self, parent: AbstractNode, child: AbstractNode) {
        let mut layout_data_ref = self.mutate_layout_data();
        let node_ldw = layout_data_ref.get().get_mut_ref();
        node_ldw.data.set_pseudo_after_node(parent, child);
    }

    pub fn is_pseudo_after(&self) -> bool {
        let layout_data_ref = self.borrow_layout_data();
        let node_ldw = layout_data_ref.get().get_ref();
        node_ldw.data.is_pseudo_after()
    }

    pub fn is_pseudo_before(&self) -> bool {
        let layout_data_ref = self.borrow_layout_data();
        let node_ldw = layout_data_ref.get().get_ref();
        node_ldw.data.is_pseudo_before()
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
            self.get_abstract().with_imm_element(|element| {
                // FIXME(pcwalton): Workaround until Rust gets multiple lifetime parameters on
                // implementations.
                f(&ThreadSafeLayoutElement {
                    element: cast::transmute_region(element),
                })
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
    pub fn traverse_postorder_mut<T:PostorderNodeMutTraversal>(mut self, traversal: &mut T)
                                  -> bool {
        if traversal.should_prune(self) {
            return true
        }

        let pseudo_elements = self.necessary_pseudo_elements();
        for pseudo_element in pseudo_elements.iter() {
            traversal.pseudo_element_process(self, *pseudo_element);
        }

        let mut opt_kid = self.first_child();
        loop {
            match opt_kid {
                None => break,
                Some(kid) => {
                    if !kid.traverse_postorder_mut(traversal) {
                        return false
                    }
                    unsafe {
                        opt_kid = kid.next_sibling();
                    }
                }
            }
        }

        traversal.process(self)
    }

    pub fn necessary_pseudo_elements(&self) -> ~[PseudoElement] {
        let mut pseudo_elements = ~[];

        let ldw = self.borrow_layout_data();
        let ldw_ref = ldw.get().get_ref();

        if ldw_ref.data.before_style.is_some() {
            pseudo_elements.push(Before);
        }
        if ldw_ref.data.after_style.is_some() {
            pseudo_elements.push(After);
        }

        return pseudo_elements
    }

    /// Returns true if this node is a text node or false otherwise.
    #[inline]
    pub fn is_text(self) -> bool {
        self.node.is_text()
    }
}

pub struct ThreadSafeLayoutNodeChildrenIterator<'a> {
    priv current_node: Option<ThreadSafeLayoutNode<'a>>,
}

impl<'a> Iterator<ThreadSafeLayoutNode<'a>> for ThreadSafeLayoutNodeChildrenIterator<'a> {
    fn next(&mut self) -> Option<ThreadSafeLayoutNode<'a>> {
        let node = self.current_node;
        self.current_node = self.current_node.and_then(|node| {
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
    fn process<'a>(&'a mut self, node: ThreadSafeLayoutNode<'a>) -> bool;

    fn pseudo_element_process(&mut self, node: ThreadSafeLayoutNode, kind: PseudoElement);
    /// Returns true if this node should be pruned. If this returns true, we skip the operation
    /// entirely and do not process any descendant nodes. This is called *before* child nodes are
    /// visited. The default implementation never prunes any nodes.
    fn should_prune<'a>(&'a self, _node: ThreadSafeLayoutNode<'a>) -> bool {
        false
    }
}

pub type UnsafeLayoutNode = (uint, uint);

pub fn layout_node_to_unsafe_layout_node(node: &LayoutNode) -> UnsafeLayoutNode {
    unsafe {
        cast::transmute_copy(node)
    }
}

/// Keeps track of the leaves of the DOM. This is used to efficiently start bottom-up traversals.
pub struct DomLeafSet {
    priv set: ConcurrentHashMap<UnsafeLayoutNode,()>,
}

impl DomLeafSet {
    /// Creates a new DOM leaf set.
    pub fn new() -> DomLeafSet {
        DomLeafSet {
            set: ConcurrentHashMap::with_locks_and_buckets(64, 256),
        }
    }

    /// Inserts a DOM node into the leaf set.
    pub fn insert(&self, node: &LayoutNode) {
        self.set.insert(layout_node_to_unsafe_layout_node(node), ());
    }

    /// Removes all DOM nodes from the set.
    pub fn clear(&self) {
        self.set.clear()
    }

    /// Iterates over the DOM nodes in the leaf set.
    pub fn iter<'a>(&'a self) -> ConcurrentHashMapIterator<'a,UnsafeLayoutNode,()> {
        self.set.iter()
    }
}

