/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Types and traits used to access the DOM from style calculation.

#![allow(unsafe_code)]

use context::SharedStyleContext;
use data::PrivateStyleData;
use domrefcell::DOMRefCell;
use element_state::ElementState;
use properties::{ComputedValues, PropertyDeclarationBlock};
use refcell::{Ref, RefMut};
use restyle_hints::{RESTYLE_DESCENDANTS, RESTYLE_LATER_SIBLINGS, RESTYLE_SELF, RestyleHint};
use selector_impl::{ElementExt, PseudoElement};
use selector_matching::ApplicableDeclarationBlock;
use sink::Push;
use std::fmt::Debug;
use std::ops::BitOr;
use std::sync::Arc;
use string_cache::{Atom, Namespace};

/// Opaque type stored in type-unsafe work queues for parallel layout.
/// Must be transmutable to and from TNode.
pub type UnsafeNode = (usize, usize);

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

pub trait TNode : Sized + Copy + Clone {
    type ConcreteElement: TElement<ConcreteNode = Self, ConcreteDocument = Self::ConcreteDocument>;
    type ConcreteDocument: TDocument<ConcreteNode = Self, ConcreteElement = Self::ConcreteElement>;
    type ConcreteRestyleDamage: TRestyleDamage;
    type ConcreteChildrenIterator: Iterator<Item = Self>;

    fn to_unsafe(&self) -> UnsafeNode;
    unsafe fn from_unsafe(n: &UnsafeNode) -> Self;

    /// Returns whether this is a text node. It turns out that this is all the style system cares
    /// about, and thus obviates the need to compute the full type id, which would be expensive in
    /// Gecko.
    fn is_text_node(&self) -> bool;

    fn is_element(&self) -> bool;

    fn dump(self);

    fn dump_style(self);

    /// Returns an iterator over this node's children.
    fn children(self) -> Self::ConcreteChildrenIterator;

    /// Converts self into an `OpaqueNode`.
    fn opaque(&self) -> OpaqueNode;

    /// While doing a reflow, the node at the root has no parent, as far as we're
    /// concerned. This method returns `None` at the reflow root.
    fn layout_parent_node(self, reflow_root: OpaqueNode) -> Option<Self>;

    fn debug_id(self) -> usize;

    fn as_element(&self) -> Option<Self::ConcreteElement>;

    fn as_document(&self) -> Option<Self::ConcreteDocument>;

    fn has_changed(&self) -> bool;

    unsafe fn set_changed(&self, value: bool);

    fn is_dirty(&self) -> bool;

    unsafe fn set_dirty(&self, value: bool);

    fn has_dirty_descendants(&self) -> bool;

    unsafe fn set_dirty_descendants(&self, value: bool);

    fn needs_dirty_on_viewport_size_changed(&self) -> bool;

    unsafe fn set_dirty_on_viewport_size_changed(&self);

    fn can_be_fragmented(&self) -> bool;

    unsafe fn set_can_be_fragmented(&self, value: bool);

    /// Borrows the PrivateStyleData without checks.
    #[inline(always)]
    unsafe fn borrow_data_unchecked(&self) -> Option<*const PrivateStyleData>;

    /// Borrows the PrivateStyleData immutably. Fails on a conflicting borrow.
    #[inline(always)]
    fn borrow_data(&self) -> Option<Ref<PrivateStyleData>>;

    /// Borrows the PrivateStyleData mutably. Fails on a conflicting borrow.
    #[inline(always)]
    fn mutate_data(&self) -> Option<RefMut<PrivateStyleData>>;

    /// Get the description of how to account for recent style changes.
    fn restyle_damage(self) -> Self::ConcreteRestyleDamage;

    /// Set the restyle damage field.
    fn set_restyle_damage(self, damage: Self::ConcreteRestyleDamage);

    fn parent_node(&self) -> Option<Self>;

    fn first_child(&self) -> Option<Self>;

    fn last_child(&self) -> Option<Self>;

    fn prev_sibling(&self) -> Option<Self>;

    fn next_sibling(&self) -> Option<Self>;


    /// Returns the style results for the given node. If CSS selector matching
    /// has not yet been performed, fails.
    fn style(&self, _context: &SharedStyleContext) -> Ref<Arc<ComputedValues>> {
        Ref::map(self.borrow_data().unwrap(), |data| data.style.as_ref().unwrap())
    }

    /// Removes the style from this node.
    fn unstyle(self) {
        self.mutate_data().unwrap().style = None;
    }

    /// XXX: It's a bit unfortunate we need to pass the current computed values
    /// as an argument here, but otherwise Servo would crash due to double
    /// borrows to return it.
    fn existing_style_for_restyle_damage<'a>(&'a self,
                                             current_computed_values: Option<&'a Arc<ComputedValues>>,
                                             pseudo: Option<&PseudoElement>)
        -> Option<&'a <Self::ConcreteRestyleDamage as TRestyleDamage>::PreExistingComputedValues>;
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

    fn as_node(&self) -> Self::ConcreteNode;

    fn style_attribute(&self) -> Option<&Arc<DOMRefCell<PropertyDeclarationBlock>>>;

    fn get_state(&self) -> ElementState;

    fn has_attr(&self, namespace: &Namespace, attr: &Atom) -> bool;
    fn attr_equals(&self, namespace: &Namespace, attr: &Atom, value: &Atom) -> bool;

    /// Properly marks nodes as dirty in response to restyle hints.
    fn note_restyle_hint(&self, hint: RestyleHint) {
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
            unsafe { parent.set_dirty_descendants(true); }
            curr = parent;
        }

        // Process hints.
        if hint.contains(RESTYLE_SELF) {
            unsafe { node.set_dirty(true); }
        // XXX(emilio): For now, dirty implies dirty descendants if found.
        } else if hint.contains(RESTYLE_DESCENDANTS) {
            unsafe { node.set_dirty_descendants(true); }
            let mut current = node.first_child();
            while let Some(node) = current {
                unsafe { node.set_dirty(true); }
                current = node.next_sibling();
            }
        }

        if hint.contains(RESTYLE_LATER_SIBLINGS) {
            let mut next = ::selectors::Element::next_sibling_element(self);
            while let Some(sib) = next {
                let sib_node = sib.as_node();
                unsafe { sib_node.set_dirty(true) };
                next = ::selectors::Element::next_sibling_element(&sib);
            }
        }
    }
}
