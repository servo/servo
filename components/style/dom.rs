/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Types and traits used to access the DOM from style calculation.

#![allow(unsafe_code)]

use atomic_refcell::{AtomicRef, AtomicRefMut};
use data::{NodeStyles, NodeData};
use element_state::ElementState;
use parking_lot::RwLock;
use properties::{ComputedValues, PropertyDeclarationBlock};
use restyle_hints::{RESTYLE_DESCENDANTS, RESTYLE_LATER_SIBLINGS, RESTYLE_SELF, RestyleHint};
use selector_impl::{ElementExt, PseudoElement};
use selector_matching::ApplicableDeclarationBlock;
use sink::Push;
use std::fmt::Debug;
use std::ops::BitOr;
use std::sync::Arc;
use string_cache::{Atom, Namespace};
use traversal::DomTraversalContext;
use util::opts;

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

#[derive(Clone, Copy, PartialEq)]
pub enum StylingMode {
    /// The node has never been styled before, and needs a full style computation.
    Initial,
    /// The node has been styled before, but needs some amount of recomputation.
    Restyle,
    /// The node does not need any style processing, but one or more of its
    /// descendants do.
    Traverse,
    /// No nodes in this subtree require style processing.
    Stop,
}

pub trait TRestyleDamage : Debug + PartialEq + BitOr<Output=Self> + Copy {
    /// The source for our current computed values in the cascade. This is a
    /// ComputedValues in Servo and a StyleContext in Gecko.
    ///
    /// This is needed because Gecko has a few optimisations for the calculation
    /// of the difference depending on which values have been used during
    /// layout.
    ///
    /// This should be obtained via TNode::existing_style_for_restyle_damage
    type PreExistingComputedValues;

    fn compute(old: &Self::PreExistingComputedValues,
               new: &Arc<ComputedValues>) -> Self;

    fn empty() -> Self;

    fn rebuild_and_reflow() -> Self;
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

pub trait TNode : Sized + Copy + Clone + NodeInfo {
    type ConcreteElement: TElement<ConcreteNode = Self, ConcreteDocument = Self::ConcreteDocument>;
    type ConcreteDocument: TDocument<ConcreteNode = Self, ConcreteElement = Self::ConcreteElement>;
    type ConcreteChildrenIterator: Iterator<Item = Self>;

    fn to_unsafe(&self) -> UnsafeNode;
    unsafe fn from_unsafe(n: &UnsafeNode) -> Self;

    fn dump(self);

    fn dump_style(self);

    /// Returns an iterator over this node's children.
    fn children(self) -> LayoutIterator<Self::ConcreteChildrenIterator>;

    /// Converts self into an `OpaqueNode`.
    fn opaque(&self) -> OpaqueNode;

    /// While doing a reflow, the node at the root has no parent, as far as we're
    /// concerned. This method returns `None` at the reflow root.
    fn layout_parent_node(self, reflow_root: OpaqueNode) -> Option<Self>;

    fn debug_id(self) -> usize;

    fn as_element(&self) -> Option<Self::ConcreteElement>;

    fn as_document(&self) -> Option<Self::ConcreteDocument>;

    /// The concept of a dirty bit doesn't exist in our new restyle algorithm.
    /// Instead, we associate restyle and change hints with nodes. However, we
    /// continue to allow the dirty bit to trigger unconditional restyles while
    /// we transition both Servo and Stylo to the new architecture.
    fn deprecated_dirty_bit_is_set(&self) -> bool;

    fn has_dirty_descendants(&self) -> bool;

    unsafe fn set_dirty_descendants(&self);

    fn needs_dirty_on_viewport_size_changed(&self) -> bool;

    unsafe fn set_dirty_on_viewport_size_changed(&self);

    fn can_be_fragmented(&self) -> bool;

    unsafe fn set_can_be_fragmented(&self, value: bool);

    /// Atomically stores the number of children of this node that we will
    /// need to process during bottom-up traversal.
    fn store_children_to_process(&self, n: isize);

    /// Atomically notes that a child has been processed during bottom-up
    /// traversal. Returns the number of children left to process.
    fn did_process_child(&self) -> isize;

    /// Returns true if this node has a styled layout frame that owns the style.
    fn frame_has_style(&self) -> bool { false }

    /// Returns the styles from the layout frame that owns them, if any.
    ///
    /// FIXME(bholley): Once we start dropping NodeData from nodes when
    /// creating frames, we'll want to teach this method to actually get
    /// style data from the frame.
    fn get_styles_from_frame(&self) -> Option<NodeStyles> { None }

    /// Returns the styling mode for this node. This is only valid to call before
    /// and during restyling, before finish_styling is invoked.
    ///
    /// See the comments around StylingMode.
    fn styling_mode(&self) -> StylingMode {
        use self::StylingMode::*;

        // Non-incremental layout impersonates Initial.
        if opts::get().nonincremental_layout {
            return Initial;
        }

        // Compute the default result if this node doesn't require processing.
        let mode_for_descendants = if self.has_dirty_descendants() {
            Traverse
        } else {
            Stop
        };

        match self.borrow_data() {
            // No node data, no style on the frame.
            None if !self.frame_has_style() => Initial,
            // No node data, style on the frame.
            None => mode_for_descendants,
            Some(d) => {
                if d.restyle_data.is_some() || self.deprecated_dirty_bit_is_set() {
                    Restyle
                } else {
                    debug_assert!(!self.frame_has_style()); // display:none etc
                    mode_for_descendants
                }
            },
        }
    }

    /// Sets up the appropriate data structures to style a node, returing a
    /// mutable handle to the node data upon which further style calculations
    /// can be performed.
    fn begin_styling(&self) -> AtomicRefMut<NodeData>;

    /// Set the style directly for a text node. This skips various unnecessary
    /// steps from begin_styling like computing the previous style.
    fn style_text_node(&self, style: Arc<ComputedValues>);

    /// Immutable borrows the NodeData.
    fn borrow_data(&self) -> Option<AtomicRef<NodeData>>;

    fn parent_node(&self) -> Option<Self>;

    fn first_child(&self) -> Option<Self>;

    fn last_child(&self) -> Option<Self>;

    fn prev_sibling(&self) -> Option<Self>;

    fn next_sibling(&self) -> Option<Self>;
}

pub trait TDocument : Sized + Copy + Clone {
    type ConcreteNode: TNode<ConcreteElement = Self::ConcreteElement, ConcreteDocument = Self>;
    type ConcreteElement: TElement<ConcreteNode = Self::ConcreteNode, ConcreteDocument = Self>;

    fn as_node(&self) -> Self::ConcreteNode;

    fn root_node(&self) -> Option<Self::ConcreteNode>;

    fn drain_modified_elements(&self) -> Vec<(Self::ConcreteElement,
                                              <Self::ConcreteElement as ElementExt>::Snapshot)>;

    fn needs_paint_from_layout(&self);
    fn will_paint(&self);
}

pub trait PresentationalHintsSynthetizer {
    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, hints: &mut V)
        where V: Push<ApplicableDeclarationBlock>;
}

pub trait TElement : PartialEq + Debug + Sized + Copy + Clone + ElementExt + PresentationalHintsSynthetizer {
    type ConcreteNode: TNode<ConcreteElement = Self, ConcreteDocument = Self::ConcreteDocument>;
    type ConcreteDocument: TDocument<ConcreteNode = Self::ConcreteNode, ConcreteElement = Self>;
    type ConcreteRestyleDamage: TRestyleDamage;

    fn as_node(&self) -> Self::ConcreteNode;

    fn style_attribute(&self) -> Option<&Arc<RwLock<PropertyDeclarationBlock>>>;

    fn get_state(&self) -> ElementState;

    fn has_attr(&self, namespace: &Namespace, attr: &Atom) -> bool;
    fn attr_equals(&self, namespace: &Namespace, attr: &Atom, value: &Atom) -> bool;

    /// Set the restyle damage field.
    fn set_restyle_damage(self, damage: Self::ConcreteRestyleDamage);

    /// XXX: It's a bit unfortunate we need to pass the current computed values
    /// as an argument here, but otherwise Servo would crash due to double
    /// borrows to return it.
    fn existing_style_for_restyle_damage<'a>(&'a self,
                                             current_computed_values: Option<&'a Arc<ComputedValues>>,
                                             pseudo: Option<&PseudoElement>)
        -> Option<&'a <Self::ConcreteRestyleDamage as TRestyleDamage> ::PreExistingComputedValues>;

    /// Properly marks nodes as dirty in response to restyle hints.
    fn note_restyle_hint<C: DomTraversalContext<Self::ConcreteNode>>(&self, hint: RestyleHint) {
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
            unsafe { parent.set_dirty_descendants(); }
            curr = parent;
        }

        // Process hints.
        if hint.contains(RESTYLE_SELF) {
            unsafe { C::ensure_node_data(&node).borrow_mut().ensure_restyle_data(); }
        // XXX(emilio): For now, dirty implies dirty descendants if found.
        } else if hint.contains(RESTYLE_DESCENDANTS) {
            unsafe { node.set_dirty_descendants(); }
            let mut current = node.first_child();
            while let Some(node) = current {
                unsafe { C::ensure_node_data(&node).borrow_mut().ensure_restyle_data(); }
                current = node.next_sibling();
            }
        }

        if hint.contains(RESTYLE_LATER_SIBLINGS) {
            let mut next = ::selectors::Element::next_sibling_element(self);
            while let Some(sib) = next {
                let sib_node = sib.as_node();
                unsafe { C::ensure_node_data(&sib_node).borrow_mut().ensure_restyle_data() };
                next = ::selectors::Element::next_sibling_element(&sib);
            }
        }
    }
}
