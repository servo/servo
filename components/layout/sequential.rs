/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements sequential traversals over the DOM and flow trees.

use app_units::Au;
use context::{LayoutContext, SharedLayoutContext};
use euclid::point::Point2D;
use flow::{PostorderFlowTraversal, PreorderFlowTraversal};
use flow::{self, Flow, ImmutableFlowUtils, InorderFlowTraversal, MutableFlowUtils};
use flow_ref::{self, FlowRef};
use fragment::FragmentBorderBoxIterator;
use generated_content::ResolveGeneratedContent;
use traversal::PostorderNodeMutTraversal;
use traversal::{AssignBSizesAndStoreOverflow, AssignISizes};
use traversal::{BubbleISizes, BuildDisplayList, ComputeAbsolutePositions};
use traversal::{DomTraversal, DomTraversalContext};
use util::opts;
use wrapper::LayoutNode;

pub fn traverse_dom_preorder<'ln, N, T>(root: N,
                                        shared_layout_context: &SharedLayoutContext)
                                        where N: LayoutNode<'ln>,
                                              T: DomTraversal<'ln, N> {
    fn doit<'a, 'ln, N, T>(context: &'a DomTraversalContext<'a>, node: N)
                           where N: LayoutNode<'ln>, T: DomTraversal<'ln, N> {
        T::process_preorder(context, node);

        for kid in node.children() {
            doit::<N, T>(context, kid);
        }

        T::process_postorder(context, node);
    }

    let layout_context = LayoutContext::new(shared_layout_context);
    let traversal_context = DomTraversalContext {
        layout_context: &layout_context,
        root: root.opaque(),
    };

    doit::<N, T>(&traversal_context, root);
}

pub fn resolve_generated_content(root: &mut FlowRef, shared_layout_context: &SharedLayoutContext) {
    fn doit(flow: &mut Flow, level: u32, traversal: &mut ResolveGeneratedContent) {
        if !traversal.should_process(flow) {
            return
        }

        traversal.process(flow, level);

        for kid in flow::mut_base(flow).children.iter_mut() {
            doit(kid, level + 1, traversal)
        }
    }

    let layout_context = LayoutContext::new(shared_layout_context);
    let mut traversal = ResolveGeneratedContent::new(&layout_context);
    doit(flow_ref::deref_mut(root), 0, &mut traversal)
}

pub fn traverse_flow_tree_preorder(root: &mut FlowRef,
                                   shared_layout_context: &SharedLayoutContext) {
    fn doit(flow: &mut Flow,
            assign_inline_sizes: AssignISizes,
            assign_block_sizes: AssignBSizesAndStoreOverflow) {
        if assign_inline_sizes.should_process(flow) {
            assign_inline_sizes.process(flow);
        }

        for kid in flow::child_iter(flow) {
            doit(kid, assign_inline_sizes, assign_block_sizes);
        }

        if assign_block_sizes.should_process(flow) {
            assign_block_sizes.process(flow);
        }
    }

    let layout_context = LayoutContext::new(shared_layout_context);

    let root = flow_ref::deref_mut(root);

    if opts::get().bubble_inline_sizes_separately {
        let bubble_inline_sizes = BubbleISizes { layout_context: &layout_context };
        {
            let root: &mut Flow = root;
            root.traverse_postorder(&bubble_inline_sizes);
        }
    }

    let assign_inline_sizes = AssignISizes                 { layout_context: &layout_context };
    let assign_block_sizes  = AssignBSizesAndStoreOverflow { layout_context: &layout_context };

    doit(root, assign_inline_sizes, assign_block_sizes);
}

pub fn build_display_list_for_subtree(root: &mut FlowRef,
                                      shared_layout_context: &SharedLayoutContext) {
    fn doit(flow: &mut Flow,
            compute_absolute_positions: ComputeAbsolutePositions,
            build_display_list: BuildDisplayList) {
        if compute_absolute_positions.should_process(flow) {
            compute_absolute_positions.process(flow);
        }

        for kid in flow::mut_base(flow).child_iter() {
            doit(kid, compute_absolute_positions, build_display_list);
        }

        if build_display_list.should_process(flow) {
            build_display_list.process(flow);
        }
    }

    let layout_context = LayoutContext::new(shared_layout_context);
    let compute_absolute_positions = ComputeAbsolutePositions { layout_context: &layout_context };
    let build_display_list         = BuildDisplayList         { layout_context: &layout_context };

    doit(flow_ref::deref_mut(root), compute_absolute_positions, build_display_list);
}

pub fn iterate_through_flow_tree_fragment_border_boxes(root: &mut FlowRef,
                                                       iterator: &mut FragmentBorderBoxIterator) {
    fn doit(flow: &mut Flow,
            level: i32,
            iterator: &mut FragmentBorderBoxIterator,
            stacking_context_position: &Point2D<Au>) {
        flow.iterate_through_fragment_border_boxes(iterator, level, stacking_context_position);

        for kid in flow::mut_base(flow).child_iter() {
            let stacking_context_position =
                if kid.is_block_flow() && kid.as_block().fragment.establishes_stacking_context() {
                    let margin = Point2D::new(kid.as_block().fragment.margin.inline_start, Au(0));
                    *stacking_context_position + flow::base(kid).stacking_relative_position + margin
                } else {
                    *stacking_context_position
                };

            // FIXME(#2795): Get the real container size.
            doit(kid, level + 1, iterator, &stacking_context_position);
        }
    }

    doit(flow_ref::deref_mut(root), 0, iterator, &Point2D::zero());
}
