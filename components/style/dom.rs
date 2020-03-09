/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Types and traits used to access the DOM from style calculation.

#![allow(unsafe_code)]
#![deny(missing_docs)]

use crate::applicable_declarations::ApplicableDeclarationBlock;
#[cfg(feature = "gecko")]
use crate::context::PostAnimationTasks;
#[cfg(feature = "gecko")]
use crate::context::UpdateAnimationsTasks;
use crate::data::ElementData;
use crate::element_state::ElementState;
use crate::font_metrics::FontMetricsProvider;
use crate::media_queries::Device;
use crate::properties::{AnimationRules, ComputedValues, PropertyDeclarationBlock};
use crate::selector_parser::{AttrValue, Lang, PseudoElement, SelectorImpl};
use crate::shared_lock::Locked;
use crate::stylist::CascadeData;
use crate::traversal_flags::TraversalFlags;
use crate::{Atom, LocalName, Namespace, WeakAtom};
use atomic_refcell::{AtomicRef, AtomicRefMut};
use selectors::matching::{ElementSelectorFlags, QuirksMode, VisitedHandlingMode};
use selectors::sink::Push;
use selectors::Element as SelectorsElement;
use servo_arc::{Arc, ArcBorrow};
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;

pub use style_traits::dom::OpaqueNode;

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
                return Some(n);
            }
        }
    }
}

/// An iterator over the DOM children of a node.
pub struct DomChildren<N>(Option<N>);
impl<N> Iterator for DomChildren<N>
where
    N: TNode,
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
    N: TNode,
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
pub trait TDocument: Sized + Copy + Clone {
    /// The concrete `TNode` type.
    type ConcreteNode: TNode<ConcreteDocument = Self>;

    /// Get this document as a `TNode`.
    fn as_node(&self) -> Self::ConcreteNode;

    /// Returns whether this document is an HTML document.
    fn is_html_document(&self) -> bool;

    /// Returns the quirks mode of this document.
    fn quirks_mode(&self) -> QuirksMode;

    /// Get a list of elements with a given ID in this document, sorted by
    /// tree position.
    ///
    /// Can return an error to signal that this list is not available, or also
    /// return an empty slice.
    fn elements_with_id<'a>(
        &self,
        _id: &Atom,
    ) -> Result<&'a [<Self::ConcreteNode as TNode>::ConcreteElement], ()>
    where
        Self: 'a,
    {
        Err(())
    }
}

/// The `TNode` trait. This is the main generic trait over which the style
/// system can be implemented.
pub trait TNode: Sized + Copy + Clone + Debug + NodeInfo + PartialEq {
    /// The concrete `TElement` type.
    type ConcreteElement: TElement<ConcreteNode = Self>;

    /// The concrete `TDocument` type.
    type ConcreteDocument: TDocument<ConcreteNode = Self>;

    /// The concrete `TShadowRoot` type.
    type ConcreteShadowRoot: TShadowRoot<ConcreteNode = Self>;

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

    /// Get this node as a ShadowRoot, if it's one.
    fn as_shadow_root(&self) -> Option<Self::ConcreteShadowRoot>;
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
            f,
            "{:?} dd={} aodd={} data={:?}",
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
        write!(
            f,
            "{:?} dd={} aodd={} data={:?} values={:?}",
            el, dd, aodd, &data, values
        )
    } else {
        write!(f, "{:?}", n)
    }
}

fn fmt_subtree<F, N: TNode>(f: &mut fmt::Formatter, stringify: &F, n: N, indent: u32) -> fmt::Result
where
    F: Fn(&mut fmt::Formatter, N) -> fmt::Result,
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

/// The ShadowRoot trait.
pub trait TShadowRoot: Sized + Copy + Clone + Debug + PartialEq {
    /// The concrete node type.
    type ConcreteNode: TNode<ConcreteShadowRoot = Self>;

    /// Get this ShadowRoot as a node.
    fn as_node(&self) -> Self::ConcreteNode;

    /// Get the shadow host that hosts this ShadowRoot.
    fn host(&self) -> <Self::ConcreteNode as TNode>::ConcreteElement;

    /// Get the style data for this ShadowRoot.
    fn style_data<'a>(&self) -> Option<&'a CascadeData>
    where
        Self: 'a;

    /// Get the list of shadow parts for this shadow root.
    fn parts<'a>(&self) -> &[<Self::ConcreteNode as TNode>::ConcreteElement]
    where
        Self: 'a,
    {
        &[]
    }

    /// Get a list of elements with a given ID in this shadow root, sorted by
    /// tree position.
    ///
    /// Can return an error to signal that this list is not available, or also
    /// return an empty slice.
    fn elements_with_id<'a>(
        &self,
        _id: &Atom,
    ) -> Result<&'a [<Self::ConcreteNode as TNode>::ConcreteElement], ()>
    where
        Self: 'a,
    {
        Err(())
    }
}

/// The element trait, the main abstraction the style crate acts over.
pub trait TElement:
    Eq + PartialEq + Debug + Hash + Sized + Copy + Clone + SelectorsElement<Impl = SelectorImpl>
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
    fn owner_doc_matches_for_testing(&self, _: &Device) -> bool {
        true
    }

    /// Whether this element should match user and author rules.
    ///
    /// We use this for Native Anonymous Content in Gecko.
    fn matches_user_and_author_rules(&self) -> bool {
        true
    }

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

    /// The ::marker pseudo-element of this element, if it exists.
    fn marker_pseudo_element(&self) -> Option<Self> {
        None
    }

    /// Execute `f` for each anonymous content child (apart from ::before and
    /// ::after) whose originating element is `self`.
    fn each_anonymous_content_child<F>(&self, _f: F)
    where
        F: FnMut(Self),
    {
    }

    /// Return whether this element is an element in the HTML namespace.
    fn is_html_element(&self) -> bool;

    /// Return whether this element is an element in the MathML namespace.
    fn is_mathml_element(&self) -> bool;

    /// Return whether this element is an element in the SVG namespace.
    fn is_svg_element(&self) -> bool;

    /// Return whether this element is an element in the XUL namespace.
    fn is_xul_element(&self) -> bool {
        false
    }

    /// Return the list of slotted nodes of this node.
    fn slotted_nodes(&self) -> &[Self::ConcreteNode] {
        &[]
    }

    /// Get this element's style attribute.
    fn style_attribute(&self) -> Option<ArcBorrow<Locked<PropertyDeclarationBlock>>>;

    /// Unset the style attribute's dirty bit.
    /// Servo doesn't need to manage ditry bit for style attribute.
    fn unset_dirty_style_attribute(&self) {}

    /// Get this element's SMIL override declarations.
    fn smil_override(&self) -> Option<ArcBorrow<Locked<PropertyDeclarationBlock>>> {
        None
    }

    /// Get the combined animation and transition rules.
    ///
    /// FIXME(emilio): Is this really useful?
    fn animation_rules(&self) -> AnimationRules {
        if !self.may_have_animations() {
            return AnimationRules(None, None);
        }

        AnimationRules(self.animation_rule(), self.transition_rule())
    }

    /// Get this element's animation rule.
    fn animation_rule(&self) -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        None
    }

    /// Get this element's transition rule.
    fn transition_rule(&self) -> Option<Arc<Locked<PropertyDeclarationBlock>>> {
        None
    }

    /// Get this element's state, for non-tree-structural pseudos.
    fn state(&self) -> ElementState;

    /// Whether this element has an attribute with a given namespace.
    fn has_attr(&self, namespace: &Namespace, attr: &LocalName) -> bool;

    /// Returns whether this element has a `part` attribute.
    fn has_part_attr(&self) -> bool;

    /// Returns whether this element exports any part from its shadow tree.
    fn exports_any_part(&self) -> bool;

    /// The ID for this element.
    fn id(&self) -> Option<&WeakAtom>;

    /// Internal iterator for the classes of this element.
    fn each_class<F>(&self, callback: F)
    where
        F: FnMut(&Atom);

    /// Internal iterator for the part names of this element.
    fn each_part<F>(&self, _callback: F)
    where
        F: FnMut(&Atom),
    {
    }

    /// Internal iterator for the part names that this element exports for a
    /// given part name.
    fn each_exported_part<F>(&self, _name: &Atom, _callback: F)
    where
        F: FnMut(&Atom),
    {
    }

    /// Whether a given element may generate a pseudo-element.
    ///
    /// This is useful to avoid computing, for example, pseudo styles for
    /// `::-first-line` or `::-first-letter`, when we know it won't affect us.
    ///
    /// TODO(emilio, bz): actually implement the logic for it.
    fn may_generate_pseudo(&self, pseudo: &PseudoElement, _primary_style: &ComputedValues) -> bool {
        // ::before/::after are always supported for now, though we could try to
        // optimize out leaf elements.

        // ::first-letter and ::first-line are only supported for block-inside
        // things, and only in Gecko, not Servo.  Unfortunately, Gecko has
        // block-inside things that might have any computed display value due to
        // things like fieldsets, legends, etc.  Need to figure out how this
        // should work.
        debug_assert!(
            pseudo.is_eager(),
            "Someone called may_generate_pseudo with a non-eager pseudo."
        );
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
            return data.has_styles() && !data.hint.has_animation_hint_or_recascade();
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
    unsafe fn set_animation_only_dirty_descendants(&self) {}

    /// Flag that this element has no descendant for animation-only restyle processing.
    ///
    /// Only safe to call with exclusive access to the element.
    unsafe fn unset_animation_only_dirty_descendants(&self) {}

    /// Clear all bits related describing the dirtiness of descendants.
    ///
    /// In Gecko, this corresponds to the regular dirty descendants bit, the
    /// animation-only dirty descendants bit, and the lazy frame construction
    /// descendants bit.
    unsafe fn clear_descendant_bits(&self) {
        self.unset_dirty_descendants();
    }

    /// Returns true if this element is a visited link.
    ///
    /// Servo doesn't support visited styles yet.
    fn is_visited_link(&self) -> bool {
        false
    }

    /// Returns true if this element is in a native anonymous subtree.
    fn is_in_native_anonymous_subtree(&self) -> bool {
        false
    }

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
    fn implemented_pseudo_element(&self) -> Option<PseudoElement> {
        None
    }

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

    /// Whether there is an ElementData container.
    fn has_data(&self) -> bool;

    /// Immutably borrows the ElementData.
    fn borrow_data(&self) -> Option<AtomicRef<ElementData>>;

    /// Mutably borrows the ElementData.
    fn mutate_data(&self) -> Option<AtomicRefMut<ElementData>>;

    /// Whether we should skip any root- or item-based display property
    /// blockification on this element.  (This function exists so that Gecko
    /// native anonymous content can opt out of this style fixup.)
    fn skip_item_display_fixup(&self) -> bool;

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
    fn may_have_animations(&self) -> bool {
        false
    }

    /// Creates a task to update various animation state on a given (pseudo-)element.
    #[cfg(feature = "gecko")]
    fn update_animations(
        &self,
        before_change_style: Option<Arc<ComputedValues>>,
        tasks: UpdateAnimationsTasks,
    );

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
        return data.hint.has_animation_hint();
    }

    /// The shadow root this element is a host of.
    fn shadow_root(&self) -> Option<<Self::ConcreteNode as TNode>::ConcreteShadowRoot>;

    /// The shadow root which roots the subtree this element is contained in.
    fn containing_shadow(&self) -> Option<<Self::ConcreteNode as TNode>::ConcreteShadowRoot>;

    /// Return the element which we can use to look up rules in the selector
    /// maps.
    ///
    /// This is always the element itself, except in the case where we are an
    /// element-backed pseudo-element, in which case we return the originating
    /// element.
    fn rule_hash_target(&self) -> Self {
        if self.is_pseudo_element() {
            self.pseudo_element_originating_element()
                .expect("Trying to collect rules for a detached pseudo-element")
        } else {
            *self
        }
    }

    /// Executes the callback for each applicable style rule data which isn't
    /// the main document's data (which stores UA / author rules).
    ///
    /// The element passed to the callback is the containing shadow host for the
    /// data if it comes from Shadow DOM.
    ///
    /// Returns whether normal document author rules should apply.
    ///
    /// TODO(emilio): We could separate the invalidation data for elements
    /// matching in other scopes to avoid over-invalidation.
    fn each_applicable_non_document_style_rule_data<'a, F>(&self, mut f: F) -> bool
    where
        Self: 'a,
        F: FnMut(&'a CascadeData, Self),
    {
        use rule_collector::containing_shadow_ignoring_svg_use;

        let target = self.rule_hash_target();
        if !target.matches_user_and_author_rules() {
            return false;
        }

        let mut doc_rules_apply = true;

        // Use the same rules to look for the containing host as we do for rule
        // collection.
        if let Some(shadow) = containing_shadow_ignoring_svg_use(target) {
            doc_rules_apply = false;
            if let Some(data) = shadow.style_data() {
                f(data, shadow.host());
            }
        }

        if let Some(shadow) = target.shadow_root() {
            if let Some(data) = shadow.style_data() {
                f(data, shadow.host());
            }
        }

        let mut current = target.assigned_slot();
        while let Some(slot) = current {
            // Slots can only have assigned nodes when in a shadow tree.
            let shadow = slot.containing_shadow().unwrap();
            if let Some(data) = shadow.style_data() {
                if data.any_slotted_rule() {
                    f(data, shadow.host());
                }
            }
            current = slot.assigned_slot();
        }

        if target.has_part_attr() {
            if let Some(mut inner_shadow) = target.containing_shadow() {
                loop {
                    let inner_shadow_host = inner_shadow.host();
                    match inner_shadow_host.containing_shadow() {
                        Some(shadow) => {
                            if let Some(data) = shadow.style_data() {
                                if data.any_part_rule() {
                                    f(data, shadow.host())
                                }
                            }
                            // TODO: Could be more granular.
                            if !shadow.host().exports_any_part() {
                                break;
                            }
                            inner_shadow = shadow;
                        },
                        None => {
                            // TODO(emilio): Should probably distinguish with
                            // MatchesDocumentRules::{No,Yes,IfPart} or
                            // something so that we could skip some work.
                            doc_rules_apply = true;
                            break;
                        },
                    }
                }
            }
        }

        doc_rules_apply
    }

    /// Does a rough (and cheap) check for whether or not transitions might need to be updated that
    /// will quickly return false for the common case of no transitions specified or running. If
    /// this returns false, we definitely don't need to update transitions but if it returns true
    /// we can perform the more thoroughgoing check, needs_transitions_update, to further
    /// reduce the possibility of false positives.
    #[cfg(feature = "gecko")]
    fn might_need_transitions_update(
        &self,
        old_values: Option<&ComputedValues>,
        new_values: &ComputedValues,
    ) -> bool;

    /// Returns true if one of the transitions needs to be updated on this element. We check all
    /// the transition properties to make sure that updating transitions is necessary.
    /// This method should only be called if might_needs_transitions_update returns true when
    /// passed the same parameters.
    #[cfg(feature = "gecko")]
    fn needs_transitions_update(
        &self,
        before_change_style: &ComputedValues,
        after_change_style: &ComputedValues,
    ) -> bool;

    /// Returns the value of the `xml:lang=""` attribute (or, if appropriate,
    /// the `lang=""` attribute) on this element.
    fn lang_attr(&self) -> Option<AttrValue>;

    /// Returns whether this element's language matches the language tag
    /// `value`.  If `override_lang` is not `None`, it specifies the value
    /// of the `xml:lang=""` or `lang=""` attribute to use in place of
    /// looking at the element and its ancestors.  (This argument is used
    /// to implement matching of `:lang()` against snapshots.)
    fn match_element_lang(&self, override_lang: Option<Option<AttrValue>>, value: &Lang) -> bool;

    /// Returns whether this element is the main body element of the HTML
    /// document it is on.
    fn is_html_document_body_element(&self) -> bool;

    /// Generate the proper applicable declarations due to presentational hints,
    /// and insert them into `hints`.
    fn synthesize_presentational_hints_for_legacy_attributes<V>(
        &self,
        visited_handling: VisitedHandlingMode,
        hints: &mut V,
    ) where
        V: Push<ApplicableDeclarationBlock>;

    /// Returns element's local name.
    fn local_name(&self) -> &<SelectorImpl as selectors::parser::SelectorImpl>::BorrowedLocalName;

    /// Returns element's namespace.
    fn namespace(&self)
        -> &<SelectorImpl as selectors::parser::SelectorImpl>::BorrowedNamespaceUrl;
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
