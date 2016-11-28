/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversing the DOM tree; the bloom filter.

use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use bloom::StyleBloom;
use context::{LocalStyleContext, SharedStyleContext, StyleContext};
use data::{ElementData, RestyleData, StoredRestyleHint};
use dom::{OpaqueNode, StylingMode, TElement, TNode};
use matching::{MatchMethods, StyleSharingResult};
use restyle_hints::{RESTYLE_DESCENDANTS, RESTYLE_LATER_SIBLINGS, RESTYLE_SELF};
use selectors::matching::StyleRelations;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use util::opts;

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
    debug!("{} taking bf", ::tid::tid());

    STYLE_BLOOM.with(|style_bloom| {
        style_bloom.borrow_mut().take()
            .unwrap_or_else(|| StyleBloom::new(context.generation))
    })
}

pub fn put_thread_local_bloom_filter(bf: StyleBloom) {
    debug!("[{}] putting bloom filter back", ::tid::tid());

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
pub fn remove_from_bloom_filter<'a, E, C>(context: &C, root: OpaqueNode, element: E)
    where E: TElement,
          C: StyleContext<'a>
{
    debug!("[{}] remove_from_bloom_filter", ::tid::tid());

    // We may have arrived to `reconstruct_flows` without entering in style
    // recalc at all due to our optimizations, nor that it's up to date, so we
    // can't ensure there's a bloom filter at all.
    let bf = STYLE_BLOOM.with(|style_bloom| {
        style_bloom.borrow_mut().take()
    });

    if let Some(mut bf) = bf {
        if context.shared_context().generation == bf.generation() {
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

pub trait DomTraversalContext<N: TNode> {
    type SharedContext: Sync + 'static;

    fn new<'a>(&'a Self::SharedContext, OpaqueNode) -> Self;

    /// Process `node` on the way down, before its children have been processed.
    fn process_preorder(&self, node: N, data: &mut PerLevelTraversalData);

    /// Process `node` on the way up, after its children have been processed.
    ///
    /// This is only executed if `needs_postorder_traversal` returns true.
    fn process_postorder(&self, node: N);

    /// Boolean that specifies whether a bottom up traversal should be
    /// performed.
    ///
    /// If it's false, then process_postorder has no effect at all.
    fn needs_postorder_traversal(&self) -> bool { true }

    /// Returns true if traversal should visit the given child.
    fn should_traverse_child(child: N) -> bool;

    /// Helper for the traversal implementations to select the children that
    /// should be enqueued for processing.
    fn traverse_children<F: FnMut(N)>(parent: N::ConcreteElement, mut f: F)
    {
        use dom::StylingMode::Restyle;

        if parent.is_display_none() {
            return;
        }

        for kid in parent.as_node().children() {
            if Self::should_traverse_child(kid) {
                if kid.as_element().map_or(false, |el| el.styling_mode() == Restyle) {
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

    fn local_context(&self) -> &LocalStyleContext;
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
pub fn style_element_in_display_none_subtree<'a, E, C, F>(element: E,
                                                          init_data: &F,
                                                          context: &'a C) -> E
    where E: TElement,
          C: StyleContext<'a>,
          F: Fn(E),
{
    // Check the base case.
    if element.get_data().is_some() {
        debug_assert!(element.is_display_none());
        return element;
    }

    // Ensure the parent is styled.
    let parent = element.parent_element().unwrap();
    let display_none_root = style_element_in_display_none_subtree(parent, init_data, context);

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
pub fn recalc_style_at<'a, E, C, D>(context: &'a C,
                                    data: &mut PerLevelTraversalData,
                                    element: E)
    where E: TElement,
          C: StyleContext<'a>,
          D: DomTraversalContext<E::ConcreteNode>
{
    let mode = element.styling_mode();
    let should_compute = element.borrow_data().map_or(true, |d| d.get_current_styles().is_none());
    debug!("recalc_style_at: {:?} (should_compute={:?} mode={:?}, data={:?})",
           element, should_compute, mode, element.borrow_data());

    let (computed_display_none, propagated_hint) = if should_compute {
        compute_style::<_, _, D>(context, data, element)
    } else {
        (false, StoredRestyleHint::empty())
    };

    // Preprocess children, computing restyle hints and handling sibling relationships.
    //
    // We don't need to do this if we're not traversing children, or if we're performing
    // initial styling.
    let will_traverse_children = !computed_display_none &&
                                 (mode == StylingMode::Restyle ||
                                  mode == StylingMode::Traverse);
    if will_traverse_children {
        preprocess_children::<_, _, D>(context, element, propagated_hint,
                                       mode == StylingMode::Restyle);
    }
}

fn compute_style<'a, E, C, D>(context: &'a C,
                              data: &mut PerLevelTraversalData,
                              element: E) -> (bool, StoredRestyleHint)
    where E: TElement,
          C: StyleContext<'a>,
          D: DomTraversalContext<E::ConcreteNode>
{
    let shared_context = context.shared_context();
    let mut bf = take_thread_local_bloom_filter(shared_context);
    // Ensure the bloom filter is up to date.
    let dom_depth = bf.insert_parents_recovering(element,
                                                 data.current_dom_depth,
                                                 shared_context.generation);

    // Update the dom depth with the up-to-date dom depth.
    //
    // Note that this is always the same than the pre-existing depth, but it can
    // change from unknown to known at this step.
    data.current_dom_depth = Some(dom_depth);

    bf.assert_complete(element);

    let mut data = unsafe { D::ensure_element_data(&element).borrow_mut() };
    debug_assert!(!data.is_persistent());

    // Check to see whether we can share a style with someone.
    let style_sharing_candidate_cache =
        &mut context.local_context().style_sharing_candidate_cache.borrow_mut();

    let sharing_result = if element.parent_element().is_none() {
        StyleSharingResult::CannotShare
    } else {
        unsafe { element.share_style_if_possible(style_sharing_candidate_cache,
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
                style_sharing_candidate_cache.insert_if_possible(&element,
                                                                 &data.current_styles().primary.values,
                                                                 relations);
            }
        }
        StyleSharingResult::StyleWasShared(index) => {
            if opts::get().style_sharing_stats {
                STYLE_SHARING_CACHE_HITS.fetch_add(1, Ordering::Relaxed);
            }
            style_sharing_candidate_cache.touch(index);
        }
    }

    // If we're restyling this element to display:none, throw away all style data
    // in the subtree, notify the caller to early-return.
    let display_none = data.current_styles().is_display_none();
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

    (display_none, data.as_restyle().map_or(StoredRestyleHint::empty(), |r| r.hint.propagate()))
}

fn preprocess_children<'a, E, C, D>(context: &'a C,
                                    element: E,
                                    mut propagated_hint: StoredRestyleHint,
                                    restyled_parent: bool)
    where E: TElement,
          C: StyleContext<'a>,
          D: DomTraversalContext<E::ConcreteNode>
{
    // Loop over all the children.
    for child in element.as_node().children() {
        // FIXME(bholley): Add TElement::element_children instead of this.
        let child = match child.as_element() {
            Some(el) => el,
            None => continue,
        };

        // Set up our lazy child restyle data.
        let mut child_data = unsafe { LazyRestyleData::<E, D>::new(&child) };

        // Propagate the parent and sibling restyle hint.
        if !propagated_hint.is_empty() {
            child_data.ensure().map(|d| d.hint.insert(&propagated_hint));
        }

        // Handle element snashots.
        if child_data.has_snapshot() {
            // Compute the restyle hint.
            let mut restyle_data = child_data.ensure().unwrap();
            let mut hint = context.shared_context().stylist
                                  .compute_restyle_hint(&child,
                                                        restyle_data.snapshot.as_ref().unwrap(),
                                                        child.get_state());

            // If the hint includes a directive for later siblings, strip
            // it out and modify the base hint for future siblings.
            if hint.contains(RESTYLE_LATER_SIBLINGS) {
                hint.remove(RESTYLE_LATER_SIBLINGS);
                propagated_hint.insert(&(RESTYLE_SELF | RESTYLE_DESCENDANTS).into());
            }

            // Insert the hint.
            if !hint.is_empty() {
                restyle_data.hint.insert(&hint.into());
            }
        }

        // If we restyled this node, conservatively mark all our children as
        // needing a re-cascade. Once we have the rule tree, we will be able
        // to distinguish between re-matching and re-cascading.
        if restyled_parent {
            child_data.ensure();
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

/// Various steps in the child preparation algorithm above may cause us to lazily
/// instantiate the ElementData on the child. Encapsulate that logic into a
/// convenient abstraction.
struct LazyRestyleData<'b, E: TElement + 'b, D: DomTraversalContext<E::ConcreteNode>> {
    data: Option<AtomicRefMut<'b, ElementData>>,
    element: &'b E,
    phantom: PhantomData<D>,
}

impl<'b, E: TElement, D: DomTraversalContext<E::ConcreteNode>> LazyRestyleData<'b, E, D> {
    /// This may lazily instantiate ElementData, and is therefore only safe to
    /// call on an element for which we have exclusive access.
    unsafe fn new(element: &'b E) -> Self {
        LazyRestyleData {
            data: None,
            element: element,
            phantom: PhantomData,
        }
    }

    fn ensure(&mut self) -> Option<&mut RestyleData> {
        if self.data.is_none() {
            let mut d = unsafe { D::ensure_element_data(self.element).borrow_mut() };
            d.restyle();
            self.data = Some(d);
        }

        self.data.as_mut().unwrap().as_restyle_mut()
    }

    /// Checks for the existence of an element snapshot without lazily instantiating
    /// anything. This allows the traversal to cheaply pass through already-styled
    /// nodes when they don't need a restyle.
    fn has_snapshot(&self) -> bool {
        // If there's no element data, we're done.
        let raw_data = self.element.get_data();
        if raw_data.is_none() {
            debug_assert!(self.data.is_none());
            return false;
        }

        // If there is element data, we still may not have committed to processing
        // the node. Carefully get a reference to the data.
        let maybe_tmp_borrow;
        let borrow_ref = match self.data {
            Some(ref d) => d,
            None => {
                maybe_tmp_borrow = raw_data.unwrap().borrow_mut();
                &maybe_tmp_borrow
            }
        };

        // Check for a snapshot.
        borrow_ref.as_restyle().map_or(false, |d| d.snapshot.is_some())
    }
}
