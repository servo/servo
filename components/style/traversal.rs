/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversing the DOM tree; the bloom filter.

use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use context::{LocalStyleContext, SharedStyleContext, StyleContext};
use data::{ElementData, RestyleData, StoredRestyleHint};
use dom::{OpaqueNode, StylingMode, TElement, TNode, UnsafeNode};
use matching::{MatchMethods, StyleSharingResult};
use restyle_hints::{RESTYLE_DESCENDANTS, RESTYLE_LATER_SIBLINGS, RESTYLE_SELF};
use selectors::bloom::BloomFilter;
use selectors::matching::StyleRelations;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use tid::tid;
use util::opts;

/// Every time we do another layout, the old bloom filters are invalid. This is
/// detected by ticking a generation number every layout.
pub type Generation = u32;

/// Style sharing candidate cache stats. These are only used when
/// `-Z style-sharing-stats` is given.
pub static STYLE_SHARING_CACHE_HITS: AtomicUsize = ATOMIC_USIZE_INIT;
pub static STYLE_SHARING_CACHE_MISSES: AtomicUsize = ATOMIC_USIZE_INIT;

/// A pair of the bloom filter used for css selector matching, and the node to
/// which it applies. This is used to efficiently do `Descendant` selector
/// matches. Thanks to the bloom filter, we can avoid walking up the tree
/// looking for ancestors that aren't there in the majority of cases.
///
/// As we walk down the DOM tree a thread-local bloom filter is built of all the
/// CSS `SimpleSelector`s which are part of a `Descendant` compound selector
/// (i.e. paired with a `Descendant` combinator, in the `next` field of a
/// `CompoundSelector`.
///
/// Before a `Descendant` selector match is tried, it's compared against the
/// bloom filter. If the bloom filter can exclude it, the selector is quickly
/// rejected.
///
/// When done styling a node, all selectors previously inserted into the filter
/// are removed.
///
/// Since a work-stealing queue is used for styling, sometimes, the bloom filter
/// will no longer be the for the parent of the node we're currently on. When
/// this happens, the thread local bloom filter will be thrown away and rebuilt.
thread_local!(
    static STYLE_BLOOM: RefCell<Option<(Box<BloomFilter>, UnsafeNode, Generation)>> = RefCell::new(None));

/// Returns the thread local bloom filter.
///
/// If one does not exist, a new one will be made for you. If it is out of date,
/// it will be cleared and reused.
pub fn take_thread_local_bloom_filter<E>(parent_element: Option<E>,
                                         root: OpaqueNode,
                                         context: &SharedStyleContext)
                                         -> Box<BloomFilter>
                                         where E: TElement {
    STYLE_BLOOM.with(|style_bloom| {
        match (parent_element, style_bloom.borrow_mut().take()) {
            // Root node. Needs new bloom filter.
            (None,     _  ) => {
                debug!("[{}] No parent, but new bloom filter!", tid());
                Box::new(BloomFilter::new())
            }
            // No bloom filter for this thread yet.
            (Some(parent), None) => {
                let mut bloom_filter = Box::new(BloomFilter::new());
                insert_ancestors_into_bloom_filter(&mut bloom_filter, parent, root);
                bloom_filter
            }
            // Found cached bloom filter.
            (Some(parent), Some((mut bloom_filter, old_node, old_generation))) => {
                if old_node == parent.as_node().to_unsafe() &&
                    old_generation == context.generation {
                    // Hey, the cached parent is our parent! We can reuse the bloom filter.
                    debug!("[{}] Parent matches (={}). Reusing bloom filter.", tid(), old_node.0);
                } else {
                    // Oh no. the cached parent is stale. I guess we need a new one. Reuse the existing
                    // allocation to avoid malloc churn.
                    bloom_filter.clear();
                    insert_ancestors_into_bloom_filter(&mut bloom_filter, parent, root);
                }
                bloom_filter
            },
        }
    })
}

pub fn put_thread_local_bloom_filter(bf: Box<BloomFilter>, unsafe_node: &UnsafeNode,
                                     context: &SharedStyleContext) {
    STYLE_BLOOM.with(move |style_bloom| {
        assert!(style_bloom.borrow().is_none(),
                "Putting into a never-taken thread-local bloom filter");
        *style_bloom.borrow_mut() = Some((bf, *unsafe_node, context.generation));
    })
}

/// "Ancestors" in this context is inclusive of ourselves.
fn insert_ancestors_into_bloom_filter<E>(bf: &mut Box<BloomFilter>,
                                         mut el: E,
                                         root: OpaqueNode)
                                         where E: TElement {
    debug!("[{}] Inserting ancestors.", tid());
    let mut ancestors = 0;
    loop {
        ancestors += 1;

        el.insert_into_bloom_filter(&mut **bf);
        el = match el.layout_parent_element(root) {
            None => break,
            Some(p) => p,
        };
    }
    debug!("[{}] Inserted {} ancestors.", tid(), ancestors);
}

pub fn remove_from_bloom_filter<'a, N, C>(context: &C, root: OpaqueNode, node: N)
    where N: TNode,
          C: StyleContext<'a>
{
    let unsafe_layout_node = node.to_unsafe();

    let (mut bf, old_node, old_generation) =
        STYLE_BLOOM.with(|style_bloom| {
            style_bloom.borrow_mut()
                       .take()
                       .expect("The bloom filter should have been set by style recalc.")
        });

    assert_eq!(old_node, unsafe_layout_node);
    assert_eq!(old_generation, context.shared_context().generation);

    match node.layout_parent_element(root) {
        None => {
            debug!("[{}] - {:X}, and deleting BF.", tid(), unsafe_layout_node.0);
            // If this is the reflow root, eat the thread-local bloom filter.
        }
        Some(parent) => {
            // Otherwise, put it back, but remove this node.
            node.as_element().map(|x| x.remove_from_bloom_filter(&mut *bf));
            let unsafe_parent = parent.as_node().to_unsafe();
            put_thread_local_bloom_filter(bf, &unsafe_parent, &context.shared_context());
        },
    };
}

pub trait DomTraversalContext<N: TNode> {
    type SharedContext: Sync + 'static;

    fn new<'a>(&'a Self::SharedContext, OpaqueNode) -> Self;

    /// Process `node` on the way down, before its children have been processed.
    fn process_preorder(&self, node: N);

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
                                    root: OpaqueNode,
                                    element: E)
    where E: TElement,
          C: StyleContext<'a>,
          D: DomTraversalContext<E::ConcreteNode>
{
    // Get the style bloom filter.
    //
    // FIXME(bholley): We need to do these even in the StylingMode::Stop case
    // to handshake with the unconditional pop during servo's bottom-up
    // traversal. We should avoid doing work here in the Stop case when we
    // redesign the bloom filter.
    let mut bf = take_thread_local_bloom_filter(element.parent_element(), root, context.shared_context());

    let mode = element.styling_mode();
    let should_compute = element.borrow_data().map_or(true, |d| d.get_current_styles().is_none());
    debug!("recalc_style_at: {:?} (should_compute={:?} mode={:?}, data={:?})",
           element, should_compute, mode, element.borrow_data());

    let (computed_display_none, propagated_hint) = if should_compute {
        compute_style::<_, _, D>(context, element, &*bf)
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

    let unsafe_layout_node = element.as_node().to_unsafe();

    // Before running the children, we need to insert our nodes into the bloom
    // filter.
    debug!("[{}] + {:X}", tid(), unsafe_layout_node.0);
    element.insert_into_bloom_filter(&mut *bf);

    // NB: flow construction updates the bloom filter on the way up.
    put_thread_local_bloom_filter(bf, &unsafe_layout_node, context.shared_context());
}

fn compute_style<'a, E, C, D>(context: &'a C,
                              element: E,
                              bloom_filter: &BloomFilter) -> (bool, StoredRestyleHint)
    where E: TElement,
          C: StyleContext<'a>,
          D: DomTraversalContext<E::ConcreteNode>
{
    let mut data = unsafe { D::ensure_element_data(&element).borrow_mut() };
    debug_assert!(!data.is_persistent());

    // Check to see whether we can share a style with someone.
    let style_sharing_candidate_cache =
        &mut context.local_context().style_sharing_candidate_cache.borrow_mut();

    let sharing_result = if element.parent_element().is_none() {
        StyleSharingResult::CannotShare
    } else {
        unsafe { element.share_style_if_possible(style_sharing_candidate_cache,
                                                 context.shared_context(), &mut data) }
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
                match_results = element.match_element(context, Some(bloom_filter));
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
