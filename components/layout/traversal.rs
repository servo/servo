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
use script_layout_interface::restyle_damage::{BUBBLE_ISIZES, REFLOW, REFLOW_OUT_OF_FLOW, REPAINT, RestyleDamage};
use script_layout_interface::wrapper_traits::{LayoutNode, ThreadSafeLayoutNode};
use std::mem;
use style::context::{LocalStyleContext, SharedStyleContext, StyleContext};
use style::dom::TNode;
use style::selector_impl::ServoSelectorImpl;
use style::traversal::{DomTraversalContext, recalc_style_at, remove_from_bloom_filter};
use style::traversal::RestyleResult;
use util::opts;
use wrapper::{LayoutNodeLayoutData, ThreadSafeLayoutNodeHelpers};

pub struct RecalcStyleAndConstructFlows<'lc> {
    context: LayoutContext<'lc>,
    root: OpaqueNode,
}

impl<'lc, N> DomTraversalContext<N> for RecalcStyleAndConstructFlows<'lc>
    where N: LayoutNode + TNode,
          N::ConcreteElement: ::selectors::Element<Impl=ServoSelectorImpl>

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

    fn process_preorder(&self, node: N) -> RestyleResult {
        // FIXME(pcwalton): Stop allocating here. Ideally this should just be
        // done by the HTML parser.
        node.initialize_data();

        recalc_style_at(&self.context, self.root, node)
    }

    fn process_postorder(&self, node: N) {
        construct_flows_at(&self.context, self.root, node);
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
        if nonincremental_layout || node.is_dirty() || node.has_dirty_descendants() {
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
        if self.should_process() {
            self.state.push_stacking_context_id(flow::base(flow).stacking_context_id);
            flow.build_display_list(&mut self.state);
            flow::mut_base(flow).restyle_damage.remove(REPAINT);
            self.state.pop_stacking_context_id();
        }

        for kid in flow::child_iter_mut(flow) {
            self.traverse(kid);
        }
    }

    #[inline]
    fn should_process(&self) -> bool {
        true
    }
}
