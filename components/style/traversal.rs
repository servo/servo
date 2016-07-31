/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversing the DOM tree; the bloom filter.

use animation;
use context::{LocalStyleContext, SharedStyleContext, StyleContext};
use dom::{OpaqueNode, TElement, TNode, TRestyleDamage, UnsafeNode};
use matching::{ApplicableDeclarations, ElementMatchMethods, MatchMethods, StyleSharingResult};
use selector_impl::SelectorImplExt;
use selectors::Element;
use selectors::bloom::BloomFilter;
use selectors::matching::StyleRelations;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use tid::tid;
use util::opts;
use values::HasViewportPercentage;

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
fn take_thread_local_bloom_filter<N>(parent_node: Option<N>,
                                     root: OpaqueNode,
                                     context: &SharedStyleContext)
                                     -> Box<BloomFilter>
                                     where N: TNode {
    STYLE_BLOOM.with(|style_bloom| {
        match (parent_node, style_bloom.borrow_mut().take()) {
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
                if old_node == parent.to_unsafe() &&
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

fn put_thread_local_bloom_filter(bf: Box<BloomFilter>, unsafe_node: &UnsafeNode,
                                 context: &SharedStyleContext) {
    STYLE_BLOOM.with(move |style_bloom| {
        assert!(style_bloom.borrow().is_none(),
                "Putting into a never-taken thread-local bloom filter");
        *style_bloom.borrow_mut() = Some((bf, *unsafe_node, context.generation));
    })
}

/// "Ancestors" in this context is inclusive of ourselves.
fn insert_ancestors_into_bloom_filter<N>(bf: &mut Box<BloomFilter>,
                                         mut n: N,
                                         root: OpaqueNode)
                                         where N: TNode {
    debug!("[{}] Inserting ancestors.", tid());
    let mut ancestors = 0;
    loop {
        ancestors += 1;

        n.insert_into_bloom_filter(&mut **bf);
        n = match n.layout_parent_node(root) {
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
            node.remove_from_bloom_filter(&mut *bf);
            let unsafe_parent = parent.to_unsafe();
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
    fn process_postorder(&self, node: N);

    /// Returns if the node should be processed by the preorder traversal (and
    /// then by the post-order one).
    ///
    /// Note that this is true unconditionally for servo, since it requires to
    /// bubble the widths bottom-up for all the DOM.
    fn should_process(&self, node: N) -> bool {
        node.is_dirty() || node.has_dirty_descendants()
    }

    /// Do an action over the child before pushing him to the work queue.
    ///
    /// By default, propagate the IS_DIRTY flag down the tree.
    #[allow(unsafe_code)]
    fn pre_process_child_hook(&self, parent: N, kid: N) {
        // NOTE: At this point is completely safe to modify either the parent or
        // the child, since we have exclusive access to both of them.
        if parent.is_dirty() {
            unsafe {
                kid.set_dirty(true);
                parent.set_dirty_descendants(true);
            }
        }
    }

    fn local_context(&self) -> &LocalStyleContext;
}

/// Determines the amount of relations where we're going to share style.
#[inline]
pub fn relations_are_shareable(relations: &StyleRelations) -> bool {
    use selectors::matching::*;
    !relations.intersects(AFFECTED_BY_UNIQUE_SELECTOR |
                          AFFECTED_BY_PSEUDO_ELEMENTS | AFFECTED_BY_STATE |
                          AFFECTED_BY_NON_COMMON_STYLE_AFFECTING_ATTRIBUTE_SELECTOR |
                          AFFECTED_BY_STYLE_ATTRIBUTE |
                          AFFECTED_BY_PRESENTATIONAL_HINTS)
}

/// Calculates the style for a single node.
#[inline]
#[allow(unsafe_code)]
pub fn recalc_style_at<'a, N, C>(context: &'a C,
                                 root: OpaqueNode,
                                 node: N)
    where N: TNode,
          C: StyleContext<'a>,
          <N::ConcreteElement as Element>::Impl: SelectorImplExt + 'a {
    // Get the parent node.
    let parent_opt = match node.parent_node() {
        Some(parent) if parent.is_element() => Some(parent),
        _ => None,
    };

    // Get the style bloom filter.
    let mut bf = take_thread_local_bloom_filter(parent_opt, root, context.shared_context());

    let nonincremental_layout = opts::get().nonincremental_layout;
    if nonincremental_layout || node.is_dirty() {
        // Remove existing CSS styles from nodes whose content has changed (e.g. text changed),
        // to force non-incremental reflow.
        if node.has_changed() {
            node.unstyle();
        }

        // Check to see whether we can share a style with someone.
        let style_sharing_candidate_cache =
            &mut context.local_context().style_sharing_candidate_cache.borrow_mut();

        let sharing_result = match node.as_element() {
            Some(element) => {
                unsafe {
                    element.share_style_if_possible(style_sharing_candidate_cache,
                                                    context.shared_context(),
                                                    parent_opt.clone())
                }
            },
            None => StyleSharingResult::CannotShare,
        };

        // Otherwise, match and cascade selectors.
        match sharing_result {
            StyleSharingResult::CannotShare => {
                let mut applicable_declarations = ApplicableDeclarations::new();

                let relations;
                let shareable_element = match node.as_element() {
                    Some(element) => {
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
                    },
                    None => {
                        relations = StyleRelations::empty();
                        if node.has_changed() {
                            node.set_restyle_damage(N::ConcreteRestyleDamage::rebuild_and_reflow())
                        }
                        None
                    },
                };

                // Perform the CSS cascade.
                unsafe {
                    node.cascade_node(context,
                                      parent_opt,
                                      &applicable_declarations);
                }

                // Add ourselves to the LRU cache.
                if let Some(element) = shareable_element {
                    style_sharing_candidate_cache.insert_if_possible(&element, relations);
                }
            }
            StyleSharingResult::StyleWasShared(index, damage) => {
                if opts::get().style_sharing_stats {
                    STYLE_SHARING_CACHE_HITS.fetch_add(1, Ordering::Relaxed);
                }
                style_sharing_candidate_cache.touch(index);
                node.set_restyle_damage(damage);
            }
        }
    } else {
        // Finish any expired transitions.
        animation::complete_expired_transitions(
            node.opaque(),
            node.mutate_data().unwrap().style.as_mut().unwrap(),
            context.shared_context()
        );
    }

    let unsafe_layout_node = node.to_unsafe();

    // Before running the children, we need to insert our nodes into the bloom
    // filter.
    debug!("[{}] + {:X}", tid(), unsafe_layout_node.0);
    node.insert_into_bloom_filter(&mut *bf);

    // NB: flow construction updates the bloom filter on the way up.
    put_thread_local_bloom_filter(bf, &unsafe_layout_node, context.shared_context());

    // Mark the node as DIRTY_ON_VIEWPORT_SIZE_CHANGE is it uses viewport
    // percentage units.
    if !node.needs_dirty_on_viewport_size_changed() {
        if let Some(element) = node.as_element() {
            if let Some(ref property_declaration_block) = *element.style_attribute() {
                if property_declaration_block.declarations().any(|d| d.0.has_viewport_percentage()) {
                    unsafe {
                        node.set_dirty_on_viewport_size_changed();
                    }
                }
            }
        }
    }
}
