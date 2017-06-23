/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Types and traits used to access the DOM from style calculation.

#![allow(unsafe_code)]
#![deny(missing_docs)]

use {Atom, Namespace, LocalName};
use applicable_declarations::ApplicableDeclarationBlock;
use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
#[cfg(feature = "gecko")] use context::UpdateAnimationsTasks;
use data::ElementData;
use element_state::ElementState;
use font_metrics::FontMetricsProvider;
use media_queries::Device;
use properties::{ComputedValues, PropertyDeclarationBlock};
#[cfg(feature = "gecko")] use properties::animated_properties::AnimationValue;
#[cfg(feature = "gecko")] use properties::animated_properties::TransitionProperty;
use rule_tree::CascadeLevel;
use selector_parser::{AttrValue, ElementExt, PreExistingComputedValues};
use selector_parser::{PseudoClassStringArg, PseudoElement};
use selectors::matching::{ElementSelectorFlags, VisitedHandlingMode};
use selectors::sink::Push;
use shared_lock::Locked;
use smallvec::VecLike;
use std::fmt;
#[cfg(feature = "gecko")] use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use stylearc::Arc;
use thread_state;

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

    /// Get this node's parent node.
    fn parent_node(&self) -> Option<Self>;

    /// Get this node's parent element if present.
    fn parent_element(&self) -> Option<Self::ConcreteElement> {
        self.parent_node().and_then(|n| n.as_element())
    }

    /// Returns an iterator over this node's children.
    fn children(&self) -> LayoutIterator<Self::ConcreteChildrenIterator>;

    /// Get this node's parent element from the perspective of a restyle
    /// traversal.
    fn traversal_parent(&self) -> Option<Self::ConcreteElement>;

    /// Get this node's children from the perspective of a restyle traversal.
    fn traversal_children(&self) -> LayoutIterator<Self::ConcreteChildrenIterator>;

    /// Returns whether `children()` and `traversal_children()` might return
    /// iterators over different nodes.
    fn children_and_traversal_children_might_differ(&self) -> bool;

    /// Converts self into an `OpaqueNode`.
    fn opaque(&self) -> OpaqueNode;

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
        writeln!(f, "DOM Subtree:")?;
        fmt_subtree(f, &|f, n| write!(f, "{:?}", n), self.0, 1)
    }
}

/// Wrapper to output the subtree along with the ElementData when formatting
/// for Debug.
pub struct ShowSubtreeData<N: TNode>(pub N);
impl<N: TNode> Debug for ShowSubtreeData<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "DOM Subtree:")?;
        fmt_subtree(f, &|f, n| fmt_with_data(f, n), self.0, 1)
    }
}

/// Wrapper to output the subtree along with the ElementData and primary
/// ComputedValues when formatting for Debug. This is extremely verbose.
pub struct ShowSubtreeDataAndPrimaryValues<N: TNode>(pub N);
impl<N: TNode> Debug for ShowSubtreeDataAndPrimaryValues<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "DOM Subtree:")?;
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
        let values = data.as_ref().and_then(|d| d.styles.get_primary());
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
        write!(f, "  ")?;
    }
    stringify(f, n)?;
    for kid in n.traversal_children() {
        writeln!(f, "")?;
        fmt_subtree(f, stringify, kid, indent + 1)?;
    }

    Ok(())
}

/// Flag that this element has a descendant for style processing, propagating
/// the bit up to the root as needed.
///
/// This is _not_ safe to call during the parallel traversal.
///
/// This is intended as a helper so Servo and Gecko can override it with custom
/// stuff if needed.
///
/// Returns whether no parent had already noted it, that is, whether we reached
/// the root during the walk up.
pub unsafe fn raw_note_descendants<E, B>(element: E) -> bool
    where E: TElement,
          B: DescendantsBit<E>,
{
    debug_assert!(!thread_state::get().is_worker());
    // TODO(emilio, bholley): Documenting the flags setup a bit better wouldn't
    // really hurt I guess.
    debug_assert!(element.get_data().is_some(),
                  "You should ensure you only flag styled elements");

    let mut curr = Some(element);
    while let Some(el) = curr {
        if B::has(el) {
            break;
        }
        B::set(el);
        curr = el.traversal_parent();
    }

    // Note: We disable this assertion on servo because of bugs. See the
    // comment around note_dirty_descendant in layout/wrapper.rs.
    if cfg!(feature = "gecko") {
        debug_assert!(element.descendants_bit_is_propagated::<B>());
    }

    curr.is_none()
}

/// A trait used to synthesize presentational hints for HTML element attributes.
pub trait PresentationalHintsSynthesizer {
    /// Generate the proper applicable declarations due to presentational hints,
    /// and insert them into `hints`.
    fn synthesize_presentational_hints_for_legacy_attributes<V>(&self,
                                                                visited_handling: VisitedHandlingMode,
                                                                hints: &mut V)
        where V: Push<ApplicableDeclarationBlock>;
}

/// The element trait, the main abstraction the style crate acts over.
pub trait TElement : Eq + PartialEq + Debug + Hash + Sized + Copy + Clone +
                     ElementExt + PresentationalHintsSynthesizer {
    /// The concrete node type.
    type ConcreteNode: TNode<ConcreteElement = Self>;

    /// Type of the font metrics provider
    ///
    /// XXXManishearth It would be better to make this a type parameter on
    /// ThreadLocalStyleContext and StyleContext
    type FontMetricsProvider: FontMetricsProvider;

    /// Get this element as a node.
    fn as_node(&self) -> Self::ConcreteNode;

    /// A debug-only check that the device's owner doc matches the actual doc
    /// we're the root of.
    ///
    /// Otherwise we may set document-level state incorrectly, like the root
    /// font-size used for rem units.
    fn owner_doc_matches_for_testing(&self, _: &Device) -> bool { true }

    /// Returns the depth of this element in the DOM.
    fn depth(&self) -> usize {
        let mut depth = 0;
        let mut curr = *self;
        while let Some(parent) = curr.traversal_parent() {
            depth += 1;
            curr = parent;
        }

        depth
    }

    /// Get this node's parent element from the perspective of a restyle
    /// traversal.
    fn traversal_parent(&self) -> Option<Self> {
        self.as_node().traversal_parent()
    }

    /// Returns the parent element we should inherit from.
    ///
    /// This is pretty much always the parent element itself, except in the case
    /// of Gecko's Native Anonymous Content, which uses the traversal parent
    /// (i.e. the flattened tree parent) and which also may need to find the
    /// closest non-NAC ancestor.
    fn inheritance_parent(&self) -> Option<Self> {
        self.parent_element()
    }

    /// The ::before pseudo-element of this element, if it exists.
    fn before_pseudo_element(&self) -> Option<Self> {
        None
    }

    /// The ::after pseudo-element of this element, if it exists.
    fn after_pseudo_element(&self) -> Option<Self> {
        None
    }

    /// Execute `f` for each anonymous content child (apart from ::before and
    /// ::after) whose originating element is `self`.
    fn each_anonymous_content_child<F>(&self, _f: F)
    where
        F: FnMut(Self),
    {}

    /// For a given NAC element, return the closest non-NAC ancestor, which is
    /// guaranteed to exist.
    fn closest_non_native_anonymous_ancestor(&self) -> Option<Self> {
        unreachable!("Servo doesn't know about NAC");
    }

    /// Get this element's style attribute.
    fn style_attribute(&self) -> Option<&Arc<Locked<PropertyDeclarationBlock>>>;

    /// Unset the style attribute's dirty bit.
    /// Servo doesn't need to manage ditry bit for style attribute.
    fn unset_dirty_style_attribute(&self) {
    }

    /// Get this element's SMIL override declarations.
    fn get_smil_override(&self) -> Option<&Arc<Locked<PropertyDeclarationBlock>>> {
        None
    }

    /// Get this element's animation rule by the cascade level.
    fn get_animation_rule_by_cascade(&self,
                                     _cascade_level: CascadeLevel)
                                     -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        None
    }

    /// Get this element's animation rule.
    fn get_animation_rule(&self)
                          -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        None
    }

    /// Get this element's transition rule.
    fn get_transition_rule(&self)
                           -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        None
    }

    /// Get this element's state, for non-tree-structural pseudos.
    fn get_state(&self) -> ElementState;

    /// Whether this element has an attribute with a given namespace.
    fn has_attr(&self, namespace: &Namespace, attr: &LocalName) -> bool;

    /// The ID for this element.
    fn get_id(&self) -> Option<Atom>;

    /// Internal iterator for the classes of this element.
    fn each_class<F>(&self, callback: F) where F: FnMut(&Atom);

    /// Get the pre-existing style to calculate restyle damage (change hints).
    ///
    /// This needs to be generic since it varies between Servo and Gecko.
    ///
    /// XXX(emilio): It's a bit unfortunate we need to pass the current computed
    /// values as an argument here, but otherwise Servo would crash due to
    /// double borrows to return it.
    fn existing_style_for_restyle_damage<'a>(&'a self,
                                             current_computed_values: &'a ComputedValues,
                                             pseudo: Option<&PseudoElement>)
                                             -> Option<&'a PreExistingComputedValues>;

    /// Whether a given element may generate a pseudo-element.
    ///
    /// This is useful to avoid computing, for example, pseudo styles for
    /// `::-first-line` or `::-first-letter`, when we know it won't affect us.
    ///
    /// TODO(emilio, bz): actually implement the logic for it.
    fn may_generate_pseudo(
        &self,
        _pseudo: &PseudoElement,
        _primary_style: &ComputedValues,
    ) -> bool {
        true
    }

    /// Returns true if this element may have a descendant needing style processing.
    ///
    /// Note that we cannot guarantee the existence of such an element, because
    /// it may have been removed from the DOM between marking it for restyle and
    /// the actual restyle traversal.
    fn has_dirty_descendants(&self) -> bool;

    /// Returns whether state or attributes that may change style have changed
    /// on the element, and thus whether the element has been snapshotted to do
    /// restyle hint computation.
    fn has_snapshot(&self) -> bool;

    /// Returns whether the current snapshot if present has been handled.
    fn handled_snapshot(&self) -> bool;

    /// Flags this element as having handled already its snapshot.
    unsafe fn set_handled_snapshot(&self);

    /// Returns whether the element's styles are up-to-date.
    fn has_current_styles(&self, data: &ElementData) -> bool {
        if self.has_snapshot() && !self.handled_snapshot() {
            return false;
        }

        data.has_styles() && !data.has_invalidations()
    }

    /// Flags an element and its ancestors with a given `DescendantsBit`.
    ///
    /// TODO(emilio): We call this conservatively from restyle_element_internal
    /// because we never flag unstyled stuff. A different setup for this may be
    /// a bit cleaner, but it's probably not worth to invest on it right now
    /// unless necessary.
    unsafe fn note_descendants<B: DescendantsBit<Self>>(&self);

    /// Flag that this element has a descendant for style processing.
    ///
    /// Only safe to call with exclusive access to the element.
    unsafe fn set_dirty_descendants(&self);

    /// Debug helper to be sure the bit is propagated.
    fn descendants_bit_is_propagated<B: DescendantsBit<Self>>(&self) -> bool {
        let mut current = Some(*self);
        while let Some(el) = current {
            if !B::has(el) { return false; }
            current = el.traversal_parent();
        }

        true
    }

    /// Flag that this element has no descendant for style processing.
    ///
    /// Only safe to call with exclusive access to the element.
    unsafe fn unset_dirty_descendants(&self);

    /// Similar to the dirty_descendants but for representing a descendant of
    /// the element needs to be updated in animation-only traversal.
    fn has_animation_only_dirty_descendants(&self) -> bool {
        false
    }

    /// Flag that this element has a descendant for animation-only restyle
    /// processing.
    ///
    /// Only safe to call with exclusive access to the element.
    unsafe fn set_animation_only_dirty_descendants(&self) {
    }

    /// Flag that this element has no descendant for animation-only restyle processing.
    ///
    /// Only safe to call with exclusive access to the element.
    unsafe fn unset_animation_only_dirty_descendants(&self) {
    }

    /// Returns true if this element is native anonymous (only Gecko has native
    /// anonymous content).
    fn is_native_anonymous(&self) -> bool { false }

    /// Returns the pseudo-element implemented by this element, if any.
    ///
    /// Gecko traverses pseudo-elements during the style traversal, and we need
    /// to know this so we can properly grab the pseudo-element style from the
    /// parent element.
    ///
    /// Note that we still need to compute the pseudo-elements before-hand,
    /// given otherwise we don't know if we need to create an element or not.
    ///
    /// Servo doesn't have to deal with this.
    fn implemented_pseudo_element(&self) -> Option<PseudoElement> { None }

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

    /// In Gecko, element has a flag that represents the element may have
    /// any type of animations or not to bail out animation stuff early.
    /// Whereas Servo doesn't have such flag.
    fn may_have_animations(&self) -> bool { false }

    /// Creates a task to update various animation state on a given (pseudo-)element.
    #[cfg(feature = "gecko")]
    fn update_animations(&self,
                         before_change_style: Option<Arc<ComputedValues>>,
                         tasks: UpdateAnimationsTasks);

    /// Returns true if the element has relevant animations. Relevant
    /// animations are those animations that are affecting the element's style
    /// or are scheduled to do so in the future.
    fn has_animations(&self) -> bool;

    /// Returns true if the element has a CSS animation.
    fn has_css_animations(&self) -> bool;

    /// Returns true if the element has a CSS transition (including running transitions and
    /// completed transitions).
    fn has_css_transitions(&self) -> bool;

    /// Returns true if the element has animation restyle hints.
    fn has_animation_restyle_hints(&self) -> bool {
        let data = match self.borrow_data() {
            Some(d) => d,
            None => return false,
        };
        return data.restyle.hint.has_animation_hint()
    }

    /// Returns the anonymous content for the current element's XBL binding,
    /// given if any.
    ///
    /// This is used in Gecko for XBL and shadow DOM.
    fn xbl_binding_anonymous_content(&self) -> Option<Self::ConcreteNode> {
        None
    }

    /// Returns the rule hash target given an element.
    fn rule_hash_target(&self) -> Self {
        let is_implemented_pseudo =
            self.implemented_pseudo_element().is_some();

        // NB: This causes use to rule has pseudo selectors based on the
        // properties of the originating element (which is fine, given the
        // find_first_from_right usage).
        if is_implemented_pseudo {
            self.closest_non_native_anonymous_ancestor().unwrap()
        } else {
            *self
        }
    }

    /// Gets declarations from XBL bindings from the element. Only gecko element could have this.
    fn get_declarations_from_xbl_bindings<V>(&self,
                                             _pseudo_element: Option<&PseudoElement>,
                                             _applicable_declarations: &mut V)
                                             -> bool
        where V: Push<ApplicableDeclarationBlock> + VecLike<ApplicableDeclarationBlock> {
        false
    }

    /// Gets the current existing CSS transitions, by |property, end value| pairs in a HashMap.
    #[cfg(feature = "gecko")]
    fn get_css_transitions_info(&self)
                                -> HashMap<TransitionProperty, Arc<AnimationValue>>;

    /// Does a rough (and cheap) check for whether or not transitions might need to be updated that
    /// will quickly return false for the common case of no transitions specified or running. If
    /// this returns false, we definitely don't need to update transitions but if it returns true
    /// we can perform the more thoroughgoing check, needs_transitions_update, to further
    /// reduce the possibility of false positives.
    #[cfg(feature = "gecko")]
    fn might_need_transitions_update(&self,
                                     old_values: Option<&ComputedValues>,
                                     new_values: &ComputedValues)
                                     -> bool;

    /// Returns true if one of the transitions needs to be updated on this element. We check all
    /// the transition properties to make sure that updating transitions is necessary.
    /// This method should only be called if might_needs_transitions_update returns true when
    /// passed the same parameters.
    #[cfg(feature = "gecko")]
    fn needs_transitions_update(&self,
                                before_change_style: &ComputedValues,
                                after_change_style: &ComputedValues)
                                -> bool;

    /// Returns true if we need to update transitions for the specified property on this element.
    #[cfg(feature = "gecko")]
    fn needs_transitions_update_per_property(&self,
                                             property: &TransitionProperty,
                                             combined_duration: f32,
                                             before_change_style: &ComputedValues,
                                             after_change_style: &ComputedValues,
                                             existing_transitions: &HashMap<TransitionProperty,
                                                                            Arc<AnimationValue>>)
                                             -> bool;

    /// Returns the value of the `xml:lang=""` attribute (or, if appropriate,
    /// the `lang=""` attribute) on this element.
    fn lang_attr(&self) -> Option<AttrValue>;

    /// Returns whether this element's language matches the language tag
    /// `value`.  If `override_lang` is not `None`, it specifies the value
    /// of the `xml:lang=""` or `lang=""` attribute to use in place of
    /// looking at the element and its ancestors.  (This argument is used
    /// to implement matching of `:lang()` against snapshots.)
    fn match_element_lang(&self,
                          override_lang: Option<Option<AttrValue>>,
                          value: &PseudoClassStringArg)
                          -> bool;
}

/// Trait abstracting over different kinds of dirty-descendants bits.
pub trait DescendantsBit<E: TElement> {
    /// Returns true if the Element has the bit.
    fn has(el: E) -> bool;
    /// Sets the bit on the Element.
    unsafe fn set(el: E);
}

/// Implementation of DescendantsBit for the regular dirty descendants bit.
pub struct DirtyDescendants;
impl<E: TElement> DescendantsBit<E> for DirtyDescendants {
    fn has(el: E) -> bool { el.has_dirty_descendants() }
    unsafe fn set(el: E) { el.set_dirty_descendants(); }
}

/// Implementation of DescendantsBit for the animation-only dirty descendants bit.
pub struct AnimationOnlyDirtyDescendants;
impl<E: TElement> DescendantsBit<E> for AnimationOnlyDirtyDescendants {
    fn has(el: E) -> bool { el.has_animation_only_dirty_descendants() }
    unsafe fn set(el: E) { el.set_animation_only_dirty_descendants(); }
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
#[derive(Debug, Eq, Hash, PartialEq)]
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
