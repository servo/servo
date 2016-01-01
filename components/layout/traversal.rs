/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversals over the DOM and flow trees, running the layout computations.

use construct::FlowConstructor;
use context::LayoutContext;
use css::matching::{ElementMatchMethods, MatchMethods, StyleSharingResult};
use flow::{PostorderFlowTraversal, PreorderFlowTraversal};
use flow::{self, Flow};
use gfx::display_list::OpaqueNode;
use incremental::{self, BUBBLE_ISIZES, REFLOW, REFLOW_OUT_OF_FLOW, REPAINT, RestyleDamage};
use script::layout_interface::ReflowGoal;
use selectors::bloom::BloomFilter;
use std::cell::RefCell;
use std::mem;
use style::context::StyleContext;
use style::dom::UnsafeNode;
use style::matching::ApplicableDeclarations;
use util::opts;
use util::tid::tid;
use wrapper::{LayoutNode, ThreadSafeLayoutNode};

/// Every time we do another layout, the old bloom filters are invalid. This is
/// detected by ticking a generation number every layout.
type Generation = u32;

/// A pair of the bloom filter used for css selector matching, and the node to
/// which it applies. This is used to efficiently do `Descendant` selector
/// matches. Thanks to the bloom filter, we can avoid walking up the tree
/// looking for ancestors that aren't there in the majority of cases.
///
/// As we walk down the DOM tree a task-local bloom filter is built of all the
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
/// this happens, the task local bloom filter will be thrown away and rebuilt.
thread_local!(
    static STYLE_BLOOM: RefCell<Option<(Box<BloomFilter>, UnsafeNode, Generation)>> = RefCell::new(None));

/// Returns the task local bloom filter.
///
/// If one does not exist, a new one will be made for you. If it is out of date,
/// it will be cleared and reused.
fn take_task_local_bloom_filter<'ln, N>(parent_node: Option<N>,
                                        root: OpaqueNode,
                                        layout_context: &LayoutContext)
                                        -> Box<BloomFilter>
                                        where N: LayoutNode<'ln> {
    STYLE_BLOOM.with(|style_bloom| {
        match (parent_node, style_bloom.borrow_mut().take()) {
            // Root node. Needs new bloom filter.
            (None,     _  ) => {
                debug!("[{}] No parent, but new bloom filter!", tid());
                box BloomFilter::new()
            }
            // No bloom filter for this thread yet.
            (Some(parent), None) => {
                let mut bloom_filter = box BloomFilter::new();
                insert_ancestors_into_bloom_filter(&mut bloom_filter, parent, root);
                bloom_filter
            }
            // Found cached bloom filter.
            (Some(parent), Some((mut bloom_filter, old_node, old_generation))) => {
                if old_node == parent.to_unsafe() &&
                    old_generation == layout_context.shared_context().generation {
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

fn put_task_local_bloom_filter(bf: Box<BloomFilter>,
                               unsafe_node: &UnsafeNode,
                               layout_context: &LayoutContext) {
    STYLE_BLOOM.with(move |style_bloom| {
        assert!(style_bloom.borrow().is_none(),
                "Putting into a never-taken task-local bloom filter");
        *style_bloom.borrow_mut() = Some((bf, *unsafe_node, layout_context.shared_context().generation));
    })
}

/// "Ancestors" in this context is inclusive of ourselves.
fn insert_ancestors_into_bloom_filter<'ln, N>(bf: &mut Box<BloomFilter>,
                                              mut n: N,
                                              root: OpaqueNode)
                                              where N: LayoutNode<'ln> {
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

#[derive(Copy, Clone)]
pub struct DomTraversalContext<'a> {
    pub layout_context: &'a LayoutContext<'a>,
    pub root: OpaqueNode,
}

pub trait DomTraversal<'ln, N: LayoutNode<'ln>>  {
    fn process_preorder<'a>(context: &'a DomTraversalContext<'a>, node: N);
    fn process_postorder<'a>(context: &'a DomTraversalContext<'a>, node: N);
}

/// FIXME(bholley): I added this now to demonstrate the usefulness of the new design.
/// This is currently unused, but will be used shortly.
#[allow(dead_code)]
pub struct RecalcStyleOnly;
impl<'ln, N: LayoutNode<'ln>> DomTraversal<'ln, N> for RecalcStyleOnly {
    fn process_preorder<'a>(context: &'a DomTraversalContext<'a>, node: N) { recalc_style_at(context, node); }
    fn process_postorder<'a>(_: &'a DomTraversalContext<'a>, _: N) {}
}

pub struct RecalcStyleAndConstructFlows;
impl<'ln, N: LayoutNode<'ln>> DomTraversal<'ln, N> for RecalcStyleAndConstructFlows {
    fn process_preorder<'a>(context: &'a DomTraversalContext<'a>, node: N) { recalc_style_at(context, node); }
    fn process_postorder<'a>(context: &'a DomTraversalContext<'a>, node: N) { construct_flows_at(context, node); }
}

/// A bottom-up, parallelizable traversal.
pub trait PostorderNodeMutTraversal<'ln, ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode<'ln>> {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&mut self, node: &ConcreteThreadSafeLayoutNode) -> bool;
}

/// The recalc-style-for-node traversal, which styles each node and must run before
/// layout computation. This computes the styles applied to each node.
#[inline]
#[allow(unsafe_code)]
fn recalc_style_at<'a, 'ln, N: LayoutNode<'ln>> (context: &'a DomTraversalContext<'a>, node: N) {
    // Initialize layout data.
    //
    // FIXME(pcwalton): Stop allocating here. Ideally this should just be done by the HTML
    // parser.
    node.initialize_data();

    // Get the parent node.
    let parent_opt = node.layout_parent_node(context.root);

    // Get the style bloom filter.
    let mut bf = take_task_local_bloom_filter(parent_opt, context.root, context.layout_context);

    let nonincremental_layout = opts::get().nonincremental_layout;
    if nonincremental_layout || node.is_dirty() {
        // Remove existing CSS styles from nodes whose content has changed (e.g. text changed),
        // to force non-incremental reflow.
        if node.has_changed() {
            let node = node.to_threadsafe();
            node.unstyle();
        }

        // Check to see whether we can share a style with someone.
        let style_sharing_candidate_cache =
            &mut context.layout_context.style_sharing_candidate_cache();

        let sharing_result = match node.as_element() {
            Some(element) => {
                unsafe {
                    element.share_style_if_possible(style_sharing_candidate_cache,
                                                    parent_opt.clone())
                }
            },
            None => StyleSharingResult::CannotShare,
        };

        // Otherwise, match and cascade selectors.
        match sharing_result {
            StyleSharingResult::CannotShare => {
                let mut applicable_declarations = ApplicableDeclarations::new();

                let shareable_element = match node.as_element() {
                    Some(element) => {
                        // Perform the CSS selector matching.
                        let stylist = unsafe { &*context.layout_context.shared_context().stylist.0 };
                        if element.match_element(stylist,
                                                 Some(&*bf),
                                                 &mut applicable_declarations) {
                            Some(element)
                        } else {
                            None
                        }
                    },
                    None => {
                        if node.has_changed() {
                            node.to_threadsafe().set_restyle_damage(
                                incremental::rebuild_and_reflow())
                        }
                        None
                    },
                };

                // Perform the CSS cascade.
                unsafe {
                    node.cascade_node(context.layout_context.shared,
                                      parent_opt,
                                      &applicable_declarations,
                                      &mut context.layout_context.applicable_declarations_cache(),
                                      &context.layout_context.shared_context().new_animations_sender);
                }

                // Add ourselves to the LRU cache.
                if let Some(element) = shareable_element {
                    style_sharing_candidate_cache.insert_if_possible(&element);
                }
            }
            StyleSharingResult::StyleWasShared(index, damage) => {
                style_sharing_candidate_cache.touch(index);
                node.to_threadsafe().set_restyle_damage(damage);
            }
        }
    }

    let unsafe_layout_node = node.to_unsafe();

    // Before running the children, we need to insert our nodes into the bloom
    // filter.
    debug!("[{}] + {:X}", tid(), unsafe_layout_node.0);
    node.insert_into_bloom_filter(&mut *bf);

    // NB: flow construction updates the bloom filter on the way up.
    put_task_local_bloom_filter(bf, &unsafe_layout_node, context.layout_context);
}

/// The flow construction traversal, which builds flows for styled nodes.
#[inline]
#[allow(unsafe_code)]
fn construct_flows_at<'a, 'ln, N: LayoutNode<'ln>>(context: &'a DomTraversalContext<'a>, node: N) {
    // Construct flows for this node.
    {
        let tnode = node.to_threadsafe();

        // Always reconstruct if incremental layout is turned off.
        let nonincremental_layout = opts::get().nonincremental_layout;
        if nonincremental_layout || node.has_dirty_descendants() {
            let mut flow_constructor = FlowConstructor::new(context.layout_context);
            if nonincremental_layout || !flow_constructor.repair_if_possible(&tnode) {
                flow_constructor.process(&tnode);
                debug!("Constructed flow for {:x}: {:x}",
                       tnode.debug_id(),
                       tnode.flow_debug_id());
            }
        }

        // Reset the layout damage in this node. It's been propagated to the
        // flow by the flow constructor.
        tnode.set_restyle_damage(RestyleDamage::empty());
    }

    unsafe {
        node.set_changed(false);
        node.set_dirty(false);
        node.set_dirty_descendants(false);
    }

    let unsafe_layout_node = node.to_unsafe();

    let (mut bf, old_node, old_generation) =
        STYLE_BLOOM.with(|style_bloom| {
            mem::replace(&mut *style_bloom.borrow_mut(), None)
            .expect("The bloom filter should have been set by style recalc.")
        });

    assert_eq!(old_node, unsafe_layout_node);
    assert_eq!(old_generation, context.layout_context.shared_context().generation);

    match node.layout_parent_node(context.root) {
        None => {
            debug!("[{}] - {:X}, and deleting BF.", tid(), unsafe_layout_node.0);
            // If this is the reflow root, eat the task-local bloom filter.
        }
        Some(parent) => {
            // Otherwise, put it back, but remove this node.
            node.remove_from_bloom_filter(&mut *bf);
            let unsafe_parent = parent.to_unsafe();
            put_task_local_bloom_filter(bf, &unsafe_parent, context.layout_context);
        },
    };
}

/// The bubble-inline-sizes traversal, the first part of layout computation. This computes
/// preferred and intrinsic inline-sizes and bubbles them up the tree.
pub struct BubbleISizes<'a> {
    pub layout_context: &'a LayoutContext<'a>,
}

impl<'a> PostorderFlowTraversal for BubbleISizes<'a> {
    #[inline]
    fn process(&self, flow: &mut Flow) {
        flow.bubble_inline_sizes();
        flow::mut_base(flow).restyle_damage.remove(BUBBLE_ISIZES);
    }

    #[inline]
    fn should_process(&self, flow: &mut Flow) -> bool {
        flow::base(flow).restyle_damage.contains(BUBBLE_ISIZES)
    }
}

/// The assign-inline-sizes traversal. In Gecko this corresponds to `Reflow`.
#[derive(Copy, Clone)]
pub struct AssignISizes<'a> {
    pub layout_context: &'a LayoutContext<'a>,
}

impl<'a> PreorderFlowTraversal for AssignISizes<'a> {
    #[inline]
    fn process(&self, flow: &mut Flow) {
        flow.assign_inline_sizes(self.layout_context);
    }

    #[inline]
    fn should_process(&self, flow: &mut Flow) -> bool {
        flow::base(flow).restyle_damage.intersects(REFLOW_OUT_OF_FLOW | REFLOW)
    }
}

/// The assign-block-sizes-and-store-overflow traversal, the last (and most expensive) part of
/// layout computation. Determines the final block-sizes for all layout objects, computes
/// positions, and computes overflow regions. In Gecko this corresponds to `Reflow` and
/// `FinishAndStoreOverflow`.
#[derive(Copy, Clone)]
pub struct AssignBSizesAndStoreOverflow<'a> {
    pub layout_context: &'a LayoutContext<'a>,
}

impl<'a> PostorderFlowTraversal for AssignBSizesAndStoreOverflow<'a> {
    #[inline]
    fn process(&self, flow: &mut Flow) {
        // Can't do anything with flows impacted by floats until we reach their inorder parent.
        // NB: We must return without resetting the restyle bits for these, as we haven't actually
        // reflowed anything!
        if flow::base(flow).flags.impacted_by_floats() {
            return
        }

        flow.assign_block_size(self.layout_context);
    }

    #[inline]
    fn should_process(&self, flow: &mut Flow) -> bool {
        flow::base(flow).restyle_damage.intersects(REFLOW_OUT_OF_FLOW | REFLOW)
    }
}

#[derive(Copy, Clone)]
pub struct ComputeAbsolutePositions<'a> {
    pub layout_context: &'a LayoutContext<'a>,
}

impl<'a> PreorderFlowTraversal for ComputeAbsolutePositions<'a> {
    #[inline]
    fn process(&self, flow: &mut Flow) {
        flow.compute_absolute_position(self.layout_context);
        flow.store_overflow(self.layout_context);
    }
}

#[derive(Copy, Clone)]
pub struct BuildDisplayList<'a> {
    pub layout_context: &'a LayoutContext<'a>,
}

impl<'a> PostorderFlowTraversal for BuildDisplayList<'a> {
    #[inline]
    fn process(&self, flow: &mut Flow) {
        flow.build_display_list(self.layout_context);
        flow::mut_base(flow).restyle_damage.remove(REPAINT);
    }

    #[inline]
    fn should_process(&self, _: &mut Flow) -> bool {
        self.layout_context.shared_context().goal == ReflowGoal::ForDisplay
    }
}
