/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements sequential traversals over the DOM and flow trees.

use context::{LayoutContext, SharedLayoutContext};
use flow::{Flow, MutableFlowUtils, PreorderFlowTraversal, PostorderFlowTraversal};
use flow;
use flow_ref::FlowRef;
use traversal::{BubbleISizes, RecalcStyleForNode, ConstructFlows};
use traversal::{AssignBSizesAndStoreOverflow, AssignISizes};
use traversal::{ComputeAbsolutePositions, BuildDisplayList};
use wrapper::LayoutNode;
use wrapper::{PostorderNodeMutTraversal};
use wrapper::{PreorderDomTraversal, PostorderDomTraversal};

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
    fn doit(flow: &mut Flow, assign_inline_sizes: AssignISizes, assign_block_sizes: AssignBSizesAndStoreOverflow) {
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

    let root = root.get_mut();

    if layout_context.shared.opts.bubble_inline_sizes_separately {
        let bubble_inline_sizes = BubbleISizes { layout_context: &layout_context };
        root.traverse_postorder(&bubble_inline_sizes);
    }

    let assign_inline_sizes = AssignISizes                 { layout_context: &layout_context };
    let assign_block_sizes  = AssignBSizesAndStoreOverflow { layout_context: &layout_context };

    doit(root, assign_inline_sizes, assign_block_sizes);
}

pub fn build_display_list_for_subtree(root: &mut FlowRef,
                                      shared_layout_context: &SharedLayoutContext) {
    fn doit(flow: &mut Flow, compute_absolute_positions: ComputeAbsolutePositions, build_display_list: BuildDisplayList) {
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

    doit(root.get_mut(), compute_absolute_positions, build_display_list);
}
