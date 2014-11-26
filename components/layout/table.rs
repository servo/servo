/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_blocks)]

use block::{BlockFlow, MarginsMayNotCollapse, ISizeAndMarginsComputer};
use block::{ISizeConstraintInput, ISizeConstraintSolution};
use construct::FlowConstructor;
use context::LayoutContext;
use floats::FloatKind;
use flow::{Flow, FlowClass, IMPACTED_BY_LEFT_FLOATS, IMPACTED_BY_RIGHT_FLOATS, ImmutableFlowUtils};
use flow::{TableFlowClass};
use fragment::{Fragment, FragmentBoundsIterator};
use layout_debug;
use model::{IntrinsicISizes, IntrinsicISizesContribution};
use table_wrapper::{TableLayout, FixedLayout, AutoLayout};
use wrapper::ThreadSafeLayoutNode;

use servo_util::geometry::Au;
use servo_util::logical_geometry::LogicalRect;
use std::cmp::max;
use std::fmt;
use style::{ComputedValues, CSSFloat};
use style::computed_values::{LPA_Auto, LPA_Length, LPA_Percentage, table_layout};
use sync::Arc;

/// A table flow corresponded to the table's internal table fragment under a table wrapper flow.
/// The properties `position`, `float`, and `margin-*` are used on the table wrapper fragment,
/// not table fragment per CSS 2.1 § 10.5.
#[deriving(Encodable)]
pub struct TableFlow {
    pub block_flow: BlockFlow,

    /// Information about the inline-sizes of each column.
    pub column_inline_sizes: Vec<ColumnInlineSize>,

    /// Table-layout property
    pub table_layout: TableLayout,
}

impl TableFlow {
    pub fn from_node_and_fragment(node: &ThreadSafeLayoutNode,
                                  fragment: Fragment)
                                  -> TableFlow {
        let mut block_flow = BlockFlow::from_node_and_fragment(node, fragment);
        let table_layout = if block_flow.fragment().style().get_table().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableFlow {
            block_flow: block_flow,
            column_inline_sizes: Vec::new(),
            table_layout: table_layout
        }
    }

    pub fn from_node(constructor: &mut FlowConstructor,
                     node: &ThreadSafeLayoutNode)
                     -> TableFlow {
        let mut block_flow = BlockFlow::from_node(constructor, node);
        let table_layout = if block_flow.fragment().style().get_table().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableFlow {
            block_flow: block_flow,
            column_inline_sizes: Vec::new(),
            table_layout: table_layout
        }
    }

    pub fn float_from_node(constructor: &mut FlowConstructor,
                           node: &ThreadSafeLayoutNode,
                           float_kind: FloatKind)
                           -> TableFlow {
        let mut block_flow = BlockFlow::float_from_node(constructor, node, float_kind);
        let table_layout = if block_flow.fragment().style().get_table().table_layout ==
                              table_layout::fixed {
            FixedLayout
        } else {
            AutoLayout
        };
        TableFlow {
            block_flow: block_flow,
            column_inline_sizes: Vec::new(),
            table_layout: table_layout
        }
    }

    /// Update the corresponding value of `self_inline_sizes` if a value of `kid_inline_sizes` has
    /// a larger value than one of `self_inline_sizes`. Returns the minimum and preferred inline
    /// sizes.
    pub fn update_column_inline_sizes(parent_inline_sizes: &mut Vec<ColumnInlineSize>,
                                      child_inline_sizes: &Vec<ColumnInlineSize>)
                                      -> IntrinsicISizes {
        let mut total_inline_sizes = IntrinsicISizes::new();
        for (parent_sizes, child_sizes) in parent_inline_sizes.iter_mut()
                                                              .zip(child_inline_sizes.iter()) {
            *parent_sizes = ColumnInlineSize {
                minimum_length: max(parent_sizes.minimum_length, child_sizes.minimum_length),
                percentage: parent_sizes.greatest_percentage(child_sizes),
                preferred: max(parent_sizes.preferred, child_sizes.preferred),
                constrained: parent_sizes.constrained || child_sizes.constrained
            };

            total_inline_sizes.minimum_inline_size = total_inline_sizes.minimum_inline_size +
                parent_sizes.minimum_length;
            total_inline_sizes.preferred_inline_size = total_inline_sizes.preferred_inline_size +
                parent_sizes.preferred;
        }
        total_inline_sizes
    }

    /// Assign block-size for table flow.
    ///
    /// TODO(#2014, pcwalton): This probably doesn't handle margin collapse right.
    ///
    /// inline(always) because this is only ever called by in-order or non-in-order top-level
    /// methods
    #[inline(always)]
    fn assign_block_size_table_base<'a>(&mut self, layout_context: &'a LayoutContext<'a>) {
        self.block_flow.assign_block_size_block_base(layout_context, MarginsMayNotCollapse);
    }
}

impl Flow for TableFlow {
    fn class(&self) -> FlowClass {
        TableFlowClass
    }

    fn as_table<'a>(&'a mut self) -> &'a mut TableFlow {
        self
    }

    fn as_immutable_table<'a>(&'a self) -> &'a TableFlow {
        self
    }

    fn as_block<'a>(&'a mut self) -> &'a mut BlockFlow {
        &mut self.block_flow
    }

    fn column_inline_sizes<'a>(&'a mut self) -> &'a mut Vec<ColumnInlineSize> {
        &mut self.column_inline_sizes
    }

    /// The specified column inline-sizes are set from column group and the first row for the fixed
    /// table layout calculation.
    /// The maximum min/pref inline-sizes of each column are set from the rows for the automatic
    /// table layout calculation.
    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!("table::bubble_inline_sizes {:x}",
                                         self.block_flow.base.debug_id());

        let mut computation = IntrinsicISizesContribution::new();
        let mut did_first_row = false;
        for kid in self.block_flow.base.child_iter() {
            debug_assert!(kid.is_proper_table_child());
            if kid.is_table_colgroup() {
                for specified_inline_size in kid.as_table_colgroup().inline_sizes.iter() {
                    self.column_inline_sizes.push(ColumnInlineSize {
                        minimum_length: match *specified_inline_size {
                            LPA_Auto | LPA_Percentage(_) => Au(0),
                            LPA_Length(length) => length,
                        },
                        percentage: match *specified_inline_size {
                            LPA_Auto | LPA_Length(_) => 0.0,
                            LPA_Percentage(percentage) => percentage,
                        },
                        preferred: Au(0),
                        constrained: false,
                    })
                }
            } else if kid.is_table_rowgroup() || kid.is_table_row() {
                // Read column inline-sizes from the table-row-group/table-row, and assign
                // inline-size=0 for the columns not defined in the column group.
                // FIXME: Need to read inline-sizes from either table-header-group OR the first
                // table-row.
                match self.table_layout {
                    FixedLayout => {
                        // Fixed table layout only looks at the first row.
                        if !did_first_row {
                            did_first_row = true;
                            for child_column_inline_size in kid.column_inline_sizes().iter() {
                                self.column_inline_sizes.push(*child_column_inline_size);
                            }
                        }
                    }
                    AutoLayout => {
                        let child_column_inline_sizes = kid.column_inline_sizes();
                        let mut child_intrinsic_sizes =
                            TableFlow::update_column_inline_sizes(&mut self.column_inline_sizes,
                                                                  child_column_inline_sizes);

                        // Add new columns if processing this row caused us to discover them.
                        let child_column_count = child_column_inline_sizes.len();
                        let parent_column_count = self.column_inline_sizes.len();
                        debug!("table until the previous row has {} column(s) and this row has {} \
                                column(s)",
                               parent_column_count,
                               child_column_count);
                        self.column_inline_sizes.reserve(child_column_count);
                        for i in range(parent_column_count, child_column_count) {
                            let inline_size_for_new_column = (*child_column_inline_sizes)[i];
                            child_intrinsic_sizes.minimum_inline_size =
                                child_intrinsic_sizes.minimum_inline_size +
                                inline_size_for_new_column.minimum_length;
                            child_intrinsic_sizes.preferred_inline_size =
                                child_intrinsic_sizes.preferred_inline_size +
                                inline_size_for_new_column.preferred;
                            self.column_inline_sizes.push(inline_size_for_new_column);
                        }

                        computation.union_block(&child_intrinsic_sizes)
                    }
                }
            }
        }

        self.block_flow.base.intrinsic_inline_sizes = computation.finish()
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments.
    /// When called on this context, the context has had its inline-size set by the parent context.
    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("table::assign_inline_sizes {:x}",
                                            self.block_flow.base.debug_id());
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "table");

        // The position was set to the containing block by the flow's parent.
        let containing_block_inline_size = self.block_flow.base.block_container_inline_size;

        let mut num_unspecified_inline_sizes = 0;
        let mut total_column_inline_size = Au(0);
        for column_inline_size in self.column_inline_sizes.iter() {
            let this_column_inline_size = column_inline_size.minimum_length;
            if this_column_inline_size == Au(0) {
                num_unspecified_inline_sizes += 1
            } else {
                total_column_inline_size = total_column_inline_size + this_column_inline_size
            }
        }

        let inline_size_computer = InternalTable;

        inline_size_computer.compute_used_inline_size(&mut self.block_flow,
                                                      layout_context,
                                                      containing_block_inline_size);

        let inline_start_content_edge = self.block_flow.fragment.border_padding.inline_start;
        let padding_and_borders = self.block_flow.fragment.border_padding.inline_start_end();
        let content_inline_size =
            self.block_flow.fragment.border_box.size.inline - padding_and_borders;

        match self.table_layout {
            FixedLayout => {
                // In fixed table layout, we distribute extra space among the unspecified columns
                // if there are any, or among all the columns if all are specified.
                if total_column_inline_size < content_inline_size &&
                        num_unspecified_inline_sizes == 0 {
                    let ratio = content_inline_size.to_subpx() / total_column_inline_size.to_subpx();
                    for column_inline_size in self.column_inline_sizes.iter_mut() {
                        column_inline_size.minimum_length = column_inline_size.minimum_length.scale_by(ratio);
                        column_inline_size.percentage = 0.0;
                    }
                } else if num_unspecified_inline_sizes != 0 {
                    let extra_column_inline_size =
                        (content_inline_size - total_column_inline_size) /
                        num_unspecified_inline_sizes;
                    for column_inline_size in self.column_inline_sizes.iter_mut() {
                        if column_inline_size.minimum_length == Au(0) &&
                                column_inline_size.percentage == 0.0 {
                            column_inline_size.minimum_length = extra_column_inline_size /
                                num_unspecified_inline_sizes
                        }
                        column_inline_size.percentage = 0.0;
                    }
                }
            }
            _ => {}
        }

        // As tables are always wrapped inside a table wrapper, they are never impacted by floats.
        self.block_flow.base.flags.remove(IMPACTED_BY_LEFT_FLOATS);
        self.block_flow.base.flags.remove(IMPACTED_BY_RIGHT_FLOATS);

        self.block_flow.propagate_assigned_inline_size_to_children(
            inline_start_content_edge,
            content_inline_size,
            Some(self.column_inline_sizes.as_slice()));
    }

    fn assign_block_size<'a>(&mut self, ctx: &'a LayoutContext<'a>) {
        debug!("assign_block_size: assigning block_size for table");
        self.assign_block_size_table_base(ctx);
    }

    fn compute_absolute_position(&mut self) {
        self.block_flow.compute_absolute_position()
    }

    fn generated_containing_block_rect(&self) -> LogicalRect<Au> {
        self.block_flow.generated_containing_block_rect()
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow.update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow.update_late_computed_block_position_if_necessary(block_position)
    }

    fn build_display_list(&mut self, layout_context: &LayoutContext) {
        self.block_flow.build_display_list(layout_context);
    }

    fn repair_style(&mut self, new_style: &Arc<ComputedValues>) {
        self.block_flow.repair_style(new_style)
    }

    fn iterate_through_fragment_bounds(&self, iterator: &mut FragmentBoundsIterator) {
        self.block_flow.iterate_through_fragment_bounds(iterator);
    }
}

impl fmt::Show for TableFlow {
    /// Outputs a debugging string describing this table flow.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableFlow: {}", self.block_flow)
    }
}

/// Table, TableRowGroup, TableRow, TableCell types.
/// Their inline-sizes are calculated in the same way and do not have margins.
pub struct InternalTable;

impl ISizeAndMarginsComputer for InternalTable {
    /// Compute the used value of inline-size, taking care of min-inline-size and max-inline-size.
    ///
    /// CSS Section 10.4: Minimum and Maximum inline-sizes
    fn compute_used_inline_size(&self,
                                block: &mut BlockFlow,
                                ctx: &LayoutContext,
                                parent_flow_inline_size: Au) {
        let input = self.compute_inline_size_constraint_inputs(block,
                                                               parent_flow_inline_size,
                                                               ctx);
        let solution = self.solve_inline_size_constraints(block, &input);

        self.set_inline_size_constraint_solutions(block, solution);
    }

    /// Solve the inline-size and margins constraints for this block flow.
    fn solve_inline_size_constraints(&self, _: &mut BlockFlow, input: &ISizeConstraintInput)
                               -> ISizeConstraintSolution {
        ISizeConstraintSolution::new(input.available_inline_size, Au(0), Au(0))
    }
}

/// Information about the inline sizes of columns within a table.
///
/// During table inline-size bubbling, we might need to store both a percentage constraint and a
/// specific width constraint. For instance, one cell might say that it wants to be 100 pixels wide
/// in the inline direction and another cell might say that it wants to take up 20% of the inline-
/// size of the table. Now because we bubble up these constraints during the bubble-inline-sizes
/// phase of layout, we don't know yet how wide the table is ultimately going to be in the inline
/// direction. As we need to pick the maximum width of all cells for a column (in this case, the
/// maximum of 100 pixels and 20% of the table), the preceding constraint means that we must
/// potentially store both a specified width *and* a specified percentage, so that the inline-size
/// assignment phase of layout will know which one to pick.
#[deriving(Clone, Encodable, Show)]
pub struct ColumnInlineSize {
    /// The preferred intrinsic inline size.
    pub preferred: Au,
    /// The largest specified size of this column as a length.
    pub minimum_length: Au,
    /// The largest specified size of this column as a percentage (`width` property).
    pub percentage: CSSFloat,
    /// Whether the column inline size is *constrained* per INTRINSIC § 4.1.
    pub constrained: bool,
}

impl ColumnInlineSize {
    /// Returns the true minimum size of this column, given the containing block's inline size.
    /// Beware that this is generally only correct for fixed table layout. (Compare CSS 2.1 §
    /// 17.5.2.1 with the algorithm in INTRINSIC § 4.)
    pub fn minimum(&self, containing_block_inline_size: Au) -> Au {
        max(self.minimum_length, containing_block_inline_size.scale_by(self.percentage))
    }

    /// Returns the higher of the two percentages specified in `self` and `other`.
    pub fn greatest_percentage(&self, other: &ColumnInlineSize) -> CSSFloat {
        if self.percentage > other.percentage {
            self.percentage
        } else {
            other.percentage
        }
    }
}
