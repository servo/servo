/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversals over the DOM and flow trees, running the layout computations.

use construct::FlowConstructor;
use context::LayoutContext;
use css::matching::{ApplicableDeclarations, MatchMethods, StyleSharingResult};
use flow::{MutableFlowUtils, PreorderFlowTraversal, PostorderFlowTraversal};
use flow::{self, Flow};
use incremental::{self, BUBBLE_ISIZES, REFLOW, REFLOW_OUT_OF_FLOW, RestyleDamage};
use script::layout_interface::ReflowGoal;
use wrapper::{ThreadSafeLayoutNode, UnsafeLayoutNode};
use wrapper::{layout_node_to_unsafe_layout_node, LayoutNode};

use selectors::bloom::BloomFilter;
use util::opts;
use util::tid::tid;

use std::cell::RefCell;
use std::mem;

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
    static STYLE_BLOOM: RefCell<Option<(Box<BloomFilter>, UnsafeLayoutNode, Generation)>> = RefCell::new(None));

/// Returns the task local bloom filter.
///
/// If one does not exist, a new one will be made for you. If it is out of date,
/// it will be cleared and reused.
fn take_task_local_bloom_filter(parent_node: Option<LayoutNode>, layout_context: &LayoutContext)
                                -> Box<BloomFilter> {
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
                insert_ancestors_into_bloom_filter(&mut bloom_filter, parent, layout_context);
                bloom_filter
            }
            // Found cached bloom filter.
            (Some(parent), Some((mut bloom_filter, old_node, old_generation))) => {
                if old_node == layout_node_to_unsafe_layout_node(&parent) &&
                    old_generation == layout_context.shared.generation {
                    // Hey, the cached parent is our parent! We can reuse the bloom filter.
                    debug!("[{}] Parent matches (={}). Reusing bloom filter.", tid(), old_node.0);
                } else {
                    // Oh no. the cached parent is stale. I guess we need a new one. Reuse the existing
                    // allocation to avoid malloc churn.
                    bloom_filter.clear();
                    insert_ancestors_into_bloom_filter(&mut bloom_filter, parent, layout_context);
                }
                bloom_filter
            },
        }
    })
}

fn put_task_local_bloom_filter(bf: Box<BloomFilter>,
                               unsafe_node: &UnsafeLayoutNode,
                               layout_context: &LayoutContext) {
    STYLE_BLOOM.with(move |style_bloom| {
        assert!(style_bloom.borrow().is_none(),
                "Putting into a never-taken task-local bloom filter");
        *style_bloom.borrow_mut() = Some((bf, *unsafe_node, layout_context.shared.generation));
    })
}

/// "Ancestors" in this context is inclusive of ourselves.
fn insert_ancestors_into_bloom_filter(bf: &mut Box<BloomFilter>,
                                      mut n: LayoutNode,
                                      layout_context: &LayoutContext) {
    debug!("[{}] Inserting ancestors.", tid());
    let mut ancestors = 0;
    loop {
        ancestors += 1;

        n.insert_into_bloom_filter(&mut **bf);
        n = match n.layout_parent_node(layout_context.shared) {
            None => break,
            Some(p) => p,
        };
    }
    debug!("[{}] Inserted {} ancestors.", tid(), ancestors);
}


/// A top-down traversal.
pub trait PreorderDomTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&self, node: LayoutNode);
}

/// A bottom-up traversal, with a optional in-order pass.
pub trait PostorderDomTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&self, node: LayoutNode);
}

/// A bottom-up, parallelizable traversal.
pub trait PostorderNodeMutTraversal {
    /// The operation to perform. Return true to continue or false to stop.
    fn process<'a>(&'a mut self, node: &ThreadSafeLayoutNode<'a>) -> bool;

    /// Returns true if this node should be pruned. If this returns true, we skip the operation
    /// entirely and do not process any descendant nodes. This is called *before* child nodes are
    /// visited. The default implementation never prunes any nodes.
    fn should_prune<'a>(&'a self, _node: &ThreadSafeLayoutNode<'a>) -> bool {
        false
    }
}

/// The recalc-style-for-node traversal, which styles each node and must run before
/// layout computation. This computes the styles applied to each node.
#[derive(Copy, Clone)]
pub struct RecalcStyleForNode<'a> {
    pub layout_context: &'a LayoutContext<'a>,
}

impl<'a> PreorderDomTraversal for RecalcStyleForNode<'a> {
    #[inline]
    #[allow(unsafe_code)]
    fn process(&self, node: LayoutNode) {
        // Initialize layout data.
        //
        // FIXME(pcwalton): Stop allocating here. Ideally this should just be done by the HTML
        // parser.
        node.initialize_layout_data();

        // Get the parent node.
        let parent_opt = node.layout_parent_node(self.layout_context.shared);

        // Get the style bloom filter.
        let mut bf = take_task_local_bloom_filter(parent_opt, self.layout_context);

        let nonincremental_layout = opts::get().nonincremental_layout;
        if nonincremental_layout || node.is_dirty() {
            // Remove existing CSS styles from nodes whose content has changed (e.g. text changed),
            // to force non-incremental reflow.
            if node.has_changed() {
                let node = ThreadSafeLayoutNode::new(&node);
                node.unstyle();
            }

            // Check to see whether we can share a style with someone.
            let style_sharing_candidate_cache =
                &mut self.layout_context.style_sharing_candidate_cache();
            let sharing_result = unsafe {
                node.share_style_if_possible(style_sharing_candidate_cache,
                                             parent_opt.clone())
            };
            // Otherwise, match and cascade selectors.
            match sharing_result {
                StyleSharingResult::CannotShare(mut shareable) => {
                    let mut applicable_declarations = ApplicableDeclarations::new();

                    if node.as_element().is_some() {
                        // Perform the CSS selector matching.
                        let stylist = unsafe { &*self.layout_context.shared.stylist };
                        node.match_node(stylist,
                                        Some(&*bf),
                                        &mut applicable_declarations,
                                        &mut shareable);
                    } else if node.has_changed() {
                        ThreadSafeLayoutNode::new(&node).set_restyle_damage(
                            incremental::rebuild_and_reflow())
                    }

                    // Perform the CSS cascade.
                    unsafe {
                        node.cascade_node(self.layout_context.shared,
                                          parent_opt,
                                          &applicable_declarations,
                                          &mut self.layout_context.applicable_declarations_cache(),
                                          &self.layout_context.shared.new_animations_sender);
                    }

                    // Add ourselves to the LRU cache.
                    if shareable {
                        if let Some(element) = node.as_element() {
                            style_sharing_candidate_cache.insert_if_possible(&element);
                        }
                    }
                }
                StyleSharingResult::StyleWasShared(index, damage) => {
                    style_sharing_candidate_cache.touch(index);
                    ThreadSafeLayoutNode::new(&node).set_restyle_damage(damage);
                }
            }
        }

        let unsafe_layout_node = layout_node_to_unsafe_layout_node(&node);

        // Before running the children, we need to insert our nodes into the bloom
        // filter.
        debug!("[{}] + {:X}", tid(), unsafe_layout_node.0);
        node.insert_into_bloom_filter(&mut *bf);

        // NB: flow construction updates the bloom filter on the way up.
        put_task_local_bloom_filter(bf, &unsafe_layout_node, self.layout_context);
    }
}

/// The flow construction traversal, which builds flows for styled nodes.
#[derive(Copy, Clone)]
pub struct ConstructFlows<'a> {
    pub layout_context: &'a LayoutContext<'a>,
}

impl<'a> PostorderDomTraversal for ConstructFlows<'a> {
    #[inline]
    #[allow(unsafe_code)]
    fn process(&self, node: LayoutNode) {
        // Construct flows for this node.
        {
            let tnode = ThreadSafeLayoutNode::new(&node);

            // Always reconstruct if incremental layout is turned off.
            let nonincremental_layout = opts::get().nonincremental_layout;
            if nonincremental_layout || node.has_dirty_descendants() {
                let mut flow_constructor = FlowConstructor::new(self.layout_context);
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
            node.set_dirty_siblings(false);
            node.set_dirty_descendants(false);
        }

        let unsafe_layout_node = layout_node_to_unsafe_layout_node(&node);

        let (mut bf, old_node, old_generation) =
            STYLE_BLOOM.with(|style_bloom| {
                mem::replace(&mut *style_bloom.borrow_mut(), None)
                .expect("The bloom filter should have been set by style recalc.")
            });

        assert_eq!(old_node, unsafe_layout_node);
        assert_eq!(old_generation, self.layout_context.shared.generation);

        match node.layout_parent_node(self.layout_context.shared) {
            None => {
                debug!("[{}] - {:X}, and deleting BF.", tid(), unsafe_layout_node.0);
                // If this is the reflow root, eat the task-local bloom filter.
            }
            Some(parent) => {
                // Otherwise, put it back, but remove this node.
                node.remove_from_bloom_filter(&mut *bf);
                let unsafe_parent = layout_node_to_unsafe_layout_node(&parent);
                put_task_local_bloom_filter(bf, &unsafe_parent, self.layout_context);
            },
        };
    }
}

/// The flow tree verification traversal. This is only on in debug builds.
#[cfg(debug)]
struct FlowTreeVerification;

#[cfg(debug)]
impl PreorderFlow for FlowTreeVerification {
    #[inline]
    fn process(&mut self, flow: &mut Flow) {
        let base = flow::base(flow);
        if !base.flags.is_leaf() && !base.flags.is_nonleaf() {
            println!("flow tree verification failed: flow wasn't a leaf or a nonleaf!");
            flow.dump();
            panic!("flow tree verification failed")
        }
    }
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
        flow.early_store_overflow(self.layout_context);
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
    }

    #[inline]
    fn should_process(&self, _: &mut Flow) -> bool {
        self.layout_context.shared.goal == ReflowGoal::ForDisplay
    }
}
