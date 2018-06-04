/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS tables.
//!
//! This follows the "More Precise Definitions of Inline Layout and Table Layout" proposal written
//! by L. David Baron (Mozilla) here:
//!
//!   http://dbaron.org/css/intrinsic/
//!
//! Hereafter this document is referred to as INTRINSIC.

#![deny(unsafe_code)]

use app_units::Au;
use block::{BlockFlow, MarginsMayCollapseFlag};
use context::LayoutContext;
use display_list::{BlockFlowDisplayListBuilding, DisplayListBuildState, StackingContextCollectionFlags};
use display_list::StackingContextCollectionState;
use euclid::Point2D;
use floats::FloatKind;
use flow::{Flow, FlowClass, OpaqueFlow};
use fragment::{Fragment, FragmentBorderBoxIterator, Overflow};
use gfx_traits::print_tree::PrintTree;
use std::fmt;
use style::computed_values::position;
use style::logical_geometry::{LogicalRect, LogicalSize};
use style::properties::ComputedValues;

#[derive(Clone, Copy, Debug, Serialize)]
pub enum TableLayout {
    Fixed,
    Auto
}

#[allow(unsafe_code)]
unsafe impl ::flow::HasBaseFlow for TableWrapperFlow {}

/// A table wrapper flow based on a block formatting context.
#[derive(Serialize)]
#[repr(C)]
pub struct TableWrapperFlow {
    pub block_flow: BlockFlow,
}

impl TableWrapperFlow {
    pub fn from_fragment(fragment: Fragment) -> TableWrapperFlow {
        TableWrapperFlow::from_fragment_and_float_kind(fragment, None)
    }

    pub fn from_fragment_and_float_kind(
        fragment: Fragment,
        float_kind: Option<FloatKind>
    ) -> TableWrapperFlow {
        TableWrapperFlow {
            block_flow: BlockFlow::from_fragment_and_float_kind(fragment, float_kind),
        }
    }
}

impl Flow for TableWrapperFlow {
    fn class(&self) -> FlowClass {
        FlowClass::TableWrapper
    }

    fn as_mut_table_wrapper(&mut self) -> &mut TableWrapperFlow {
        self
    }

    fn as_table_wrapper(&self) -> &TableWrapperFlow {
        self
    }

    fn as_mut_block(&mut self) -> &mut BlockFlow {
        &mut self.block_flow
    }

    fn as_block(&self) -> &BlockFlow {
        &self.block_flow
    }

    fn mark_as_root(&mut self) {
        self.block_flow.mark_as_root();
    }

    fn bubble_inline_sizes(&mut self) {
        self.block_flow.bubble_inline_sizes();
    }

    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        debug!("assign_inline_sizes({}): assigning inline_size for flow",
               if self.block_flow.base.flags.is_float() {
                   "floated table_wrapper"
               } else {
                   "table_wrapper"
               });

        let shared_context = layout_context.shared_context();
        self.block_flow.initialize_container_size_for_root(shared_context);

        let inline_start_content_edge = self.block_flow.fragment.border_box.start.i;
        let content_inline_size = self.block_flow.fragment.border_box.size.inline;
        let inline_end_content_edge = self.block_flow.fragment.border_padding.inline_end +
                                      self.block_flow.fragment.margin.inline_end;

        self.block_flow.propagate_assigned_inline_size_to_children(
            shared_context,
            inline_start_content_edge,
            inline_end_content_edge,
            content_inline_size,
            |_child_flow, _, _, _, _, _| {}
        )
    }

    fn assign_block_size(&mut self, layout_context: &LayoutContext) {
        debug!("assign_block_size: assigning block_size for table_wrapper");
        let remaining = self.block_flow.assign_block_size_block_base(
            layout_context,
            None,
            MarginsMayCollapseFlag::MarginsMayNotCollapse);
        debug_assert!(remaining.is_none());
    }

    fn compute_stacking_relative_position(&mut self, layout_context: &LayoutContext) {
        self.block_flow.compute_stacking_relative_position(layout_context)
    }

    fn place_float_if_applicable<'a>(&mut self) {
        self.block_flow.place_float_if_applicable()
    }

    fn assign_block_size_for_inorder_child_if_necessary(&mut self,
                                                        layout_context: &LayoutContext,
                                                        parent_thread_id: u8,
                                                        content_box: LogicalRect<Au>)
                                                        -> bool {
        self.block_flow.assign_block_size_for_inorder_child_if_necessary(layout_context,
                                                                         parent_thread_id,
                                                                         content_box)
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow.update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow.update_late_computed_block_position_if_necessary(block_position)
    }

    fn generated_containing_block_size(&self, flow: OpaqueFlow) -> LogicalSize<Au> {
        self.block_flow.generated_containing_block_size(flow)
    }

    fn build_display_list(&mut self, state: &mut DisplayListBuildState) {
        self.block_flow.build_display_list(state);
    }

    fn collect_stacking_contexts(&mut self, state: &mut StackingContextCollectionState) {
        self.block_flow.collect_stacking_contexts_for_block(
            state,
            StackingContextCollectionFlags::NEVER_CREATES_CONTAINING_BLOCK |
            StackingContextCollectionFlags::NEVER_CREATES_CLIP_SCROLL_NODE);
    }

    fn repair_style(&mut self, new_style: &::ServoArc<ComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn compute_overflow(&self) -> Overflow {
        self.block_flow.compute_overflow()
    }

    fn iterate_through_fragment_border_boxes(&self,
                                             iterator: &mut FragmentBorderBoxIterator,
                                             level: i32,
                                             stacking_context_position: &Point2D<Au>) {
        self.block_flow.iterate_through_fragment_border_boxes(iterator, level, stacking_context_position)
    }

    fn mutate_fragments(&mut self, mutator: &mut FnMut(&mut Fragment)) {
        self.block_flow.mutate_fragments(mutator)
    }

    fn print_extra_flow_children(&self, print_tree: &mut PrintTree) {
        self.block_flow.print_extra_flow_children(print_tree);
    }

    fn positioning(&self) -> position::T {
        self.block_flow.positioning()
    }
}

impl fmt::Debug for TableWrapperFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.block_flow.base.flags.is_float() {
            write!(f, "TableWrapperFlow(Float): {:?}", self.block_flow)
        } else {
            write!(f, "TableWrapperFlow: {:?}", self.block_flow)
        }
    }
}
