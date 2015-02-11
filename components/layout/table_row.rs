/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_blocks)]

use block::BlockFlow;
use block::ISizeAndMarginsComputer;
use construct::FlowConstructor;
use context::LayoutContext;
use flow::{FlowClass, Flow, ImmutableFlowUtils};
use flow;
use fragment::{Fragment, FragmentBorderBoxIterator};
use layout_debug;
use table::{ColumnComputedInlineSize, ColumnIntrinsicInlineSize, InternalTable};
use model::MaybeAuto;
use wrapper::ThreadSafeLayoutNode;

use geom::{Point2D, Rect};
use servo_util::geometry::Au;
use servo_util::logical_geometry::LogicalRect;
use std::cmp::max;
use std::fmt;
use style::properties::ComputedValues;
use style::values::computed::LengthOrPercentageOrAuto;
use std::sync::Arc;

/// A single row of a table.
#[derive(RustcEncodable)]
pub struct TableRowFlow {
    pub block_flow: BlockFlow,

    /// Information about the intrinsic inline-sizes of each cell.
    pub cell_intrinsic_inline_sizes: Vec<CellIntrinsicInlineSize>,

    /// Information about the computed inline-sizes of each column.
    pub column_computed_inline_sizes: Vec<ColumnComputedInlineSize>,
}

/// Information about the column inline size and span for each cell.
#[derive(RustcEncodable, Copy)]
pub struct CellIntrinsicInlineSize {
    /// Inline sizes that this cell contributes to the column.
    pub column_size: ColumnIntrinsicInlineSize,
    /// The column span of this cell.
    pub column_span: u32,
}

impl TableRowFlow {
    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode,
                                  fragment: Fragment)
                                  -> TableRowFlow {
        TableRowFlow {
            block_flow: BlockFlow::from_node_and_fragment(node, fragment),
            cell_intrinsic_inline_sizes: Vec::new(),
            column_computed_inline_sizes: Vec::new(),
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableRowFlow {
        TableRowFlow {
            block_flow: BlockFlow::from_node(constructor, node),
            cell_intrinsic_inline_sizes: Vec::new(),
            column_computed_inline_sizes: Vec::new()
        }
    }

    pub fn fragment<'a>(&'a mut self) -> &'a Fragment {
        &self.block_flow.fragment
    }

    fn initialize_offsets(&mut self) -> (Au, Au, Au) {
        // TODO: If border-collapse: collapse, block_start_offset, block_end_offset, and
        // inline_start_offset should be updated. Currently, they are set as Au(0).
        (Au(0), Au(0), Au(0))
    }

    /// Assign block-size for table-row flow.
    ///
    /// TODO(pcwalton): This doesn't handle floats and positioned elements right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_block_size_table_row_base<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        let (block_start_offset, _, _) = self.initialize_offsets();

        let /* mut */ cur_y = block_start_offset;

        // Per CSS 2.1 § 17.5.3, find max_y = max(computed `block-size`, minimum block-size of all
        // cells).
        let mut max_y = Au(0);
        for kid in self.block_flow.base.child_iter() {
            kid.place_float_if_applicable(layout_context);
            if !flow::base(kid).flags.is_float() {
                kid.assign_block_size_for_inorder_child_if_necessary(layout_context);
            }

            {
                let child_fragment = kid.as_table_cell().fragment();
                // TODO: Percentage block-size
                let child_specified_block_size =
                    MaybeAuto::from_style(child_fragment.style().content_block_size(),
                                          Au::new(0)).specified_or_zero();
                max_y = max(max_y,
                            child_specified_block_size +
                                child_fragment.border_padding.block_start_end());
            }
            let child_node = flow::mut_base(kid);
            child_node.position.start.b = cur_y;
            max_y = max(max_y, child_node.position.size.block);
        }

        let mut block_size = max_y;
        // TODO: Percentage block-size
        block_size = match MaybeAuto::from_style(self.block_flow
                                                     .fragment
                                                     .style()
                                                     .content_block_size(),
                                                 Au(0)) {
            MaybeAuto::Auto => block_size,
            MaybeAuto::Specified(value) => max(value, block_size)
        };
        // cur_y = cur_y + block-size;

        // Assign the block-size of own fragment
        //
        // FIXME(pcwalton): Take `cur_y` into account.
        let mut position = self.block_flow.fragment.border_box;
        position.size.block = block_size;
        self.block_flow.fragment.border_box = position;
        self.block_flow.base.position.size.block = block_size;

        // Assign the block-size of kid fragments, which is the same value as own block-size.
        for kid in self.block_flow.base.child_iter() {
            {
                let kid_fragment = kid.as_table_cell().mut_fragment();
                let mut position = kid_fragment.border_box;
                position.size.block = block_size;
                kid_fragment.border_box = position;
            }
            let child_node = flow::mut_base(kid);
            child_node.position.size.block = block_size;
        }
    }
}

impl Flow for TableRowFlow {
    fn class(&self) -> FlowClass {
        FlowClass::TableRow
    }

    fn as_table_row<'a>(&'a mut self) -> &'a mut TableRowFlow {
        self
    }

    fn as_immutable_table_row<'a>(&'a self) -> &'a TableRowFlow {
        self
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    fn column_intrinsic_inline_sizes<'a>(&'a mut self) -> &'a mut Vec<ColumnIntrinsicInlineSize> {
        panic!("can't call column_intrinsic_inline_sizes() on table row")
    }

    fn column_computed_inline_sizes<'a>(&'a mut self) -> &'a mut Vec<ColumnComputedInlineSize> {
        &mut self.column_computed_inline_sizes
    }

    /// Recursively (bottom-up) determines the context's preferred and minimum inline-sizes. When
    /// called on this context, all child contexts have had their min/pref inline-sizes set. This
    /// function must decide min/pref inline-sizes based on child context inline-sizes and
    /// dimensions of any fragments it is responsible for flowing.
    /// Min/pref inline-sizes set by this function are used in automatic table layout calculation.
    /// The specified column inline-sizes of children cells are used in fixed table layout
    /// calculation.
    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!("table_row::bubble_inline_sizes {:x}",
                                         self.block_flow.base.debug_id());

        // Bubble up the specified inline-sizes from child table cells.
        let (mut min_inline_size, mut pref_inline_size) = (Au(0), Au(0));
        for kid in self.block_flow.base.child_iter() {
            assert!(kid.is_table_cell());

            // Collect the specified column inline-size of the cell. This is used in both fixed and
            // automatic table layout calculation.
            let child_specified_inline_size;
            let child_column_span;
            {
                let child_table_cell = kid.as_table_cell();
                child_specified_inline_size = child_table_cell.fragment()
                                                              .style()
                                                              .content_inline_size();
                child_column_span = child_table_cell.column_span
            }

            // Collect minimum and preferred inline-sizes of the cell for automatic table layout
            // calculation.
            let child_base = flow::mut_base(kid);
            let child_column_inline_size = ColumnIntrinsicInlineSize {
                minimum_length: match child_specified_inline_size {
                    LengthOrPercentageOrAuto::Auto | LengthOrPercentageOrAuto::Percentage(_) => {
                        child_base.intrinsic_inline_sizes.minimum_inline_size
                    }
                    LengthOrPercentageOrAuto::Length(length) => length,
                },
                percentage: match child_specified_inline_size {
                    LengthOrPercentageOrAuto::Auto | LengthOrPercentageOrAuto::Length(_) => 0.0,
                    LengthOrPercentageOrAuto::Percentage(percentage) => percentage,
                },
                preferred: child_base.intrinsic_inline_sizes.preferred_inline_size,
                constrained: match child_specified_inline_size {
                    LengthOrPercentageOrAuto::Length(_) => true,
                    LengthOrPercentageOrAuto::Auto | LengthOrPercentageOrAuto::Percentage(_) => false,
                },
            };
            min_inline_size = min_inline_size + child_column_inline_size.minimum_length;
            pref_inline_size = pref_inline_size + child_column_inline_size.preferred;
            self.cell_intrinsic_inline_sizes.push(CellIntrinsicInlineSize {
                column_size: child_column_inline_size,
                column_span: child_column_span,
            });
        }
        self.block_flow.base.intrinsic_inline_sizes.minimum_inline_size = min_inline_size;
        self.block_flow.base.intrinsic_inline_sizes.preferred_inline_size = max(min_inline_size,
                                                                                pref_inline_size);
    }

    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("table_row::assign_inline_sizes {:x}",
                                         self.block_flow.base.debug_id());
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "table_row");

        // The position was set to the containing block by the flow's parent.
        let containing_block_inline_size = self.block_flow.base.block_container_inline_size;
        // FIXME: In case of border-collapse: collapse, inline_start_content_edge should be
        // border_inline_start.
        let inline_start_content_edge = Au(0);

        let inline_size_computer = InternalTable;
        inline_size_computer.compute_used_inline_size(&mut self.block_flow,
                                                      layout_context,
                                                      containing_block_inline_size);

        // Spread out the completed inline sizes among columns with spans > 1.
        let mut computed_inline_size_for_cells = Vec::new();
        let mut column_computed_inline_size_iterator = self.column_computed_inline_sizes.iter();
        for cell_intrinsic_inline_size in self.cell_intrinsic_inline_sizes.iter() {
            // Start with the computed inline size for the first column in the span.
            let mut column_computed_inline_size =
                match column_computed_inline_size_iterator.next() {
                    Some(column_computed_inline_size) => *column_computed_inline_size,
                    None => {
                        // We're in fixed layout mode and there are more cells in this row than
                        // columns we know about. According to CSS 2.1 § 17.5.2.1, the behavior is
                        // now undefined. So just use zero.
                        //
                        // FIXME(pcwalton): $10 says this isn't Web compatible.
                        ColumnComputedInlineSize {
                            size: Au(0),
                        }
                    }
                };

            // Add in computed inline sizes for any extra columns in the span.
            for _ in range(1, cell_intrinsic_inline_size.column_span) {
                let extra_column_computed_inline_size =
                    match column_computed_inline_size_iterator.next() {
                        Some(column_computed_inline_size) => column_computed_inline_size,
                        None => break,
                    };
                column_computed_inline_size.size = column_computed_inline_size.size +
                    extra_column_computed_inline_size.size;
            }

            computed_inline_size_for_cells.push(column_computed_inline_size)
        }

        // Push those inline sizes down to the cells.
        self.block_flow.propagate_assigned_inline_size_to_children(
            layout_context,
            inline_start_content_edge,
            containing_block_inline_size,
            Some(computed_inline_size_for_cells.as_slice()));
    }

    fn assign_block_size<'a>(&mut self, ctx: &'a LayoutContext<'a>) {
        debug!("assign_block_size: assigning block_size for table_row");
        self.assign_block_size_table_row_base(ctx);
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
}

impl fmt::Debug for TableRowFlow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableRowFlow: {:?}", self.block_flow.fragment)
    }
}
