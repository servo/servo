/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversing the DOM tree; the bloom filter.

use atomic_refcell::AtomicRefCell;
use context::{LocalStyleContext, SharedStyleContext, StyleContext};
use data::NodeData;
use dom::{NodeInfo, OpaqueNode, StylingMode, TElement, TNode, UnsafeNode};
use matching::{ApplicableDeclarations, MatchMethods, StyleSharingResult};
use selectors::bloom::BloomFilter;
use selectors::matching::StyleRelations;
use std::cell::RefCell;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use tid::tid;
use util::opts;

/// Every time we do another layout, the old bloom filters are invalid. This is
/// detected by ticking a generation number every layout.
pub type Generation = u32;

/// This enum tells us about whether we can stop restyling or not after styling
/// an element.
///
/// So far this only happens where a display: none node is found.
#[derive(Clone, Copy, PartialEq)]
pub enum RestyleResult {
    Continue,
    Stop,
}

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
        el = match el.as_node().layout_parent_node(root).and_then(|x| x.as_element()) {
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

    match node.layout_parent_node(root) {
        None => {
            debug!("[{}] - {:X}, and deleting BF.", tid(), unsafe_layout_node.0);
            // If this is the reflow root, eat the thread-local bloom filter.
        }
        Some(parent) => {
            // Otherwise, put it back, but remove this node.
            node.as_element().map(|x| x.remove_from_bloom_filter(&mut *bf));
            let unsafe_parent = parent.to_unsafe();
            put_thread_local_bloom_filter(bf, &unsafe_parent, &context.shared_context());
        },
    };
}

pub trait DomTraversalContext<N: TNode> {
    type SharedContext: Sync + 'static;

    fn new<'a>(&'a Self::SharedContext, OpaqueNode) -> Self;

    /// Process `node` on the way down, before its children have been processed.
    fn process_preorder(&self, node: N) -> RestyleResult;

    /// Process `node` on the way up, after its children have been processed.
    ///
    /// This is only executed if `needs_postorder_traversal` returns true.
    fn process_postorder(&self, node: N);

    /// Boolean that specifies whether a bottom up traversal should be
    /// performed.
    ///
    /// If it's false, then process_postorder has no effect at all.
    fn needs_postorder_traversal(&self) -> bool { true }

    /// Helper for the traversal implementations to select the children that
    /// should be enqueued for processing.
    fn traverse_children<F: FnMut(N)>(parent: N, mut f: F)
    {
        // If we enqueue any children for traversal, we need to set the dirty
        // descendants bit. Avoid doing it more than once.
        let mut marked_dirty_descendants = false;

        for kid in parent.children() {
            if kid.styling_mode() != StylingMode::Stop {
                if !marked_dirty_descendants {
                    unsafe { parent.set_dirty_descendants(); }
                    marked_dirty_descendants = true;
                }
                f(kid);
            }
        }
    }

    /// Ensures the existence of the NodeData, and returns it. This can't live
    /// on TNode because of the trait-based separation between Servo's script
    /// and layout crates.
    fn ensure_node_data(node: &N) -> &AtomicRefCell<NodeData>;

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

pub fn ensure_element_styled<'a, E, C>(element: E,
                                       context: &'a C)
    where E: TElement,
          C: StyleContext<'a>
{
    let mut display_none = false;
    ensure_element_styled_internal(element, context, &mut display_none);
}

#[allow(unsafe_code)]
fn ensure_element_styled_internal<'a, E, C>(element: E,
                                            context: &'a C,
                                            parents_had_display_none: &mut bool)
    where E: TElement,
          C: StyleContext<'a>
{
    use properties::longhands::display::computed_value as display;

    // NB: The node data must be initialized here.

    // We need to go to the root and ensure their style is up to date.
    //
    // This means potentially a bit of wasted work (usually not much). We could
    // add a flag at the node at which point we stopped the traversal to know
    // where should we stop, but let's not add that complication unless needed.
    let parent = element.parent_element();
    if let Some(parent) = parent {
        ensure_element_styled_internal(parent, context, parents_had_display_none);
    }

    // Common case: our style is already resolved and none of our ancestors had
    // display: none.
    //
    // We only need to mark whether we have display none, and forget about it,
    // our style is up to date.
    let node = element.as_node();
    if let Some(data) = node.borrow_data() {
        if let Some(style) = data.get_current_styles().map(|x| &x.primary) {
            if !*parents_had_display_none {
                *parents_had_display_none = style.get_box().clone_display() == display::T::none;
                return;
            }
        }
    }

    // Otherwise, our style might be out of date. Time to do selector matching
    // if appropriate and cascade the node.
    //
    // Note that we could add the bloom filter's complexity here, but that's
    // probably not necessary since we're likely to be matching only a few
    // nodes, at best.
    let mut applicable_declarations = ApplicableDeclarations::new();
    let stylist = &context.shared_context().stylist;

    element.match_element(&**stylist,
                          None,
                          &mut applicable_declarations);

    unsafe {
        element.cascade_node(context, parent, &applicable_declarations);
    }
}

/// Calculates the style for a single node.
#[inline]
#[allow(unsafe_code)]
pub fn recalc_style_at<'a, E, C, D>(context: &'a C,
                                    root: OpaqueNode,
                                    element: E) -> RestyleResult
    where E: TElement,
          C: StyleContext<'a>,
          D: DomTraversalContext<E::ConcreteNode>
{
    // Get the style bloom filter.
    let mut bf = take_thread_local_bloom_filter(element.parent_element(), root, context.shared_context());

    let mut restyle_result = RestyleResult::Continue;
    let mode = element.as_node().styling_mode();
    debug_assert!(mode != StylingMode::Stop, "Parent should not have enqueued us");
    if mode != StylingMode::Traverse {
        // Check to see whether we can share a style with someone.
        let style_sharing_candidate_cache =
            &mut context.local_context().style_sharing_candidate_cache.borrow_mut();

        let sharing_result = if element.parent_element().is_none() {
            StyleSharingResult::CannotShare
        } else {
            unsafe { element.share_style_if_possible(style_sharing_candidate_cache, context.shared_context()) }
        };

        // Otherwise, match and cascade selectors.
        match sharing_result {
            StyleSharingResult::CannotShare => {
                let mut applicable_declarations = ApplicableDeclarations::new();

                let relations;
                let shareable_element = {
                    if opts::get().style_sharing_stats {
                        STYLE_SHARING_CACHE_MISSES.fetch_add(1, Ordering::Relaxed);
                    }

                    // Perform the CSS selector matching.
                    let stylist = &context.shared_context().stylist;

                    relations = element.match_element(&**stylist,
                                                      Some(&*bf),
                                                      &mut applicable_declarations);

                    debug!("Result of selector matching: {:?}", relations);

                    if relations_are_shareable(&relations) {
                        Some(element)
                    } else {
                        None
                    }
                };

                // Perform the CSS cascade.
                unsafe {
                    restyle_result = element.cascade_node(context,
                                                          element.parent_element(),
                                                          &applicable_declarations);
                }

                // Add ourselves to the LRU cache.
                if let Some(element) = shareable_element {
                    style_sharing_candidate_cache.insert_if_possible(&element, relations);
                }
            }
            StyleSharingResult::StyleWasShared(index, damage, cached_restyle_result) => {
                restyle_result = cached_restyle_result;
                if opts::get().style_sharing_stats {
                    STYLE_SHARING_CACHE_HITS.fetch_add(1, Ordering::Relaxed);
                }
                style_sharing_candidate_cache.touch(index);
                element.set_restyle_damage(damage);
            }
        }
    }

    // If we restyled this node, conservatively mark all our children as needing
    // processing. The eventual algorithm we're designing does this in a more granular
    // fashion.
    if mode == StylingMode::Restyle && restyle_result == RestyleResult::Continue {
        for kid in element.as_node().children() {
            let mut data = D::ensure_node_data(&kid).borrow_mut();
            if kid.is_text_node() {
                data.ensure_restyle_data();
            } else {
                data.gather_previous_styles(|| kid.get_styles_from_frame());
                if data.previous_styles().is_some() {
                    data.ensure_restyle_data();
                }
            }
        }
    }

    let unsafe_layout_node = element.as_node().to_unsafe();

    // Before running the children, we need to insert our nodes into the bloom
    // filter.
    debug!("[{}] + {:X}", tid(), unsafe_layout_node.0);
    element.insert_into_bloom_filter(&mut *bf);

    // NB: flow construction updates the bloom filter on the way up.
    put_thread_local_bloom_filter(bf, &unsafe_layout_node, context.shared_context());

    restyle_result
}
