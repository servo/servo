/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_code)]

use block::{BlockFlow, ISizeAndMarginsComputer};
use context::LayoutContext;
use flow::{FlowClass, Flow};
use fragment::{Fragment, FragmentBorderBoxIterator};
use layout_debug;
use style::computed_values::{border_collapse, border_spacing};
use table::{ColumnComputedInlineSize, ColumnIntrinsicInlineSize, InternalTable, TableLikeFlow};
use table_row::{self, CollapsedBordersForRow};
use wrapper::ThreadSafeLayoutNode;

use geom::{Point2D, Rect};
use rustc_serialize::{Encoder, Encodable};
use std::fmt;
use std::iter::{IntoIterator, Iterator, Peekable};
use std::sync::Arc;
use style::properties::ComputedValues;
use util::geometry::Au;
use util::logical_geometry::LogicalRect;

/// A table formatting context.
pub struct TableRowGroupFlow {
    /// Fields common to all block flows.
    pub block_flow: BlockFlow,

    /// Information about the intrinsic inline-sizes of each column.
    pub column_intrinsic_inline_sizes: Vec<ColumnIntrinsicInlineSize>,

    /// Information about the actual inline sizes of each column.
    pub column_computed_inline_sizes: Vec<ColumnComputedInlineSize>,

    /// The spacing for this rowgroup.
    pub spacing: border_spacing::T,

    /// Information about the borders for each cell that we bubble up to our parent. This is only
    /// computed if `border-collapse` is `collapse`.
    pub preliminary_collapsed_borders: CollapsedBordersForRow,

    /// The final width of the borders in the inline direction for each cell, computed by the
    /// entire table and pushed down into each row during inline size computation.
    pub collapsed_inline_direction_border_widths_for_table: Vec<Au>,

    /// The final width of the borders in the block direction for each cell, computed by the
    /// entire table and pushed down into each row during inline size computation.
    pub collapsed_block_direction_border_widths_for_table: Vec<Au>,
}

impl Encodable for TableRowGroupFlow {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        self.block_flow.encode(e)
    }
}

impl TableRowGroupFlow {
    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode, fragment: Fragment)
                                  -> TableRowGroupFlow {
        TableRowGroupFlow {
            block_flow: BlockFlow::from_node_and_fragment(node, fragment),
            column_intrinsic_inline_sizes: Vec::new(),
            column_computed_inline_sizes: Vec::new(),
            spacing: border_spacing::T {
                horizontal: Au(0),
                vertical: Au(0),
            },
            preliminary_collapsed_borders: CollapsedBordersForRow::new(),
            collapsed_inline_direction_border_widths_for_table: Vec::new(),
            collapsed_block_direction_border_widths_for_table: Vec::new(),
        }
    }

    pub fn fragment<'a>(&'a mut self) -> &'a Fragment {
        &self.block_flow.fragment
    }

    pub fn populate_collapsed_border_spacing<'a,I>(
            &mut self,
            collapsed_inline_direction_border_widths_for_table: &[Au],
            collapsed_block_direction_border_widths_for_table: &mut Peekable<I>)
            where I: Iterator<Item=&'a Au> {
        self.collapsed_inline_direction_border_widths_for_table.clear();
        self.collapsed_inline_direction_border_widths_for_table
            .extend(collapsed_inline_direction_border_widths_for_table.into_iter().map(|x| *x));

        for _ in range(0, self.block_flow.base.children.len()) {
            if let Some(collapsed_block_direction_border_width_for_table) =
                    collapsed_block_direction_border_widths_for_table.next() {
                self.collapsed_block_direction_border_widths_for_table
                    .push(*collapsed_block_direction_border_width_for_table)
            }
        }
        if let Some(collapsed_block_direction_border_width_for_table) =
                collapsed_block_direction_border_widths_for_table.peek() {
            self.collapsed_block_direction_border_widths_for_table
                .push(**collapsed_block_direction_border_width_for_table)
        }
    }
}

impl Flow for TableRowGroupFlow {
    fn class(&self) -> FlowClass {
        FlowClass::TableRowGroup
    }

    fn as_table_rowgroup<'a>(&'a mut self) -> &'a mut TableRowGroupFlow {
        self
    }

    fn as_immutable_table_rowgroup<'a>(&'a self) -> &'a TableRowGroupFlow {
        self
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    fn as_immutable_block(&self) -> &BlockFlow {
        &self.block_flow
    }

    fn column_intrinsic_inline_sizes<'a>(&'a mut self) -> &'a mut Vec<ColumnIntrinsicInlineSize> {
        &mut self.column_intrinsic_inline_sizes
    }

    fn column_computed_inline_sizes<'a>(&'a mut self) -> &'a mut Vec<ColumnComputedInlineSize> {
        &mut self.column_computed_inline_sizes
    }

    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!("table_rowgroup::bubble_inline_sizes {:x}",
                                         self.block_flow.base.debug_id());
        // Proper calculation of intrinsic sizes in table layout requires access to the entire
        // table, which we don't have yet. Defer to our parent.
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments.
    /// When called on this context, the context has had its inline-size set by the parent context.
    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("table_rowgroup::assign_inline_sizes {:x}",
                                            self.block_flow.base.debug_id());
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "table_rowgroup");

        // The position was set to the containing block by the flow's parent.
        let containing_block_inline_size = self.block_flow.base.block_container_inline_size;
        let (inline_start_content_edge, inline_end_content_edge) = (Au(0), Au(0));
        let content_inline_size = containing_block_inline_size;

        let inline_size_computer = InternalTable;
        let border_collapse = self.block_flow.fragment.style.get_inheritedtable().border_collapse;
        inline_size_computer.compute_used_inline_size(&mut self.block_flow,
                                                      layout_context,
                                                      containing_block_inline_size,
                                                      border_collapse);

        let column_computed_inline_sizes = self.column_computed_inline_sizes.as_slice();
        let border_spacing = self.spacing;
        let collapsed_inline_direction_border_widths_for_table =
            self.collapsed_inline_direction_border_widths_for_table.as_slice();
        let mut collapsed_block_direction_border_widths_for_table =
            self.collapsed_block_direction_border_widths_for_table.iter().peekable();
        self.block_flow.propagate_assigned_inline_size_to_children(layout_context,
                                                                   inline_start_content_edge,
                                                                   inline_end_content_edge,
                                                                   content_inline_size,
                                                                   |child_flow,
                                                                    child_index,
                                                                    content_inline_size,
                                                                    writing_mode,
                                                                    inline_start_margin_edge| {
            table_row::propagate_column_inline_sizes_to_child(
                child_flow,
                child_index,
                content_inline_size,
                writing_mode,
                column_computed_inline_sizes,
                &border_spacing,
                &None,
                inline_start_margin_edge);

            if border_collapse == border_collapse::T::collapse {
                let child_table_row = child_flow.as_table_row();
                child_table_row.populate_collapsed_border_spacing(
                    collapsed_inline_direction_border_widths_for_table.as_slice(),
                    &mut collapsed_block_direction_border_widths_for_table);
            }
        });
    }

    fn assign_block_size<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        debug!("assign_block_size: assigning block_size for table_rowgroup");
        self.block_flow.assign_block_size_for_table_like_flow(layout_context,
                                                              self.spacing.vertical)
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow.update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow.update_late_computed_block_position_if_necessary(block_position)
    }

    fn build_display_list(&mut self, layout_context: &LayoutContext) {
        debug!("build_display_list_table_rowgroup: same process as block flow");
        self.block_flow.build_display_list(layout_context)
    }

    fn repair_style(&mut self, new_style: &Arc<ComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn compute_overflow(&self) -> Rect<Au> {
        self.block_flow.compute_overflow()
    }

    fn generated_containing_block_rect(&self) -> LogicalRect<Au> {
        self.block_flow.generated_containing_block_rect()
    }

    fn iterate_through_fragment_border_boxes(&self,
                                             iterator: &mut FragmentBorderBoxIterator,
                                             stacking_context_position: &Point2D<Au>) {
        self.block_flow.iterate_through_fragment_border_boxes(iterator, stacking_context_position)
    }

    fn mutate_fragments(&mut self, mutator: &mut FnMut(&mut Fragment)) {
        self.block_flow.mutate_fragments(mutator)
    }
}

impl fmt::Debug for TableRowGroupFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableRowGroupFlow: {:?}", self.block_flow.fragment)
    }
}
