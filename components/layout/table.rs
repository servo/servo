/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS table formatting contexts.

#![deny(unsafe_code)]

use app_units::Au;
use block::{BlockFlow, CandidateBSizeIterator, ISizeAndMarginsComputer};
use block::{ISizeConstraintInput, ISizeConstraintSolution};
use context::LayoutContext;
use display_list_builder::{BlockFlowDisplayListBuilding, BorderPaintingMode};
use display_list_builder::{DisplayListBuildState, StackingContextCollectionFlags, StackingContextCollectionState};
use euclid::Point2D;
use flow;
use flow::{BaseFlow, EarlyAbsolutePositionInfo, Flow, FlowClass, ImmutableFlowUtils, OpaqueFlow};
use flow_list::MutFlowListIterator;
use fragment::{Fragment, FragmentBorderBoxIterator, Overflow};
use gfx_traits::print_tree::PrintTree;
use layout_debug;
use model::{IntrinsicISizes, IntrinsicISizesContribution, MaybeAuto};
use std::cmp;
use std::fmt;
use style::computed_values::{border_collapse, border_spacing, table_layout};
use style::context::SharedStyleContext;
use style::logical_geometry::LogicalSize;
use style::properties::ComputedValues;
use style::servo::restyle_damage::ServoRestyleDamage;
use style::values::CSSFloat;
use style::values::computed::LengthOrPercentageOrAuto;
use table_row::{self, CellIntrinsicInlineSize, CollapsedBorder, CollapsedBorderProvenance};
use table_row::TableRowFlow;
use table_wrapper::TableLayout;

#[allow(unsafe_code)]
unsafe impl ::flow::HasBaseFlow for TableFlow {}

/// A table flow corresponded to the table's internal table fragment under a table wrapper flow.
/// The properties `position`, `float`, and `margin-*` are used on the table wrapper fragment,
/// not table fragment per CSS 2.1 § 10.5.
#[derive(Serialize)]
#[repr(C)]
pub struct TableFlow {
    pub block_flow: BlockFlow,

    /// Information about the intrinsic inline-sizes of each column, computed bottom-up during
    /// intrinsic inline-size bubbling.
    pub column_intrinsic_inline_sizes: Vec<ColumnIntrinsicInlineSize>,

    /// Information about the actual inline sizes of each column, computed top-down during actual
    /// inline-size bubbling.
    pub column_computed_inline_sizes: Vec<ColumnComputedInlineSize>,

    /// The final width of the borders in the inline direction for each cell, computed by the
    /// entire table and pushed down into each row during inline size computation.
    pub collapsed_inline_direction_border_widths_for_table: Vec<Au>,

    /// The final width of the borders in the block direction for each cell, computed by the
    /// entire table and pushed down into each row during inline size computation.
    pub collapsed_block_direction_border_widths_for_table: Vec<Au>,

    /// Table-layout property
    pub table_layout: TableLayout,
}

impl TableFlow {
    pub fn from_fragment(fragment: Fragment) -> TableFlow {
        let mut block_flow = BlockFlow::from_fragment(fragment);
        let table_layout =
            if block_flow.fragment().style().get_table().table_layout == table_layout::T::Fixed {
                TableLayout::Fixed
            } else {
                TableLayout::Auto
            };
        TableFlow {
            block_flow: block_flow,
            column_intrinsic_inline_sizes: Vec::new(),
            column_computed_inline_sizes: Vec::new(),
            collapsed_inline_direction_border_widths_for_table: Vec::new(),
            collapsed_block_direction_border_widths_for_table: Vec::new(),
            table_layout: table_layout
        }
    }

    /// Update the corresponding value of `self_inline_sizes` if a value of `kid_inline_sizes` has
    /// a larger value than one of `self_inline_sizes`. Returns the minimum and preferred inline
    /// sizes.
    fn update_automatic_column_inline_sizes(
            parent_inline_sizes: &mut Vec<ColumnIntrinsicInlineSize>,
            child_cell_inline_sizes: &[CellIntrinsicInlineSize],
            surrounding_size: Au)
            -> IntrinsicISizes {
        let mut total_inline_sizes = IntrinsicISizes {
            minimum_inline_size: surrounding_size,
            preferred_inline_size: surrounding_size,
        };
        let mut column_index = 0;
        let mut incoming_rowspan = vec![];

        for child_cell_inline_size in child_cell_inline_sizes {
            // Skip any column occupied by a cell from a previous row.
            while column_index < incoming_rowspan.len() && incoming_rowspan[column_index] != 1 {
                if incoming_rowspan[column_index] > 1 {
                    incoming_rowspan[column_index] -= 1;
                }
                column_index += 1;
            }
            for _ in 0..child_cell_inline_size.column_span {
                if column_index < parent_inline_sizes.len() {
                    // We already have some intrinsic size information for this column. Merge it in
                    // according to the rules specified in INTRINSIC § 4.
                    let parent_sizes = &mut parent_inline_sizes[column_index];
                    if child_cell_inline_size.column_span > 1 {
                        // TODO(pcwalton): Perform the recursive algorithm specified in INTRINSIC §
                        // 4. For now we make this column contribute no width.
                    } else {
                        let column_size = &child_cell_inline_size.column_size;
                        *parent_sizes = ColumnIntrinsicInlineSize {
                            minimum_length: cmp::max(parent_sizes.minimum_length,
                                                     column_size.minimum_length),
                            percentage: parent_sizes.greatest_percentage(column_size),
                            preferred: cmp::max(parent_sizes.preferred, column_size.preferred),
                            constrained: parent_sizes.constrained || column_size.constrained,
                        }
                    }
                } else {
                    // We discovered a new column. Initialize its data.
                    debug_assert!(column_index == parent_inline_sizes.len());
                    if child_cell_inline_size.column_span > 1 {
                        // TODO(pcwalton): Perform the recursive algorithm specified in INTRINSIC §
                        // 4. For now we make this column contribute no width.
                        parent_inline_sizes.push(ColumnIntrinsicInlineSize::new())
                    } else {
                        parent_inline_sizes.push(child_cell_inline_size.column_size)
                    }
                }

                total_inline_sizes.minimum_inline_size +=
                    parent_inline_sizes[column_index].minimum_length;
                total_inline_sizes.preferred_inline_size +=
                    parent_inline_sizes[column_index].preferred;

                // If this cell spans later rows, record its rowspan.
                if child_cell_inline_size.row_span > 1 {
                    if incoming_rowspan.len() < column_index + 1 {
                        incoming_rowspan.resize(column_index + 1, 0);
                    }
                    incoming_rowspan[column_index] = child_cell_inline_size.row_span;
                }

                column_index += 1
            }
        }

        total_inline_sizes
    }

    /// Updates the minimum and preferred inline-size calculation for a single row. This is
    /// factored out into a separate function because we process children of rowgroups too.
    fn update_column_inline_sizes_for_row(row: &TableRowFlow,
                                          column_inline_sizes: &mut Vec<ColumnIntrinsicInlineSize>,
                                          computation: &mut IntrinsicISizesContribution,
                                          first_row: bool,
                                          table_layout: TableLayout,
                                          surrounding_inline_size: Au) {
        // Read column inline-sizes from the table-row, and assign inline-size=0 for the columns
        // not defined in the column group.
        //
        // FIXME: Need to read inline-sizes from either table-header-group OR the first table-row.
        match table_layout {
            TableLayout::Fixed => {
                // Fixed table layout only looks at the first row.
                //
                // FIXME(pcwalton): This is really inefficient. We should stop after the first row!
                if first_row {
                    for cell_inline_size in &row.cell_intrinsic_inline_sizes {
                        column_inline_sizes.push(cell_inline_size.column_size);
                    }
                }
            }
            TableLayout::Auto => {
                computation.union_block(&TableFlow::update_automatic_column_inline_sizes(
                    column_inline_sizes,
                    &row.cell_intrinsic_inline_sizes,
                    surrounding_inline_size))
            }
        }
    }

    /// Returns the effective spacing per cell, taking the value of `border-collapse` into account.
    pub fn spacing(&self) -> border_spacing::T {
        let style = self.block_flow.fragment.style();
        match style.get_inheritedtable().border_collapse {
            border_collapse::T::Separate => style.get_inheritedtable().border_spacing,
            border_collapse::T::Collapse => border_spacing::T::zero(),
        }
    }

    pub fn total_horizontal_spacing(&self) -> Au {
        let num_columns = self.column_intrinsic_inline_sizes.len();
        if num_columns == 0 {
            return Au(0);
        }
        self.spacing().horizontal() * (num_columns as i32 + 1)
    }
}

impl Flow for TableFlow {
    fn class(&self) -> FlowClass {
        FlowClass::Table
    }

    fn as_mut_table(&mut self) -> &mut TableFlow {
        self
    }

    fn as_table(&self) -> &TableFlow {
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

    /// The specified column inline-sizes are set from column group and the first row for the fixed
    /// table layout calculation.
    /// The maximum min/pref inline-sizes of each column are set from the rows for the automatic
    /// table layout calculation.
    fn bubble_inline_sizes(&mut self) {
        let _scope = layout_debug_scope!("table::bubble_inline_sizes {:x}",
                                         self.block_flow.base.debug_id());

        // Get column inline sizes from colgroups
        for kid in self.block_flow.base.child_iter_mut().filter(|kid| kid.is_table_colgroup()) {
            for specified_inline_size in &kid.as_mut_table_colgroup().inline_sizes {
                self.column_intrinsic_inline_sizes.push(ColumnIntrinsicInlineSize {
                    minimum_length: match *specified_inline_size {
                        LengthOrPercentageOrAuto::Auto |
                        LengthOrPercentageOrAuto::Calc(_) |
                        LengthOrPercentageOrAuto::Percentage(_) => Au(0),
                        LengthOrPercentageOrAuto::Length(length) => Au::from(length),
                    },
                    percentage: match *specified_inline_size {
                        LengthOrPercentageOrAuto::Auto |
                        LengthOrPercentageOrAuto::Calc(_) |
                        LengthOrPercentageOrAuto::Length(_) => 0.0,
                        LengthOrPercentageOrAuto::Percentage(percentage) => percentage.0,
                    },
                    preferred: Au(0),
                    constrained: false,
                })
            }
        }

        self.collapsed_inline_direction_border_widths_for_table = Vec::new();
        self.collapsed_block_direction_border_widths_for_table = vec![Au(0)];

        let collapsing_borders = self.block_flow
                                     .fragment
                                     .style
                                     .get_inheritedtable()
                                     .border_collapse == border_collapse::T::Collapse;
        let table_inline_collapsed_borders = if collapsing_borders {
            Some(TableInlineCollapsedBorders {
                start: CollapsedBorder::inline_start(&*self.block_flow.fragment.style,
                                                     CollapsedBorderProvenance::FromTable),
                end: CollapsedBorder::inline_end(&*self.block_flow.fragment.style,
                                                 CollapsedBorderProvenance::FromTable),
            })
        } else {
            None
        };

        let mut computation = IntrinsicISizesContribution::new();
        let mut previous_collapsed_block_end_borders =
            PreviousBlockCollapsedBorders::FromTable(CollapsedBorder::block_start(
                    &*self.block_flow.fragment.style,
                    CollapsedBorderProvenance::FromTable));
        let mut first_row = true;
        let (border_padding, _) = self.block_flow.fragment.surrounding_intrinsic_inline_size();

        {
            let mut iterator = TableRowIterator::new(&mut self.block_flow.base).peekable();
            while let Some(row) = iterator.next() {
                TableFlow::update_column_inline_sizes_for_row(
                        row,
                        &mut self.column_intrinsic_inline_sizes,
                        &mut computation,
                        first_row,
                        self.table_layout,
                        border_padding);
                if collapsing_borders {
                    let next_index_and_sibling = iterator.peek();
                    let next_collapsed_borders_in_block_direction =
                        match next_index_and_sibling {
                            Some(next_sibling) => {
                                NextBlockCollapsedBorders::FromNextRow(
                                    &next_sibling.as_table_row()
                                                 .preliminary_collapsed_borders
                                                 .block_start)
                            }
                            None => {
                                NextBlockCollapsedBorders::FromTable(
                                    CollapsedBorder::block_end(&*self.block_flow.fragment.style,
                                                               CollapsedBorderProvenance::FromTable))
                            }
                        };
                    perform_border_collapse_for_row(row,
                        table_inline_collapsed_borders.as_ref().unwrap(),
                        previous_collapsed_block_end_borders,
                        next_collapsed_borders_in_block_direction,
                        &mut self.collapsed_inline_direction_border_widths_for_table,
                        &mut self.collapsed_block_direction_border_widths_for_table);
                    previous_collapsed_block_end_borders =
                        PreviousBlockCollapsedBorders::FromPreviousRow(
                            row.final_collapsed_borders.block_end.clone());
                }
                first_row = false
            };
        }

        let total_horizontal_spacing = self.total_horizontal_spacing();
        let mut style_specified_intrinsic_inline_size =
            self.block_flow
                .fragment
                .style_specified_intrinsic_inline_size()
                .finish();
        style_specified_intrinsic_inline_size.minimum_inline_size -= total_horizontal_spacing;
        style_specified_intrinsic_inline_size.preferred_inline_size -= total_horizontal_spacing;
        computation.union_block(&style_specified_intrinsic_inline_size);
        computation.surrounding_size += total_horizontal_spacing;

        self.block_flow.base.intrinsic_inline_sizes = computation.finish()
    }

    /// Recursively (top-down) determines the actual inline-size of child contexts and fragments.
    /// When called on this context, the context has had its inline-size set by the parent context.
    fn assign_inline_sizes(&mut self, layout_context: &LayoutContext) {
        let _scope = layout_debug_scope!("table::assign_inline_sizes {:x}",
                                            self.block_flow.base.debug_id());
        debug!("assign_inline_sizes({}): assigning inline_size for flow", "table");

        let shared_context = layout_context.shared_context();
        // The position was set to the containing block by the flow's parent.
        // FIXME: The code for distributing column widths should really be placed under table_wrapper.rs.
        let containing_block_inline_size = self.block_flow.base.block_container_inline_size;

        let mut constrained_column_inline_sizes_indices = vec![];
        let mut unspecified_inline_sizes_indices = vec![];
        for (idx, column_inline_size) in self.column_intrinsic_inline_sizes.iter().enumerate() {
            if column_inline_size.constrained {
                constrained_column_inline_sizes_indices.push(idx);
            } else if column_inline_size.percentage == 0.0 {
                unspecified_inline_sizes_indices.push(idx);
            }
        }

        let inline_size_computer = InternalTable;
        inline_size_computer.compute_used_inline_size(&mut self.block_flow,
                                                      shared_context,
                                                      containing_block_inline_size);

        let inline_start_content_edge = self.block_flow.fragment.border_padding.inline_start;
        let inline_end_content_edge = self.block_flow.fragment.border_padding.inline_end;
        let padding_and_borders = self.block_flow.fragment.border_padding.inline_start_end();
        let spacing_per_cell = self.spacing();
        let total_horizontal_spacing = self.total_horizontal_spacing();
        let content_inline_size = self.block_flow.fragment.border_box.size.inline -
            padding_and_borders - total_horizontal_spacing;
        let mut remaining_inline_size = content_inline_size;

        match self.table_layout {
            TableLayout::Fixed => {
                self.column_computed_inline_sizes.clear();

                // https://drafts.csswg.org/css2/tables.html#fixed-table-layout
                for column_inline_size in &self.column_intrinsic_inline_sizes {
                    if column_inline_size.constrained {
                        self.column_computed_inline_sizes.push(ColumnComputedInlineSize {
                            size: column_inline_size.minimum_length,
                        });
                        remaining_inline_size -= column_inline_size.minimum_length;
                    } else if column_inline_size.percentage != 0.0 {
                        let size = remaining_inline_size.scale_by(column_inline_size.percentage);
                        self.column_computed_inline_sizes.push(ColumnComputedInlineSize {
                            size: size,
                        });
                        remaining_inline_size -= size;
                    } else {
                        // Set the size to 0 now, distribute the remaining widths later
                        self.column_computed_inline_sizes.push(ColumnComputedInlineSize {
                            size: Au(0),
                        });
                    }
                }

                // Distribute remaining content inline size
                if unspecified_inline_sizes_indices.len() > 0 {
                    for &index in &unspecified_inline_sizes_indices {
                        self.column_computed_inline_sizes[index].size =
                            remaining_inline_size.scale_by(1.0 / unspecified_inline_sizes_indices.len() as f32);
                    }
                } else {
                    let total_minimum_size = self.column_intrinsic_inline_sizes
                        .iter()
                        .filter(|size| size.constrained)
                        .map(|size| size.minimum_length.0 as f32)
                        .sum::<f32>();

                    for &index in &constrained_column_inline_sizes_indices {
                        self.column_computed_inline_sizes[index].size +=
                            remaining_inline_size.scale_by(
                                self.column_computed_inline_sizes[index].size.0 as f32 / total_minimum_size);
                    }
                }
            }
            _ => {
                // The table wrapper already computed the inline-sizes and propagated them down
                // to us.
            }
        }

        let column_computed_inline_sizes = &self.column_computed_inline_sizes;
        let collapsed_inline_direction_border_widths_for_table =
            &self.collapsed_inline_direction_border_widths_for_table;
        let mut collapsed_block_direction_border_widths_for_table =
            self.collapsed_block_direction_border_widths_for_table.iter().peekable();
        let mut incoming_rowspan = vec![];
        self.block_flow.propagate_assigned_inline_size_to_children(shared_context,
                                                                   inline_start_content_edge,
                                                                   inline_end_content_edge,
                                                                   content_inline_size,
                                                                   |child_flow,
                                                                    _child_index,
                                                                    _content_inline_size,
                                                                    writing_mode,
                                                                    _inline_start_margin_edge,
                                                                    _inline_end_margin_edge| {
            table_row::propagate_column_inline_sizes_to_child(
                child_flow,
                writing_mode,
                column_computed_inline_sizes,
                &spacing_per_cell,
                &mut incoming_rowspan);
            if child_flow.is_table_row() {
                let child_table_row = child_flow.as_mut_table_row();
                child_table_row.populate_collapsed_border_spacing(
                    collapsed_inline_direction_border_widths_for_table,
                    &mut collapsed_block_direction_border_widths_for_table);
            } else if child_flow.is_table_rowgroup() {
                let child_table_rowgroup = child_flow.as_mut_table_rowgroup();
                child_table_rowgroup.populate_collapsed_border_spacing(
                    collapsed_inline_direction_border_widths_for_table,
                    &mut collapsed_block_direction_border_widths_for_table);
            }
        });
    }

    fn assign_block_size(&mut self, _: &LayoutContext) {
        debug!("assign_block_size: assigning block_size for table");
        let vertical_spacing = self.spacing().vertical();
        self.block_flow.assign_block_size_for_table_like_flow(vertical_spacing)
    }

    fn compute_stacking_relative_position(&mut self, layout_context: &LayoutContext) {
        self.block_flow.compute_stacking_relative_position(layout_context)
    }

    fn generated_containing_block_size(&self, flow: OpaqueFlow) -> LogicalSize<Au> {
        self.block_flow.generated_containing_block_size(flow)
    }

    fn update_late_computed_inline_position_if_necessary(&mut self, inline_position: Au) {
        self.block_flow.update_late_computed_inline_position_if_necessary(inline_position)
    }

    fn update_late_computed_block_position_if_necessary(&mut self, block_position: Au) {
        self.block_flow.update_late_computed_block_position_if_necessary(block_position)
    }

    fn build_display_list(&mut self, state: &mut DisplayListBuildState) {
        let border_painting_mode = match self.block_flow
                                             .fragment
                                             .style
                                             .get_inheritedtable()
                                             .border_collapse {
            border_collapse::T::Separate => BorderPaintingMode::Separate,
            border_collapse::T::Collapse => BorderPaintingMode::Hidden,
        };

        self.block_flow.build_display_list_for_block(state, border_painting_mode);
    }

    fn collect_stacking_contexts(&mut self, state: &mut StackingContextCollectionState) {
        // Stacking contexts are collected by the table wrapper.
        self.block_flow.collect_stacking_contexts_for_block(state,
            StackingContextCollectionFlags::NEVER_CREATES_STACKING_CONTEXT);
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
}

impl fmt::Debug for TableFlow {
    /// Outputs a debugging string describing this table flow.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TableFlow: {:?}", self.block_flow)
    }
}

/// Table, TableRowGroup, TableRow, TableCell types.
/// Their inline-sizes are calculated in the same way and do not have margins.
pub struct InternalTable;

impl ISizeAndMarginsComputer for InternalTable {
    /// Compute the used value of inline-size, taking care of min-inline-size and max-inline-size.
    ///
    /// CSS Section 10.4: Minimum and Maximum inline-sizes
    fn compute_used_inline_size(
        &self,
        block: &mut BlockFlow,
        shared_context: &SharedStyleContext,
        parent_flow_inline_size: Au
    ) {
        let mut input = self.compute_inline_size_constraint_inputs(block,
                                                                   parent_flow_inline_size,
                                                                   shared_context);

        // Tables are always at least as wide as their minimum inline size.
        let minimum_inline_size =
            block.base.intrinsic_inline_sizes.minimum_inline_size -
            block.fragment.border_padding.inline_start_end();
        input.available_inline_size = cmp::max(input.available_inline_size, minimum_inline_size);

        let solution = self.solve_inline_size_constraints(block, &input);
        self.set_inline_size_constraint_solutions(block, solution);
    }

    /// Solve the inline-size and margins constraints for this block flow.
    fn solve_inline_size_constraints(&self, _: &mut BlockFlow, input: &ISizeConstraintInput)
                                     -> ISizeConstraintSolution {
        ISizeConstraintSolution::new(input.available_inline_size, Au(0), Au(0))
    }
}

/// Information about the intrinsic inline sizes of columns within a table.
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
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ColumnIntrinsicInlineSize {
    /// The preferred intrinsic inline size.
    pub preferred: Au,
    /// The largest specified size of this column as a length.
    pub minimum_length: Au,
    /// The largest specified size of this column as a percentage (`width` property).
    pub percentage: CSSFloat,
    /// Whether the column inline size is *constrained* per INTRINSIC § 4.1.
    pub constrained: bool,
}

impl ColumnIntrinsicInlineSize {
    /// Returns a newly-initialized `ColumnIntrinsicInlineSize` with all fields blank.
    pub fn new() -> ColumnIntrinsicInlineSize {
        ColumnIntrinsicInlineSize {
            preferred: Au(0),
            minimum_length: Au(0),
            percentage: 0.0,
            constrained: false,
        }
    }

    /// Returns the higher of the two percentages specified in `self` and `other`.
    pub fn greatest_percentage(&self, other: &ColumnIntrinsicInlineSize) -> CSSFloat {
        if self.percentage > other.percentage {
            self.percentage
        } else {
            other.percentage
        }
    }
}

/// The actual inline size for each column.
///
/// TODO(pcwalton): There will probably be some `border-collapse`-related info in here too
/// eventually.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ColumnComputedInlineSize {
    /// The computed size of this inline column.
    pub size: Au,
}

pub trait VecExt<T> {
    fn push_or_set(&mut self, index: usize, value: T) -> &mut T;
    fn get_mut_or_push(&mut self, index: usize, zero: T) -> &mut T;
}

impl<T> VecExt<T> for Vec<T> {
    fn push_or_set(&mut self, index: usize, value: T) -> &mut T {
        if index < self.len() {
            self[index] = value
        } else {
            debug_assert!(index == self.len());
            self.push(value)
        }
        &mut self[index]
    }

    fn get_mut_or_push(&mut self, index: usize, zero: T) -> &mut T {
        if index >= self.len() {
            debug_assert!(index == self.len());
            self.push(zero)
        }
        &mut self[index]
    }
}

/// Updates the border styles in the block direction for a single row. This function should
/// only be called if border collapsing is on. It is factored out into a separate function
/// because we process children of rowgroups too.
fn perform_border_collapse_for_row(child_table_row: &mut TableRowFlow,
                                   table_inline_borders: &TableInlineCollapsedBorders,
                                   previous_block_borders: PreviousBlockCollapsedBorders,
                                   next_block_borders: NextBlockCollapsedBorders,
                                   inline_spacing: &mut Vec<Au>,
                                   block_spacing: &mut Vec<Au>) {
    // TODO mbrubeck: Take rowspan and colspan into account.
    let number_of_borders_inline_direction = child_table_row.preliminary_collapsed_borders.inline.len();
    // Compute interior inline borders.
    for (i, this_inline_border) in child_table_row.preliminary_collapsed_borders
                                                  .inline
                                                  .iter_mut()
                                                  .enumerate() {
        child_table_row.final_collapsed_borders.inline.push_or_set(i, *this_inline_border);
        if i == 0 {
            child_table_row.final_collapsed_borders.inline[i].combine(&table_inline_borders.start);
        } else if i + 1 == number_of_borders_inline_direction {
            child_table_row.final_collapsed_borders.inline[i].combine(&table_inline_borders.end);
        }

        let inline_spacing = inline_spacing.get_mut_or_push(i, Au(0));
        *inline_spacing = cmp::max(*inline_spacing, child_table_row.final_collapsed_borders.inline[i].width)
    }

    // Compute block-start borders.
    let block_start_borders = &mut child_table_row.final_collapsed_borders.block_start;
    *block_start_borders = child_table_row.preliminary_collapsed_borders.block_start.clone();
    for (i, this_border) in block_start_borders.iter_mut().enumerate() {
        match previous_block_borders {
            PreviousBlockCollapsedBorders::FromPreviousRow(ref previous_block_borders) => {
                if previous_block_borders.len() > i {
                    this_border.combine(&previous_block_borders[i]);
                }
            }
            PreviousBlockCollapsedBorders::FromTable(table_border) => {
                this_border.combine(&table_border);
            }
        }
    }

    // Compute block-end borders.
    let next_block = &mut child_table_row.final_collapsed_borders.block_end;
    block_spacing.push(Au(0));
    let block_spacing = block_spacing.last_mut().unwrap();
    for (i, this_block_border) in child_table_row.preliminary_collapsed_borders
                                                 .block_end
                                                 .iter()
                                                 .enumerate() {
        let next_block = next_block.push_or_set(i, *this_block_border);
        match next_block_borders {
            NextBlockCollapsedBorders::FromNextRow(next_block_borders) => {
                if next_block_borders.len() > i {
                    next_block.combine(&next_block_borders[i])
                }
            }
            NextBlockCollapsedBorders::FromTable(ref next_block_borders) => {
                next_block.combine(next_block_borders);
            }
        }
        *block_spacing = cmp::max(*block_spacing, next_block.width)
    }
}

/// Encapsulates functionality shared among all table-like flows: for now, tables and table
/// rowgroups.
pub trait TableLikeFlow {
    /// Lays out the rows of a table.
    fn assign_block_size_for_table_like_flow(&mut self, block_direction_spacing: Au);
}

impl TableLikeFlow for BlockFlow {
    fn assign_block_size_for_table_like_flow(&mut self, block_direction_spacing: Au) {
        debug_assert!(self.fragment.style.get_inheritedtable().border_collapse ==
                      border_collapse::T::Separate || block_direction_spacing == Au(0));

        if self.base.restyle_damage.contains(ServoRestyleDamage::REFLOW) {
            // Our current border-box position.
            let block_start_border_padding = self.fragment.border_padding.block_start;
            let mut current_block_offset = block_start_border_padding;
            let mut has_rows = false;

            // At this point, `current_block_offset` is at the content edge of our box. Now iterate
            // over children.
            for kid in self.base.child_iter_mut() {
                // Account for spacing or collapsed borders.
                if kid.is_table_row() {
                    has_rows = true;
                    let child_table_row = kid.as_table_row();
                    current_block_offset = current_block_offset +
                        match self.fragment.style.get_inheritedtable().border_collapse {
                            border_collapse::T::Separate => block_direction_spacing,
                            border_collapse::T::Collapse => {
                                child_table_row.collapsed_border_spacing.block_start
                            }
                        }
                }

                // At this point, `current_block_offset` is at the border edge of the child.
                flow::mut_base(kid).position.start.b = current_block_offset;

                // Move past the child's border box. Do not use the `translate_including_floats`
                // function here because the child has already translated floats past its border
                // box.
                let kid_base = flow::mut_base(kid);
                current_block_offset = current_block_offset + kid_base.position.size.block;
            }

            // Compute any explicitly-specified block size.
            // Can't use `for` because we assign to
            // `candidate_block_size_iterator.candidate_value`.
            let mut block_size = current_block_offset - block_start_border_padding;
            let mut candidate_block_size_iterator = CandidateBSizeIterator::new(
                &self.fragment,
                self.base.block_container_explicit_block_size);
            while let Some(candidate_block_size) = candidate_block_size_iterator.next() {
                candidate_block_size_iterator.candidate_value =
                    match candidate_block_size {
                        MaybeAuto::Auto => block_size,
                        MaybeAuto::Specified(value) => value
                    };
            }

            // Adjust `current_block_offset` as necessary to account for the explicitly-specified
            // block-size.
            block_size = candidate_block_size_iterator.candidate_value;
            let delta = block_size - (current_block_offset - block_start_border_padding);
            current_block_offset = current_block_offset + delta;

            // Take border, padding, and spacing into account.
            let block_end_offset = self.fragment.border_padding.block_end +
                if has_rows { block_direction_spacing } else { Au(0) };
            current_block_offset = current_block_offset + block_end_offset;

            // Now that `current_block_offset` is at the block-end of the border box, compute the
            // final border box position.
            self.fragment.border_box.size.block = current_block_offset;
            self.fragment.border_box.start.b = Au(0);
            self.base.position.size.block = current_block_offset;

            // Write in the size of the relative containing block for children. (This information
            // is also needed to handle RTL.)
            for kid in self.base.child_iter_mut() {
                flow::mut_base(kid).early_absolute_position_info = EarlyAbsolutePositionInfo {
                    relative_containing_block_size: self.fragment.content_box().size,
                    relative_containing_block_mode: self.fragment.style().writing_mode,
                };
            }
        }

        self.base.restyle_damage.remove(ServoRestyleDamage::REFLOW_OUT_OF_FLOW | ServoRestyleDamage::REFLOW);
    }
}

/// Inline collapsed borders for the table itself.
#[derive(Debug)]
struct TableInlineCollapsedBorders {
    /// The table border at the start of the inline direction.
    start: CollapsedBorder,
    /// The table border at the end of the inline direction.
    end: CollapsedBorder,
}

enum PreviousBlockCollapsedBorders {
    FromPreviousRow(Vec<CollapsedBorder>),
    FromTable(CollapsedBorder),
}

enum NextBlockCollapsedBorders<'a> {
    FromNextRow(&'a [CollapsedBorder]),
    FromTable(CollapsedBorder),
}

/// Iterator over all the rows of a table
struct TableRowIterator<'a> {
    kids: MutFlowListIterator<'a>,
    grandkids: Option<MutFlowListIterator<'a>>,
}

impl<'a> TableRowIterator<'a> {
    fn new(base: &'a mut BaseFlow) -> Self {
        TableRowIterator {
            kids: base.child_iter_mut(),
            grandkids: None,
        }
    }
}

impl<'a> Iterator for TableRowIterator<'a> {
    type Item = &'a mut TableRowFlow;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // If we're inside a rowgroup, iterate through the rowgroup's children.
        if let Some(ref mut grandkids) = self.grandkids {
            if let Some(grandkid) = grandkids.next() {
                return Some(grandkid.as_mut_table_row())
            }
        }
        // Otherwise, iterate through the table's children.
        self.grandkids = None;
        match self.kids.next() {
            Some(kid) => {
                if kid.is_table_rowgroup() {
                    self.grandkids = Some(flow::mut_base(kid).child_iter_mut());
                    self.next()
                } else if kid.is_table_row() {
                    Some(kid.as_mut_table_row())
                } else {
                    self.next() // Skip children that are not rows or rowgroups
                }
            }
            None => None
        }
    }
}
