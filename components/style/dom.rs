/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Types and traits used to access the DOM from style calculation.

#![allow(unsafe_code)]

use {Atom, Namespace, LocalName};
use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use data::ElementData;
use element_state::ElementState;
use parking_lot::RwLock;
use properties::{ComputedValues, PropertyDeclarationBlock};
use selector_parser::{ElementExt, PreExistingComputedValues, PseudoElement};
use sink::Push;
use std::fmt;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use stylist::ApplicableDeclarationBlock;

pub use style_traits::UnsafeNode;

/// An opaque handle to a node, which, unlike UnsafeNode, cannot be transformed
/// back into a non-opaque representation. The only safe operation that can be
/// performed on this node is to compare it to another opaque handle or to another
/// OpaqueNode.
///
/// Layout and Graphics use this to safely represent nodes for comparison purposes.
/// Because the script task's GC does not trace layout, node data cannot be safely stored in layout
/// data structures. Also, layout code tends to be faster when the DOM is not being accessed, for
/// locality reasons. Using `OpaqueNode` enforces this invariant.
#[derive(Clone, PartialEq, Copy, Debug, Hash, Eq)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
pub struct OpaqueNode(pub usize);

impl OpaqueNode {
    /// Returns the address of this node, for debugging purposes.
    #[inline]
    pub fn id(&self) -> usize {
        self.0
    }
}

/// Simple trait to provide basic information about the type of an element.
///
/// We avoid exposing the full type id, since computing it in the general case
/// would be difficult for Gecko nodes.
pub trait NodeInfo {
    fn is_element(&self) -> bool;
    fn is_text_node(&self) -> bool;

    // Comments, doctypes, etc are ignored by layout algorithms.
    fn needs_layout(&self) -> bool { self.is_element() || self.is_text_node() }
}

pub struct LayoutIterator<T>(pub T);
impl<T, I> Iterator for LayoutIterator<T> where T: Iterator<Item=I>, I: NodeInfo {
    type Item = I;
    fn next(&mut self) -> Option<I> {
        loop {
            // Filter out nodes that layout should ignore.
            let n = self.0.next();
            if n.is_none() || n.as_ref().unwrap().needs_layout() {
                return n
            }
        }
    }
}

pub trait TNode : Sized + Copy + Clone + Debug + NodeInfo {
    type ConcreteElement: TElement<ConcreteNode = Self>;
    type ConcreteChildrenIterator: Iterator<Item = Self>;

    fn to_unsafe(&self) -> UnsafeNode;
    unsafe fn from_unsafe(n: &UnsafeNode) -> Self;

    /// Returns an iterator over this node's children.
    fn children(self) -> LayoutIterator<Self::ConcreteChildrenIterator>;

    /// Converts self into an `OpaqueNode`.
    fn opaque(&self) -> OpaqueNode;

    fn parent_element(&self) -> Option<Self::ConcreteElement> {
        self.parent_node().and_then(|n| n.as_element())
    }

    fn debug_id(self) -> usize;

    fn as_element(&self) -> Option<Self::ConcreteElement>;

    fn needs_dirty_on_viewport_size_changed(&self) -> bool;

    unsafe fn set_dirty_on_viewport_size_changed(&self);

    fn can_be_fragmented(&self) -> bool;

    unsafe fn set_can_be_fragmented(&self, value: bool);

    fn parent_node(&self) -> Option<Self>;
}

/// Wrapper to output the ElementData along with the node when formatting for
/// Debug.
pub struct ShowData<N: TNode>(pub N);
impl<N: TNode> Debug for ShowData<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_with_data(f, self.0)
    }
}

/// Wrapper to output the primary computed values along with the node when
/// formatting for Debug. This is very verbose.
pub struct ShowDataAndPrimaryValues<N: TNode>(pub N);
impl<N: TNode> Debug for ShowDataAndPrimaryValues<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_with_data_and_primary_values(f, self.0)
    }
}

/// Wrapper to output the subtree rather than the single node when formatting
/// for Debug.
pub struct ShowSubtree<N: TNode>(pub N);
impl<N: TNode> Debug for ShowSubtree<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "DOM Subtree:"));
        fmt_subtree(f, &|f, n| write!(f, "{:?}", n), self.0, 1)
    }
}

/// Wrapper to output the subtree along with the ElementData when formatting
/// for Debug.
pub struct ShowSubtreeData<N: TNode>(pub N);
impl<N: TNode> Debug for ShowSubtreeData<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "DOM Subtree:"));
        fmt_subtree(f, &|f, n| fmt_with_data(f, n), self.0, 1)
    }
}

/// Wrapper to output the subtree along with the ElementData and primary
/// ComputedValues when formatting for Debug. This is extremely verbose.
pub struct ShowSubtreeDataAndPrimaryValues<N: TNode>(pub N);
impl<N: TNode> Debug for ShowSubtreeDataAndPrimaryValues<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "DOM Subtree:"));
        fmt_subtree(f, &|f, n| fmt_with_data_and_primary_values(f, n), self.0, 1)
    }
}

fn fmt_with_data<N: TNode>(f: &mut fmt::Formatter, n: N) -> fmt::Result {
    if let Some(el) = n.as_element() {
        write!(f, "{:?} dd={} data={:?}", el, el.has_dirty_descendants(), el.borrow_data())
    } else {
        write!(f, "{:?}", n)
    }
}

fn fmt_with_data_and_primary_values<N: TNode>(f: &mut fmt::Formatter, n: N) -> fmt::Result {
    if let Some(el) = n.as_element() {
        let dd = el.has_dirty_descendants();
        let data = el.borrow_data();
        let styles = data.as_ref().and_then(|d| d.get_styles());
        let values = styles.map(|s| &s.primary.values);
        write!(f, "{:?} dd={} data={:?} values={:?}", el, dd, &data, values)
    } else {
        write!(f, "{:?}", n)
    }
}

fn fmt_subtree<F, N: TNode>(f: &mut fmt::Formatter, stringify: &F, n: N, indent: u32)
                            -> fmt::Result
    where F: Fn(&mut fmt::Formatter, N) -> fmt::Result
{
    for _ in 0..indent {
        try!(write!(f, "  "));
    }
    try!(stringify(f, n));
    for kid in n.children() {
        try!(writeln!(f, ""));
        try!(fmt_subtree(f, stringify, kid, indent + 1));
    }

    Ok(())
}

pub trait PresentationalHintsSynthetizer {
    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, hints: &mut V)
        where V: Push<ApplicableDeclarationBlock>;
}

pub trait TElement : PartialEq + Debug + Sized + Copy + Clone + ElementExt + PresentationalHintsSynthetizer {
    type ConcreteNode: TNode<ConcreteElement = Self>;

    fn as_node(&self) -> Self::ConcreteNode;

    /// While doing a reflow, the element at the root has no parent, as far as we're
    /// concerned. This method returns `None` at the reflow root.
    fn layout_parent_element(self, reflow_root: OpaqueNode) -> Option<Self> {
        if self.as_node().opaque() == reflow_root {
            None
        } else {
            self.parent_element()
        }
    }

    fn style_attribute(&self) -> Option<&Arc<RwLock<PropertyDeclarationBlock>>>;

    fn get_state(&self) -> ElementState;

    fn has_attr(&self, namespace: &Namespace, attr: &LocalName) -> bool;
    fn attr_equals(&self, namespace: &Namespace, attr: &LocalName, value: &Atom) -> bool;

    /// XXX: It's a bit unfortunate we need to pass the current computed values
    /// as an argument here, but otherwise Servo would crash due to double
    /// borrows to return it.
    fn existing_style_for_restyle_damage<'a>(&'a self,
                                             current_computed_values: Option<&'a Arc<ComputedValues>>,
                                             pseudo: Option<&PseudoElement>)
                                             -> Option<&'a PreExistingComputedValues>;

    /// Returns true if this element may have a descendant needing style processing.
    ///
    /// Note that we cannot guarantee the existence of such an element, because
    /// it may have been removed from the DOM between marking it for restyle and
    /// the actual restyle traversal.
    fn has_dirty_descendants(&self) -> bool;

    /// Flag that this element has a descendant for style processing.
    ///
    /// Only safe to call with exclusive access to the element.
    unsafe fn set_dirty_descendants(&self);

    /// Flag that this element has no descendant for style processing.
    ///
    /// Only safe to call with exclusive access to the element.
    unsafe fn unset_dirty_descendants(&self);

    /// Atomically stores the number of children of this node that we will
    /// need to process during bottom-up traversal.
    fn store_children_to_process(&self, n: isize);

    /// Atomically notes that a child has been processed during bottom-up
    /// traversal. Returns the number of children left to process.
    fn did_process_child(&self) -> isize;

    /// Gets a reference to the ElementData container.
    fn get_data(&self) -> Option<&AtomicRefCell<ElementData>>;

    /// Immutably borrows the ElementData.
    fn borrow_data(&self) -> Option<AtomicRef<ElementData>> {
        self.get_data().map(|x| x.borrow())
    }

    /// Mutably borrows the ElementData.
    fn mutate_data(&self) -> Option<AtomicRefMut<ElementData>> {
        self.get_data().map(|x| x.borrow_mut())
    }

    /// Whether we should skip any root- or item-based display property
    /// blockification on this element.  (This function exists so that Gecko
    /// native anonymous content can opt out of this style fixup.)
    fn skip_root_and_item_based_display_fixup(&self) -> bool;
}

/// TNode and TElement aren't Send because we want to be careful and explicit
/// about our parallel traversal. However, there are certain situations
/// (including but not limited to the traversal) where we need to send DOM
/// objects to other threads.

#[derive(Clone, Debug, PartialEq)]
pub struct SendNode<N: TNode>(N);
unsafe impl<N: TNode> Send for SendNode<N> {}
impl<N: TNode> SendNode<N> {
    pub unsafe fn new(node: N) -> Self {
        SendNode(node)
    }
}
impl<N: TNode> Deref for SendNode<N> {
    type Target = N;
    fn deref(&self) -> &N {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct SendElement<E: TElement>(E);
unsafe impl<E: TElement> Send for SendElement<E> {}
impl<E: TElement> SendElement<E> {
    pub unsafe fn new(el: E) -> Self {
        SendElement(el)
    }
}
impl<E: TElement> Deref for SendElement<E> {
    type Target = E;
    fn deref(&self) -> &E {
        &self.0
    }
}
