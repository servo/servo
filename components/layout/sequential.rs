/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements sequential traversals over the DOM and flow trees.

use context::{LayoutContext, SharedLayoutContext};
use flow::{self, Flow, ImmutableFlowUtils, MutableFlowUtils, PostorderFlowTraversal};
use flow::{PreorderFlowTraversal};
use flow_ref::FlowRef;
use fragment::FragmentBorderBoxIterator;
use traversal::{BubbleISizes, RecalcStyleForNode, ConstructFlows};
use traversal::{AssignBSizesAndStoreOverflow, AssignISizes};
use traversal::{ComputeAbsolutePositions, BuildDisplayList};
use wrapper::LayoutNode;
use wrapper::{PostorderNodeMutTraversal};
use wrapper::{PreorderDomTraversal, PostorderDomTraversal};

use geom::point::Point2D;
use servo_util::geometry::{Au, ZERO_POINT};
use servo_util::opts;

pub fn traverse_dom_preorder(root: LayoutNode,
                             shared_layout_context: &SharedLayoutContext) {
    fn doit(node: LayoutNode, recalc_style: RecalcStyleForNode, construct_flows: ConstructFlows) {
        recalc_style.process(node);

        for kid in node.children() {
            doit(kid, recalc_style, construct_flows);
        }

        construct_flows.process(node);
    }

    let layout_context  = LayoutContext::new(shared_layout_context);
    let recalc_style    = RecalcStyleForNode { layout_context: &layout_context };
    let construct_flows = ConstructFlows     { layout_context: &layout_context };

    doit(root, recalc_style, construct_flows);
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

    let root = &mut **root;

    if opts::get().bubble_inline_sizes_separately {
        let bubble_inline_sizes = BubbleISizes { layout_context: &layout_context };
        root.traverse_postorder(&bubble_inline_sizes);
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

    doit(&mut **root, compute_absolute_positions, build_display_list);
}

pub fn iterate_through_flow_tree_fragment_border_boxes(root: &mut FlowRef,
                                                       iterator: &mut FragmentBorderBoxIterator) {
    fn doit(flow: &mut Flow,
            iterator: &mut FragmentBorderBoxIterator,
            stacking_context_position: &Point2D<Au>) {
        flow.iterate_through_fragment_border_boxes(iterator, stacking_context_position);

        for kid in flow::mut_base(flow).child_iter() {
            let stacking_context_position =
                if kid.is_block_flow() && kid.as_block().fragment.establishes_stacking_context() {
                    *stacking_context_position + flow::base(kid).stacking_relative_position
                } else {
                    *stacking_context_position
                };

            // FIXME(#2795): Get the real container size.
            doit(kid, iterator, &stacking_context_position);
        }
    }

    doit(&mut **root, iterator, &ZERO_POINT);
}
