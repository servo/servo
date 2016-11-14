/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Types and traits used to access the DOM from style calculation.

#![allow(unsafe_code)]

use {Atom, Namespace, LocalName};
use atomic_refcell::{AtomicRef, AtomicRefCell};
use data::{ElementStyles, ElementData};
use element_state::ElementState;
use parking_lot::RwLock;
use properties::{ComputedValues, PropertyDeclarationBlock};
use properties::longhands::display::computed_value as display;
use restyle_hints::{RESTYLE_DESCENDANTS, RESTYLE_LATER_SIBLINGS, RESTYLE_SELF, RestyleHint};
use selector_impl::{ElementExt, PseudoElement, RestyleDamage};
use selector_matching::ApplicableDeclarationBlock;
use sink::Push;
use std::fmt::Debug;
use std::ops::BitOr;
use std::sync::Arc;
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

    /// While doing a reflow, the node at the root has no parent, as far as we're
    /// concerned. This method returns `None` at the reflow root.
    fn layout_parent_element(self, reflow_root: OpaqueNode) -> Option<Self::ConcreteElement>;

    fn debug_id(self) -> usize;

    fn as_element(&self) -> Option<Self::ConcreteElement>;

    fn needs_dirty_on_viewport_size_changed(&self) -> bool;

    unsafe fn set_dirty_on_viewport_size_changed(&self);

    fn can_be_fragmented(&self) -> bool;

    unsafe fn set_can_be_fragmented(&self, value: bool);

    fn parent_node(&self) -> Option<Self>;

    fn first_child(&self) -> Option<Self>;

    fn last_child(&self) -> Option<Self>;

    fn prev_sibling(&self) -> Option<Self>;

    fn next_sibling(&self) -> Option<Self>;
}

pub trait PresentationalHintsSynthetizer {
    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, hints: &mut V)
        where V: Push<ApplicableDeclarationBlock>;
}

pub trait TElement : PartialEq + Debug + Sized + Copy + Clone + ElementExt + PresentationalHintsSynthetizer {
    type ConcreteNode: TNode<ConcreteElement = Self>;

    fn as_node(&self) -> Self::ConcreteNode;

    fn style_attribute(&self) -> Option<&Arc<RwLock<PropertyDeclarationBlock>>>;

    fn get_state(&self) -> ElementState;

    fn has_attr(&self, namespace: &Namespace, attr: &LocalName) -> bool;
    fn attr_equals(&self, namespace: &Namespace, attr: &LocalName, value: &Atom) -> bool;

    /// Set the restyle damage field.
    fn set_restyle_damage(self, damage: RestyleDamage);

    /// XXX: It's a bit unfortunate we need to pass the current computed values
    /// as an argument here, but otherwise Servo would crash due to double
    /// borrows to return it.
    fn existing_style_for_restyle_damage<'a>(&'a self,
                                             current_computed_values: Option<&'a Arc<ComputedValues>>,
                                             pseudo: Option<&PseudoElement>)
        -> Option<&'a <RestyleDamage as TRestyleDamage>::PreExistingComputedValues>;

    /// The concept of a dirty bit doesn't exist in our new restyle algorithm.
    /// Instead, we associate restyle and change hints with nodes. However, we
    /// continue to allow the dirty bit to trigger unconditional restyles while
    /// we transition both Servo and Stylo to the new architecture.
    fn deprecated_dirty_bit_is_set(&self) -> bool;

    fn has_dirty_descendants(&self) -> bool;

    unsafe fn set_dirty_descendants(&self);

    /// Atomically stores the number of children of this node that we will
    /// need to process during bottom-up traversal.
    fn store_children_to_process(&self, n: isize);

    /// Atomically notes that a child has been processed during bottom-up
    /// traversal. Returns the number of children left to process.
    fn did_process_child(&self) -> isize;

    /// Returns true if this element's current style is display:none. Only valid
    /// to call after styling.
    fn is_display_none(&self) -> bool {
        self.borrow_data().unwrap()
            .current_styles().primary
            .get_box().clone_display() == display::T::none
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

        let mut mode = match self.borrow_data() {
            // No element data, no style on the frame.
            None if !self.frame_has_style() => Initial,
            // No element data, style on the frame.
            None => mode_for_descendants,
            // We have element data. Decide below.
            Some(d) => {
                if d.has_current_styles() {
                    // The element has up-to-date style.
                    debug_assert!(!self.frame_has_style());
                    debug_assert!(d.restyle_data.is_none());
                    mode_for_descendants
                } else {
                    // The element needs processing.
                    if d.previous_styles().is_some() {
                        Restyle
                    } else {
                        Initial
                    }
                }
            },
        };

        // Handle the deprecated dirty bit. This should go away soon.
        if mode != Initial && self.deprecated_dirty_bit_is_set() {
            mode = Restyle;
        }
        mode

    }

    /// Immutable borrows the ElementData.
    fn borrow_data(&self) -> Option<AtomicRef<ElementData>>;

    /// Gets a reference to the ElementData container.
    fn get_data(&self) -> Option<&AtomicRefCell<ElementData>>;

    /// Properly marks nodes as dirty in response to restyle hints.
    fn note_restyle_hint<C: DomTraversalContext<Self::ConcreteNode>>(&self, hint: RestyleHint) {
        // Bail early if there's no restyling to do.
        if hint.is_empty() {
            return;
        }

        // If the restyle hint is non-empty, we need to restyle either this element
        // or one of its siblings. Mark our ancestor chain as having dirty descendants.
        let mut curr = *self;
        while let Some(parent) = curr.parent_element() {
            if parent.has_dirty_descendants() { break }
            unsafe { parent.set_dirty_descendants(); }
            curr = parent;
        }

        // Process hints.
        if hint.contains(RESTYLE_SELF) {
            unsafe { let _ = C::prepare_for_styling(self); }
        // XXX(emilio): For now, dirty implies dirty descendants if found.
        } else if hint.contains(RESTYLE_DESCENDANTS) {
            unsafe { self.set_dirty_descendants(); }
            let mut current = self.first_child_element();
            while let Some(el) = current {
                unsafe { let _ = C::prepare_for_styling(&el); }
                current = el.next_sibling_element();
            }
        }

        if hint.contains(RESTYLE_LATER_SIBLINGS) {
            let mut next = ::selectors::Element::next_sibling_element(self);
            while let Some(sib) = next {
                unsafe { let _ = C::prepare_for_styling(&sib); }
                next = ::selectors::Element::next_sibling_element(&sib);
            }
        }
    }
}
