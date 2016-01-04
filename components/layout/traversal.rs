/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversals over the DOM and flow trees, running the layout computations.

#![allow(unsafe_code)]

use construct::FlowConstructor;
use context::{LayoutContext, SharedLayoutContext};
use flow::{PostorderFlowTraversal, PreorderFlowTraversal};
use flow::{self, Flow};
use gfx::display_list::OpaqueNode;
use incremental::{BUBBLE_ISIZES, REFLOW, REFLOW_OUT_OF_FLOW, REPAINT, RestyleDamage};
use script::layout_interface::ReflowGoal;
use selectors::bloom::BloomFilter;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use style::context::{LocalStyleContext, SharedStyleContext, StyleContext};
use style::dom::{TNode, TRestyleDamage, UnsafeNode};
use style::matching::{ApplicableDeclarations, ElementMatchMethods, MatchMethods, StyleSharingResult};
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
                                        context: &SharedStyleContext)
                                        -> Box<BloomFilter>
                                        where N: TNode<'ln> {
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

fn put_task_local_bloom_filter(bf: Box<BloomFilter>,
                               unsafe_node: &UnsafeNode,
                               context: &SharedStyleContext) {
    STYLE_BLOOM.with(move |style_bloom| {
        assert!(style_bloom.borrow().is_none(),
                "Putting into a never-taken task-local bloom filter");
        *style_bloom.borrow_mut() = Some((bf, *unsafe_node, context.generation));
    })
}

/// "Ancestors" in this context is inclusive of ourselves.
fn insert_ancestors_into_bloom_filter<'ln, N>(bf: &mut Box<BloomFilter>,
                                              mut n: N,
                                              root: OpaqueNode)
                                              where N: TNode<'ln> {
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

pub trait DomTraversalContext<'ln, N: TNode<'ln>>  {
    type SharedContext: Sync + 'static;
    fn new<'a>(&'a Self::SharedContext, OpaqueNode) -> Self;
    fn process_preorder(&self, node: N);
    fn process_postorder(&self, node: N);
}

/// FIXME(bholley): I added this now to demonstrate the usefulness of the new design.
/// This is currently unused, but will be used shortly.

#[allow(dead_code)]
pub struct StandaloneStyleContext<'a> {
    pub shared: &'a SharedStyleContext,
    cached_local_style_context: Rc<LocalStyleContext>,
}

impl<'a> StandaloneStyleContext<'a> {
    pub fn new(_: &'a SharedStyleContext) -> Self { panic!("Not implemented") }
}

impl<'a> StyleContext<'a> for StandaloneStyleContext<'a> {
    fn shared_context(&self) -> &'a SharedStyleContext {
        &self.shared
    }

    fn local_context(&self) -> &LocalStyleContext {
        &self.cached_local_style_context
    }
}

#[allow(dead_code)]
pub struct RecalcStyleOnly<'lc> {
    context: StandaloneStyleContext<'lc>,
    root: OpaqueNode,
}

impl<'lc, 'ln, N: TNode<'ln>> DomTraversalContext<'ln, N> for RecalcStyleOnly<'lc> {
    type SharedContext = SharedStyleContext;
    fn new<'a>(shared: &'a Self::SharedContext, root: OpaqueNode) -> Self {
        // See the comment in RecalcStyleAndConstructFlows::new for an explanation of why this is
        // necessary.
        let shared_lc: &'lc SharedStyleContext = unsafe { mem::transmute(shared) };
        RecalcStyleOnly {
            context: StandaloneStyleContext::new(shared_lc),
            root: root,
        }
    }

    fn process_preorder(&self, node: N) { recalc_style_at(&self.context, self.root, node); }
    fn process_postorder(&self, _: N) {}
}

pub struct RecalcStyleAndConstructFlows<'lc> {
    context: LayoutContext<'lc>,
    root: OpaqueNode,
}

impl<'lc, 'ln, N: LayoutNode<'ln>> DomTraversalContext<'ln, N> for RecalcStyleAndConstructFlows<'lc> {
    type SharedContext = SharedLayoutContext;
    fn new<'a>(shared: &'a Self::SharedContext, root: OpaqueNode) -> Self {
        // FIXME(bholley): This transmutation from &'a to &'lc is very unfortunate, but I haven't
        // found a way to avoid it despite spending several days on it (and consulting Manishearth,
        // brson, and nmatsakis).
        //
        // The crux of the problem is that parameterizing DomTraversalContext on the lifetime of
        // the SharedContext doesn't work for a variety of reasons [1]. However, the code in
        // parallel.rs needs to be able to use the DomTraversalContext trait (or something similar)
        // to stack-allocate a struct (a generalized LayoutContext<'a>) that holds a borrowed
        // SharedContext, which means that the struct needs to be parameterized on a lifetime.
        // Given the aforementioned constraint, the only way to accomplish this is to avoid
        // propagating the borrow lifetime from the struct to the trait, but that means that the
        // new() method on the trait cannot require the lifetime of its argument to match the
        // lifetime of the Self object it creates.
        //
        // This could be solved with an associated type with an unbound lifetime parameter, but
        // that would require higher-kinded types, which don't exist yet and probably aren't coming
        // for a while.
        //
        // So we transmute. :-(
        //
        // [1] For example, the WorkQueue type needs to be parameterized on the concrete type of
        // DomTraversalContext::SharedContext, and the WorkQueue lifetime is similar to that of the
        // LayoutTask, generally much longer than that of a given SharedLayoutContext borrow.
        let shared_lc: &'lc SharedLayoutContext = unsafe { mem::transmute(shared) };
        RecalcStyleAndConstructFlows {
            context: LayoutContext::new(shared_lc),
            root: root,
        }
    }

    fn process_preorder(&self, node: N) { recalc_style_at(&self.context, self.root, node); }
    fn process_postorder(&self, node: N) { construct_flows_at(&self.context, self.root, node); }
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
fn recalc_style_at<'a, 'ln, N: TNode<'ln>, C: StyleContext<'a>> (context: &'a C, root: OpaqueNode, node: N) {
    // Initialize layout data.
    //
    // FIXME(pcwalton): Stop allocating here. Ideally this should just be done by the HTML
    // parser.
    node.initialize_data();

    // Get the parent node.
    let parent_opt = node.layout_parent_node(root);

    // Get the style bloom filter.
    let mut bf = take_task_local_bloom_filter(parent_opt, root, context.shared_context());

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
                        let stylist = unsafe { &*context.shared_context().stylist.0 };
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
                            node.set_restyle_damage(N::ConcreteRestyleDamage::rebuild_and_reflow())
                        }
                        None
                    },
                };

                // Perform the CSS cascade.
                unsafe {
                    node.cascade_node(&context.shared_context(),
                                      parent_opt,
                                      &applicable_declarations,
                                      &mut context.local_context().applicable_declarations_cache.borrow_mut(),
                                      &context.shared_context().new_animations_sender);
                }

                // Add ourselves to the LRU cache.
                if let Some(element) = shareable_element {
                    style_sharing_candidate_cache.insert_if_possible(&element);
                }
            }
            StyleSharingResult::StyleWasShared(index, damage) => {
                style_sharing_candidate_cache.touch(index);
                node.set_restyle_damage(damage);
            }
        }
    }

    let unsafe_layout_node = node.to_unsafe();

    // Before running the children, we need to insert our nodes into the bloom
    // filter.
    debug!("[{}] + {:X}", tid(), unsafe_layout_node.0);
    node.insert_into_bloom_filter(&mut *bf);

    // NB: flow construction updates the bloom filter on the way up.
    put_task_local_bloom_filter(bf, &unsafe_layout_node, context.shared_context());
}

/// The flow construction traversal, which builds flows for styled nodes.
#[inline]
#[allow(unsafe_code)]
fn construct_flows_at<'a, 'ln, N: LayoutNode<'ln>>(context: &'a LayoutContext<'a>, root: OpaqueNode, node: N) {
    // Construct flows for this node.
    {
        let tnode = node.to_threadsafe();

        // Always reconstruct if incremental layout is turned off.
        let nonincremental_layout = opts::get().nonincremental_layout;
        if nonincremental_layout || node.has_dirty_descendants() {
            let mut flow_constructor = FlowConstructor::new(context);
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
    assert_eq!(old_generation, context.shared_context().generation);

    match node.layout_parent_node(root) {
        None => {
            debug!("[{}] - {:X}, and deleting BF.", tid(), unsafe_layout_node.0);
            // If this is the reflow root, eat the task-local bloom filter.
        }
        Some(parent) => {
            // Otherwise, put it back, but remove this node.
            node.remove_from_bloom_filter(&mut *bf);
            let unsafe_parent = parent.to_unsafe();
            put_task_local_bloom_filter(bf, &unsafe_parent, &context.shared_context());
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
