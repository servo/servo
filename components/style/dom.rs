/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Types and traits used to access the DOM from style calculation.

#![allow(unsafe_code)]
#![deny(missing_docs)]

use {Atom, Namespace, LocalName};
use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
use data::ElementData;
use element_state::ElementState;
use properties::{ComputedValues, PropertyDeclarationBlock};
use selector_parser::{ElementExt, PreExistingComputedValues, PseudoElement};
use selectors::matching::ElementSelectorFlags;
use shared_lock::Locked;
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
    /// Whether this node is an element.
    fn is_element(&self) -> bool;
    /// Whether this node is a text node.
    fn is_text_node(&self) -> bool;

    /// Whether this node needs layout.
    ///
    /// Comments, doctypes, etc are ignored by layout algorithms.
    fn needs_layout(&self) -> bool { self.is_element() || self.is_text_node() }
}

/// A node iterator that only returns node that don't need layout.
pub struct LayoutIterator<T>(pub T);

impl<T, I> Iterator for LayoutIterator<T>
    where T: Iterator<Item=I>,
          I: NodeInfo,
{
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

/// The `TNode` trait. This is the main generic trait over which the style
/// system can be implemented.
pub trait TNode : Sized + Copy + Clone + Debug + NodeInfo {
    /// The concrete `TElement` type.
    type ConcreteElement: TElement<ConcreteNode = Self>;

    /// A concrete children iterator type in order to iterate over the `Node`s.
    ///
    /// TODO(emilio): We should eventually replace this with the `impl Trait`
    /// syntax.
    type ConcreteChildrenIterator: Iterator<Item = Self>;

    /// Convert this node in an `UnsafeNode`.
    fn to_unsafe(&self) -> UnsafeNode;

    /// Get a node back from an `UnsafeNode`.
    unsafe fn from_unsafe(n: &UnsafeNode) -> Self;

    /// Returns an iterator over this node's children.
    fn children(self) -> LayoutIterator<Self::ConcreteChildrenIterator>;

    /// Converts self into an `OpaqueNode`.
    fn opaque(&self) -> OpaqueNode;

    /// Get this node's parent element if present.
    fn parent_element(&self) -> Option<Self::ConcreteElement> {
        self.parent_node().and_then(|n| n.as_element())
    }

    /// A debug id, only useful, mm... for debugging.
    fn debug_id(self) -> usize;

    /// Get this node as an element, if it's one.
    fn as_element(&self) -> Option<Self::ConcreteElement>;

    /// Whether this node needs to be laid out on viewport size change.
    fn needs_dirty_on_viewport_size_changed(&self) -> bool;

    /// Mark this node as needing layout on viewport size change.
    unsafe fn set_dirty_on_viewport_size_changed(&self);

    /// Whether this node can be fragmented. This is used for multicol, and only
    /// for Servo.
    fn can_be_fragmented(&self) -> bool;

    /// Set whether this node can be fragmented.
    unsafe fn set_can_be_fragmented(&self, value: bool);

    /// Get this node's parent node.
    fn parent_node(&self) -> Option<Self>;

    /// Whether this node is in the document right now needed to clear the
    /// restyle data appropriately on some forced restyles.
    fn is_in_doc(&self) -> bool;
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
        let values = styles.map(|s| s.primary.values());
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

/// A trait used to synthesize presentational hints for HTML element attributes.
pub trait PresentationalHintsSynthetizer {
    /// Generate the proper applicable declarations due to presentational hints,
    /// and insert them into `hints`.
    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self, hints: &mut V)
        where V: Push<ApplicableDeclarationBlock>;
}

/// The animation rules. The first one is for Animation cascade level, and the second one is for
/// Transition cascade level.
pub struct AnimationRules(pub Option<Arc<Locked<PropertyDeclarationBlock>>>,
                          pub Option<Arc<Locked<PropertyDeclarationBlock>>>);

/// The element trait, the main abstraction the style crate acts over.
pub trait TElement : PartialEq + Debug + Sized + Copy + Clone + ElementExt + PresentationalHintsSynthetizer {
    /// The concrete node type.
    type ConcreteNode: TNode<ConcreteElement = Self>;

    /// Get this element as a node.
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

    /// Get this element's style attribute.
    fn style_attribute(&self) -> Option<&Arc<Locked<PropertyDeclarationBlock>>>;

    /// Get this element's animation rules.
    fn get_animation_rules(&self, _pseudo: Option<&PseudoElement>) -> AnimationRules {
        AnimationRules(None, None)
    }

    /// Get this element's animation rule.
    fn get_animation_rule(&self, _pseudo: Option<&PseudoElement>)
                          -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        None
    }

    /// Get this element's transition rule.
    fn get_transition_rule(&self, _pseudo: Option<&PseudoElement>)
                           -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        None
    }

    /// Get this element's state, for non-tree-structural pseudos.
    fn get_state(&self) -> ElementState;

    /// Whether this element has an attribute with a given namespace.
    fn has_attr(&self, namespace: &Namespace, attr: &LocalName) -> bool;

    /// Whether an attribute value equals `value`.
    fn attr_equals(&self, namespace: &Namespace, attr: &LocalName, value: &Atom) -> bool;

    /// Get the pre-existing style to calculate restyle damage (change hints).
    ///
    /// This needs to be generic since it varies between Servo and Gecko.
    ///
    /// XXX(emilio): It's a bit unfortunate we need to pass the current computed
    /// values as an argument here, but otherwise Servo would crash due to
    /// double borrows to return it.
    fn existing_style_for_restyle_damage<'a>(&'a self,
                                             current_computed_values: &'a Arc<ComputedValues>,
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

    /// Sets selector flags, which indicate what kinds of selectors may have
    /// matched on this element and therefore what kind of work may need to
    /// be performed when DOM state changes.
    ///
    /// This is unsafe, like all the flag-setting methods, because it's only safe
    /// to call with exclusive access to the element. When setting flags on the
    /// parent during parallel traversal, we use SequentialTask to queue up the
    /// set to run after the threads join.
    unsafe fn set_selector_flags(&self, flags: ElementSelectorFlags);

    /// Returns true if the element has all the specified selector flags.
    fn has_selector_flags(&self, flags: ElementSelectorFlags) -> bool;

    /// Creates a task to update CSS Animations on a given (pseudo-)element.
    /// Note: Gecko only.
    fn update_animations(&self, _pseudo: Option<&PseudoElement>);

    /// Returns true if the element has a CSS animation.
    fn has_css_animations(&self, _pseudo: Option<&PseudoElement>) -> bool;
}

/// TNode and TElement aren't Send because we want to be careful and explicit
/// about our parallel traversal. However, there are certain situations
/// (including but not limited to the traversal) where we need to send DOM
/// objects to other threads.
///
/// That's the reason why `SendNode` exists.
#[derive(Clone, Debug, PartialEq)]
pub struct SendNode<N: TNode>(N);
unsafe impl<N: TNode> Send for SendNode<N> {}
impl<N: TNode> SendNode<N> {
    /// Unsafely construct a SendNode.
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

/// Same reason as for the existence of SendNode, SendElement does the proper
/// things for a given `TElement`.
#[derive(Debug, PartialEq)]
pub struct SendElement<E: TElement>(E);
unsafe impl<E: TElement> Send for SendElement<E> {}
impl<E: TElement> SendElement<E> {
    /// Unsafely construct a SendElement.
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
