/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

use std::fmt;
use std::iter::{Iterator, Peekable};

use app_units::Au;
use base::print_tree::PrintTree;
use euclid::default::Point2D;
use log::{debug, trace};
use serde::{Serialize, Serializer};
use style::computed_values::{border_collapse, border_spacing};
use style::logical_geometry::LogicalSize;
use style::properties::ComputedValues;

use crate::block::{BlockFlow, ISizeAndMarginsComputer};
use crate::context::LayoutContext;
use crate::display_list::{
    DisplayListBuildState, StackingContextCollectionFlags, StackingContextCollectionState,
};
use crate::flow::{Flow, FlowClass, OpaqueFlow};
use crate::fragment::{Fragment, FragmentBorderBoxIterator, Overflow};
use crate::table::{ColumnIntrinsicInlineSize, InternalTable, TableLikeFlow};
use crate::{layout_debug, layout_debug_scope};

#[allow(unsafe_code)]
unsafe impl crate::flow::HasBaseFlow for TableRowGroupFlow {}

/// A table formatting context.
#[repr(C)]
pub struct TableRowGroupFlow {
    /// Fields common to all block flows.
    pub block_flow: BlockFlow,

    /// Information about the intrinsic inline-sizes of each column.
    pub column_intrinsic_inline_sizes: Vec<ColumnIntrinsicInlineSize>,

    /// The spacing for this rowgroup.
    pub spacing: border_spacing::T,

    /// The final width of the borders in the inline direction for each cell, computed by the
    /// entire table and pushed down into each row during inline size computation.
    pub collapsed_inline_direction_border_widths_for_table: Vec<Au>,

    /// The final width of the borders in the block direction for each cell, computed by the
    /// entire table and pushed down into each row during inline size computation.
    pub collapsed_block_direction_border_widths_for_table: Vec<Au>,
}

impl Serialize for TableRowGroupFlow {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.block_flow.serialize(serializer)
    }
}

impl TableRowGroupFlow {
    pub fn from_fragment(fragment: Fragment) -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_fragment(fragment),
            column_intrinsic_inline_sizes: Vec::new(),
            spacing: border_spacing::T::zero(),
            collapsed_inline_direction_border_widths_for_table: Vec::new(),
            collapsed_block_direction_border_widths_for_table: Vec::new(),
        }
    }

    pub fn populate_collapsed_border_spacing<'a, I>(
        &mut self,
        collapsed_inline_direction_border_widths_for_table: &[Au],
        collapsed_block_direction_border_widths_for_table: &mut Peekable<I>,
    ) where
        I: Iterator<Item = &'a Au>,
    {
        self.collapsed_inline_direction_border_widths_for_table
            .clear();
        self.collapsed_inline_direction_border_widths_for_table
            .extend(
                collapsed_inline_direction_border_widths_for_table
                    .iter()
                    .copied(),
            );

        for _ in 0..self.block_flow.base.children.len() {
            if let Some(collapsed_block_direction_border_width_for_table) =
                collapsed_block_direction_border_widths_for_table.next()
            {
                self.collapsed_block_direction_border_widths_for_table
                    .push(*collapsed_block_direction_border_width_for_table)
            }
        }
        if let Some(collapsed_block_direction_border_width_for_table) =
            collapsed_block_direction_border_widths_for_table.peek()
        {
            self.collapsed_block_direction_border_widths_for_table
                .push(**collapsed_block_direction_border_width_for_table)
        }
    }
}

impl Flow for TableRowGroupFlow {
    fn class(&self) -> FlowClass {
        FlowClass::TableRowGroup
    }

    fn as_mut_table_rowgroup(&mut self) -> &mut TableRowGroupFlow {
        self
    }

    fn as_table_rowgroup(&self) -> &TableRowGroupFlow {
        self
    }

    fn as_mut_block(&mut self) -> &mut BlockFlow {
        &mut self.block_flow
    }

    fn as_block(&self) -> &BlockFlow {
        &self.block_flow
    }

    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!(
            "table_rowgroup::bubble_inline_sizes {:x}",
            self.block_flow.base.debug_id()
        );
        // Proper calculation of intrinsic sizes in table layout requires access to the entire
        // table, which we don't have yet. Defer to our parent.
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments.
    /// When called on this context, the context has had its inline-size set by the parent context.
    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!(
            "table_rowgroup::assign_inline_sizes {:x}",
            self.block_flow.base.debug_id()
        );
        debug!(
            "assign_inline_sizes({}): assigning inline_size for flow",
            "table_rowgroup"
        );
        trace!("TableRowGroupFlow before assigning: {:?}", &self);

        let shared_context = layout_context.shared_context();
        // The position was set to the containing block by the flow's parent.
        let containing_block_inline_size = self.block_flow.base.block_container_inline_size;
        let (inline_start_content_edge, inline_end_content_edge) = (Au(0), Au(0));
        let content_inline_size = containing_block_inline_size;

        let border_collapse = self
            .block_flow
            .fragment
            .style
            .get_inherited_table()
            .border_collapse;
        let inline_size_computer = InternalTable;
        inline_size_computer.compute_used_inline_size(
            &mut self.block_flow,
            shared_context,
            containing_block_inline_size,
        );

        let collapsed_inline_direction_border_widths_for_table =
            &self.collapsed_inline_direction_border_widths_for_table;
        let mut collapsed_block_direction_border_widths_for_table = self
            .collapsed_block_direction_border_widths_for_table
            .iter()
            .peekable();
        self.block_flow.propagate_assigned_inline_size_to_children(
            shared_context,
            inline_start_content_edge,
            inline_end_content_edge,
            content_inline_size,
            |child_flow,
             _child_index,
             _content_inline_size,
             _writing_mode,
             _inline_start_margin_edge,
             _inline_end_margin_edge| {
                if border_collapse == border_collapse::T::Collapse {
                    let child_table_row = child_flow.as_mut_table_row();
                    child_table_row.populate_collapsed_border_spacing(
                        collapsed_inline_direction_border_widths_for_table,
                        &mut collapsed_block_direction_border_widths_for_table,
                    );
                }
            },
        );

        trace!("TableRowGroupFlow after assigning: {:?}", &self);
    }

    fn assign_block_size(&mut self, lc: &LayoutContext) {
        debug!("assign_block_size: assigning block_size for table_rowgroup");
        trace!("TableRowGroupFlow before assigning: {:?}", &self);

        self.block_flow
            .assign_block_size_for_table_like_flow(self.spacing.vertical(), lc);

        trace!("TableRowGroupFlow after assigning: {:?}", &self);
    }

    fn compute_stacking_relative_position(&mut self, layout_context: &LayoutContext) {
        self.block_flow
            .compute_stacking_relative_position(layout_context)
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow
            .update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow
            .update_late_computed_block_position_if_necessary(block_position)
    }

    fn build_display_list(&mut self, _: &mut DisplayListBuildState) {
        use style::servo::restyle_damage::ServoRestyleDamage;

        // we skip setting the damage in TableCellStyleInfo::build_display_list()
        // because we only have immutable access
        self.block_flow
            .fragment
            .restyle_damage
            .remove(ServoRestyleDamage::REPAINT);
    }

    fn collect_stacking_contexts(&mut self, state: &mut StackingContextCollectionState) {
        self.block_flow.collect_stacking_contexts_for_block(
            state,
            StackingContextCollectionFlags::POSITION_NEVER_CREATES_CONTAINING_BLOCK,
        );
    }

    fn repair_style(&mut self, new_style: &crate::ServoArc<ComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn compute_overflow(&self) -> Overflow {
        self.block_flow.compute_overflow()
    }

    fn contains_roots_of_absolute_flow_tree(&self) -> bool {
        self.block_flow.contains_roots_of_absolute_flow_tree()
    }

    fn is_absolute_containing_block(&self) -> bool {
        self.block_flow.is_absolute_containing_block()
    }

    fn generated_containing_block_size(&self, flow: OpaqueFlow) -> LogicalSize<Au> {
        self.block_flow.generated_containing_block_size(flow)
    }

    fn iterate_through_fragment_border_boxes(
        &self,
        iterator: &mut dyn FragmentBorderBoxIterator,
        level: i32,
        stacking_context_position: &Point2D<Au>,
    ) {
        self.block_flow.iterate_through_fragment_border_boxes(
            iterator,
            level,
            stacking_context_position,
        )
    }

    fn mutate_fragments(&mut self, mutator: &mut dyn FnMut(&mut Fragment)) {
        self.block_flow.mutate_fragments(mutator)
    }

    fn print_extra_flow_children(&self, print_tree: &mut PrintTree) {
        self.block_flow.print_extra_flow_children(print_tree);
    }
}

impl fmt::Debug for TableRowGroupFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableRowGroupFlow: {:?}", self.block_flow)
    }
}
