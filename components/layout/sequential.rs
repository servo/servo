/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Implements sequential traversals over the DOM and flow trees.

use app_units::Au;
use context::{LayoutContext, SharedLayoutContext};
use display_list_builder::DisplayListBuildState;
use euclid::point::Point2D;
use floats::SpeculatedFloatPlacement;
use flow::{self, Flow, ImmutableFlowUtils, InorderFlowTraversal, MutableFlowUtils, OpaqueFlow};
use flow::{PostorderFlowTraversal, PreorderFlowTraversal};
use flow::IS_ABSOLUTELY_POSITIONED;
use fragment::FragmentBorderBoxIterator;
use generated_content::ResolveGeneratedContent;
use gfx_traits::ScrollRootId;
use script_layout_interface::wrapper_traits::LayoutNode;
use style::context::StyleContext;
use style::servo::restyle_damage::{REFLOW, STORE_OVERFLOW};
use traversal::{AssignBSizes, AssignISizes, BubbleISizes, BuildDisplayList};
use util::opts;

pub use style::sequential::traverse_dom;

pub fn resolve_generated_content(root: &mut Flow, shared_layout_context: &SharedLayoutContext) {
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
    doit(root, 0, &mut traversal)
}

pub fn traverse_flow_tree_preorder(root: &mut Flow,
                                   shared_layout_context: &SharedLayoutContext) {
    fn doit(flow: &mut Flow,
            assign_inline_sizes: AssignISizes,
            assign_block_sizes: AssignBSizes) {
        if assign_inline_sizes.should_process(flow) {
            assign_inline_sizes.process(flow);
        }

        for kid in flow::child_iter_mut(flow) {
            doit(kid, assign_inline_sizes, assign_block_sizes);
        }

        if assign_block_sizes.should_process(flow) {
            assign_block_sizes.process(flow);
        }
    }

    let layout_context = LayoutContext::new(shared_layout_context);

    if opts::get().bubble_inline_sizes_separately {
        let bubble_inline_sizes = BubbleISizes { layout_context: &layout_context };
        {
            let root: &mut Flow = root;
            root.traverse_postorder(&bubble_inline_sizes);
        }
    }

    let assign_inline_sizes = AssignISizes { shared_context: layout_context.shared_context() };
    let assign_block_sizes  = AssignBSizes { layout_context: &layout_context };

    doit(root, assign_inline_sizes, assign_block_sizes);
}

pub fn build_display_list_for_subtree<'a>(flow_root: &mut Flow,
                                          shared_layout_context: &'a SharedLayoutContext)
                                          -> DisplayListBuildState<'a> {
    let mut state = DisplayListBuildState::new(shared_layout_context,
                                               flow::base(flow_root).stacking_context_id);
    flow_root.collect_stacking_contexts(&mut state.root_stacking_context, ScrollRootId::root());

    let mut build_display_list = BuildDisplayList { state: state };
    build_display_list.traverse(flow_root);
    build_display_list.state
}

/// Invokes a callback for each fragment belonging to the given node as well as all fragments
/// along the path leading to it.
///
/// The callback receives the stacking-relative border box of each fragment, for convenience.
///
/// FIXME(pcwalton): At some point, this should change to not iterate over fragments unless they
/// actually belong to the node.
pub fn for_each_fragment_of_node_and_ancestors<N>(root_flow: &mut Flow,
                                                  node: N,
                                                  iterator: &mut FragmentBorderBoxIterator)
                                                  where N: LayoutNode {
    // Find the path from the root to the node.
    //
    // FIXME(pcwalton): This would be unnecessary if we had parent pointers in the flow tree. I
    // think they're going to be inevitable.
    let mut path = flow::flow_tree_path_for_node(node);

    // If it's present, remove the root flow from the path, as we start there.
    loop {
        if let Some(&target) = path.last() {
            if OpaqueFlow::from_flow(root_flow) == target {
                path.pop();
                continue
            }
        }
        break
    }

    iterate(root_flow, 0, iterator, &mut path, &Point2D::zero());

    fn iterate(flow: &mut Flow,
               level: i32,
               iterator: &mut FragmentBorderBoxIterator,
               path: &mut Vec<OpaqueFlow>,
               stacking_context_position: &Point2D<Au>) {
        // Iterate through the border boxes of this flow.
        flow.iterate_through_fragment_border_boxes(iterator, level, stacking_context_position);

        // Are we done?
        if path.is_empty() {
            return
        }

        // Find the next child in the path.
        let target_child = *path.last().expect("Path ended prematurely!");
        for kid in flow::mut_base(flow).child_iter_mut() {
            if OpaqueFlow::from_flow(kid) == target_child {
                path.pop();
                let stacking_context_position =
                    stacking_context_position_for_child(kid, stacking_context_position);
                return iterate(kid, level + 1, iterator, path, &stacking_context_position)
            }
        }

        unreachable!("Didn't find next flow in path!")
    }

    fn stacking_context_position_for_child(child: &Flow, parent_position: &Point2D<Au>)
                                           -> Point2D<Au> {
        if child.is_block_flow() && child.as_block().fragment.establishes_stacking_context() {
            // FIXME(#2795): Get the real container size.
            let margin = Point2D::new(child.as_block().fragment.margin.inline_start, Au(0));
            *parent_position + flow::base(child).stacking_relative_position + margin
        } else {
            *parent_position
        }
    }
}

pub fn store_overflow(layout_context: &LayoutContext, flow: &mut Flow) {
    if !flow::base(flow).restyle_damage.contains(STORE_OVERFLOW) {
        return
    }

    for mut kid in flow::mut_base(flow).child_iter_mut() {
        store_overflow(layout_context, kid);
    }

    flow.store_overflow(layout_context);

    flow::mut_base(flow).restyle_damage.remove(STORE_OVERFLOW);
}

/// Guesses how much inline size will be taken up by floats on the left and right sides of the
/// given flow. This is needed to speculatively calculate the inline sizes of block formatting
/// contexts. The speculation typically succeeds, but if it doesn't we have to lay it out again.
pub fn guess_float_placement(flow: &mut Flow) {
    if !flow::base(flow).restyle_damage.intersects(REFLOW) {
        return
    }

    let mut floats_in = SpeculatedFloatPlacement::compute_floats_in_for_first_child(flow);
    for kid in flow::mut_base(flow).child_iter_mut() {
        if flow::base(kid).flags.contains(IS_ABSOLUTELY_POSITIONED) {
            // Do not propagate floats in or out, but do propogate between kids.
            guess_float_placement(kid);
        } else {
            floats_in.compute_floats_in(kid);
            flow::mut_base(kid).speculated_float_placement_in = floats_in;
            guess_float_placement(kid);
            floats_in = flow::base(kid).speculated_float_placement_out;
        }
    }
    floats_in.compute_floats_out(flow);
    flow::mut_base(flow).speculated_float_placement_out = floats_in
}
