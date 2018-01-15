/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Types and traits used to access the DOM from style calculation.

#![allow(unsafe_code)]
#![deny(missing_docs)]

use {Atom, Namespace, LocalName};
use applicable_declarations::ApplicableDeclarationBlock;
use atomic_refcell::{AtomicRef, AtomicRefCell, AtomicRefMut};
#[cfg(feature = "gecko")] use context::PostAnimationTasks;
#[cfg(feature = "gecko")] use context::UpdateAnimationsTasks;
use data::ElementData;
use element_state::ElementState;
use font_metrics::FontMetricsProvider;
use media_queries::Device;
use properties::{AnimationRules, ComputedValues, PropertyDeclarationBlock};
#[cfg(feature = "gecko")] use properties::LonghandId;
#[cfg(feature = "gecko")] use properties::animated_properties::AnimationValue;
use rule_tree::CascadeLevel;
use selector_parser::{AttrValue, PseudoClassStringArg, PseudoElement, SelectorImpl};
use selectors::Element as SelectorsElement;
use selectors::matching::{ElementSelectorFlags, QuirksMode, VisitedHandlingMode};
use selectors::sink::Push;
use servo_arc::{Arc, ArcBorrow};
use shared_lock::Locked;
use std::fmt;
#[cfg(feature = "gecko")] use hash::FnvHashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use stylist::{StyleRuleCascadeData, Stylist};
use traversal_flags::TraversalFlags;

/// An opaque handle to a node, which, unlike UnsafeNode, cannot be transformed
/// back into a non-opaque representation. The only safe operation that can be
/// performed on this node is to compare it to another opaque handle or to another
/// OpaqueNode.
///
/// Layout and Graphics use this to safely represent nodes for comparison purposes.
/// Because the script task's GC does not trace layout, node data cannot be safely stored in layout
/// data structures. Also, layout code tends to be faster when the DOM is not being accessed, for
/// locality reasons. Using `OpaqueNode` enforces this invariant.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf, Deserialize, Serialize))]
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
}

/// A node iterator that only returns node that don't need layout.
pub struct LayoutIterator<T>(pub T);

impl<T, N> Iterator for LayoutIterator<T>
where
    T: Iterator<Item = N>,
    N: NodeInfo,
{
    type Item = N;

    fn next(&mut self) -> Option<N> {
        loop {
            let n = self.0.next()?;
            // Filter out nodes that layout should ignore.
            if n.is_text_node() || n.is_element() {
                return Some(n)
            }
        }
    }
}

/// An iterator over the DOM children of a node.
pub struct DomChildren<N>(Option<N>);
impl<N> Iterator for DomChildren<N>
where
    N: TNode
{
    type Item = N;

    fn next(&mut self) -> Option<N> {
        let n = self.0.take()?;
        self.0 = n.next_sibling();
        Some(n)
    }
}

/// An iterator over the DOM descendants of a node in pre-order.
pub struct DomDescendants<N> {
    previous: Option<N>,
    scope: N,
}

impl<N> Iterator for DomDescendants<N>
where
    N: TNode
{
    type Item = N;

    #[inline]
    fn next(&mut self) -> Option<N> {
        let prev = self.previous.take()?;
        self.previous = prev.next_in_preorder(Some(self.scope));
        self.previous
    }
}

/// The `TDocument` trait, to represent a document node.
pub trait TDocument : Sized + Copy + Clone {
    /// The concrete `TNode` type.
    type ConcreteNode: TNode<ConcreteDocument = Self>;

    /// Get this document as a `TNode`.
    fn as_node(&self) -> Self::ConcreteNode;

    /// Returns whether this document is an HTML document.
    fn is_html_document(&self) -> bool;

    /// Returns the quirks mode of this document.
    fn quirks_mode(&self) -> QuirksMode;

    /// Get a list of elements with a given ID in this document, sorted by
    /// document position.
    ///
    /// Can return an error to signal that this list is not available, or also
    /// return an empty slice.
    fn elements_with_id(
        &self,
        _id: &Atom,
    ) -> Result<&[<Self::ConcreteNode as TNode>::ConcreteElement], ()> {
        Err(())
    }
}

/// The `TNode` trait. This is the main generic trait over which the style
/// system can be implemented.
pub trait TNode : Sized + Copy + Clone + Debug + NodeInfo + PartialEq {
    /// The concrete `TElement` type.
    type ConcreteElement: TElement<ConcreteNode = Self>;

    /// The concrete `TDocument` type.
    type ConcreteDocument: TDocument<ConcreteNode = Self>;

    /// Get this node's parent node.
    fn parent_node(&self) -> Option<Self>;

    /// Get this node's first child.
    fn first_child(&self) -> Option<Self>;

    /// Get this node's first child.
    fn last_child(&self) -> Option<Self>;

    /// Get this node's previous sibling.
    fn prev_sibling(&self) -> Option<Self>;

    /// Get this node's next sibling.
    fn next_sibling(&self) -> Option<Self>;

    /// Get the owner document of this node.
    fn owner_doc(&self) -> Self::ConcreteDocument;

    /// Iterate over the DOM children of a node.
    fn dom_children(&self) -> DomChildren<Self> {
        DomChildren(self.first_child())
    }

    /// Returns whether the node is attached to a document.
    fn is_in_document(&self) -> bool;

    /// Iterate over the DOM children of a node, in preorder.
    fn dom_descendants(&self) -> DomDescendants<Self> {
        DomDescendants {
            previous: Some(*self),
            scope: *self,
        }
    }

    /// Returns the next children in pre-order, optionally scoped to a subtree
    /// root.
    #[inline]
    fn next_in_preorder(&self, scoped_to: Option<Self>) -> Option<Self> {
        if let Some(c) = self.first_child() {
            return Some(c);
        }

        if Some(*self) == scoped_to {
            return None;
        }

        let mut current = *self;
        loop {
            if let Some(s) = current.next_sibling() {
                return Some(s);
            }

            let parent = current.parent_node();
            if parent == scoped_to {
                return None;
            }

            current = parent.expect("Not a descendant of the scope?");
        }
    }

    /// Get this node's parent element from the perspective of a restyle
    /// traversal.
    fn traversal_parent(&self) -> Option<Self::ConcreteElement>;

    /// Get this node's parent element if present.
    fn parent_element(&self) -> Option<Self::ConcreteElement> {
        self.parent_node().and_then(|n| n.as_element())
    }

    /// Converts self into an `OpaqueNode`.
    fn opaque(&self) -> OpaqueNode;

    /// A debug id, only useful, mm... for debugging.
    fn debug_id(self) -> usize;

    /// Get this node as an element, if it's one.
    fn as_element(&self) -> Option<Self::ConcreteElement>;

    /// Get this node as a document, if it's one.
    fn as_document(&self) -> Option<Self::ConcreteDocument>;
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
#[cfg(feature = "servo")]
pub struct ShowSubtreeDataAndPrimaryValues<N: TNode>(pub N);
#[cfg(feature = "servo")]
impl<N: TNode> Debug for ShowSubtreeDataAndPrimaryValues<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "DOM Subtree:")?;
        fmt_subtree(f, &|f, n| fmt_with_data_and_primary_values(f, n), self.0, 1)
    }
}

fn fmt_with_data<N: TNode>(f: &mut fmt::Formatter, n: N) -> fmt::Result {
    if let Some(el) = n.as_element() {
        write!(
            f, "{:?} dd={} aodd={} data={:?}",
            el,
            el.has_dirty_descendants(),
            el.has_animation_only_dirty_descendants(),
            el.borrow_data(),
       )
    } else {
        write!(f, "{:?}", n)
    }
}

#[cfg(feature = "servo")]
fn fmt_with_data_and_primary_values<N: TNode>(f: &mut fmt::Formatter, n: N) -> fmt::Result {
    if let Some(el) = n.as_element() {
        let dd = el.has_dirty_descendants();
        let aodd = el.has_animation_only_dirty_descendants();
        let data = el.borrow_data();
        let values = data.as_ref().and_then(|d| d.styles.get_primary());
        write!(f, "{:?} dd={} aodd={} data={:?} values={:?}", el, dd, aodd, &data, values)
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
    if let Some(e) = n.as_element() {
        for kid in e.traversal_children() {
            writeln!(f, "")?;
            fmt_subtree(f, stringify, kid, indent + 1)?;
        }
    }

    Ok(())
}

/// The element trait, the main abstraction the style crate acts over.
pub trait TElement
    : Eq
    + PartialEq
    + Debug
    + Hash
    + Sized
    + Copy
    + Clone
    + SelectorsElement<Impl = SelectorImpl>
{
    /// The concrete node type.
    type ConcreteNode: TNode<ConcreteElement = Self>;

    /// A concrete children iterator type in order to iterate over the `Node`s.
    ///
    /// TODO(emilio): We should eventually replace this with the `impl Trait`
    /// syntax.
    type TraversalChildrenIterator: Iterator<Item = Self::ConcreteNode>;

    /// Type of the font metrics provider
    ///
    /// XXXManishearth It would be better to make this a type parameter on
    /// ThreadLocalStyleContext and StyleContext
    type FontMetricsProvider: FontMetricsProvider + Send;

    /// Get this element as a node.
    fn as_node(&self) -> Self::ConcreteNode;

    /// A debug-only check that the device's owner doc matches the actual doc
    /// we're the root of.
    ///
    /// Otherwise we may set document-level state incorrectly, like the root
    /// font-size used for rem units.
    fn owner_doc_matches_for_testing(&self, _: &Device) -> bool { true }

    /// Whether this element should match user and author rules.
    ///
    /// We use this for Native Anonymous Content in Gecko.
    fn matches_user_and_author_rules(&self) -> bool { true }

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

    /// The style scope of this element is a node that represents which rules
    /// apply to the element.
    ///
    /// In Servo, where we don't know about Shadow DOM or XBL, the style scope
    /// is always the document.
    fn style_scope(&self) -> Self::ConcreteNode {
        self.as_node().owner_doc().as_node()
    }

    /// Get this node's parent element from the perspective of a restyle
    /// traversal.
    fn traversal_parent(&self) -> Option<Self> {
        self.as_node().traversal_parent()
    }

    /// Get this node's children from the perspective of a restyle traversal.
    fn traversal_children(&self) -> LayoutIterator<Self::TraversalChildrenIterator>;

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

    /// Return whether this element is an element in the HTML namespace.
    fn is_html_element(&self) -> bool;

    /// Returns whether this element is a <html:slot> element.
    fn is_html_slot_element(&self) -> bool {
        self.get_local_name() == &*local_name!("slot") &&
        self.is_html_element()
    }

    /// Return the list of slotted nodes of this node.
    fn slotted_nodes(&self) -> &[Self::ConcreteNode] {
        &[]
    }

    /// For a given NAC element, return the closest non-NAC ancestor, which is
    /// guaranteed to exist.
    fn closest_non_native_anonymous_ancestor(&self) -> Option<Self> {
        unreachable!("Servo doesn't know about NAC");
    }

    /// Get this element's style attribute.
    fn style_attribute(&self) -> Option<ArcBorrow<Locked<PropertyDeclarationBlock>>>;

    /// Unset the style attribute's dirty bit.
    /// Servo doesn't need to manage ditry bit for style attribute.
    fn unset_dirty_style_attribute(&self) {
    }

    /// Get this element's SMIL override declarations.
    fn get_smil_override(&self) -> Option<ArcBorrow<Locked<PropertyDeclarationBlock>>> {
        None
    }

    /// Get this element's animation rule by the cascade level.
    fn get_animation_rule_by_cascade(&self,
                                     _cascade_level: CascadeLevel)
                                     -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        None
    }

    /// Get the combined animation and transition rules.
    fn get_animation_rules(&self) -> AnimationRules {
        if !self.may_have_animations() {
            return AnimationRules(None, None)
        }

        AnimationRules(
            self.get_animation_rule(),
            self.get_transition_rule(),
        )
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

    /// Whether a given element may generate a pseudo-element.
    ///
    /// This is useful to avoid computing, for example, pseudo styles for
    /// `::-first-line` or `::-first-letter`, when we know it won't affect us.
    ///
    /// TODO(emilio, bz): actually implement the logic for it.
    fn may_generate_pseudo(
        &self,
        pseudo: &PseudoElement,
        _primary_style: &ComputedValues,
    ) -> bool {
        // ::before/::after are always supported for now, though we could try to
        // optimize out leaf elements.

        // ::first-letter and ::first-line are only supported for block-inside
        // things, and only in Gecko, not Servo.  Unfortunately, Gecko has
        // block-inside things that might have any computed display value due to
        // things like fieldsets, legends, etc.  Need to figure out how this
        // should work.
        debug_assert!(pseudo.is_eager(),
                      "Someone called may_generate_pseudo with a non-eager pseudo.");
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

    /// Returns whether the element's styles are up-to-date for |traversal_flags|.
    fn has_current_styles_for_traversal(
        &self,
        data: &ElementData,
        traversal_flags: TraversalFlags,
    ) -> bool {
        if traversal_flags.for_animation_only() {
            // In animation-only restyle we never touch snapshots and don't
            // care about them. But we can't assert '!self.handled_snapshot()'
            // here since there are some cases that a second animation-only
            // restyle which is a result of normal restyle (e.g. setting
            // animation-name in normal restyle and creating a new CSS
            // animation in a SequentialTask) is processed after the normal
            // traversal in that we had elements that handled snapshot.
            return data.has_styles() &&
                   !data.hint.has_animation_hint_or_recascade();
        }

        if self.has_snapshot() && !self.handled_snapshot() {
            return false;
        }

        data.has_styles() && !data.hint.has_non_animation_invalidations()
    }

    /// Returns whether the element's styles are up-to-date after traversal
    /// (i.e. in post traversal).
    fn has_current_styles(&self, data: &ElementData) -> bool {
        if self.has_snapshot() && !self.handled_snapshot() {
            return false;
        }

        data.has_styles() &&
        // TODO(hiro): When an animating element moved into subtree of
        // contenteditable element, there remains animation restyle hints in
        // post traversal. It's generally harmless since the hints will be
        // processed in a next styling but ideally it should be processed soon.
        //
        // Without this, we get failures in:
        //   layout/style/crashtests/1383319.html
        //   layout/style/crashtests/1383001.html
        //
        // https://bugzilla.mozilla.org/show_bug.cgi?id=1389675 tracks fixing
        // this.
        !data.hint.has_non_animation_invalidations()
    }

    /// Flag that this element has a descendant for style processing.
    ///
    /// Only safe to call with exclusive access to the element.
    unsafe fn set_dirty_descendants(&self);

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

    /// Clear all bits related describing the dirtiness of descendants.
    ///
    /// In Gecko, this corresponds to the regular dirty descendants bit, the
    /// animation-only dirty descendants bit, and the lazy frame construction
    /// descendants bit.
    unsafe fn clear_descendant_bits(&self) { self.unset_dirty_descendants(); }

    /// Clear all element flags related to dirtiness.
    ///
    /// In Gecko, this corresponds to the regular dirty descendants bit, the
    /// animation-only dirty descendants bit, the lazy frame construction bit,
    /// and the lazy frame construction descendants bit.
    unsafe fn clear_dirty_bits(&self) { self.unset_dirty_descendants(); }

    /// Returns true if this element is a visited link.
    ///
    /// Servo doesn't support visited styles yet.
    fn is_visited_link(&self) -> bool { false }

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

    /// Gets a reference to the ElementData container, or creates one.
    ///
    /// Unsafe because it can race to allocate and leak if not used with
    /// exclusive access to the element.
    unsafe fn ensure_data(&self) -> AtomicRefMut<ElementData>;

    /// Clears the element data reference, if any.
    ///
    /// Unsafe following the same reasoning as ensure_data.
    unsafe fn clear_data(&self);

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

    /// Creates a task to process post animation on a given element.
    #[cfg(feature = "gecko")]
    fn process_post_animation(&self, tasks: PostAnimationTasks);

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
        return data.hint.has_animation_hint()
    }

    /// Returns the anonymous content for the current element's XBL binding,
    /// given if any.
    ///
    /// This is used in Gecko for XBL and shadow DOM.
    fn xbl_binding_anonymous_content(&self) -> Option<Self::ConcreteNode> {
        None
    }

    /// Return the element which we can use to look up rules in the selector
    /// maps.
    ///
    /// This is always the element itself, except in the case where we are an
    /// element-backed pseudo-element, in which case we return the originating
    /// element.
    fn rule_hash_target(&self) -> Self {
        if self.implemented_pseudo_element().is_some() {
            self.closest_non_native_anonymous_ancestor()
                .expect("Trying to collect rules for a detached pseudo-element")
        } else {
            *self
        }
    }

    /// Implements Gecko's `nsBindingManager::WalkRules`.
    ///
    /// Returns whether to cut off the inheritance.
    fn each_xbl_stylist<'a, F>(&self, _: F) -> bool
    where
        Self: 'a,
        F: FnMut(AtomicRef<'a, Stylist>),
    {
        false
    }

    /// Executes the callback for each applicable style rule data which isn't
    /// the main document's data (which stores UA / author rules).
    ///
    /// Returns whether normal document author rules should apply.
    fn each_applicable_non_document_style_rule_data<'a, F>(&self, mut f: F) -> bool
    where
        Self: 'a,
        F: FnMut(AtomicRef<'a, StyleRuleCascadeData>, QuirksMode),
    {
        let cut_off_inheritance = self.each_xbl_stylist(|stylist| {
            let quirks_mode = stylist.quirks_mode();
            f(
                AtomicRef::map(stylist, |stylist| stylist.normal_author_cascade_data()),
                quirks_mode,
            )
        });

        let mut current = self.assigned_slot();
        while let Some(slot) = current {
            slot.each_xbl_stylist(|stylist| {
                let quirks_mode = stylist.quirks_mode();
                if stylist.slotted_author_cascade_data().is_some() {
                    f(
                        AtomicRef::map(stylist, |stylist| stylist.slotted_author_cascade_data().unwrap()),
                        quirks_mode,
                    )
                }
            });

            current = slot.assigned_slot();
        }

        cut_off_inheritance
    }

    /// Gets the current existing CSS transitions, by |property, end value| pairs in a FnvHashMap.
    #[cfg(feature = "gecko")]
    fn get_css_transitions_info(&self)
                                -> FnvHashMap<LonghandId, Arc<AnimationValue>>;

    /// Does a rough (and cheap) check for whether or not transitions might need to be updated that
    /// will quickly return false for the common case of no transitions specified or running. If
    /// this returns false, we definitely don't need to update transitions but if it returns true
    /// we can perform the more thoroughgoing check, needs_transitions_update, to further
    /// reduce the possibility of false positives.
    #[cfg(feature = "gecko")]
    fn might_need_transitions_update(
        &self,
        old_values: Option<&ComputedValues>,
        new_values: &ComputedValues
    ) -> bool;

    /// Returns true if one of the transitions needs to be updated on this element. We check all
    /// the transition properties to make sure that updating transitions is necessary.
    /// This method should only be called if might_needs_transitions_update returns true when
    /// passed the same parameters.
    #[cfg(feature = "gecko")]
    fn needs_transitions_update(
        &self,
        before_change_style: &ComputedValues,
        after_change_style: &ComputedValues
    ) -> bool;

    /// Returns true if we need to update transitions for the specified property on this element.
    #[cfg(feature = "gecko")]
    fn needs_transitions_update_per_property(
        &self,
        property: &LonghandId,
        combined_duration: f32,
        before_change_style: &ComputedValues,
        after_change_style: &ComputedValues,
        existing_transitions: &FnvHashMap<LonghandId, Arc<AnimationValue>>
    ) -> bool;

    /// Returns the value of the `xml:lang=""` attribute (or, if appropriate,
    /// the `lang=""` attribute) on this element.
    fn lang_attr(&self) -> Option<AttrValue>;

    /// Returns whether this element's language matches the language tag
    /// `value`.  If `override_lang` is not `None`, it specifies the value
    /// of the `xml:lang=""` or `lang=""` attribute to use in place of
    /// looking at the element and its ancestors.  (This argument is used
    /// to implement matching of `:lang()` against snapshots.)
    fn match_element_lang(
        &self,
        override_lang: Option<Option<AttrValue>>,
        value: &PseudoClassStringArg
    ) -> bool;

    /// Returns whether this element is the main body element of the HTML
    /// document it is on.
    fn is_html_document_body_element(&self) -> bool;

    /// Generate the proper applicable declarations due to presentational hints,
    /// and insert them into `hints`.
    fn synthesize_presentational_hints_for_legacy_attributes<V>(
        &self,
        visited_handling: VisitedHandlingMode,
        hints: &mut V,
    )
    where
        V: Push<ApplicableDeclarationBlock>;
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
