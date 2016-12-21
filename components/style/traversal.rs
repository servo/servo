/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversing the DOM tree; the bloom filter.

use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use bloom::StyleBloom;
use context::{SharedStyleContext, StyleContext};
use data::{ElementData, StoredRestyleHint};
use dom::{OpaqueNode, TElement, TNode};
use matching::{MatchMethods, StyleSharingResult};
use restyle_hints::{RESTYLE_DESCENDANTS, RESTYLE_SELF};
use selector_parser::RestyleDamage;
use selectors::Element;
use selectors::matching::StyleRelations;
use servo_config::opts;
use std::cell::RefCell;
use std::mem;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use stylist::Stylist;

/// Every time we do another layout, the old bloom filters are invalid. This is
/// detected by ticking a generation number every layout.
pub type Generation = u32;

/// Style sharing candidate cache stats. These are only used when
/// `-Z style-sharing-stats` is given.
pub static STYLE_SHARING_CACHE_HITS: AtomicUsize = ATOMIC_USIZE_INIT;
pub static STYLE_SHARING_CACHE_MISSES: AtomicUsize = ATOMIC_USIZE_INIT;

thread_local!(
    static STYLE_BLOOM: RefCell<Option<StyleBloom>> = RefCell::new(None));

/// Returns the thread local bloom filter.
///
/// If one does not exist, a new one will be made for you. If it is out of date,
/// it will be cleared and reused.
pub fn take_thread_local_bloom_filter(context: &SharedStyleContext)
                                      -> StyleBloom
{
    trace!("{} taking bf", ::tid::tid());

    STYLE_BLOOM.with(|style_bloom| {
        style_bloom.borrow_mut().take()
            .unwrap_or_else(|| StyleBloom::new(context.generation))
    })
}

pub fn put_thread_local_bloom_filter(bf: StyleBloom) {
    trace!("[{}] putting bloom filter back", ::tid::tid());

    STYLE_BLOOM.with(move |style_bloom| {
        debug_assert!(style_bloom.borrow().is_none(),
                     "Putting into a never-taken thread-local bloom filter");
        *style_bloom.borrow_mut() = Some(bf);
    })
}

/// Remove `element` from the bloom filter if it's the last element we inserted.
///
/// Restores the bloom filter if this is not the root of the reflow.
///
/// This is mostly useful for sequential traversal, where the element will
/// always be the last one.
pub fn remove_from_bloom_filter<E: TElement>(context: &SharedStyleContext,
                                             root: OpaqueNode, element: E)
{
    trace!("[{}] remove_from_bloom_filter", ::tid::tid());

    // We may have arrived to `reconstruct_flows` without entering in style
    // recalc at all due to our optimizations, nor that it's up to date, so we
    // can't ensure there's a bloom filter at all.
    let bf = STYLE_BLOOM.with(|style_bloom| {
        style_bloom.borrow_mut().take()
    });

    if let Some(mut bf) = bf {
        if context.generation == bf.generation() {
            bf.maybe_pop(element);

            // If we're the root of the reflow, just get rid of the bloom
            // filter.
            //
            // FIXME: We might want to just leave it in TLS? You don't do 4k
            // allocations every day. Also, this just clears one thread's bloom
            // filter, which is... not great?
            if element.as_node().opaque() != root {
                put_thread_local_bloom_filter(bf);
            }
        }
    }
}

// NB: Keep this as small as possible, please!
#[derive(Clone, Debug)]
pub struct PerLevelTraversalData {
    pub current_dom_depth: Option<usize>,
}

/// This structure exists to enforce that callers invoke pre_traverse, and also
/// to pass information from the pre-traversal into the primary traversal.
pub struct PreTraverseToken {
    traverse: bool,
    unstyled_children_only: bool,
}

impl PreTraverseToken {
    pub fn should_traverse(&self) -> bool {
        self.traverse
    }

    pub fn traverse_unstyled_children_only(&self) -> bool {
        self.unstyled_children_only
    }
}

/// Enum to prevent duplicate logging.
pub enum LogBehavior {
    MayLog,
    DontLog,
}
use self::LogBehavior::*;
impl LogBehavior {
    fn allow(&self) -> bool { match *self { MayLog => true, DontLog => false, } }
}

pub trait DomTraversal<N: TNode> : Sync {
    type ThreadLocalContext: Send;

    /// Process `node` on the way down, before its children have been processed.
    fn process_preorder(&self, data: &mut PerLevelTraversalData,
                        thread_local: &mut Self::ThreadLocalContext, node: N);

    /// Process `node` on the way up, after its children have been processed.
    ///
    /// This is only executed if `needs_postorder_traversal` returns true.
    fn process_postorder(&self, thread_local: &mut Self::ThreadLocalContext, node: N);

    /// Boolean that specifies whether a bottom up traversal should be
    /// performed.
    ///
    /// If it's false, then process_postorder has no effect at all.
    fn needs_postorder_traversal() -> bool { true }

    /// Must be invoked before traversing the root element to determine whether
    /// a traversal is needed. Returns a token that allows the caller to prove
    /// that the call happened.
    ///
    /// The unstyled_children_only parameter is used in Gecko to style newly-
    /// appended children without restyling the parent.
    fn pre_traverse(root: N::ConcreteElement, stylist: &Stylist,
                    unstyled_children_only: bool)
                    -> PreTraverseToken
    {
        if unstyled_children_only {
            return PreTraverseToken {
                traverse: true,
                unstyled_children_only: true,
            };
        }

        // Expand the snapshot, if any. This is normally handled by the parent, so
        // we need a special case for the root.
        //
        // Expanding snapshots here may create a LATER_SIBLINGS restyle hint, which
        // we will drop on the floor. To prevent missed restyles, we assert against
        // restyling a root with later siblings.
        if let Some(mut data) = root.mutate_data() {
            if let Some(r) = data.as_restyle_mut() {
                debug_assert!(root.next_sibling_element().is_none());
                let _later_siblings = r.expand_snapshot(root, stylist);
            }
        }

        PreTraverseToken {
            traverse: Self::node_needs_traversal(root.as_node()),
            unstyled_children_only: false,
        }
    }

    /// Returns true if traversal should visit a text node. The style system never
    /// processes text nodes, but Servo overrides this to visit them for flow
    /// construction when necessary.
    fn text_node_needs_traversal(node: N) -> bool { debug_assert!(node.is_text_node()); false }

    /// Returns true if traversal is needed for the given node and subtree.
    fn node_needs_traversal(node: N) -> bool {
        // Non-incremental layout visits every node.
        if cfg!(feature = "servo") && opts::get().nonincremental_layout {
            return true;
        }

        match node.as_element() {
            None => Self::text_node_needs_traversal(node),
            Some(el) => {
                // If the dirty descendants bit is set, we need to traverse no
                // matter what. Skip examining the ElementData.
                if el.has_dirty_descendants() {
                    return true;
                }

                // Check the element data. If it doesn't exist, we need to visit
                // the element.
                let data = match el.borrow_data() {
                    Some(d) => d,
                    None => return true,
                };

                // Check what kind of element data we have. If it's Initial or Persistent,
                // we're done.
                let restyle = match *data {
                    ElementData::Initial(ref i) => return i.is_none(),
                    ElementData::Persistent(_) => return false,
                    ElementData::Restyle(ref r) => r,
                };

                // Check whether we have any selector matching or re-cascading to
                // do in this subtree.
                debug_assert!(restyle.snapshot.is_none(), "Snapshots should already be expanded");
                if !restyle.hint.is_empty() || restyle.recascade {
                    return true;
                }

                // Servo uses the post-order traversal for flow construction, so
                // we need to traverse any element with damage so that we can perform
                // fixup / reconstruction on our way back up the tree.
                if cfg!(feature = "servo") && restyle.damage != RestyleDamage::empty() {
                    return true;
                }

                false
            },
        }
    }

    /// Returns true if traversal of this element's children is allowed. We use
    /// this to cull traversal of various subtrees.
    ///
    /// This may be called multiple times when processing an element, so we pass
    /// a parameter to keep the logs tidy.
    fn should_traverse_children(parent: N::ConcreteElement, parent_data: &ElementData,
                                log: LogBehavior) -> bool
    {
        // See the comment on `cascade_node` for why we allow this on Gecko.
        debug_assert!(cfg!(feature = "gecko") || parent_data.has_current_styles());

        // If the parent computed display:none, we don't style the subtree.
        if parent_data.styles().is_display_none() {
            if log.allow() { debug!("Parent {:?} is display:none, culling traversal", parent); }
            return false;
        }

        // Gecko-only XBL handling.
        //
        // If we're computing initial styles and the parent has a Gecko XBL
        // binding, that binding may inject anonymous children and remap the
        // explicit children to an insertion point (or hide them entirely). It
        // may also specify a scoped stylesheet, which changes the rules that
        // apply within the subtree. These two effects can invalidate the result
        // of property inheritance and selector matching (respectively) within the
        // subtree.
        //
        // To avoid wasting work, we defer initial styling of XBL subtrees
        // until frame construction, which does an explicit traversal of the
        // unstyled children after shuffling the subtree. That explicit
        // traversal may in turn find other bound elements, which get handled
        // in the same way.
        //
        // We explicitly avoid handling restyles here (explicitly removing or
        // changing bindings), since that adds complexity and is rarer. If it
        // happens, we may just end up doing wasted work, since Gecko
        // recursively drops Servo ElementData when the XBL insertion parent of
        // an Element is changed.
        if cfg!(feature = "gecko") && parent_data.is_styled_initial() &&
           parent_data.styles().primary.values.has_moz_binding() {
            if log.allow() { debug!("Parent {:?} has XBL binding, deferring traversal", parent); }
            return false;
        }

        return true;

    }

    /// Helper for the traversal implementations to select the children that
    /// should be enqueued for processing.
    fn traverse_children<F: FnMut(N)>(parent: N::ConcreteElement, mut f: F)
    {
        // Check if we're allowed to traverse past this element.
        if !Self::should_traverse_children(parent, &parent.borrow_data().unwrap(), MayLog) {
            return;
        }

        for kid in parent.as_node().children() {
            if Self::node_needs_traversal(kid) {
                let el = kid.as_element();
                if el.as_ref().and_then(|el| el.borrow_data())
                              .map_or(false, |d| d.is_restyle())
                {
                    unsafe { parent.set_dirty_descendants(); }
                }
                f(kid);
            }
        }
    }

    /// Ensures the existence of the ElementData, and returns it. This can't live
    /// on TNode because of the trait-based separation between Servo's script
    /// and layout crates.
    ///
    /// This is only safe to call in top-down traversal before processing the
    /// children of |element|.
    unsafe fn ensure_element_data(element: &N::ConcreteElement) -> &AtomicRefCell<ElementData>;

    /// Clears the ElementData attached to this element, if any.
    ///
    /// This is only safe to call in top-down traversal before processing the
    /// children of |element|.
    unsafe fn clear_element_data(element: &N::ConcreteElement);

    fn shared_context(&self) -> &SharedStyleContext;

    fn create_thread_local_context(&self) -> Self::ThreadLocalContext;
}

/// Determines the amount of relations where we're going to share style.
#[inline]
pub fn relations_are_shareable(relations: &StyleRelations) -> bool {
    use selectors::matching::*;
    !relations.intersects(AFFECTED_BY_ID_SELECTOR |
                          AFFECTED_BY_PSEUDO_ELEMENTS | AFFECTED_BY_STATE |
                          AFFECTED_BY_NON_COMMON_STYLE_AFFECTING_ATTRIBUTE_SELECTOR |
                          AFFECTED_BY_STYLE_ATTRIBUTE |
                          AFFECTED_BY_PRESENTATIONAL_HINTS)
}

/// Handles lazy resolution of style in display:none subtrees. See the comment
/// at the callsite in query.rs.
pub fn style_element_in_display_none_subtree<E, F>(context: &StyleContext<E>,
                                                   element: E, init_data: &F) -> E
    where E: TElement,
          F: Fn(E),
{
    // Check the base case.
    if element.get_data().is_some() {
        // See the comment on `cascade_node` for why we allow this on Gecko.
        debug_assert!(cfg!(feature = "gecko") || element.borrow_data().unwrap().has_current_styles());
        debug_assert!(element.borrow_data().unwrap().styles().is_display_none());
        return element;
    }

    // Ensure the parent is styled.
    let parent = element.parent_element().unwrap();
    let display_none_root = style_element_in_display_none_subtree(context, parent, init_data);

    // Initialize our data.
    init_data(element);

    // Resolve our style.
    let mut data = element.mutate_data().unwrap();
    let match_results = element.match_element(context, None);
    unsafe {
        let shareable = match_results.primary_is_shareable();
        element.cascade_node(context, &mut data, Some(parent),
                             match_results.primary,
                             match_results.per_pseudo,
                             shareable);
    }

    display_none_root
}

/// Calculates the style for a single node.
#[inline]
#[allow(unsafe_code)]
pub fn recalc_style_at<E, D>(traversal: &D,
                             traversal_data: &mut PerLevelTraversalData,
                             context: &mut StyleContext<E>,
                             element: E,
                             mut data: &mut AtomicRefMut<ElementData>)
    where E: TElement,
          D: DomTraversal<E::ConcreteNode>
{
    debug_assert!(data.as_restyle().map_or(true, |r| r.snapshot.is_none()),
                  "Snapshots should be expanded by the caller");

    let compute_self = !data.has_current_styles();
    let mut inherited_style_changed = false;

    debug!("recalc_style_at: {:?} (compute_self={:?}, dirty_descendants={:?}, data={:?})",
           element, compute_self, element.has_dirty_descendants(), data);

    // Compute style for this element if necessary.
    if compute_self {
        inherited_style_changed = compute_style(traversal, traversal_data, context, element, &mut data);
    }

    // Now that matching and cascading is done, clear the bits corresponding to
    // those operations and compute the propagated restyle hint.
    let empty_hint = StoredRestyleHint::empty();
    let propagated_hint = match data.as_restyle_mut() {
        None => empty_hint,
        Some(r) => {
            r.recascade = false;
            mem::replace(&mut r.hint, empty_hint).propagate()
        },
    };
    debug_assert!(data.has_current_styles());
    trace!("propagated_hint={:?}, inherited_style_changed={:?}", propagated_hint, inherited_style_changed);

    // Preprocess children, propagating restyle hints and handling sibling relationships.
    if D::should_traverse_children(element, &data, DontLog) &&
       (element.has_dirty_descendants() || !propagated_hint.is_empty() || inherited_style_changed) {
        preprocess_children(traversal, element, propagated_hint, inherited_style_changed);
    }
}

// Computes style, returning true if the inherited styles changed for this
// element.
//
// FIXME(bholley): This should differentiate between matching and cascading,
// since we have separate bits for each now.
fn compute_style<E, D>(_traversal: &D,
                       traversal_data: &mut PerLevelTraversalData,
                       context: &mut StyleContext<E>,
                       element: E,
                       mut data: &mut AtomicRefMut<ElementData>) -> bool
    where E: TElement,
          D: DomTraversal<E::ConcreteNode>,
{
    let shared_context = context.shared;
    let mut bf = take_thread_local_bloom_filter(shared_context);
    // Ensure the bloom filter is up to date.
    let dom_depth = bf.insert_parents_recovering(element,
                                                 traversal_data.current_dom_depth,
                                                 shared_context.generation);

    // Update the dom depth with the up-to-date dom depth.
    //
    // Note that this is always the same than the pre-existing depth, but it can
    // change from unknown to known at this step.
    traversal_data.current_dom_depth = Some(dom_depth);

    bf.assert_complete(element);

    // Check to see whether we can share a style with someone.
    let sharing_result = if element.parent_element().is_none() {
        StyleSharingResult::CannotShare
    } else {
        unsafe { element.share_style_if_possible(&mut context.thread_local.style_sharing_candidate_cache,
                                                 shared_context, &mut data) }
    };

    // Otherwise, match and cascade selectors.
    match sharing_result {
        StyleSharingResult::CannotShare => {
            let match_results;
            let shareable_element = {
                if opts::get().style_sharing_stats {
                    STYLE_SHARING_CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
                }

                // Perform the CSS selector matching.
                match_results = element.match_element(context, Some(bf.filter()));
                if match_results.primary_is_shareable() {
                    Some(element)
                } else {
                    None
                }
            };
            let relations = match_results.relations;

            // Perform the CSS cascade.
            unsafe {
                let shareable = match_results.primary_is_shareable();
                element.cascade_node(context, &mut data,
                                     element.parent_element(),
                                     match_results.primary,
                                     match_results.per_pseudo,
                                     shareable);
            }

            // Add ourselves to the LRU cache.
            if let Some(element) = shareable_element {
                context.thread_local
                       .style_sharing_candidate_cache
                       .insert_if_possible(&element, &data.styles().primary.values, relations);
            }
        }
        StyleSharingResult::StyleWasShared(index) => {
            if opts::get().style_sharing_stats {
                STYLE_SHARING_CACHE_HITS.fetch_add(1, Ordering::Relaxed);
            }
            context.thread_local.style_sharing_candidate_cache.touch(index);
        }
    }

    // If we're restyling this element to display:none, throw away all style data
    // in the subtree, notify the caller to early-return.
    let display_none = data.styles().is_display_none();
    if display_none {
        debug!("New element style is display:none - clearing data from descendants.");
        clear_descendant_data(element, &|e| unsafe { D::clear_element_data(&e) });
    }

    // TODO(emilio): It's pointless to insert the element in the parallel
    // traversal, but it may be worth todo it for sequential restyling. What we
    // do now is trying to recover it which in that case is really cheap, so
    // we'd save a few instructions, but probably not worth given the added
    // complexity.
    put_thread_local_bloom_filter(bf);

    // FIXME(bholley): Compute this accurately from the call to CalcStyleDifference.
    let inherited_styles_changed = true;

    inherited_styles_changed
}

fn preprocess_children<E, D>(traversal: &D,
                             element: E,
                             mut propagated_hint: StoredRestyleHint,
                             parent_inherited_style_changed: bool)
    where E: TElement,
          D: DomTraversal<E::ConcreteNode>
{
    // Loop over all the children.
    for child in element.as_node().children() {
        // FIXME(bholley): Add TElement::element_children instead of this.
        let child = match child.as_element() {
            Some(el) => el,
            None => continue,
        };

        let mut child_data = unsafe { D::ensure_element_data(&child).borrow_mut() };
        if child_data.is_unstyled_initial() {
            continue;
        }

        let mut restyle_data = match child_data.restyle() {
            Some(d) => d,
            None => continue,
        };

        // Propagate the parent and sibling restyle hint.
        if !propagated_hint.is_empty() {
            restyle_data.hint.insert(&propagated_hint);
        }

        // Handle element snapshots.
        let stylist = &traversal.shared_context().stylist;
        let later_siblings = restyle_data.expand_snapshot(child, stylist);
        if later_siblings {
            propagated_hint.insert(&(RESTYLE_SELF | RESTYLE_DESCENDANTS).into());
        }

        // If properties that we inherited from the parent changed, we need to recascade.
        //
        // FIXME(bholley): Need to handle explicitly-inherited reset properties somewhere.
        if parent_inherited_style_changed {
            restyle_data.recascade = true;
        }
    }
}

pub fn clear_descendant_data<E: TElement, F: Fn(E)>(el: E, clear_data: &F) {
    for kid in el.as_node().children() {
        if let Some(kid) = kid.as_element() {
            // We maintain an invariant that, if an element has data, all its ancestors
            // have data as well. By consequence, any element without data has no
            // descendants with data.
            if kid.get_data().is_some() {
                clear_data(kid);
                clear_descendant_data(kid, clear_data);
            }
        }
    }

    unsafe { el.unset_dirty_descendants(); }
}
