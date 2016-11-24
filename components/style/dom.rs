/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Types and traits used to access the DOM from style calculation.

#![allow(unsafe_code)]

use {Atom, Namespace, LocalName};
use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use data::{ElementStyles, ElementData};
use element_state::ElementState;
use parking_lot::RwLock;
use properties::{ComputedValues, PropertyDeclarationBlock};
use selector_parser::{ElementExt, PseudoElement, RestyleDamage};
use sink::Push;
use std::fmt::Debug;
use std::ops::{BitOr, BitOrAssign};
use std::sync::Arc;
use stylist::ApplicableDeclarationBlock;
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

#[derive(Clone, Copy, Debug, PartialEq)]
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

pub trait TRestyleDamage : BitOr<Output=Self> + BitOrAssign + Copy + Debug + PartialEq {
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

    fn is_empty(&self) -> bool {
        *self == Self::empty()
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

pub trait TNode : Sized + Copy + Clone + NodeInfo {
    type ConcreteElement: TElement<ConcreteNode = Self>;
    type ConcreteChildrenIterator: Iterator<Item = Self>;

    fn to_unsafe(&self) -> UnsafeNode;
    unsafe fn from_unsafe(n: &UnsafeNode) -> Self;

    fn dump(self);

    fn dump_style(self);

    /// Returns an iterator over this node's children.
    fn children(self) -> LayoutIterator<Self::ConcreteChildrenIterator>;

    /// Converts self into an `OpaqueNode`.
    fn opaque(&self) -> OpaqueNode;

    fn layout_parent_element(self, reflow_root: OpaqueNode) -> Option<Self::ConcreteElement> {
        if self.opaque() == reflow_root {
            None
        } else {
            self.parent_node().and_then(|n| n.as_element())
        }
    }

    fn debug_id(self) -> usize;

    fn as_element(&self) -> Option<Self::ConcreteElement>;

    fn needs_dirty_on_viewport_size_changed(&self) -> bool;

    unsafe fn set_dirty_on_viewport_size_changed(&self);

    fn can_be_fragmented(&self) -> bool;

    unsafe fn set_can_be_fragmented(&self, value: bool);

    fn parent_node(&self) -> Option<Self>;
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
        -> Option<&'a <RestyleDamage as TRestyleDamage>::PreExistingComputedValues>;

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

    /// Returns true if this element's current style is display:none. Only valid
    /// to call after styling.
    fn is_display_none(&self) -> bool {
        self.borrow_data().unwrap().current_styles().is_display_none()
    }

    /// Returns true if this node has a styled layout frame that owns the style.
    fn frame_has_style(&self) -> bool { false }

    /// Returns the styles from the layout frame that owns them, if any.
    ///
    /// FIXME(bholley): Once we start dropping ElementData from nodes when
    /// creating frames, we'll want to teach this method to actually get
    /// style data from the frame.
    fn get_styles_from_frame(&self) -> Option<ElementStyles> { None }

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
            // No element data, no style on the frame.
            None if !self.frame_has_style() => Initial,
            // No element data, style on the frame.
            None => mode_for_descendants,
            // We have element data. Decide below.
            Some(d) => match *d {
                ElementData::Restyle(_) => Restyle,
                ElementData::Persistent(_) => mode_for_descendants,
                ElementData::Initial(None) => Initial,
                // We previously computed the initial style for this element
                // and then never consumed it. This is arguably a bug, since
                // it means we either styled an element unnecessarily, or missed
                // an opportunity to coalesce style traversals. However, this
                // happens now for various reasons, so we just let it slide and
                // treat it as persistent for now.
                ElementData::Initial(Some(_)) => mode_for_descendants,
            },
        }
    }

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
}
