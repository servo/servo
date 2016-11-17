/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversals over the DOM and flow trees, running the layout computations.

use construct::FlowConstructor;
use context::{LayoutContext, SharedLayoutContext};
use display_list_builder::DisplayListBuildState;
use flow::{self, PreorderFlowTraversal};
use flow::{CAN_BE_FRAGMENTED, Flow, ImmutableFlowUtils, PostorderFlowTraversal};
use gfx::display_list::OpaqueNode;
use script_layout_interface::wrapper_traits::{LayoutNode, ThreadSafeLayoutNode};
use std::mem;
use style::atomic_refcell::AtomicRefCell;
use style::context::{LocalStyleContext, SharedStyleContext, StyleContext};
use style::data::ElementData;
use style::dom::{StylingMode, TElement, TNode};
use style::selector_impl::RestyleDamage;
use style::servo::restyle_damage::{BUBBLE_ISIZES, REFLOW, REFLOW_OUT_OF_FLOW, REPAINT};
use style::traversal::{DomTraversalContext, put_thread_local_bloom_filter};
use style::traversal::{recalc_style_at, remove_from_bloom_filter};
use style::traversal::take_thread_local_bloom_filter;
use util::opts;
use wrapper::{GetRawData, LayoutNodeHelpers, LayoutNodeLayoutData};

pub struct RecalcStyleAndConstructFlows<'lc> {
    context: LayoutContext<'lc>,
    root: OpaqueNode,
}

#[allow(unsafe_code)]
impl<'lc, N> DomTraversalContext<N> for RecalcStyleAndConstructFlows<'lc>
    where N: LayoutNode + TNode,
          N::ConcreteElement: TElement

{
    type SharedContext = SharedLayoutContext;
    #[allow(unsafe_code)]
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
        // So we transmute. :-( This is safe because the DomTravesalContext is stack-allocated on
        // the worker thread while processing a WorkUnit, whereas the borrowed SharedContext is
        // live for the entire duration of the restyle. This really could _almost_ compile: all
        // we'd need to do is change the signature to to |new<'a: 'lc>|, and everything would
        // work great. But we can't do that, because that would cause a mismatch with the signature
        // in the trait we're implementing, and we can't mention 'lc in that trait at all for the
        // reasons described above.
        //
        // [1] For example, the WorkQueue type needs to be parameterized on the concrete type of
        // DomTraversalContext::SharedContext, and the WorkQueue lifetime is similar to that of the
        // LayoutThread, generally much longer than that of a given SharedLayoutContext borrow.
        let shared_lc: &'lc SharedLayoutContext = unsafe { mem::transmute(shared) };
        RecalcStyleAndConstructFlows {
            context: LayoutContext::new(shared_lc),
            root: root,
        }
    }

    fn process_preorder(&self, node: N) {
        // FIXME(pcwalton): Stop allocating here. Ideally this should just be
        // done by the HTML parser.
        node.initialize_data();

        if node.is_text_node() {
            // FIXME(bholley): Stop doing this silly work to maintain broken bloom filter
            // invariants.
            //
            // Longer version: The bloom filter is entirely busted for parallel traversal. Because
            // parallel traversal is breadth-first, each sibling rejects the bloom filter set up
            // by the previous sibling (which is valid for children, not siblings) and recreates
            // it. Similarly, the fixup performed in the bottom-up traversal is useless, because
            // threads perform flow construction up the parent chain until they find a parent with
            // other unprocessed children, at which point they bail to the work queue and find a
            // different node.
            //
            // Nevertheless, the remove_from_bloom_filter call at the end of flow construction
            // asserts that the bloom filter is valid for the current node. This breaks when we
            // stop calling recalc_style_at for text nodes, because the recursive chain of
            // construct_flows_at calls is no longer necessarily rooted in a call that sets up the
            // thread-local bloom filter for the leaf node.
            //
            // The bloom filter stuff is all going to be rewritten, so we just hackily duplicate
            // the bloom filter manipulation from recalc_style_at to maintain invariants.
            let parent = node.parent_node().unwrap().as_element();
            let bf = take_thread_local_bloom_filter(parent, self.root, self.context.shared_context());
            put_thread_local_bloom_filter(bf, &node.to_unsafe(), self.context.shared_context());
        } else {
            let el = node.as_element().unwrap();
            recalc_style_at::<_, _, Self>(&self.context, self.root, el);
        }
    }

    fn process_postorder(&self, node: N) {
        construct_flows_at(&self.context, self.root, node);
    }

    fn should_traverse_child(parent: N::ConcreteElement, child: N) -> bool {
        // If the parent is display:none, we don't need to do anything.
        if parent.is_display_none() {
            return false;
        }

        match child.as_element() {
            // Elements should be traversed if they need styling or flow construction.
            Some(el) => el.styling_mode() != StylingMode::Stop ||
                        el.as_node().to_threadsafe().restyle_damage() != RestyleDamage::empty(),

            // Text nodes never need styling. However, there are two cases they may need
            // flow construction:
            // (1) They child doesn't yet have layout data (preorder traversal initializes it).
            // (2) The parent element has restyle damage (so the text flow also needs fixup).
            None => child.get_raw_data().is_none() ||
                    parent.as_node().to_threadsafe().restyle_damage() != RestyleDamage::empty(),
        }
    }

    unsafe fn ensure_element_data(element: &N::ConcreteElement) -> &AtomicRefCell<ElementData> {
        element.as_node().initialize_data();
        element.get_data().unwrap()
    }

    unsafe fn clear_element_data(element: &N::ConcreteElement) {
        element.as_node().clear_data();
    }

    fn local_context(&self) -> &LocalStyleContext {
        self.context.local_context()
    }
}

/// A bottom-up, parallelizable traversal.
pub trait PostorderNodeMutTraversal<ConcreteThreadSafeLayoutNode: ThreadSafeLayoutNode> {
    /// The operation to perform. Return true to continue or false to stop.
    fn process(&mut self, node: &ConcreteThreadSafeLayoutNode);
}

/// The flow construction traversal, which builds flows for styled nodes.
#[inline]
#[allow(unsafe_code)]
fn construct_flows_at<'a, N: LayoutNode>(context: &'a LayoutContext<'a>, root: OpaqueNode, node: N) {
    // Construct flows for this node.
    {
        let tnode = node.to_threadsafe();

        // Always reconstruct if incremental layout is turned off.
        let nonincremental_layout = opts::get().nonincremental_layout;
        if nonincremental_layout || tnode.restyle_damage() != RestyleDamage::empty() ||
           node.as_element().map_or(false, |el| el.has_dirty_descendants()) {
            let mut flow_constructor = FlowConstructor::new(context);
            if nonincremental_layout || !flow_constructor.repair_if_possible(&tnode) {
                flow_constructor.process(&tnode);
                debug!("Constructed flow for {:x}: {:x}",
                       tnode.debug_id(),
                       tnode.flow_debug_id());
            }
        }

        tnode.clear_restyle_damage();
    }

    unsafe { node.clear_dirty_bits(); }
    remove_from_bloom_filter(context, root, node);
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
    pub shared_context: &'a SharedStyleContext,
}

impl<'a> PreorderFlowTraversal for AssignISizes<'a> {
    #[inline]
    fn process(&self, flow: &mut Flow) {
        flow.assign_inline_sizes(self.shared_context);
    }

    #[inline]
    fn should_process(&self, flow: &mut Flow) -> bool {
        flow::base(flow).restyle_damage.intersects(REFLOW_OUT_OF_FLOW | REFLOW)
    }
}

/// The assign-block-sizes-and-store-overflow traversal, the last (and most expensive) part of
/// layout computation. Determines the final block-sizes for all layout objects and computes
/// positions. In Gecko this corresponds to `Reflow`.
#[derive(Copy, Clone)]
pub struct AssignBSizes<'a> {
    pub layout_context: &'a LayoutContext<'a>,
}

impl<'a> PostorderFlowTraversal for AssignBSizes<'a> {
    #[inline]
    fn process(&self, flow: &mut Flow) {
        // Can't do anything with anything that floats might flow through until we reach their
        // inorder parent.
        //
        // NB: We must return without resetting the restyle bits for these, as we haven't actually
        // reflowed anything!
        if flow.floats_might_flow_through() {
            return
        }

        flow.assign_block_size(self.layout_context);
    }

    #[inline]
    fn should_process(&self, flow: &mut Flow) -> bool {
        let base = flow::base(flow);
        base.restyle_damage.intersects(REFLOW_OUT_OF_FLOW | REFLOW) &&
        // The fragmentation countainer is responsible for calling Flow::fragment recursively
        !base.flags.contains(CAN_BE_FRAGMENTED)
    }
}

#[derive(Copy, Clone)]
pub struct ComputeAbsolutePositions<'a> {
    pub layout_context: &'a SharedLayoutContext,
}

impl<'a> PreorderFlowTraversal for ComputeAbsolutePositions<'a> {
    #[inline]
    fn process(&self, flow: &mut Flow) {
        flow.compute_absolute_position(self.layout_context);
    }
}

pub struct BuildDisplayList<'a> {
    pub state: DisplayListBuildState<'a>,
}

impl<'a> BuildDisplayList<'a> {
    #[inline]
    pub fn traverse(&mut self, flow: &mut Flow) {
        let new_stacking_context =
            flow::base(flow).stacking_context_id != self.state.stacking_context_id();
        if new_stacking_context {
            self.state.push_stacking_context_id(flow::base(flow).stacking_context_id);
        }

        let new_scroll_root =
            flow::base(flow).scroll_root_id != self.state.scroll_root_id();
        if new_scroll_root {
            self.state.push_scroll_root_id(flow::base(flow).scroll_root_id);
        }

        if self.should_process() {
            flow.build_display_list(&mut self.state);
            flow::mut_base(flow).restyle_damage.remove(REPAINT);
        }

        for kid in flow::child_iter_mut(flow) {
            self.traverse(kid);
        }

        if new_stacking_context {
            self.state.pop_stacking_context_id();
        }

        if new_scroll_root {
            self.state.pop_scroll_root_id();
        }
    }

    #[inline]
    fn should_process(&self) -> bool {
        true
    }
}
