/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Traversals over the DOM and flow trees, running the layout computations.

use construct::FlowConstructor;
use context::{LayoutContext, ScopedThreadLocalLayoutContext, SharedLayoutContext};
use display_list_builder::DisplayListBuildState;
use flow::{self, PreorderFlowTraversal};
use flow::{CAN_BE_FRAGMENTED, Flow, ImmutableFlowUtils, PostorderFlowTraversal};
use script_layout_interface::wrapper_traits::{LayoutNode, ThreadSafeLayoutNode};
use servo_config::opts;
use style::atomic_refcell::AtomicRefCell;
use style::context::{SharedStyleContext, StyleContext};
use style::data::ElementData;
use style::dom::{TElement, TNode};
use style::selector_parser::RestyleDamage;
use style::servo::restyle_damage::{BUBBLE_ISIZES, REFLOW, REFLOW_OUT_OF_FLOW, REPAINT};
use style::traversal::{DomTraversal, recalc_style_at};
use style::traversal::PerLevelTraversalData;
use wrapper::{GetRawData, LayoutNodeHelpers, LayoutNodeLayoutData};

pub struct RecalcStyleAndConstructFlows {
    shared: SharedLayoutContext,
}

impl RecalcStyleAndConstructFlows {
    pub fn shared_layout_context(&self) -> &SharedLayoutContext {
        &self.shared
    }
}

impl RecalcStyleAndConstructFlows {
    /// Creates a traversal context, taking ownership of the shared layout context.
    pub fn new(shared: SharedLayoutContext) -> Self {
        RecalcStyleAndConstructFlows {
            shared: shared,
        }
    }

    /// Consumes this traversal context, returning ownership of the shared layout
    /// context to the caller.
    pub fn destroy(self) -> SharedLayoutContext {
        self.shared
    }
}

#[allow(unsafe_code)]
impl<N> DomTraversal<N> for RecalcStyleAndConstructFlows
    where N: LayoutNode + TNode,
          N::ConcreteElement: TElement
{
    type ThreadLocalContext = ScopedThreadLocalLayoutContext<N::ConcreteElement>;

    fn process_preorder(&self, traversal_data: &mut PerLevelTraversalData,
                        thread_local: &mut Self::ThreadLocalContext, node: N) {
        // FIXME(pcwalton): Stop allocating here. Ideally this should just be
        // done by the HTML parser.
        node.initialize_data();

        if !node.is_text_node() {
            let el = node.as_element().unwrap();
            let mut data = el.mutate_data().unwrap();
            let mut context = StyleContext {
                shared: &self.shared.style_context,
                thread_local: &mut thread_local.style_context,
            };
            recalc_style_at(self, traversal_data, &mut context, el, &mut data);
        }
    }

    fn process_postorder(&self, thread_local: &mut Self::ThreadLocalContext, node: N) {
        let context = LayoutContext::new(&self.shared);
        construct_flows_at(&context, thread_local, node);
    }

    fn text_node_needs_traversal(node: N) -> bool {
        // Text nodes never need styling. However, there are two cases they may need
        // flow construction:
        // (1) They child doesn't yet have layout data (preorder traversal initializes it).
        // (2) The parent element has restyle damage (so the text flow also needs fixup).
        node.get_raw_data().is_none() ||
        node.parent_node().unwrap().to_threadsafe().restyle_damage() != RestyleDamage::empty()
    }

    unsafe fn ensure_element_data(element: &N::ConcreteElement) -> &AtomicRefCell<ElementData> {
        element.as_node().initialize_data();
        element.get_data().unwrap()
    }

    unsafe fn clear_element_data(element: &N::ConcreteElement) {
        element.as_node().clear_data();
    }

    fn shared_context(&self) -> &SharedStyleContext {
        &self.shared.style_context
    }

    fn create_thread_local_context(&self) -> Self::ThreadLocalContext {
        ScopedThreadLocalLayoutContext::new(&self.shared)
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
fn construct_flows_at<'a, N>(context: &LayoutContext<'a>,
                             _thread_local: &mut ScopedThreadLocalLayoutContext<N::ConcreteElement>,
                             node: N)
    where N: LayoutNode,
{
    debug!("construct_flows_at: {:?}", node);

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
                debug!("Constructed flow for {:?}: {:x}",
                       tnode,
                       tnode.flow_debug_id());
            }
        }
    }

    if let Some(el) = node.as_element() {
        el.mutate_data().unwrap().persist();
        unsafe { el.unset_dirty_descendants(); }
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
